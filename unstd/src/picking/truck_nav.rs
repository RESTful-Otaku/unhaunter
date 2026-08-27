//! Console-style gamepad navigation for the truck computer screen.
//!
//! The truck UI is built from plain Bevy `Interaction` widgets. In Bevy 0.16
//! that component is owned by bevy_ui's `ui_focus_system`, which only reacts to
//! the *real* mouse — synthetic pointer/`PickingInteraction` events never reach
//! it. Instead of emulating a cursor, this navigator writes [`Interaction`]
//! directly on the focused element, so every existing click/hover handler works
//! unchanged:
//!
//! * D-Pad / left stick moves focus to the nearest element in that direction
//!   (geometric navigation, like console UIs).
//! * LB / RB (`CycleInventory` / `SwapHands` bindings, idle while in the truck)
//!   step through the enabled tab row.
//! * A (`Confirm`) presses the focused element; tapping fires instant buttons,
//!   holding drives hold-to-activate buttons (craft, end mission).
//! * B (`Drop`) leaves the truck (handled by `untruck`).
//!
//! A small frame is drawn around the focused element so the selection is always
//! visible. Mouse users are unaffected — Bevy's own pointer path keeps working.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedNode, FocusPolicy};
use bevy::window::PrimaryWindow;
use bevy_persistent::Persistent;
use bevy_picking::Pickable;
use uncore::colors;
use uncore::components::truck::TruckUI;
use uncore::components::truck_ui::{TabContents, TabState, TruckTab};
use uncore::components::truck_ui_button::TruckUIButton;
use uncore::input::{ActionState, GamepadStatus, PlayerAction};
use uncore::rumble::RumbleFeedback;
use uncore::states::GameState;
use unsettings::bindings::{ControlBindings, InputDeviceMode};

/// Minimum node size (px) for an element to be considered navigable; also
/// filters out `Display::None` subtrees, which collapse to zero-size rects.
const MIN_ELEMENT_SIZE: f32 = 6.0;

/// Extra pixels the focus frame extends beyond the highlighted element.
const HIGHLIGHT_BORDER: f32 = 2.0;

/// Per-truck-session navigation state.
#[derive(Resource, Debug, Default)]
pub struct TruckNavState {
    /// Currently highlighted element.
    pub focused: Option<Entity>,
    /// A tab press in progress; the matching release is sent next frame so
    /// `update_tab_interactions` sees the Pressed -> Hovered transition that
    /// finalizes the selection.
    pub pending_tab_release: Option<Entity>,
    /// Tab whose content was last active, to detect switches.
    pub active_tab: Option<TabContents>,
}

/// A navigable interactive element.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    entity: Entity,
    center: Vec2,
    size: Vec2,
}

/// Marker for the focus frame drawn around the selected element.
#[derive(Component)]
pub struct TruckFocusHighlight;

/// Registers the focus frame lifecycle; the nav system itself is registered by
/// `untruck` so it can be ordered in front of the Interaction consumers.
pub struct TruckNavPlugin;

impl Plugin for TruckNavPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TruckNavState>()
            .add_systems(OnEnter(GameState::Truck), spawn_highlight)
            .add_systems(OnExit(GameState::Truck), despawn_highlight);
    }
}

fn spawn_highlight(mut commands: Commands) {
    commands.spawn((
        TruckFocusHighlight,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-1000.0),
            top: Val::Px(-1000.0),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            border: UiRect::all(Val::Px(HIGHLIGHT_BORDER)),
            ..default()
        },
        BorderColor(colors::TRUCKUI_ACCENT3_COLOR),
        BackgroundColor(Color::NONE),
        FocusPolicy::Pass,
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
        ZIndex(1000),
    ));
}

fn despawn_highlight(mut commands: Commands, q: Query<Entity, With<TruckFocusHighlight>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
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
/// out of view and anything not under the truck root.
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
        candidates.push(Candidate {
            entity,
            center,
            size,
        });
    }
    candidates
}

/// The truck UI spawns [`TruckUI`] on more than one root (the screen itself and
/// the bottom help bar). Use whichever root owns the most interactive elements.
fn collect_best_root(
    q_roots: &Query<Entity, With<TruckUI>>,
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
) -> (Entity, Vec<Candidate>) {
    let mut best_root = Entity::PLACEHOLDER;
    let mut best: Vec<Candidate> = Vec::new();
    for root in q_roots.iter() {
        let candidates = collect_candidates(root, viewport, q_elements, q_child_of);
        if candidates.len() > best.len() {
            best_root = root;
            best = candidates;
        }
    }
    (best_root, best)
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

/// The element a fresh focus should land on: the top-left-most element of the
/// active tab's content (tabs excluded so we dive straight into the panel),
/// falling back to any non-tab element.
fn default_target(
    candidates: &[Candidate],
    tab_entities: &HashSet<Entity>,
    active_tab: Option<&TabContents>,
    q_contents: &Query<(Entity, &TabContents)>,
    q_child_of: &Query<&ChildOf>,
) -> Option<Candidate> {
    let content_root = active_tab.and_then(|tab| {
        q_contents
            .iter()
            .find(|(_, c)| *c == tab)
            .map(|(entity, _)| entity)
    });
    if let Some(root) = content_root {
        let mut inside: Vec<Candidate> = candidates
            .iter()
            .copied()
            .filter(|c| {
                !tab_entities.contains(&c.entity) && is_under_root(c.entity, root, q_child_of)
            })
            .collect();
        if !inside.is_empty() {
            inside.sort_by(|a, b| {
                a.center
                    .y
                    .total_cmp(&b.center.y)
                    .then(a.center.x.total_cmp(&b.center.x))
            });
            return inside.first().copied();
        }
    }
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
}

/// The next enabled tab after `step` (`-1` previous / `+1` next), or `None`
/// when there is nothing to switch to.
fn next_tab(
    tab_headers: &[(Candidate, &TruckTab)],
    selected: &TabContents,
    step: i32,
) -> Option<Candidate> {
    let mut ordered: Vec<(Candidate, &TruckTab)> = tab_headers
        .iter()
        .filter(|(_, tab)| tab.state != TabState::Disabled)
        .copied()
        .collect();
    if ordered.len() <= 1 {
        return None;
    }
    ordered.sort_by(|a, b| a.0.center.x.total_cmp(&b.0.center.x));
    let idx = ordered
        .iter()
        .position(|(_, tab)| tab.contents == *selected)
        .unwrap_or(0);
    let next = (idx as i32 + step).rem_euclid(ordered.len() as i32) as usize;
    ordered.get(next).map(|(cand, _)| *cand)
}

/// Writes `value` onto the element only when it differs, so repeat writes do
/// not spam `Changed<Interaction>` observers.
fn set_interaction(query: &mut Query<&mut Interaction>, entity: Entity, value: Interaction) {
    if let Ok(mut interaction) = query.get_mut(entity)
        && *interaction != value
    {
        *interaction = value;
    }
}

fn position_highlight(query: &mut Query<&mut Node, With<TruckFocusHighlight>>, cand: Candidate) {
    for mut node in query.iter_mut() {
        node.left = Val::Px(cand.center.x - cand.size.x * 0.5 - HIGHLIGHT_BORDER);
        node.top = Val::Px(cand.center.y - cand.size.y * 0.5 - HIGHLIGHT_BORDER);
        node.width = Val::Px(cand.size.x + HIGHLIGHT_BORDER * 2.0);
        node.height = Val::Px(cand.size.y + HIGHLIGHT_BORDER * 2.0);
    }
}

fn hide_highlight(query: &mut Query<&mut Node, With<TruckFocusHighlight>>) {
    for mut node in query.iter_mut() {
        node.left = Val::Px(-1000.0);
        node.top = Val::Px(-1000.0);
    }
}

/// Drives console-style focus navigation for the truck UI. Registered by
/// `untruck`, ordered before the systems that consume `Interaction`.
#[allow(clippy::too_many_arguments)]
pub fn truck_focus_nav(
    actions: Res<ActionState>,
    bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
    game_state: Res<State<GameState>>,
    windows: Query<(&Window, Entity), With<PrimaryWindow>>,
    mut state: ResMut<TruckNavState>,
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
    q_child_of: Query<&ChildOf>,
    q_tabs: Query<(Entity, &TruckTab)>,
    q_contents: Query<(Entity, &TabContents)>,
    mut q_interactions: Query<&mut Interaction>,
    mut q_highlight: Query<&mut Node, With<TruckFocusHighlight>>,
    mut ev_rumble: EventWriter<RumbleFeedback>,
) {
    if *game_state.get() != GameState::Truck
        || matches!(bindings.device_mode, InputDeviceMode::KeyboardAndMouse)
        || !gamepad_status.is_any_connected()
    {
        state.focused = None;
        state.pending_tab_release = None;
        hide_highlight(&mut q_highlight);
        return;
    }
    let Ok((window, _)) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());

    let (_root, candidates) = collect_best_root(&q_roots, viewport, &q_elements, &q_child_of);
    if candidates.is_empty() {
        state.focused = None;
        state.pending_tab_release = None;
        hide_highlight(&mut q_highlight);
        return;
    }

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

    // Complete a tab press started with LB/RB or A: the release half that makes
    // `update_tab_interactions` witness Pressed -> Hovered and select the tab.
    if let Some(tab) = state.pending_tab_release.take() {
        set_interaction(&mut q_interactions, tab, Interaction::Hovered);
    }

    let now_selected = selected_tab_contents(&q_tabs);

    // Entering the screen (or losing focus to a tab switch / hidden element):
    // dive into the active tab's content.
    let focus_valid = state
        .focused
        .is_some_and(|e| candidates.iter().any(|c| c.entity == e));
    if !focus_valid {
        if let Some(target) = default_target(
            &candidates,
            &tab_entities,
            now_selected.as_ref(),
            &q_contents,
            &q_child_of,
        ) {
            state.focused = Some(target.entity);
            set_interaction(&mut q_interactions, target.entity, Interaction::Hovered);
            position_highlight(&mut q_highlight, target);
        }
        state.active_tab = now_selected;
        return;
    }

    // Tab switching with the shoulder buttons (LB/RB). Click-through keeps a
    // single selection code path for mouse and controller alike.
    let switch_dir: Option<i32> = if actions.just_pressed(PlayerAction::CycleInventory) {
        Some(-1)
    } else if actions.just_pressed(PlayerAction::SwapHands) {
        Some(1)
    } else {
        None
    };
    if let Some(step) = switch_dir
        && let Some(selected) = now_selected.as_ref()
        && let Some(target) = next_tab(&tab_headers, selected, step)
    {
        set_interaction(&mut q_interactions, target.entity, Interaction::Pressed);
        state.pending_tab_release = Some(target.entity);
        state.focused = Some(target.entity);
        position_highlight(&mut q_highlight, target);
        ev_rumble.write(RumbleFeedback::Light);
        return;
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
        && let Some(next) = best_candidate(focused.center, dir, &candidates)
    {
        if let Some(old) = state.focused {
            set_interaction(&mut q_interactions, old, Interaction::None);
        }
        state.focused = Some(next.entity);
        set_interaction(&mut q_interactions, next.entity, Interaction::Hovered);
        position_highlight(&mut q_highlight, next);
        state.pending_tab_release = None;
        return;
    }

    // A / Cross: press the focused element, and keep it pressed while held so
    // hold-to-activate buttons (craft, end mission) can accumulate time.
    if actions.just_pressed(PlayerAction::Confirm)
        && let Some(focused) = state.focused
    {
        set_interaction(&mut q_interactions, focused, Interaction::Pressed);
        if tab_entities.contains(&focused) {
            // Selecting a tab is a Pressed -> Hovered sequence.
            state.pending_tab_release = Some(focused);
        }
        ev_rumble.write(RumbleFeedback::Light);
        return;
    }
    if actions.pressed(PlayerAction::Confirm)
        && let Some(focused) = state.focused
    {
        set_interaction(&mut q_interactions, focused, Interaction::Pressed);
        return;
    }
    if actions.just_released(PlayerAction::Confirm)
        && let Some(focused) = state.focused
    {
        set_interaction(&mut q_interactions, focused, Interaction::Hovered);
        return;
    }

    // Steady state: re-assert the hover on the focused element (Bevy's UI focus
    // system clears it every screen when the mouse is elsewhere).
    if let Some(focused) = state.focused {
        set_interaction(&mut q_interactions, focused, Interaction::Hovered);
    }

    // React to tab switches triggered by any other means (e.g. mouse clicks):
    // dive into the newly opened panel.
    if state.active_tab != now_selected {
        state.active_tab = now_selected.clone();
        if let Some(target) = default_target(
            &candidates,
            &tab_entities,
            now_selected.as_ref(),
            &q_contents,
            &q_child_of,
        ) {
            state.focused = Some(target.entity);
            set_interaction(&mut q_interactions, target.entity, Interaction::Hovered);
            position_highlight(&mut q_highlight, target);
        }
        return;
    }

    if let Some(focused) = state
        .focused
        .and_then(|e| candidates.iter().find(|c| c.entity == e))
    {
        position_highlight(&mut q_highlight, *focused);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    use bevy::math::Rect;
    use uncore::types::truck_button::TruckButtonType;

    fn element(world: &mut World, parent: Entity, x: f32, y: f32, w: f32, h: f32) -> Entity {
        let e = world
            .spawn((
                Interaction::default(),
                GlobalTransform::from_xyz(x, y, 0.0),
                ComputedNode {
                    size: Vec2::new(w, h),
                    ..default()
                },
            ))
            .id();
        world.entity_mut(e).insert(ChildOf(parent));
        e
    }

    fn button(world: &mut World, parent: Entity, x: f32, y: f32, w: f32, h: f32) -> Entity {
        let e = element(world, parent, x, y, w, h);
        world
            .entity_mut(e)
            .insert(TruckUIButton::from(TruckButtonType::ExitTruck));
        e
    }

    fn tab(
        world: &mut World,
        parent: Entity,
        x: f32,
        contents: TabContents,
        state: TabState,
    ) -> Entity {
        let e = element(world, parent, x, 40.0, 90.0, 30.0);
        world.entity_mut(e).insert(TruckTab {
            tabname: contents.name().to_string(),
            state,
            contents,
        });
        e
    }

    fn content_root(world: &mut World, parent: Entity, contents: TabContents) -> Entity {
        let e = world
            .spawn((
                contents,
                GlobalTransform::default(),
                ComputedNode::default(),
                Node::default(),
            ))
            .id();
        world.entity_mut(e).insert(ChildOf(parent));
        e
    }

    fn collect_under(root: Entity, viewport: Vec2, world: &mut World) -> Vec<Candidate> {
        let mut state = SystemState::<(
            Query<
                (
                    Entity,
                    &ComputedNode,
                    &GlobalTransform,
                    Option<&CalculatedClip>,
                    Option<&TruckUIButton>,
                ),
                With<Interaction>,
            >,
            Query<&ChildOf>,
        )>::from_world(world);
        let (q_elements, q_child_of) = state.get(world);
        let out = collect_candidates(root, viewport, &q_elements, &q_child_of);
        let _ = (q_elements, q_child_of);
        out
    }

    #[test]
    fn best_candidate_picks_nearest_ahead_with_perp_penalty() {
        let mut world = World::new();
        let root = world.spawn(TruckUI).id();
        // same row, 40px to the right; and one diagonal above it.
        let right = button(&mut world, root, 60.0, 20.0, 40.0, 16.0);
        let diagonal = button(&mut world, root, 90.0, 0.0, 40.0, 16.0);
        let _ = diagonal;
        let candidates = collect_under(root, Vec2::new(800.0, 600.0), &mut world);
        let from = Vec2::new(20.0, 20.0);
        let chosen = best_candidate(from, Vec2::X, &candidates).map(|c| c.entity);
        assert_eq!(chosen, Some(right));

        let above = best_candidate(from, Vec2::NEG_Y, &candidates).map(|c| c.entity);
        assert_ne!(above, chosen);
    }

    #[test]
    fn best_candidate_requires_forward_motion() {
        let mut world = World::new();
        let root = world.spawn(TruckUI).id();
        // Only a button strictly to the left; asking for "right" finds nothing.
        button(&mut world, root, -40.0, 20.0, 40.0, 16.0);
        let candidates = collect_under(root, Vec2::new(800.0, 600.0), &mut world);
        assert!(best_candidate(Vec2::ZERO, Vec2::X, &candidates).is_none());
    }

    #[test]
    fn collect_candidates_filters_disabled_hidden_and_clipped() {
        let mut world = World::new();
        let root = world.spawn(TruckUI).id();
        let live = button(&mut world, root, 20.0, 20.0, 40.0, 16.0);
        let disabled = button(&mut world, root, 80.0, 20.0, 40.0, 16.0);
        world
            .entity_mut(disabled)
            .get_mut::<TruckUIButton>()
            .unwrap()
            .disabled = true;
        // Element collapsed to zero size (Display::None subtree).
        element(&mut world, root, 140.0, 20.0, 0.0, 0.0);
        // Element clipped out of view entirely.
        let clipped = element(&mut world, root, 400.0, 400.0, 40.0, 16.0);
        world.entity_mut(clipped).insert(CalculatedClip {
            clip: Rect::new(0.0, 0.0, 200.0, 200.0),
        });
        // Element parked outside the window.
        element(&mut world, root, 5000.0, 20.0, 40.0, 16.0);

        let candidates = collect_under(root, Vec2::new(800.0, 600.0), &mut world);
        let ids: Vec<Entity> = candidates.iter().map(|c| c.entity).collect();
        assert_eq!(ids, vec![live]);
    }

    #[test]
    fn collect_best_root_picks_the_richer_root() {
        let mut world = World::new();
        let root_main = world.spawn(TruckUI).id();
        let root_help = world.spawn(TruckUI).id();
        for i in 0..3 {
            button(
                &mut world,
                root_main,
                20.0 + i as f32 * 60.0,
                100.0,
                40.0,
                16.0,
            );
        }
        button(&mut world, root_help, 20.0, 600.0, 40.0, 16.0);

        let viewport = Vec2::new(800.0, 700.0);
        let mut state = SystemState::<(
            Query<Entity, With<TruckUI>>,
            Query<
                (
                    Entity,
                    &ComputedNode,
                    &GlobalTransform,
                    Option<&CalculatedClip>,
                    Option<&TruckUIButton>,
                ),
                With<Interaction>,
            >,
            Query<&ChildOf>,
        )>::from_world(&mut world);
        let (q_roots, q_elements, q_child_of) = state.get(&world);
        let (root, candidates) = collect_best_root(&q_roots, viewport, &q_elements, &q_child_of);
        let _ = (q_roots, q_elements, q_child_of);
        state.apply(&mut world);
        assert_eq!(root, root_main);
        assert_eq!(candidates.len(), 3);
    }

    #[test]
    fn is_under_root_walks_the_ancestry_chain() {
        let mut world = World::new();
        let grandparent = world.spawn_empty().id();
        let parent = world.spawn_empty().id();
        world.entity_mut(parent).insert(ChildOf(grandparent));
        let child = world.spawn_empty().id();
        world.entity_mut(child).insert(ChildOf(parent));

        let mut state = SystemState::<Query<&ChildOf>>::from_world(&mut world);
        let q_child_of = state.get(&world);
        assert!(is_under_root(child, grandparent, &q_child_of));
        assert!(is_under_root(parent, grandparent, &q_child_of));
        assert!(!is_under_root(grandparent, child, &q_child_of));
        assert!(!is_under_root(child, Entity::from_raw(999), &q_child_of));
    }

    #[test]
    fn default_target_dives_into_the_active_tab_and_skips_tabs() {
        let mut world = World::new();
        let root = world.spawn(TruckUI).id();
        let loadout_tab = tab(
            &mut world,
            root,
            10.0,
            TabContents::Loadout,
            TabState::Selected,
        );
        let journal_tab = tab(
            &mut world,
            root,
            110.0,
            TabContents::Journal,
            TabState::Default,
        );
        let journal_root = content_root(&mut world, root, TabContents::Journal);
        let j1 = button(&mut world, journal_root, 130.0, 90.0, 40.0, 16.0);
        let _j2 = button(&mut world, journal_root, 130.0, 130.0, 40.0, 16.0);

        let mut state = SystemState::<(
            Query<
                (
                    Entity,
                    &ComputedNode,
                    &GlobalTransform,
                    Option<&CalculatedClip>,
                    Option<&TruckUIButton>,
                ),
                With<Interaction>,
            >,
            Query<&ChildOf>,
            Query<(Entity, &TabContents)>,
        )>::from_world(&mut world);
        let (q_elements, q_child_of, q_contents) = state.get(&world);
        let candidates =
            collect_candidates(root, Vec2::new(800.0, 700.0), &q_elements, &q_child_of);
        let tab_entities: HashSet<Entity> = HashSet::from([loadout_tab, journal_tab]);
        let target = default_target(
            &candidates,
            &tab_entities,
            Some(&TabContents::Journal),
            &q_contents,
            &q_child_of,
        );
        let _ = (q_elements, q_child_of, q_contents);
        assert_eq!(target.map(|c| c.entity), Some(j1));
    }

    #[test]
    fn next_tab_wraps_and_skips_disabled_tabs() {
        let mut world = World::new();
        let root = world.spawn(TruckUI).id();
        let loadout = tab(
            &mut world,
            root,
            10.0,
            TabContents::Loadout,
            TabState::Selected,
        );
        let journal = tab(
            &mut world,
            root,
            110.0,
            TabContents::Journal,
            TabState::Default,
        );
        let map_disabled = tab(
            &mut world,
            root,
            210.0,
            TabContents::LocationMap,
            TabState::Disabled,
        );

        let viewport = Vec2::new(800.0, 700.0);
        let candidates = collect_under(root, viewport, &mut world);
        let mut state = SystemState::<Query<(Entity, &TruckTab)>>::from_world(&mut world);
        let q_tabs = state.get(&world);
        let tab_headers: Vec<(Candidate, &TruckTab)> = q_tabs
            .iter()
            .filter_map(|(entity, tab)| {
                candidates
                    .iter()
                    .find(|c| c.entity == entity)
                    .map(|c| (*c, tab))
            })
            .collect();
        let _ = q_tabs;

        // Next from Loadout goes to Journal; disabled LocationMap is skipped entirely.
        assert_eq!(
            next_tab(&tab_headers, &TabContents::Loadout, 1).map(|c| c.entity),
            Some(journal)
        );
        // Wrapping backwards from Loadout lands on Journal (last enabled tab).
        assert_eq!(
            next_tab(&tab_headers, &TabContents::Loadout, -1).map(|c| c.entity),
            Some(journal)
        );
        // Forward from Journal wraps around to Loadout.
        assert_eq!(
            next_tab(&tab_headers, &TabContents::Journal, 1).map(|c| c.entity),
            Some(loadout)
        );
        assert_ne!(map_disabled, loadout);
        assert_ne!(map_disabled, journal);
    }
}
