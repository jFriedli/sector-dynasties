# Contributing

Sector Dynasties is built by many autonomous agents (and humans) working
concurrently. If you're an agent, read `AGENTS.md` first; it's the canonical,
persistent workflow (issue claiming, worktrees, high-collision files). This file
covers local setup and PR conventions for anyone.

## Local setup

Rust (stable) plus the WASM target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup target add wasm32-unknown-unknown
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Node 20+ (Node 24 is what this repo was bootstrapped and tested against).

```bash
cd web && npm install
```

## Everyday commands

```bash
cargo test --workspace                 # Rust tests
cargo run -p sim-cli -- run --seed 42 --years 5   # headless run
npm --prefix web run build:wasm        # rebuild the wasm bridge after touching Rust
npm --prefix web run dev               # frontend dev server
npm --prefix web test                  # frontend tests, including the copy lint
```

See `docs/TESTING.md` for the full check list (formatting, linting, clippy, the
soak test) and `docs/ARCHITECTURE.md` for the module map.

## Branching, issues, and PRs

- Branch format: `issue-<number>-short-name`.
- One issue per PR wherever practical; link it with `Closes #N`.
- PR description should say what changed, why, and which checks you ran locally.
- Keep issues out of the diary business: a claim comment, an occasional blocker
  note, and the final PR are enough (see `AGENTS.md`).

## Labels

Area labels: `simulation`, `worldgen`, `economy`, `trade`, `characters`, `dynasty`,
`politics`, `business`, `population`, `culture`, `events`, `ui`, `map`, `portraits`,
`assets`, `persistence`, `testing`, `tooling`, `performance`, `content`,
`architecture`.

Type/status labels: `bug`, `refactor`, `epic`, `good-first-agent`, `parallel-safe`,
`high-collision`, `needs-user-asset`, `blocked`, `status:ready`.

Size labels: `size:small`, `size:medium`, `size:large`.

An issue normally carries one or more area labels, one type/status label if
relevant, and a size label. `status:ready` marks an issue as safe to claim right
now; its absence usually means the issue is still being scoped or is blocked on
something.

## Code style

- Rust: `cargo fmt` defaults, `clippy` clean with `-D warnings`.
- TypeScript/React: Prettier defaults (see `web/.prettierrc.json`), ESLint clean.
- No large speculative abstractions; see the scope-discipline notes in
  `GAME_DESIGN.md` and `docs/ROADMAP.md`.

## Issue format (for anyone opening new issues)

```
### Goal
One short explanation.

### Acceptance criteria
A short checklist.

### Notes
Only important architectural constraints, dependencies, or context.

### Testing
What must be demonstrated.
```

Keep issues concise. Leave room for the implementer's judgment; add "Implementation
details are intentionally flexible..." when it's genuinely true.
