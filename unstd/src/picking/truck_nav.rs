//! Focus navigation for the truck computer screen with a gamepad.
//!
//! The truck UI is built from plain Bevy `Interaction` widgets driven by
//! pointer positions. Instead of duplicating that logic, this navigator snaps
//! the virtual gamepad cursor ([`super::gamepad_pointer`]) between interactive
//! elements:
//!
//! * D-Pad / left stick moves the cursor to the nearest element in that
//!   direction (geometric navigation, like console UIs).
//! * Shoulder buttons (`CycleInventory` / `SwapHands` bindings, idle while in
//!   the truck) click through the tab row.
//! * A presses/releases whatever is under the cursor (handled by the pointer
//!   plugin), including hold-to-complete buttons.
//! * B leaves the truck (handled by `untruck`).
//!
//! Mouse users are unaffected: the navigator only reacts to menu actions and
//! simply repositions the same virtual pointer the game already had.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedNode};
use bevy::window::PrimaryWindow;
use bevy_persistent::Persistent;
use bevy_picking::pointer::{PointerAction, PointerButton, PointerInput};
use uncore::components::truck::TruckUI;
use uncore::components::truck_ui::{TabContents, TabState, TruckTab};
use uncore::components::truck_ui_button::TruckUIButton;
use uncore::input::{ActionState, GamepadStatus, PlayerAction};
use uncore::rumble::RumbleFeedback;
use uncore::states::GameState;
use unsettings::bindings::{ControlBindings, InputDeviceMode};

use super::gamepad_pointer::{GamepadCursorState, gamepad_pointer_id, window_location};

/// Minimum node size (px) for an element to be considered navigable; also
/// filters out `Display::None` subtrees, which collapse to zero-size rects.
const MIN_ELEMENT_SIZE: f32 = 6.0;

/// One pending half of a synthetic click on a widget.
#[derive(Debug, Clone, Copy)]
enum ClickPhase {
    Press,
    Release,
}

#[derive(Resource, Debug, Default)]
struct TruckNavState {
    /// Currently highlighted element.
    focused: Option<Entity>,
    /// Synthetic click in progress: (target, phase).
    pending_click: Option<(Entity, ClickPhase)>,
    /// Tab whose content was last active, to detect switches.
    active_tab: Option<TabContents>,
}

/// A navigable interactive element.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    entity: Entity,
    center: Vec2,
}

/// Adds gamepad focus navigation for the truck UI. Runs alongside
/// [`super::gamepad_pointer::GamepadPointerPlugin`] (registered together).
pub(crate) struct TruckNavPlugin;

impl Plugin for TruckNavPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TruckNavState>()
            .add_systems(Update, truck_focus_nav.run_if(in_state(GameState::Truck)));
    }
}

/// Whether the currently selected tab's content changed since last frame.
fn selected_tab_contents(q_tabs: &Query<(Entity, &TruckTab)>) -> Option<TabContents> {
    q_tabs
        .iter()
        .find(|(_, tab)| tab.state == TabState::Selected)
        .map(|(_, tab)| tab.contents.clone())
}

/// Walks the `ChildOf` chain to see whether `entity` lives under `root`.
fn is_under_root(entity: Entity, root: Entity, q_child_of: &Query<&ChildOf>) -> bool {
    let mut current = entity;
    loop {
        if current == root {
            return true;
        }
        match q_child_of.get(current) {
            Ok(parent) => current = parent.parent(),
            Err(_) => return false,
        }
    }
}

/// Collects every navigable interactive element of the truck UI.
///
/// Filters out disabled widgets, zero-size (hidden) subtrees, elements clipped
/// out of view (scrolled lists) and anything not under the [`TruckUI`] root.
#[allow(clippy::too_many_arguments)]
fn collect_candidates(
    root: Entity,
    viewport: Vec2,
    q_elements: &Query<
        (
            Entity,
            &ComputedNode,
            &GlobalTransform,
            Option<&CalculatedClip>,
            Option<&TruckUIButton>,
        ),
        With<Interaction>,
    >,
    q_child_of: &Query<&ChildOf>,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for (entity, computed, transform, clip, button) in q_elements.iter() {
        // Skip disabled buttons (e.g. depleted craft button).
        if button.is_some_and(|b| b.disabled) {
            continue;
        }
        if !is_under_root(entity, root, q_child_of) {
            continue;
        }
        let size = computed.size();
        if size.x < MIN_ELEMENT_SIZE || size.y < MIN_ELEMENT_SIZE {
            continue;
        }
        let center = transform.translation().xy();
        if let Some(clip) = clip
            && !clip.clip.contains(center)
        {
            continue;
        }
        // Ignore anything parked outside the visible screen.
        if center.x < -MIN_ELEMENT_SIZE
            || center.y < -MIN_ELEMENT_SIZE
            || center.x > viewport.x + MIN_ELEMENT_SIZE
            || center.y > viewport.y + MIN_ELEMENT_SIZE
        {
            continue;
        }
        candidates.push(Candidate { entity, center });
    }
    candidates
}

/// Picks the best candidate located in `dir` from `from`.
///
/// Scores by forward distance plus twice the perpendicular offset, so the
/// nearest element roughly ahead wins over far-away aligned ones.
fn best_candidate(from: Vec2, dir: Vec2, candidates: &[Candidate]) -> Option<Candidate> {
    let mut best: Option<(f32, Candidate)> = None;
    for cand in candidates {
        let delta = cand.center - from;
        let proj = delta.dot(dir);
        // Require meaningful forward movement.
        if proj <= 2.0 {
            continue;
        }
        let perp = (delta - proj * dir).length();
        let cost = proj + 2.0 * perp;
        if best.is_none_or(|(best_cost, _)| cost < best_cost) {
            best = Some((cost, *cand));
        }
    }
    best.map(|(_, cand)| cand)
}

/// The element a fresh focus should land on: the top-left-most content
/// element (tabs excluded so we dive straight into the panel below them).
fn default_target(candidates: &[Candidate], tab_entities: &HashSet<Entity>) -> Option<Candidate> {
    candidates
        .iter()
        .copied()
        .filter(|c| !tab_entities.contains(&c.entity))
        .min_by(|a, b| {
            a.center
                .y
                .total_cmp(&b.center.y)
                .then(a.center.x.total_cmp(&b.center.x))
        })
        .or_else(|| {
            candidates
                .iter()
                .copied()
                .min_by(|a, b| a.center.y.total_cmp(&b.center.y))
        })
}

/// Moves focus to `candidate`, teleporting the virtual cursor there and
/// emitting a matching move event so the picking backend follows.
fn focus_on(
    candidate: Candidate,
    cursor: &mut GamepadCursorState,
    state: &mut TruckNavState,
    window_entity: Entity,
    ev_pointer: &mut EventWriter<PointerInput>,
) {
    let location = match window_location(window_entity, cursor.position) {
        Some(loc) => loc,
        None => return,
    };
    let delta = cursor.teleport(candidate.center);
    ev_pointer.write(PointerInput::new(
        gamepad_pointer_id(),
        location,
        PointerAction::Move { delta },
    ));
    state.focused = Some(candidate.entity);
}

/// Emits half of a synthetic click on `target`; call again next frame for the
/// matching release. Splitting across frames lets `Changed<Interaction>`
/// observers observe the press before it turns back into a hover.
fn synthesize_click(
    target: Candidate,
    phase: ClickPhase,
    cursor: &mut GamepadCursorState,
    window_entity: Entity,
    ev_pointer: &mut EventWriter<PointerInput>,
) {
    let Some(location) = window_location(window_entity, target.center) else {
        return;
    };
    // Keep the backend's idea of the pointer position in sync first.
    let delta = cursor.teleport(target.center);
    ev_pointer.write(PointerInput::new(
        gamepad_pointer_id(),
        location.clone(),
        PointerAction::Move { delta },
    ));
    let action = match phase {
        ClickPhase::Press => PointerAction::Press(PointerButton::Primary),
        ClickPhase::Release => PointerAction::Release(PointerButton::Primary),
    };
    ev_pointer.write(PointerInput::new(gamepad_pointer_id(), location, action));
}

#[allow(clippy::too_many_arguments)]
fn truck_focus_nav(
    actions: Res<ActionState>,
    bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
    game_state: Res<State<GameState>>,
    windows: Query<(&Window, Entity), With<PrimaryWindow>>,
    mut cursor: ResMut<GamepadCursorState>,
    mut state: ResMut<TruckNavState>,
    mut ev_pointer: EventWriter<PointerInput>,
    mut ev_rumble: EventWriter<RumbleFeedback>,
    q_roots: Query<Entity, With<TruckUI>>,
    q_elements: Query<
        (
            Entity,
            &ComputedNode,
            &GlobalTransform,
            Option<&CalculatedClip>,
            Option<&TruckUIButton>,
        ),
        With<Interaction>,
    >,
    q_tabs: Query<(Entity, &TruckTab)>,
    q_child_of: Query<&ChildOf>,
) {
    if *game_state.get() != GameState::Truck
        || matches!(bindings.device_mode, InputDeviceMode::KeyboardAndMouse)
        || !gamepad_status.is_any_connected()
    {
        state.pending_click = None;
        return;
    }
    let Ok((window, window_entity)) = windows.single() else {
        return;
    };
    let Ok(root) = q_roots.single() else {
        return;
    };

    let candidates = collect_candidates(
        root,
        Vec2::new(window.width(), window.height()),
        &q_elements,
        &q_child_of,
    );

    // Map of tab-header entities for quick classification.
    let tab_headers: Vec<(Candidate, &TruckTab)> = q_tabs
        .iter()
        .filter_map(|(entity, tab)| {
            candidates
                .iter()
                .find(|c| c.entity == entity)
                .map(|c| (*c, tab))
        })
        .collect();
    let tab_entities: HashSet<Entity> = tab_headers.iter().map(|(c, _)| c.entity).collect();

    // Finish any synthetic click in progress (release phase).
    if let Some((target_entity, phase)) = state.pending_click {
        if let Some(target) = candidates
            .iter()
            .copied()
            .find(|c| c.entity == target_entity)
        {
            synthesize_click(target, phase, &mut cursor, window_entity, &mut ev_pointer);
        }
        state.pending_click = match phase {
            ClickPhase::Press => Some((target_entity, ClickPhase::Release)),
            ClickPhase::Release => None,
        };
        return;
    }

    let now_selected = selected_tab_contents(&q_tabs);

    // Validate existing focus: it may have been hidden (tab switch) or
    // despawned; fall through to re-acquire below when stale.
    let focus_valid = state
        .focused
        .is_some_and(|e| candidates.iter().any(|c| c.entity == e));

    // Entering the screen (or losing focus): dive into the active tab.
    if !focus_valid {
        if let Some(target) = default_target(&candidates, &tab_entities) {
            focus_on(
                target,
                &mut cursor,
                &mut state,
                window_entity,
                &mut ev_pointer,
            );
        }
        state.active_tab = now_selected;
        // Consume no movement this frame; give the player a beat to react.
        return;
    }

    // Tab switching with the shoulder buttons (LB/RB bindings are idle while
    // in the truck). Click-through keeps a single selection code path for
    // mouse and controller alike.
    let switch_dir: Option<i32> = if actions.just_pressed(PlayerAction::CycleInventory) {
        Some(-1)
    } else if actions.just_pressed(PlayerAction::SwapHands) {
        Some(1)
    } else {
        None
    };
    if let Some(step) = switch_dir
        && let Some(selected) = now_selected.clone()
    {
        // Switchable tabs sorted left-to-right, skipping disabled ones.
        let mut ordered: Vec<&(Candidate, &TruckTab)> = tab_headers
            .iter()
            .filter(|(_, tab)| tab.state != TabState::Disabled)
            .collect();
        ordered.sort_by(|a, b| a.0.center.x.total_cmp(&b.0.center.x));
        if ordered.len() > 1
            && let Some(idx) = ordered.iter().position(|(_, tab)| tab.contents == selected)
        {
            let next_idx = (idx as i32 + step).rem_euclid(ordered.len() as i32) as usize;
            let (target, _) = ordered[next_idx];
            synthesize_click(
                *target,
                ClickPhase::Press,
                &mut cursor,
                window_entity,
                &mut ev_pointer,
            );
            state.pending_click = Some((target.entity, ClickPhase::Release));
            state.focused = Some(target.entity);
            ev_rumble.write(RumbleFeedback::Light);
            return;
        }
    }

    // Geometric focus movement with d-pad / left stick.
    let dir = if actions.just_pressed(PlayerAction::MenuLeft) {
        Some(Vec2::NEG_X)
    } else if actions.just_pressed(PlayerAction::MenuRight) {
        Some(Vec2::X)
    } else if actions.just_pressed(PlayerAction::MenuUp) {
        Some(Vec2::NEG_Y)
    } else if actions.just_pressed(PlayerAction::MenuDown) {
        Some(Vec2::Y)
    } else {
        None
    };
    if let Some(dir) = dir
        && let Some(focused) = state
            .focused
            .and_then(|e| candidates.iter().find(|c| c.entity == e))
    {
        if let Some(next) = best_candidate(focused.center, dir, &candidates) {
            focus_on(
                next,
                &mut cursor,
                &mut state,
                window_entity,
                &mut ev_pointer,
            );
        }
        return;
    }

    // React to tab switches triggered by any means (our own click-through,
    // mouse clicks): dive into the newly opened panel.
    if state.active_tab != now_selected {
        state.active_tab = now_selected;
        if let Some(target) = default_target(&candidates, &tab_entities) {
            focus_on(
                target,
                &mut cursor,
                &mut state,
                window_entity,
                &mut ev_pointer,
            );
        }
    }
}
