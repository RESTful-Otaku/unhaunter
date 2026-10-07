### Unreleased

**Quality & Tooling**

* Added a CI workflow (`.github/workflows/ci.yml`) that runs on `main` and feature branches: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, the full test suite, a native binary build, and builds of the QA/dev tools (`ghost_list`, `ghost_radio`, `walkie_voice_generator`) so they cannot silently rot.

**Rendering & VFX**

* Added a **hunt-warning vignette**: a red pulse at the screen edges during the pre-hunt warning, ramping with the ghost's warning intensity. Previously the only cues were the ghost turning red (invisible if it is dark or off-screen) and the walkie audio.
* Fixed the hunt-warning intensity ramp. It was normalised against a hard-coded `10.0` while the warning window is `5.0` seconds, so intensity started at `0.5` and only ever reached `0.95` — the warning never began from silence and never hit full strength. The window length is now a single `HUNT_WARNING_SECS` constant that both the timer and the ramp derive from, with tests locking the range, monotonicity and clamping. This also smooths the electromagnetic interference on gear (flashlight, EMF/ion/geiger meters, spirit box, ...), which was previously jumping to near-full the instant a warning began instead of ramping in.
* Fixed the **Photo Camera** ignoring electromagnetic interference. It implemented the interference handler but never called it, so its "flash misfires during strong interference" behaviour was unreachable dead code — the camera was the only electronic tool that could not be disrupted by a ghost winding up to hunt. All 13 electronic tools now respond.
* Turned off the development `DEBUG_HUNTS` flag, which was left `true` and logged ghost rage/hunt state periodically in every normal playthrough.

**Accessibility**

* Added a **Hunt Warning Flash** toggle in Video settings (default on) to disable the new vignette pulse for photosensitive players. The hunt is still telegraphed by audio and the ghost turning red.
* Tuned the hunt-warning vignette for readability and safety: during the chase it holds a **sustained low level** rather than full strength (a full-strength red cast masked the scene exactly when you need to see it), and the pulse runs at **~1 Hz**, well below the 3-60 Hz photosensitive-epilepsy range, using a `sin²` curve so it reads as a slow breathing glow rather than a strobe.

**UI/UX Polish**

* The Settings menu navigation bar now reflects the **live control bindings and active device** (e.g. `[A]/[B]` on a controller, `[Esc]`/`[Enter]` on keyboard) instead of hard-coded keys.
* Rebinding a gamepad button can now be **cancelled** with the bound Back/Start button, mirroring keyboard Escape; the cancel prompt is device-aware.
* The keyboard/gamepad binding lists no longer hard-code `Press [Escape] to go back` — the prompt uses the live Back binding, so it shows `[Start]` on a controller.

**Upstream Sync**

* Ported upstream's fix for **truck hold-button auto re-push** (`upstream/fix/truck-ui-button-cooldown`, commit `41be4a90`). Holding a hold-button (Craft Repellent, End Mission) through its activation re-fired it, because `holding` was cleared on activation while `Interaction` stayed `Pressed`, so the next frame began a fresh hold. Each re-hold also orphaned the previous hold-progress sound. `TruckUIButton` gains `cooldown_timer` and `require_release`, a `truck_button_cooldown_system` ticks the former, and the hold sound is despawned on activation and on gating.
* Upstream's `upstream/fix/b23-flashlight-overheat-sound-spam` fix was **not** ported: this tree already guards the overheat ding with `self.status != FlashlightStatus::Off`, so it plays once per overheat event rather than per frame.
* Fixed **B18: the ghost ignored haunted objects on large maps** (upstream bug investigation `B18`, branch `bug-investigation-b18`, commits `32f27c14` / `9f04dc64` / `2e9b987e`). Object influence fell off as `1/(d²+1)`, so an object ten tiles away contributed under 1% of its charge and was swamped by the base destination score of `1.0` — the ghost effectively ignored `GhostInfluence` at the ranges found on bigger maps. Influence now falls off linearly as `1/(d + 5)` with a reduced vertical attenuation factor (`20.0` → `12.0`), destination samples doubled `10` → `20`, and the vertical attenuation no longer over-penalises cross-floor pull.
* Added **floor-settle hysteresis** to stop the ghost ping-ponging between floors while wandering. The ghost now tracks `floor_stay_timer`, and a floor change is penalised 20× more until it has held its current floor for 8 seconds. The penalty is waived entirely during a hunt so the ghost still chases across floors and cannot be kited.
* **Balance note:** the influence falloff curve, sample count and floor penalty are gameplay-visible values. They are changed here because they *are* the upstream fix, not as independent tuning — the ghost now visibly seeks out haunted objects, which is the intended behaviour.
* Reviewed and **not** ported: `b09-shared-truck-sounds` and `fix/truck-ui-button-cooldown` networking halves (both require `bevy_replicon` authority/replication infrastructure this tree does not have); `dev-kira-audio` (a `bevy_kira_audio` backend migration — a large dependency change needing its own plan per the repo's upgrade policy); `55-add-reverbs-to-sound-effects` (upstream's own commit message states the effect "doesn't work as expected ... more of a single delay effect"); `feat/quinnet-transport-migration` (networking); `copilot/fix-tar-gz-folder-issue` (packaging, unrelated to this fork's layout).
* Deferred: `environmental-tension-miasma-hazards` (13 commits) adds miasma-driven hazard particles that chase and damage the player. **The mechanic is now implemented here** — see **Miasma Hazards** below. Only the upstream-specific `img/particle_spark.png` visual was not taken; this tree reuses the existing miasma puff art with a hot tint, which keeps the art direction consistent and avoids introducing a second particle visual language for one effect.

**Miasma Hazards**

* Rooms where miasma pressure builds past a high threshold now occasionally **condense a hazard**: a drifting ember that homes in on the visible player, bounces off walls, and burns them on contact for as long as they stand in it. Hiding is safe. This gives the miasma system — previously pure visual density — an actual consequence, and reads diegetically: the room itself becomes dangerous where the ghost is haunting it.
* Hazards consume the pressure that produced them, are capped at 10 at once, live for 10 seconds, and fade in and out rather than popping in or out. They are lit by the map's lighting field, so they disappear in darkness instead of glowing through it, and hidden players are never targeted by damage.
* Ported in substance from upstream's `environmental-tension-miasma-hazards` branch, minus its replication layer (this tree is single-player).

**Gameplay Fixes**

* Fixed the EMF Meter's journal hint staying stuck on: once it read EMF Level 5, the blink never cleared when the reading dropped. It now clears correctly.
* Implemented the Thermometer's long-standing TODO for temperature-threshold feedback: crossing into the **freezing** range now plays a distinct chime and shows a snowflake icon, while unusually **hot** readings buzz and show a hot-springs icon. A hysteresis band prevents the cue from flapping at the threshold.
* Added a **stamina bar** to the in-game HUD. Sprinting, recovering and exhaustion were previously invisible; the bar now appears while stamina is being used or regenerating and turns amber while exhausted, giving clear feedback on when you can sprint again.
* Removed four `panic!()` calls from click-to-move stair pathfinding: unexpected stair orientations (rare map configurations) now log a warning and fall back to safe waypoints instead of crashing the game.
* Removed leftover `dbg!()`/`println!` debug output from runtime systems (ghost door events, tile loading, level finalisation, player vitals), reducing log spam and per-frame overhead. Test-only diagnostics were converted to assertions.
* Fixed a grab conflict where a single [Grab] press could both retrieve deployed gear and pick up nearby scenery when the player stood in a narrow overlap zone. The two systems now share one radius and the gear-retrieval system yields while furniture is held, so a press always does exactly one thing.
* Fixed the truck's **Craft Repellent button** having an unreliable enabled state. Three systems wrote its `disabled` flag every frame with no ordering between them, so whichever ran last silently discarded the others' conditions. Two of them also latched the button off and never cleared it, so returning a full, unopened flask to refund the allowance could not re-enable crafting for the rest of the truck visit. `journal::button_system` is now the single owner and folds in all three conditions (ghost selected, flask can be filled, allowance remaining), with tests covering the allowance, the limit cap, and the refund/re-enable path.
* The Craft Repellent button no longer relabels itself **"End Mission - No More Repellents"** when the allowance is spent. It is not the end-mission button and does nothing when pressed in that state, so the label sent players down the wrong path; it now reads "Out of Repellent Bottles" and shows how many crafts remain otherwise.

**Mission Summary**

* The **"First Ghost Expelled"** achievement is now actually awarded and shown with a trophy banner on the summary the first time you unhaunt a ghost (previously the flag existed but was never set or displayed).
* The summary now shows **XP earned** for the mission and a green **"⬆ LEVEL UP! Lv X → Lv Y"** banner whenever the run advances the player's level, closing the progression feedback loop.
* The mission-selection list now shows each map's **Best** score (for the active difficulty) next to its grade badge, giving players a concrete target to beat.
* A **"NEW PERSONAL BEST!"** banner (with the previous and new score) now appears on the summary screen whenever a successful run beats the stored best for that map and difficulty, adding a clear replay incentive.
* The end-of-mission screen now shows the mission statistics that were previously calculated but never displayed: **Ghosts unhaunted**, **Repellent charges used** and **Average Sanity**. These already fed into the final score, so players can now see the numbers behind their grade.

**Accessibility & Granular Settings**

* Added **Fullscreen** (Windowed / Borderless / Exclusive) and **V-Sync** (Auto / On / Off) to Settings → Video, applied live to the window and persisted. V-Sync Off gives the lowest input latency for competitive play.
* **UI Scale** is now a Video setting (80%–120%). It applies live to every menu, the HUD and the truck computer, and persists across sessions.
* Finished wiring previously inert audio options: **Audio Positioning** (Screen Space / Isometric / Character Relative) now controls 3D spatialisation, and **Feedback EQ** applies a gentle tone trim. Both are selectable in Settings → Audio.
* The in-game control legend now reflects live rebinds and the active device instead of hard-coded keys, so customising controls is immediately visible.
* Removed the obsolete `CharacterControls` setting (superseded by per-action rebinding). Existing profiles migrate automatically.

**Stability & Tooling**

* Fixed a failing `unwalkiecore` test so the intended one-time-event priority downgrade (VeryHigh → VeryLow after a single play) is verified correctly, matching upstream behaviour.
* Fixed an out-of-date `object_charge` bug where attractive-object tracking leaked across missions via reused entity IDs; stale entries are now pruned each frame.
* Replaced a panic-prone, infinite-looping input path in the `ghost_radio` dev tool: it now accepts bounds-checked 1–N choices and exits cleanly on EOF / Ctrl-D.
* Fixed a main-menu bug where the looping title song was despawned and respawned every frame whenever the audio output was silent (e.g. muted). The song now only despawns after being explicitly asked to.
* Fixed stale miasma/room data when loading a new map of the same dimensions: the board now emits an explicit "initialize" signal on full load instead of relying on a size comparison, and out-of-range room tiles are ignored defensively.
* Fixed the "sanity dropping in darkness" walkie-talkie warning, which never fired because it queried a component that was never attached. It now reads the actual light level at the player's tile.
* Removed a duplicate `walkie_voice_generator` binary target that triggered a Cargo output-filename warning; the tool now builds only from its own member crate.
* Workspace-wide clippy cleanup (collapsible matches, redundant borrows, `sort_by_key`, `?` operator) and `cargo fmt` compliance.
* Added native CLI options: `--verbose`/`-v` (repeatable log verbosity) and `--mute` (silence audio at startup) for automated QA and benchmarking runs.

**Gamepad Support**

* Full gamepad/controller support on top of the existing keyboard controls (any device works simultaneously by default).
* New "Controls" section in Settings: choose input device mode, view connected gamepads, rebind every keyboard key and gamepad button, and tune stick deadzone/sensitivity/response curve and aim inversion.
* Right analog stick aims the flashlight; movement supports full analog speed.
* Menus can be navigated with the D-Pad, left stick, A/B (or equivalent) buttons.
* In-game screens (truck computer, pause) are controllable with the left stick driving an on-screen cursor and A clicking.
* The truck computer now has full console-style controller navigation: shoulder buttons switch tabs, D-Pad/stick snaps between tools and journal entries, A selects (hold for hold-to-complete buttons), B leaves the truck.
* Rumble feedback for hunts, deaths, gear interactions and evidence recording, with enable/disable and strength settings.
* The main menu shows connected controllers; help bars, pause screen and the in-game control legend now reflect the active control scheme and live rebinds.

**Utility Gear & Settings**

* Six previously dormant tools are now fully functional utility gear in challenge missions: the Compass points toward the ghost, the Ion Meter reads ionization from miasma and ghost presence, the E-Static Meter alarms on ghost proximity, the Thermal Imager sweeps for cold spots, the Motion Sensor is a deployable tripwire, and the Photo Camera can capture anomalies on film.
* Utility gear assists exploration and tracking but never provides evidence on its own.
* Video settings are now available: window size and aspect ratio presets apply immediately and persist across sessions.

### Version 0.3.1 - 2025-07-10

**Mouse Aiming**

* Now you can hover with the mouse around and the flashlight will follow.
* Flashlight visual enhanced for long range scan.
* Clicking now will make the character walk towards the cursor.
* Clicking nearby something activable/interactive, will also interact with it, as if it were the [E] key.
* Right click will enable/disable right hand equipment, like the [R] key.
* Scrollwheel on the mouse will cycle through inventory like [Q].
* Walking around with mouse has pathfinding and indicators for waypoints. Works on stairs too.

**Journal UI**

* Discard evidence hidden away, now the user has to Shift+Click.
* Simpler, better Ghost filtering logic.
* If only one ghost is possible, it auto-selects. If the ghost selected is no longer possible, it de-selects.
* Improved UI element visibility with adjusted border thickness and padding.
* Implemented repellent crafting limits and refund system based on difficulty level.
* Player automatically exits truck after crafting repellent.
* Added GrayMan to TmpEMFUVOrbs ghost set - now we have 6 ghosts for missions 4, 5 and 6.

**Gameplay Improvements:**

* Miasma can now spread through half-walls and other see-through obstacles.
* Temperature spreads more realistically through walls and obstacles, including through stairs.
* Repellent particles now change color upon hitting ghosts (electric blue for correct, bright red for incorrect).
* Smart hint system provides feedback for incorrect repellent usage.
* Adjusted Temps/Geiger to be easier and more reliable to get.
* Increased probability of evidence hints to help player progression.
* Made sanity warnings more frequent and responsive.
* Ambient audio drops just before hunts.
* Lock down evidences given by walkie talkie hints.
* Ghost is now easy to expel, needing less repellent to do so.

**Walkie-Talkie System:**

* Ghost hunt warnings are now only triggered if the player is inside the location and the ghost is sufficiently healthy.
* Hunt warnings now repeat at normal frequency for better player awareness.
* System now ensures players are properly informed when repellent can be crafted.
* Walkie-talkie hints for player sanity and wellbeing are now more sensitive and frequent.
* Increased repeat frequency for hints about crafting repellents when enough evidence is gathered.
* Walkie event priority is reduced if messages have been played before to avoid repetition.
* Adjusted timing and delays for better hint delivery.

**UI/UX Enhancements:**

* Increased visible duration for hint UI messages.
* Better camera reference point for improved player centering.
* Adjusted flashlight lighting effect.
* Prevented hover interactions on mostly invisible UI elements - fixes UI elements being hovered and sticky.
* Ghost now goes black under night vision instead of green - this is to avoid players confusing this with UV reactions.

**Other:**

* Bevy upgraded to version 0.16
* Complete overhaul of the `ghost_list` CLI tool.

**Fixes:**

* Fixed Spirit Box False Positives: Resolved bug causing incorrect spirit box readings.
* Fixed error on negative miasma pressure that got NaN into player position and direction, making the game randomly unplayable.
* Fix for random crash when expelling an entity (sound file format error)
* Fix collision bug when a door is closed on top of the player.
* WASM: Prevent accidental closing of the tab via Ctrl+W.
* Fixed bug where ghost guess auto-selection wasn't always working properly.
* Fixed ghost guess resource to use difficulty-specific ghost sets.
* Fixed log spam when music volume setting was zero.


### Version 0.3.0 - 2025-05-31

* New campaign with 15 new maps!
* Progression system with money, experience and leveling up.
* Walkie Talkie buddy extended aggressively with an additional Hint UI.
* Now missions are graded from A to F.
* New design for the menu.
* The menu now works with the mouse as well as keyboard.
* Multi-floor support and stairs. Maps can now have multiple floors.
* Controls are now fully configurable via config file. (User must edit file manually)
* Flashlight and lighting rework to make it look moodier.
* Additional spritesheets.
* New contributed map `Tarin Library` (thanks!!)
* New website! https://www.unhaunter.com

### Version 0.2.7 - 2025-03-26

**Features:**

*   **Walkie Talkie Buddy:** Introduced an NPC companion who provides hints and commentary
  via walkie-talkie, primarily aimed at assisting players on easier difficulties. #94
    *   Includes UI text display accompanying the audio messages.
    *   Provides contextual messages for events like:
        *   starting a mission
        *   forgetting gear
        *   pre-hunt warnings
    *   Added more message variety and tuned activation conditions.
*   **Hold-to-Activate Truck Buttons :** Implemented a hold duration requirement for critical
  truck UI buttons (Craft Repellent, End Mission) to prevent accidental activation.
  Includes visual progress bar and audio feedback during the hold. #32
*   **Auto-Hiding Mouse Cursor:** The mouse cursor now automatically hides after a short
  period of inactivity during gameplay, enhancing immersion. #74
*   **Basic Spatial Audio:** Implemented initial spatial audio (volume-based positioning)
  for sound effects, providing better directional awareness. #54
*   **Look at left hand:** `Left Control` now allows to focus and see the left hand gear,
  and set the evidence on the left hand. #27

**Changes:**

*   **Music volume:** Increased music volume, which allows for a very loud music at 100%. #92
*   **Snappier Camera:** Adjusted for a faster, snappier feel and improved responsiveness. #93
*   **WASM now allows for resizing:** Improved WASM CSS handling so in-browser gaming uses the full browser window. #63

**Fixes:**

*   **UI:** Corrected visibility issues with the in-game control key legend.
*   **Gear - Recorder:** Removed an experimental false reading mechanic that could be confusing.
*   **Spirit Box:** Limited the effective range of the Spirit Box to require closer proximity to the ghost. #68
*   **Replaced duplicate ghosts:** Domovoi and Wisp were duplicates so they have been renamed to new unique ghosts. #87

**Other:**

*   **Tools:** Added an internal `ghost_list` developer tool for viewing ghost/evidence statistics. #86
*   **Kokoro TTS:** Added notes and setup instructions for Text-to-Speech tooling experimentation.
*   **Documentation:** Updated README and internal developer notes. #37
*   **Wiki:** Added the basis for the wiki on GitHub with pages for evidence, ghosts, etc. #38 #40

### Version 0.2.6 - 2025-03-09

**Features:**

*   **Miasma System:** Introduced a dynamic miasma (fog) system that affects gameplay.
    The miasma's density and movement are simulated, and it interacts with the environment and player.
*   **Electromagnetic Interference (EMI):** Added a new mechanic where the ghost,
    particularly during its pre-hunt warning phase, emits electromagnetic interference.
    This affects electronic gear (Flashlight, EMF Meter, Recorder, Red Torch, Videocam),
    causing glitches, malfunctions, and false readings. This adds a layer of challenge
    and realism to using electronic equipment.
*   **Stamina System Integration:** The miasma now directly impacts the player's stamina.
    Higher miasma density in a location increases the rate at which stamina depletes while sprinting.
    This encourages players to avoid or quickly traverse miasma-filled areas.

**Changes:**

*   **Lighting System Refactoring:**
    * The lighting system has been significantly refactored for improved performance and organization.
    * Light propagation now uses a pre-baked data structure along with wave edges, allowing for dynamic light calculation with closed doors and windows.
    * The van's light is sampled by an offset to avoid darkening.

*   **Ghost Behavior:**
    * Ghost rage mechanics were refined, now considering player presence in the location overall.
    * Added a pre-hunt warning state for the ghost, including visual and auditory cues.

*   **Difficulty Adjustments:**
    * Updated hunt provocation radius values across different difficulty levels to improve game balance.

*   **Gear Adjustments:**
    * Added Electromagnetic interference to electronic gear.


### Version 0.2.5 - 2025-02-08

**Features:**

*   **Screenspace orthogonal movement**: Now players can choose to move the
    character relative to the screen or relative to the map.
*   **Settings**: Added gameplay + audio settings which are saved to disk or
    local storage for WASM.

**Changes:**

*   Bumped Bevy version to 0.15

**Other:**

*   Migrated core features and systems to separate crates for better organization and maintainability.

### Version 0.2.4 - 2024-12-30

**Features:**

* **Manual with 5 chapters!**
  * Created a whole manual with 5 chapters to ease in new people into the game.
  * Each chapter is linked to a difficulty to help people read the critical things on the game.
* **Ghost Rage adjusted:**
    * Ghosts now get angrier with more aggressive actions.
    * Ghost hunting duration is no longer increased by the anger.
    * A new cool-down logic for hunting has been implemented on top of the ghost rage.

**Changes:**

* **Gear:**
  * Added support to use different gears in the truck depending on the chosen difficulty.
  * Adjusted `EMF` meter sensibility to adapt to the difficulty.
  * Improved `Red Torch` and `Video Cam` lighting.
* **UI:**
  * Refactored code to support tutorial mode and normal user manual.
  * Minor UI fixes.
* **Upgraded:**
  * Bevy 0.14
  * Tiled 0.12

**Fixes:**
* Fixed a small bug that was making the light levels to be too low.
* Code has been cleaned and commented, improving overall quality.

### Version 0.2.3 - 2024-06-23

**Features:**

* **New Consumable Items:**
    * Salt: You can drop salt on the ground, and if the ghost walks over it,
      it will leave a trace of salt where it goes. These traces are only visible
      under UV light.
    * Quartz Stone: It absorbs the ghost's hunting energy, effectively shortening
      hunts and protecting the player. The stone gradually cracks and eventually
      breaks after repeated uses.
    * Sage: Burn it and smoke the ghost, and it will calm it down for 30 seconds.
      During hunts, it will confuse the ghost and make it lose track of the player.
* **New Menu: Map Hub:**
    * Provides a clear and dedicated menu for selecting the map and difficulty
      level before starting a new game.
    * Displays all available maps in a list, allowing for easy browsing and selection.
    * Presents a separate screen for choosing from a range of 16 difficulty levels,
      each with a description of its unique challenges.
* **Expanded Difficulty System:**
    * Offers 16 distinct difficulty levels, ranging from "Novice Investigator"
      to "Master Guardian", providing a wide range of challenges for players
      of all skill levels.
    * Each difficulty level affects various aspects of the game, including
      ghost behavior, environment conditions, player attributes, and scoring.
    * Difficulties now customize the available equipment for the player.
* **Improved Ghost Expulsion Feedback:**
    * When a ghost is successfully expelled, it now fades out over 5 seconds
      while emitting smoke particles, creating a more noticeable and satisfying
      visual effect.
    * The ghost's breach also fades out, indicating its permanent
      departure from the location.
    * Two distinct roar sounds now play during the expulsion, adding to the
      dramatic effect.

**Fixes:**

* **Lighting:** Several adjustments have been made to the lighting system to
  create a more visually appealing and atmospheric environment.
* **Quartz Stone:** The Quartz Stone's energy absorption rate is now adjusted
  based on the selected difficulty level, making it more effective at higher difficulty levels.
* **Repellent Flask:** The behavior of the Repellent Flask particles has been
  improved. They now spread more realistically, preventing them from clumping
  together or moving in an unnatural way.
* **Hiding Mechanics:**
    * Players can no longer hide while carrying any items,
      preventing conflicts with other actions.
    * The player's sprite now changes color when hiding,
      providing a visual indicator of their hidden state.
    * Hiding now requires the player to hold down the "E" key
      for a short duration, reducing accidental triggers.
* **Truck Journal:** The Truck Journal UI now filters possible evidence more
  effectively based on the selected ghost type, streamlining the evidence selection process.
* **Windows:** Fixed a bug that prevented map tileset images from loading correctly
  in the Windows build, ensuring compatibility and a consistent experience across platforms.
* **General:**
    * Addressed several warnings and clippy lints to improve
      code quality and maintainability.
    * Fixed minor UI issues and typos in the game.

### Version 0.2.2 - 2024-06-02

**Features:**

* **Item Grab & Drop:**
    * Items are now dropped to the center of the tile and it
      checks for other items to prevent dropping items in occupied tiles.
* **Deployable Gear:**
    * Players can now press `G` to drop the gear to the ground.
    * `F` will pick up the gear if there's space for it.
    * Gear continues to work while on the ground
* **New Sound effects:**
    * The ghost will snore, roar and more on different scenarios.
    * The player will hear a loud heardbeat when the health is low.
    * The backgroud music will fade out and be replaced by unsettling sounds
      when the sanity is low.
* **Basic Spatial sound:**
    * Now the sounds will have a volume (not panning) depending on the distance.

**Fixes:**
  * Increased overall brightness of the game.
  * Ensure the torch iluminates the player itself.
  * Fix player sprite to follow flashlight ilumination.
  * Lower difficulty by making ghost a bit friendlier.
  * Now torches have occlusion too.
  * Prevent the ghost from re-hunting when the player re-enters location.


### Version 0.2.1 - 2024-05-26

**Features:**

* **Ghost Events:**
    * The ghost can now slam a door closed.
    * The ghost can also flicker lightly the lights of the room.
* **Hiding Mechanics:**
    * Player can now press `E` near tables, beds, and other objects to hide itself.
    * The ghost will be confused when hunting for the player.
    * The player will need to press `E` again to be able to move.
* **Grab & Drop Items:**
    * Players can now pick up light items with `F`
    * While carrying an item they're slower, they can drop the item with `G`
* **Object-Ghost Interaction System:**
    * Introduced a new system that allows players to influence the ghost's
      behavior by manipulating objects in the environment.
    * Objects are assigned hidden properties (`Attractive`, `Repulsive`) that
      affect how the ghost interacts with them.
    * The ghost's roaming behavior is now influenced by the proximity
      and charge levels of objects.
    * Players can provoke hunts by placing `Repulsive` objects near
      the ghost's breach and by removing `Attractive` objects from the location.
    * Added visual glow effects to objects based on their properties
      and the active light source, revealing their influence on the ghost.

**Maps:**

* **Updated University map:**
    * As we now depend on having objects, it was time to get an update to the
      big university map to add all sorts of objects. It looks pretty cool!
* **New Tutorial: Glass House:**
    * A simple map where all walls are transparent so that it is easy to see,
      and players can learn the ghost evidences at their own pace.

**Fixes:**

* **Lighting:** Improved lighting system and see in the dark.
* **Reduced difficulty:** With the new object interaction system it was becoming
  much more difficult as there's more need to explore, so sanity loss and ghost rage
  was reduced. Temperatures now update in a more aggressive way to simplify tracking
  the ghost as it now roams much more.

**Tools:**

* **Ghost Radio Tool:**  Added a console-based demo tool for testing and
  experimenting with simplified ghost communication.


### Version 0.2.0 - 2024-04-02

Features:
  - New tutorial for basic movement, doors, light switches and torch.
  - Improved Pixel-perfect shader.
  - Improved see in the dark view with additional exposure range and bluish
    tones to convey the sensation of a dark place.
  - Added "ambient light" mixing mode for the shader for improved visuals.
  - Sanity now slowly increases when outside or when the ghost is very far.
  - Added NPC dialogs for use in tutorials.
  - Auto-trigger NPC dialogs by proximity.
  - New sprites and utilities to be used mainly by tutorials.

Fixes:
  - Sort maps by filename in the main menu.
  - Prevent van from auto-opening if far away.
  - Do not animate player walking while the game is paused.

### Version 0.1.9 - 2024-03-24

Features:
  - Add Equipment description in Loadout UI on hover, also tracks the evidence
    status.
  - Rearrange key help legend to fit the key info closer to the items that the
    keys act upon.

Fixes:
  - Found and fixed the issue with bad WASM performance and stutter. It was
    related to Bevy's trace feature. It has been disabled by default. Now works
    on Firefox too.
  - Prevent the player inventory in Loadout UI to wrap (WASM)


### Version 0.1.8 - 2024-03-23

Features:
- Make WASM window smaller

Other:
- Fix UI issues after `cargo update` breaks the interface

### Version 0.1.7 - 2024-03-21

Features:
- WASM version autodeployed from `main` branch


### Version 0.1.6 - 2024-03-20

Features
- New Tab in Truck for Loadout that is now the default
- Able to select the gear the player wants to carry
- Limit the maximum amount of gear of the player to 2 hands + 2 extra inventory items
- Skip empty inventory slots when cycling
- Truck UI is shown when starting the game so player can begin by choosing gear
- Two extra disabled tabs added in Truck UI for future use

Other:
- Refactor Truck UI code

### Version 0.1.5 - 2024-03-15

Features:
- Game UI rework to show clearly gear on left and right hand
- Next item to be grabbed by [Q] key is shown on screen
- [TAB] now toggles the left hand, [T] swaps left and right
- Vignetting added for health display, better visibilty
- Vignetting now also displays sanity to encourage players to go to the van
- Prevent flashlight from turning off when overheating
- Breach and ghost now slowly pulsate to make them more obvious
- Ghost can now warp long distances

Fixes:
- Typo on help for Ghost Orbs evidence
- Prevent Ghost range/hunt from being stuck high while outside
- Sanity now is no longer lost while outside of the location
- Added feedback on the "Craft Repellent" button

### Version 0.1.4 - 2024-03-14

Changes:
- WASM support (with problems)
- Deployed WASM to https://deavid.github.io/unhaunter/

### Version 0.1.3 - 2024-03-12

Features:
- Added main menu song
- Added two new maps (small house and school)
- Added map selector in main menu screen
- Added sanity system + added it to summary scoring
- Added ghost hunting mechanic + summary scoring
- Added evidence quick selector
- Record evidence outside of the truck
- Instructions visible on screen all-time, gear tied to evidence.

Performance fixes:
- Don’t draw invisible tiles
- Avoid updating UI / Texts if they don’t change
- Decoupled walking speed from FPS
- Optimized Temperature field update to avoid considering cells with no heat sources
- Optimized realtime light rendering to constrain checks to tiles nearby to the player

Fixes:
- Added sprite to prevent occlusion leak from top-left corner of maps
- Player sprite was being “colored” from old & new code which caused flickering. Removed old code.
- Camera / player sprite stutter fix

Other:
- Migrated from Bevy 0.12 to 0.13
- Added instructions to profile Unhaunter
- Lots of refactor 🙂


### Version 0.1.0 - 2024-03-06

Initial MVP Release.

