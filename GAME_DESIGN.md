# Game design principles

"Sector Dynasties" is a placeholder codename. Rename freely once a better one
sticks; it appears in the repo name, package names, and a few doc titles, all of
which are mechanical find-and-replace.

## The core fantasy

You do not control a state or an army. You control the current head of a family.
The game is about building, preserving, transforming, or losing an intergenerational
dynasty inside a living economic, political, social, and corporate simulation. You
can start ordinary and, over generations, build wealth, reputation, political
influence, businesses, institutional control, and social status. A later generation
may inherit a powerful dynasty and squander it, split it through inheritance, rebuild
it, enter politics, change industries, or redirect the family entirely. The world
keeps moving whether or not you do.

Warfare is not a primary system and the game must stay compelling without it.

## Individuals vs. statistics

Most inhabitants are never individual characters; they're statistical population
groups (`sim_core::world::PopulationGroup`). Individual simulation is reserved for
the player's dynasty, people personally known to them, and people who become
independently notable (politicians, executives, major owners, scientists,
celebrities, institutional leaders, activists). A population member can be promoted
into a named character when they become relevant, and simplified back when they
stop being one, if that's technically useful.

## Government as components, not an enum

A country's government is described by independent institutional dimensions
(`GovernmentProfile`: federalism, franchise, economic liberalism, press freedom
today; more can be added) rather than one fixed type. The displayed government name
is derived from the components. This is what lets governments drift, reform, and
diverge gradually instead of jumping between fixed states.

## Culture as dimensions

Culture should be built from dimensions, traits, norms, and values, not just
flavor text, and it should feed real systems: family expectations, nepotism
tolerance, marriage, career prestige, entrepreneurship, political behavior,
consumption, migration, and attitudes toward institutions and technology. What
counts as normal family loyalty in one culture can be a nepotism scandal in
another. (Not yet modeled in the bootstrap slice; see the `culture` backlog area.)

## Economy

Systemic, not scripted. Countries and cities specialize based on resources,
infrastructure, education, capital, institutions, geography, and policy. Production
chains create real dependencies (ore to metal to machinery, for instance). Use
aggregated quantities and market flows, never per-unit cargo simulation. Prices
should react to supply and demand; shortages should propagate.

## Nepotism is a central, contextual mechanic

Giving a relative a position can buy loyalty, control, and family stability while
costing competence, reputation, morale, or institutional trust, and the size of
that trade-off depends on the country's and character's culture. Design nepotism
events and mechanics so the "right" call is genuinely contextual, not always good or
always bad.

## Politics without a magic influence currency

Political influence should come from legible sources: relationships, family ties,
employment, corporate power, media ownership, campaign support, reputation,
institutional positions, debt relationships, trade dependence, lobbying, public
opinion, unions, professional organizations, universities, banks, religious
institutions, and parties. A family can become extremely powerful without formally
controlling a country, by controlling enough of what a country depends on.

## Events tell stories that arise from state

The event system (not yet built) should be data-driven and contextual: character
traits/skills/job/employer/business/relationships/family/culture/country/city/
economy/politics/wealth/age/education/reputation/history/flags all matter. Choices
should have consequences, some immediate and some that surface years later. Prefer
events that arise from simulation state over repetitive random popups. Adding an
event pack should never require editing core simulation code; see
`docs/CONTENT_GUIDE.md`.

## Procedural history

The game should eventually generate a believable world with a history behind it
(old companies, past recessions, migration waves, founding families, mergers,
colonial histories, border changes, tech shifts, scandals) so the player feels like
they entered an existing world. Procedural history must stay reproducible from a
seed, same as everything else; see `docs/ARCHITECTURE.md`'s determinism section.

## Technology and structural change

Long campaigns should see real structural shifts (automation, AI, longevity,
genetic engineering, terraforming, new energy/propulsion, synthetic food, advanced
manufacturing, robotics, communications) that create and destroy industries. A
dynasty that dominated one industry for a century should sometimes have to adapt or
fall behind.

## Scope discipline

Favor systems that create emergent stories over systems that exist because other
grand strategy games have them. Model causes, not scripted outcomes, wherever
practical. Do not build every system to full depth immediately; the vertical slice
matters more than any one system's completeness. See `docs/ROADMAP.md`.

## Player-facing writing

Player-facing text (events, descriptions, tooltips, names, tutorials,
notifications, generated prose, localization) must never contain an em dash or a
semicolon. This is enforced by an automated copy lint
(`web/tests/copy-lint.test.ts`) over everything exported from `web/src/content/`.
See `docs/CONTENT_GUIDE.md`.
