# AGENTS.md — working notes for AI agents on this repo

## What this is

Unhaunter: a ~35k-LOC, 25-crate Rust game built on Bevy 0.16. Ghost-hunting
horror game: you survey haunted houses, collect evidence, and expel ghosts
before they hunt you.

Workspace crates (all under the repo root, not a `crates/` dir):
`uncore` (shared components/resources), `unstd` (plugins/root), `ungame`,
`unghost`, `ungear`, `ungearitems`, `unlight`, `unfog`, `unmenu`, `unmenusettings`,
`uncoremenu`, `unmaphub`, `unmapload`, `unprofile`, `unsettings`, `unsummary`,
`unwalkiecore`, `unwalkie`, `unlevelselector`, `unhaunter` (binary),
`uncampaign`, `unart`, `unrng`, `unecs`, `unscenes`, `uncustomd`.

## Build environment (REQUIRED)

Build deps come from Homebrew — there is **no sudo** (Bazzite/immutable host).
Every cargo command needs this env var:

```bash
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/systemd/lib/pkgconfig:/home/linuxbrew/.linuxbrew/opt/xorgproto/share/pkgconfig"
```

## Validation gate

Run all four before every commit:

```bash
export PKG_CONFIG_PATH="..."   # as above
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --workspace --all-features
```

Then build and smoke-test the game:

```bash
cargo build --bin unhaunter_game          # and: cargo build --release --bin unhaunter_game
timeout 14 cargo run --bin unhaunter_game -- --mute
```

**Exit 124 from the smoke test is the success case** — it means the game booted
and was still running when the timeout fired. Any panic or non-124 exit is a
regression.

Other QA/benchmark binaries that must keep building:
`--bin ghost_list`, `--bin ghost_radio`, `--bin walkie_voice_generator`.

`.github/workflows/ci.yml` runs the same gate, so CI failures mirror these.

## Git workflow

- `main` stays stable. **Work on a branch**, one validated slice per branch:
  `feat/...` or `fix/...`.
- Branch off `main` → implement → `cargo fmt --all` → validate → build + smoke →
  update `CHANGELOG.md` under `### Unreleased` → commit →
  `git checkout main && git merge --no-ff <branch>`.
- Feature branches are **retained** as rollback points; do not delete them.
- Git identity is set **repo-locally** (`RESTful-Otaku
  <restful-otaku@users.noreply.github.com>`); global config is untouched.

## Upstream sync

Remote `upstream` = `deavid/unhaunter`.

**Do not merge upstream `dev-deavid` casually.** It has 631 commits that
restructure the entire workspace (crates moved under `crates/`, networking
added). It is effectively a rewrite, not a safe merge. When upstream fixes
something we need, cherry-pick the specific change (this is how the
`unwalkiecore` test-expectation fix was taken).

## Code conventions

- Follow the surrounding style; `rustfmt` is authoritative.
- Clippy is `-D warnings`, including `--all-targets --all-features`. New code must
  be warning-free (no `#[allow]` unless there's a documented reason).
- Settings live in `unsettings` as `Persistent<T>` resources persisted as RON in
  `~/.config/unhaunter-game/config/`. When adding a field to a settings struct,
  add `#[serde(default)]` at the **struct** level so existing config files still
  load, and add a test covering the missing-field case.
- Precedents worth following: prefer removing a `panic!()` with a safe fallback
  over changing balance/tuning; prefer a warning log plus fallback over `dbg!` or
  `println!` in runtime paths.
- Watch for dead/disconnected code — this codebase has shipped components that
  are inserted but never read, and stale cross-mission state. When touching a
  system, check its component marker is actually queried.
- `DEBUG_HUNTS` in `unghost/src/ghost.rs` is `false`; don't re-enable it by
  accident (it caused per-playthrough log spam).

## CLI

`unhaunter_game` accepts `--mute` and `--verbose`.
