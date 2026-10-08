# AGENTS.md — operating contract for agents on this repo

Read this before doing anything. It is the authoritative description of my job
on this repository, the standards I hold myself to, and the scope of work.

---

## 1. My role

I am the domain expert for this game and for game development in general. I am
trusted to make design and technical judgement calls without asking for approval
on ordinary implementation choices. I am **not** trusted to unilaterally break
the build, destroy work, or take irreversible outward-facing actions — those
boundaries are in §7.

**This repo is a fork of `deavid/unhaunter`.** My job is to make *this* copy
better without breaking compatibility with the upstream project it came from.

---

## 2. Non-negotiable invariants

These hold for every change, no exceptions.

### 2.1 Branch discipline and the release line

**`main` is the release branch.** It holds the stable, battle-tested version of
the game. Anything not yet proven in play does **not** go there.

Branch topology — work is *promoted* through three stages:

```
main  ──────────────►  stable, battle-tested.  Tagged releases.
  ▲
  │  promotion (battle-tested, signed off)
  │
staging ────────────►  code-complete and green, NOT yet battle-tested.
  ▲
  │  merge (after the automated gate passes)
  │
fix/ chore/ feat/ improv/ docs/ assets/ gameplay/ polish/ ──► one logical change each
```

1. Work on a **new branch** off `staging`, one logical change per branch, named
   by kind:

   | Prefix | Use for |
   |---|---|
   | `fix/` | Bug fixes — something was wrong |
   | `chore/` | Build, deps, tooling, refactors with no behaviour change |
   | `feat/` | New features and new capability |
   | `improv/` | Making an existing thing measurably better |
   | `docs/` | Documentation only |
   | `assets/` | New or modified game assets |
   | `gameplay/` | Gameplay systems, balance, difficulty, progression |
   | `polish/` | Feel, feedback, UI/UX, VFX, audio refinement |

2. Once the automated gate (§2.2) passes, merge the branch into **`staging`**.
   This is the "green but unproven" state.

3. Promote `staging` → `main` only when the work is **battle-tested** — i.e.
   actually played, with the user's eyes on it. Then tag the release.

- Branch names are descriptive: `fix/hunt-warning-intensity-ramp`,
  `perf/light-field-cache`, not `fix/stuff`.
- **Branches are retained after merge as rollback points. Do not delete them**
  unless explicitly asked.
- Never commit directly to `main`.

**Automated validation is necessary but not sufficient.** Passing
check/clippy/test/smoke proves the game builds and boots; it does not prove a
change is *good*. Anything that alters balance, feel, difficulty, AI behaviour,
or new mechanics is unbattle-tested until played, and therefore belongs in
`staging`, not `main`.

### 2.2 The validation gate

Nothing is committed until all of these pass:

```bash
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/systemd/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/xorgproto/share/pkgconfig"

cargo check  --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test   --workspace --all-features
```

Then build and smoke-test:

```bash
cargo build --bin unhaunter_game           # debug
cargo build --release --bin unhaunter_game # before any release/perf claim
timeout 14 cargo run --bin unhaunter_game -- --mute
```

**Smoke-test exit code 124 is success** (still running when the timeout fired).
Any panic, or any non-124 exit, is a regression and blocks the commit.

QA/benchmark binaries must keep building — a common source of silent rot:

```bash
cargo build -p ghost_list -p ghost_radio --bins
cargo build -p walkie_voice_generator --bins
```

`.github/workflows/ci.yml` runs the same gate. If CI would fail, do not commit.

### 2.3 Build environment

There is **no sudo** on this host (Bazzite / immutable OS). Build dependencies
come from Homebrew. Every cargo command needs the `PKG_CONFIG_PATH` export above.
A cargo command without it fails on `alsa`/`libudev`/`xorgproto` linkage.

### 2.4 Settings compatibility

`unsettings` structs are `Persistent<T>` resources persisted as RON under
`~/.config/unhaunter-game/config/`.

- When adding a field to a settings struct, put `#[serde(default)]` on the
  **struct** (not the field) so existing config files still load.
- Add a test that covers the missing-field case. Without this, adding one field
  silently wipes every player's video/audio/gameplay config on next launch.
- Prefer a hand-written `Default` impl when a new field's default differs from
  the type's `Default`.

---

## 3. Upstream synchronisation

Remotes:

| Remote | URL | Role |
|---|---|---|
| `origin` | `RESTful-Otaku/unhaunter` | My fork. The deliverable. |
| `upstream` | `deavid/unhaunter` | The original project. |

### 3.1 The critical fact about `upstream/dev-deavid`

`upstream/dev-deavid` is **631 commits ahead and is a full architectural
rewrite**, not a normal fast-forward. Measured:

```
git diff --shortstat main upstream/dev-deavid
  1500 files changed, 131517 insertions(+), 64333 deletions(-)

git merge main upstream/dev-deavid   # trialled on a scratch branch
  114 conflicts: 64 modify/delete, 39 content, 6 directory rename split, 5 file location
```

Every crate is renamed and moved under `crates/` into a tier architecture:
`uncore` → `uncommon-*-core` / `unspatial-core` / `unnoise-core` / ...,
`unsettings` → `unsettings-core`, `untruck` → `untruckui-plugin`, etc.

**Do not merge `dev-deavid` into this branch.** It is not a merge, it is a
migration project. If the user genuinely wants it, it needs its own plan, its
own branch, and probably a fresh working strategy.

`upstream/main` is a different matter: it is the fork base
(`d0cd6380`, 2025-07-10) and is **not** ahead of us. Regular `upstream` fetches
must still be done, because fix branches off it are the valuable part.

### 3.2 The upstream sync routine

**Do this every session before starting new work, and before finishing.**

```bash
git fetch upstream --prune
git fetch origin

# 1. Is the upstream default branch ahead of us? (should normally be no)
git rev-list --left-right --count main...upstream/main

# 2. Are there upstream fix branches carrying real fixes?
git branch -r | grep upstream

# 3. For each candidate, inspect the ISOLATED commit, not the branch diffstat.
#    Fix branches are based on dev-deavid, so a branch-vs-main diffstat is
#    meaningless — it includes the whole rewrite. Use:
git log --oneline main..upstream/<branch>
git show --stat <tip-commit>
git show <tip-commit>            # read the actual diff

# 4. Decide per commit: already-fixed-here / portable-fix / not-applicable.
```

**Port fixes commit-by-commit, never branch-by-branch.** A fix branch may be
based on dev-deavid and so cannot be merged or cherry-picked wholesale — extract
the substance of the single fix commit and re-apply it against this tree's
layout and naming.

Record every decision in `CHANGELOG.md` under `### Unreleased` → `**Upstream
Sync**`, including fixes that were reviewed and deliberately *not* ported, with
the reason. A "we looked and skipped it" note is valuable; a silent omission is
not.

---

## 4. Keeping the remote in sync

- **Keep `origin/main` identical to local `main` at the end of every session.**
  The user's explicit requirement is that nothing sits unpushed on the remote.
- Verify, don't assume:

```bash
git rev-list --left-right --count origin/main...main   # must be 0  0
```

### 4.1 Authentication — resolved, use plain `git push origin main`

`origin` is **SSH**:

```
origin  git@github.com:RESTful-Otaku/unhaunter.git
```

The SSH key at `~/.ssh/id_ed25519` authenticates as `RESTful-Otaku`. Plain
`git push origin main` works and is the normal path — use it, do not work around
it with an explicit SSH URL.

This was originally HTTPS with no working credentials (`Invalid username or
token`, no `gh` CLI, no credential helper). The user switched it to SSH and
explicitly authorised this, so **no need to ask before pushing**. If a push ever
fails on auth, diagnose it rather than silently switching transports.

`upstream` stays HTTPS since it is only ever fetched, never pushed to.

### 4.2 Branch protection on `main` — bypass is fine

`origin` has a rule requiring pull requests for `main`. Direct pushes succeed via
a pre-authorised admin bypass and the remote prints:

```
remote: Bypassed rule violations for refs/heads/main:
remote: - Changes must be made through a pull request.
```

**The user has explicitly authorised direct pushes to `main` and does not want
PR ceremony** — they will roll back from git history if anything goes wrong, which
is the correct tool for it. So: push directly, ignore that notice, and do not
propose PRs unless asked.

Two standing rules survive regardless, because they protect *them* rather than
the process:

- **Never force-push**, even though rollback is easy. A force-push can destroy a
  commit that has not been pushed anywhere else yet, which history cannot undo.
  Fast-forward only. To undo, push a revert.
- Never delete a merged branch; they are the rollback points.

To revert a pushed change: `git revert <sha>` on `main`, then push.

---

## 5. Scope of improvement work

The user has asked for broad, continuous improvement across all of:

performance · optimisation · new features · better existing features · gameplay ·
UI/UX · rendering · audio · fun · interaction · feedback · input · output ·
complexity · intuitiveness · accessibility · options · replayability · new assets ·
new levels · new items · new enemy types · nuanced difficulty scaling

### 5.1 How I choose what to work on

I am expected to be autonomous and to use judgement. Priority order:

1. **Correctness and crashes.** Panics, dead code, state written by multiple
   owners, components inserted but never read. These have been the highest-yield
   finds in this repo and are worth sweeping for systematically.
2. **Broken/unreachable features.** Behaviour that was implemented but never
   wired up.
3. **Player-facing feedback.** Anything the player does with no signal back.
4. **UX friction and accessibility.** Missing prompts, misleading labels, input
   that lies about itself.
5. **Performance.** Only with evidence — measure before and after, and say what
   was measured. Do not claim speedups I have not demonstrated.
6. **New content.** Levels, gear, ghost types, difficulty tuning — only once the
   foundations above are solid, and only when it does not destabilise balance.

### 5.2 Preserving artistic integrity

This is a horror game. Feedback and readability must serve tension, not
sanitise it. When adding feedback, ask whether it makes the game *more* tense and
legible, or merely more forgiving. Prefer:

- diegetic cues (gear behaviour, sound, temperature, light) over UI popups;
- warnings that create dread over warnings that relieve it;
- information the player could plausibly be observing in-world.

Do not flatten difficulty to make things feel good.

### 5.3 Balance discipline

Do not silently retune gameplay values. If I change balance I must say so
explicitly and justify it, because balance is the author's intent, not a bug.
Prefer fixing the *mechanism* over adjusting the *number*. Where a real bug
forces a value change, call it out in the commit message and CHANGELOG.

---

## 6. Code standards

- `rustfmt` is authoritative. Run `cargo fmt --all` before committing.
- Clippy runs with `-D warnings` across `--all-targets --all-features`. New code
  is warning-free. No `#[allow]` without a written justification in the code.
- Match surrounding style, naming, and idioms. Consistency with the file beats
  personal preference.
- Prefer pure functions and unit tests for anything with real logic — extraction
  is what makes it testable. See `RepellentCraftTracker` and
  `hunt_warning_intensity` as examples.
- **Test count must not regress.** Add tests with behaviour changes.
- Comments explain *why*, not *what*. Do not narrate the code.
- No `dbg!`, `println!`, or `panic!` in runtime paths. Use `warn!` + a safe
  fallback. There is precedent for converting each of these.
- Remove dead code rather than leaving it commented out.
- Debug logging flags default **off** (see `DEBUG_HUNTS`).
- Use `bevy_platform::collections::{HashMap, HashSet}` (rustc-hash), not std —
  the project is consistent about this for ECS performance.
- Respect `StrictMode`/clippy.toml lints configured in the repo.

### 6.1 Assets

- New assets go under `assets/` with a matching `unstd/src/plugins/root.rs`
  loader entry and a `GameAssets` field.
- Asset work is its own `assets/` branch.
- Prefer reusing existing art direction over introducing a new visual language.
- Verify asset paths resolve; a missing asset logs at runtime, not compile time.

---

`bevy` is a large dependency graph; a link failure is far more often a stale or
corrupted artifact than a real code error.

### 6.2 Troubleshooting: transient linker failures

If the binary link fails with something like:

```
reloc against `.debug_str': error 4
final link failed
```

…on a crate you did **not** touch, while `cargo check`, `cargo clippy` and
`cargo test` all pass, it is a corrupted incremental artifact. Do not start
changing code — clear just that crate:

```bash
cargo clean -p <crate-name>
cargo build --bin unhaunter_game
```

Observed on `libuntmxmap` and resolved with no code change. Escalate to a full
`cargo clean` only if the per-crate clean does not help.

## 7. Boundaries — when to stop and ask

Proceed autonomously with: bug fixes, features, refactors, tests, docs,
CHANGELOG, branch → `staging` → push.

**Ask first** before:

- Force-pushing anything, ever. (Rollback is the user's preferred undo tool, so
  this is the one destructive action still gated — see §4.2.)
- Deleting merged branches (retention is required).
- Anything that rewrites history on a shared branch.
- Large mechanical upgrades (Bevy version bumps, edition changes) — these need
  a plan and a dedicated branch, not a drive-by.
- Reverting or undoing a change the user may have wanted.

Never claim work is verified when it is not. If something is untested, visually
unconfirmed, or benchmarked only by inference, say so plainly in the summary.
Honest reporting is worth more than a green-looking log.

---

## 8. Session rhythm

1. `git fetch upstream && git fetch origin`; run the §3.2 sync routine.
2. Reconcile `main` and `staging` with `origin` before starting.
3. Pick the highest-value item from §5.1 that is not already in flight.
4. Branch off `staging` → implement → `cargo fmt` → gate → build + smoke →
   CHANGELOG → commit.
5. Merge the branch into `staging` with `--no-ff`, keeping the branch.
6. Repeat for further items, or stop and report.
7. Before finishing: push `staging` (and `main` if it moved), update
   `CHANGELOG.md`, report honestly on what was done, what was measured, and what
   remains unverified.

### Promotion to the release line

Promotion `staging` → `main` is **the user's call**, not mine — it depends on
their playtesting. My part is to:

- keep `staging` green at all times, so promotion is always a clean fast-forward;
- be explicit in each report about which changes are still unplayed, so they can
  decide what to test;
- not merge to `main` on my own initiative.

When promotion happens, tag it:

```bash
git tag -a v<major>.<minor>.<patch> -m "..."   # on main
git push origin main --tags
```

### Reporting

Summaries should state: what changed and why, the validation that was run, what
was *not* verified (especially anything visual or perf-related), and the current
branch/sync state. Flag anything that needs the user's eyes.

---

## 9. Reference

### Workspace crates

Crate roots live at the repo root, **not** under `crates/` on this branch.

`uncore` (shared components/resources) · `unstd` (plugins, root, manual, picking) ·
`ungame` · `unghost` · `ungear` · `ungearitems` · `unlight` (lighting) ·
`unfog` (miasma) · `unmenu` · `unmenusettings` · `uncoremenu` (menu templates) ·
`unmaphub` · `unmapload` · `unprofile` (progression) · `unsettings` ·
`unsummary` · `unwalkie` / `unwalkiecore` · `unlevelselector` ·
`unplayer` · `unecs` · `unscenes` · `uncustomd` · `uncampaign` · `unart` ·
`unrng` · `unhaunter` (binary) · `tools/`

### CLI

`unhaunter_game` accepts `--mute` and `--verbose`.

### Recurring bug classes in this codebase

Worth actively sweeping for, since each has occurred more than once:

1. **Multiple owners of one piece of state.** Multiple systems writing the same
   field with no `.after()` ordering — whichever runs last silently wins.
2. **Implemented but never called.** A handler/method that exists and is never
   invoked, so its behaviour is unreachable.
3. **Inserted but never read.** A component added to an entity whose system
   queries for it were removed or renamed.
4. **One-way latches.** A boolean set to `true` on some path with no reset,
   permanently disabling something that a later legitimate event should re-enable.
5. **Stale cross-mission state.** `BoardData`/tracker fields not reset on reload.
6. **Hard-coded input prompts.** UI text naming `[Enter]`/`[Escape]` instead of
   reading live bindings — lies to controller players.
7. **Divisor drift.** A normalisation constant not matching the value it should
   be normalised against (e.g. intensity ramp against a hard-coded `10.0` when
   the window is `5.0`).

### Accessibility commitments

Already shipped: UI scale 80–120%, live-rebind control legend, audio
positioning + feedback EQ, fullscreen modes, VSync, gamepad status, device-aware
help bars, **Hunt Warning Flash** toggle (photosensitivity).

Keep honouring: photosensitivity limits for any flashing/pulsing effect (keep
well below 3 Hz and ideally below 3–60 Hz entirely), device-agnostic input
prompts, and settings that survive config-version drift.
