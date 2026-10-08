# AGENTS.md

Operating contract for agents on this repo. Read before starting; keep it short
enough to actually read.

## 1. Scope

Unhaunter — Rust/Bevy 0.16 horror game, ~25 crates at the repo root. Fork of
`deavid/unhaunter`; my job is to improve this copy without breaking upstream
compatibility. I am the domain expert and choose implementation details
autonomously; I am not trusted to break the build, lose work, or take
irreversible actions.

**KISS.** Simple stupid when it works fine. Complexity only when it measurably
improves the thing. Do not add abstraction, indirection, or config for problems
that do not exist. If a change needs a paragraph to justify, make it smaller.

## 2. Branch model

`main` is the **release** line: stable, battle-tested, tagged. Work is promoted
through three stages and never skips one.

```
feature branch  ──merge──►  staging  ──playtest──►  main
                  (gate)     (green but       (human
                               unproven)        decision)
```

- Branch prefixes: `fix/` `chore/` `feat/` `improv/` `docs/` `assets/`
  `gameplay/` `polish/` `perf/` `ci/`. One logical change each; descriptive
  names (`fix/hunt-warning-intensity-ramp`, not `fix/stuff`).
- New branches fork from **`staging`**, not `main`.
- Merging to `staging` means the automated gate passed. That is *necessary, not
  sufficient* — it proves the game builds and boots, not that a change is good.
- **Promoting `staging` → `main` is the user's call, never mine.** It needs their
  playtesting. Then tag: `git tag -a vX.Y.Z -m "..." && git push origin main --tags`.
- Never commit straight to `main`.
- Keep merged branches forever; they are the rollback points. Revert with
  `git revert`, not by rewriting history.

An old divergent branch that once occupied the name `staging` was archived to
`archive/staging-2024` before the name was reused.

## 3. Gate

Nothing merges until all of this passes:

```bash
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/systemd/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/xorgproto/share/pkgconfig"

cargo fmt --all && cargo fmt --all -- --check
cargo check  --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test   --workspace --all-features
cargo build  --bin unhaunter_game
timeout 14 cargo run --bin unhaunter_game -- --mute   # exit 124 = still alive = PASS
```

The `PKG_CONFIG_PATH` export is required on every cargo command: no sudo on this
host, build deps come from Homebrew. QA/benchmark binaries must keep building —
`cargo build -p ghost_list -p ghost_radio -p walkie_voice_generator --bins`.

Test count must not regress; `.github/ci/check-test-count.sh` enforces it.

CI (`.github/workflows/`) mirrors this gate and adds a headless boot check.
`release-candidate.yml` vets promotions to `main` and states plainly what
automation does *not* prove (feel, balance, audio, visuals, performance).

### If the link fails

`reloc against '.debug_str'` on a crate you did not touch, with check/clippy/test
green, is a corrupted artifact — not a code error:

```bash
cargo clean -p <crate> && cargo build --bin unhaunter_game
```

## 4. Upstream sync

Fetch both remotes each session. `main` tracks `upstream/main` closely (fork
base `d0cd6380`).

**Never merge `upstream/dev-deavid`.** It is 631 commits and a full rewrite:
1500 files, +131k/−64k, 114 conflicts. Every crate is renamed and moved under
`crates/` into a tier layout. It is a migration project, not a merge.

Real fixes arrive as branches off it. **Port them commit-by-commit, never
branch-by-branch** — inspect the isolated commit, not the branch diffstat (which
includes the whole rewrite):

```bash
git log --oneline upstream/dev-deavid..upstream/<branch>
git show --stat <tip>
git show <tip>
```

Decide per commit: already-fixed-here / portable / not-applicable. Record every
decision, including deliberate skips and why, in `CHANGELOG.md` under
`**Upstream Sync**`. "Reviewed and skipped because…" is valuable; silence is not.

## 5. Remote

`origin` is SSH (`git@github.com:RESTful-Otaku/unhaunter.git`). Plain
`git push origin main` works and is authorised — no need to ask. The remote has
PR-only branch protection with a pre-authorised bypass; the resulting notice is
expected. `upstream` is fetch-only over HTTPS.

Keep `origin/main` and `origin/staging` identical to local at session end
(`git rev-list --left-right --count origin/main...main` must read `0 0`).

**Never force-push.** It can destroy an unpushed commit, which history cannot
undo. Fast-forward only.

## 6. What to work on

Order:

1. **Correctness.** Panics, dead code, state with multiple owners, components
   inserted but never read, one-way latches, stale cross-mission state. This is
   where the real finds have been.
2. **Unreachable features** — implemented but never wired up.
3. **Missing feedback** — the player acts and gets nothing back.
4. **UX friction and accessibility** — absent prompts, misleading labels, input
   that lies about the active device.
5. **Performance** — only with a before/after measurement, and say what was
   measured. No unverified speedup claims.
6. **New content** — levels, gear, ghost types, difficulty. Once the above are
   solid, and without destabilising balance.

### Recurring bug classes here

Each of these has occurred more than once, so sweep for them deliberately:

- **Multiple owners of one field** — several systems writing it with no `.after()`
  ordering; last writer silently wins.
- **Implemented but never called** — unreachable behaviour.
- **Inserted but never read** — component whose reader was removed or renamed.
- **One-way latch** — set `true` on some path, never reset, permanently disabling
  something a later event should re-enable.
- **Stale cross-mission state** — trackers not reset on reload.
- **Hard-coded input prompts** — `[Enter]`/`[Escape]` instead of live bindings.
- **Divisor drift** — a normalisation constant that no longer matches the value it
  normalises against.

### Judgement calls

- This is horror. Feedback should build dread, not defuse it. Prefer diegetic
  cues (gear behaviour, sound, cold, light) over UI popups.
- Do not silently retune balance. Balance is the author's intent. Prefer fixing
  the mechanism over adjusting the number; if a real bug forces a value change,
  say so in the commit and changelog.
- Settings structs: add `#[serde(default)]` at **struct** level and a
  missing-field test. Without it, one new field silently wipes every player's
  config on next launch.
- Photosensitivity: keep any flashing effect well below the 3–60 Hz range.
- Debug flags default **off** (`DEBUG_HUNTS`).

## 7. Code

- `rustfmt` is authoritative. Clippy is `-D warnings` on
  `--all-targets --all-features`; no `#[allow]` without a written reason.
- Match the surrounding file's style over personal preference.
- Extract pure functions and unit-test anything with real logic.
  `RepellentCraftTracker` and `hunt_warning_intensity` are the pattern.
- Comments explain *why*, not *what*. Do not narrate code.
- No `dbg!`, `println!`, or `panic!` in runtime paths — `warn!` plus a safe
  fallback. Remove dead code rather than commenting it out.
- `bevy_platform::collections::{HashMap, HashSet}`, not std.
- New assets need an `assets/` branch, an entry in
  `unstd/src/plugins/root.rs`, and a `GameAssets` field. Prefer reusing existing
  art over a new visual language.

## 8. Boundaries

Ask first: force-pushing; deleting a merged branch; large mechanical upgrades
(Bevy/edition bumps — need a plan and their own branch); undoing something the
user may have wanted.

Proceed freely: fixes, features, refactors, tests, docs, changelog, branch →
`staging` → push.

**Never claim verification you do not have.** Visual, audio, and performance
claims are unproven until observed. Honest reporting beats a green-looking log.

## 9. Session rhythm

1. Fetch both remotes; run the §4 sync routine.
2. Reconcile `main` and `staging` with `origin`.
3. Pick the highest-value item from §6 not already in flight.
4. Branch off `staging` → implement → `fmt` → gate → smoke → changelog → commit.
5. Merge into `staging` with `--no-ff`; keep the branch.
6. Push both branches; report what changed, what was measured, what is
   unverified, and what needs the user's eyes.
