# Agent instructions

Canonical, persistent rules for Codex/Claude workers (and anyone else) contributing
to Sector Dynasties. Many agents work in this repository concurrently; the rules
below exist to keep that low-collision. If anything here conflicts with a stale
comment elsewhere, this file wins.

## Before you start

1. Read `README.md`, `docs/ARCHITECTURE.md`, and `docs/ROADMAP.md`.
2. Pick an issue labeled `status:ready` that nobody has claimed. Prefer
   `good-first-agent` or `parallel-safe` issues if you're unsure where to start.
   Do not claim an issue that is not labeled `status:ready` (it may be blocked,
   under design, or intentionally held back).
3. Check the issue's "Notes" section for dependencies before starting.

## Claiming work

Comment on the issue to claim it (a single short line: "Claiming this."). Don't
narrate progress into the issue afterward beyond a concise blocker note if you get
stuck; the PR is where the real detail goes. If an issue has been claimed but shows
no activity or linked PR for an extended period, it's fair to comment and pick it
up (link why you believe it's abandoned).

## Branching and worktrees

Branch format: `issue-<number>-short-name`, off `main`.

```bash
git worktree add ../sector-dynasties-issue-42 -b issue-42-short-name origin/main
```

Worktrees let multiple agents (or one agent working on multiple issues) operate
without stepping on each other's working directories. Prefer a worktree over
switching branches in a shared checkout.

## High-collision files

Avoid unrelated issues touching these in the same PR; if your issue genuinely needs
to change one, say so explicitly in the PR description:

- `crates/sim-core/src/state.rs` (the central `SimState` struct: many systems will
  eventually want a field here; add narrowly and justify it)
- `crates/sim-core/src/invariants.rs` (shared invariant list: additive changes are
  usually fine, but check for merge conflicts with other in-flight PRs)
- `crates/sim-core/src/save.rs` (save format migrations: additive per schema bump,
  but only touch this when you're actually bumping `SAVE_SCHEMA_VERSION`)
- `crates/sim-core/src/lib.rs`, `Cargo.toml` files (module/dependency wiring)
  and `web/package.json` (frontend dependency wiring)
- `.github/workflows/ci.yml`, `AGENTS.md`, `CONTRIBUTING.md` (process files: change
  these deliberately, not as a side effect of an unrelated PR)

Module boundaries (see `docs/ARCHITECTURE.md`) exist specifically so most issues
never need to touch the same file as another in-flight issue. If your task keeps
pulling you into a shared file, that's a signal to split the work or add a new
module instead of growing an existing one.

## Making changes

- Keep `sim-core` free of I/O, rendering, and any WASM- or browser-specific code.
  If you're tempted to special-case `#[cfg(target_arch = "wasm32")]` inside
  `sim-core`, the logic probably belongs in `sim-wasm` instead.
- Add tests in the style already established (see `docs/TESTING.md`): a
  determinism test for new randomness, an invariant + a test that it's caught for
  new validity rules, and don't weaken an existing same-seed/round-trip test to
  make your change pass.
- Player-facing text must never contain an em dash or a semicolon; see
  `docs/CONTENT_GUIDE.md`. The copy lint will catch violations under
  `web/src/content/`, but the rule applies to any player-facing string anywhere.
- Never invent binary Unreal-style or otherwise fake assets. If you need art that
  can't be sourced or generated, file a `needs-user-asset` issue (see
  `docs/ASSETS.md`) and use an honest placeholder instead.
- Improve adjacent design when it clearly improves the game or architecture, but if
  the improvement is substantial and not required for your current issue, file a
  follow-up issue instead of scope-creeping the PR.
- Run the relevant checks locally before opening a PR (see `docs/TESTING.md` for
  the full list): `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, and from `web/`:
  `npm run build:wasm && npm run lint && npm run format:check && npx tsc -b &&
  npm test && npm run build`.

## Opening a PR

- One issue per PR wherever practical.
- PR description: what changed and why, and how it was tested (which commands you
  ran, not just "tests pass"). Link the issue (`Closes #N`).
- Don't self-merge unless the repo owner has told you self-merge is standing for
  this repository; otherwise wait for review or CI-only merge policy as configured.
- If you discover a good follow-up opportunity while working, open a concise issue
  for it rather than expanding your current PR's scope.

## Issue tracker etiquette

Not a diary. A claim comment, an occasional short blocker note, and the final PR
are enough. Don't narrate routine work into comments.

## Windows and Linux split

Kali/Linux agents: portable Rust simulation core, CLI, frontend, and CI work.
Windows agents (or anyone on Windows): anything that needs a native Windows
toolchain check for the eventual Tauri packaging, and validating that Rust/wasm-pack
/Node commands documented here actually work unmodified on Windows (see ADR 0001's
"known gaps"). File a bug against this doc if a documented command doesn't work on
Windows as written.

## Never

- Never allocate per-tick in a hot loop without a reason; profile before
  optimizing, but don't introduce obviously wasteful allocation either.
- Never couple `sim-core` to a rendering or audio dependency.
- Never commit generated WASM output (`web/src/wasm/`) or `web/node_modules`,
  `web/dist`, or `/target`.
- Never claim an issue not labeled `status:ready`, and never skip the copy lint
  rule for "just this once" text.
