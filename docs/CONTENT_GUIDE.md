# Content guide

For agents writing events, copy, or other data-driven content, not engine code.

## Player-facing writing rule

Player-facing text (event text, descriptions, tooltips, names where relevant,
tutorials, notifications, generated prose, and localization content) must never
contain:

- An em dash (`—`).
- A semicolon (`;`).

This does not apply to punctuation required inside source code. Use a period, a
comma, "and," or two shorter sentences instead. This is enforced automatically:
`web/tests/copy-lint.test.ts` scans every string exported from `web/src/content/`
and fails the build on a violation. Put new player-facing strings there (or in a
new module under that directory) so the lint covers them without extra wiring.

Prefer concise natural language. Do not fill the UI with lore dumps. A tooltip is
not the place for three paragraphs of backstory.

## Where content will live

`web/src/content/` today holds one small module (`copy.ts`) for the bootstrap
slice's UI strings. As the event system (M6, see `docs/ROADMAP.md`) lands, event
packs should live as structured, schema-validated data (JSON or a small typed
format) under a dedicated content directory, separate from the code that
interprets them, per the project's moddability goal. That directory and schema
don't exist yet; the first event-framework issue should establish them; don't
invent an ad hoc format in an unrelated PR.

## Writing events (once the framework lands)

Events should be contextual: driven by character traits/skills/job/employer/
business/relationships/family/culture/country/city/economic and political
conditions/industry/wealth/age/education/reputation/history/flags, not by a fixed
schedule. Prefer events that arise from simulation state over repetitive random
popups. Choices should have consequences, some immediate and some delayed. See
`GAME_DESIGN.md`'s "Events tell stories that arise from state" section for the
design intent, and look for `content`-labeled event-pack issues for scoped,
self-contained authoring work (for example: corporate career events, mining
industry events, succession events, nepotism events, migration events).

## Tone

Sector Dynasties is not a joke game, but it doesn't need to be grim either. Write
like a sharp, economical narrator, not a tutorial. Avoid explaining mechanics in
prose the player can see for themselves in the UI.
