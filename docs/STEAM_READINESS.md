# Steam readiness

Steam is a future distribution and optional-integration platform, not a
dependency of the game. This doc records the constraints that keep it that
way, so nothing built before a real Steam integration effort makes that
effort harder.

## What must remain true

- **The game runs fully without Steam.** Native saves are plain files
  (`SimState::to_json`/`from_json`, see `docs/ARCHITECTURE.md`) in a normal
  per-user application data or save directory, resolved through a
  cross-platform API (e.g. Tauri's path resolver once the desktop shell
  exists; see `docs/adr/0001-simulation-core-stack.md`), never a
  Steam-specific location.
- **Steam Cloud, if added, synchronizes the same native save files.** It is
  a transport for the existing save format, not a second persistence
  system. `sim-core` must never know Steam exists; a Cloud sync adapter
  would live alongside a filesystem storage adapter, both outside
  `sim-core`.
- **Steamworks stays out of `sim-core` and `sim-wasm`.** Achievements, rich
  presence, and overlay hooks are presentation/platform concerns. If a game
  rule ever needs to react to one (e.g. an achievement unlocking something
  in-game, which is unlikely but not impossible), the trigger belongs in the
  desktop host layer, translated into a normal command/event `sim-core`
  already understands, not a Steamworks call inside the simulation.
- **The browser and Linux dev builds are never Steam-gated.** Steam is a
  Windows/Linux/desktop distribution channel for the eventual Tauri
  package; it says nothing about how the game is built or tested day to
  day.

## What is deliberately not built yet

Steamworks SDK integration, achievements, Cloud, rich presence, overlay
compatibility, Deck-specific input handling, and Workshop/mod distribution
are all out of scope until the desktop build itself exists and there's a
reason to ship on Steam. Building them now would be speculative platform
work ahead of the packaging it depends on (see `docs/ROADMAP.md` milestone
M9, and issues #101-#103 for the browser/Linux/Windows packaging spikes
this would build on).

## Backlog

Tracked as a single follow-up issue rather than several speculative ones;
see the `packaging` and `persistence` labels for when packaging work makes
these concrete.
