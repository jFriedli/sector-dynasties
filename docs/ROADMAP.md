# Roadmap

Milestones are sequencing phases. GitHub labels (see `CONTRIBUTING.md`) carry the
system area (`economy`, `politics`, `worldgen`, ...) independently of milestone, so
an issue's milestone answers "when" and its labels answer "what part of the game."

Epics (tracking issues, labeled `epic`) exist for major systems and link the
actionable issues under them; they are not meant to be closed by one PR.

## M0 — Foundation and Architecture (complete)

Repository structure, the Rust/WASM/React architecture spike, deterministic
simulation foundations (`SimRng`, `SimClock`, `SimState`), headless CLI, save/load,
invariant checks, CI, and this documentation set. See ADR 0001.

## M1 — Vertical Slice

Grow the bootstrap slice into something that already produces small stories: a
slightly richer generated sector, a controllable dynasty head with basic aging and
succession, a first career, a first business archetype, a minimal event or two, and
a UI that shows the dynasty and lets the player advance time and make a choice. The
goal is breadth across the whole loop, not depth in any one system.

## M2 — World, Population, and Culture Foundations

Grow `worldgen` toward real procedural history. Add population dynamics (migration
pressure, social mobility, consumption) and the first culture dimensions, and wire
culture into at least one system (nepotism tolerance is a good first target).

## M3 — Dynasty, Careers, and Succession

Marriage, children, inheritance and ownership fragmentation, nepotism mechanics,
career progression across a couple of fields (start with corporate and
academic/government), and richer succession rules.

## M4 — Economy, Resources, and Trade

Real production chains (raw to intermediate to finished goods), a market/pricing
model, trade routes and transport modes, and at least three distinct business
archetypes with genuinely different pressures.

## M5 — Politics and Institutions

Government-from-components in more depth, political positions, lobbying/favor
interactions, and the first non-currency influence mechanics (relationships,
ownership, media, debt).

## M6 — Events and Procedural History

The data-driven event framework, the first event packs (see the `content` area for
event-authoring issues), and procedural pre-player history generation.

## M7 — UI and Visualization

The map, the dynasty/character UI, economy and political visualization, and the
portrait pipeline (procedural/placeholder first; see `ASSETS.md`).

## M8 — Persistence, AI, and Developer Tools

Full save/autosave/migration support, rule-based NPC/organization decision making,
and the developer inspector tooling (state inspector, timeline, RNG/tick display,
invariant failure reporting).

## M9 — Performance, Accessibility, and Packaging

Profiling and scaling work for population-scale simulation, accessibility passes,
and packaging for browser, Linux, and Windows (Tauri, per ADR 0001, once there's a
reason to ship native).

## M10 — Content Expansion and Balancing

Additional event packs, additional business/career/culture content, and balancing
passes once enough of the loop exists to balance meaningfully.

## Sequencing notes

- Don't schedule months of engine work before anything playable exists: M1 is
  deliberately "thin but complete," touching every layer of the stack.
- Later milestones are not strict gates. An agent finding a clean opportunity to
  start M4/M5 work while M2/M3 are still filling in is fine as long as the
  dependency is real (check the issue's "Notes" section) and not just convenient.
