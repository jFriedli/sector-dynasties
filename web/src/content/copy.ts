// Player-facing text lives here, separate from component logic, so the
// copy lint (tests/copy-lint.test.ts) can scan every string in one place
// and so a future content pipeline can localize or data-drive it without
// touching component code. See docs/CONTENT_GUIDE.md.
//
// Rule: no em dash, no semicolon, anywhere in these values.

export const copy = {
  title: "Sector Dynasties",
  subtitle: "A dynasty grows one generation at a time.",
  advanceOneYear: "Advance one year",
  yearLabel: "Year",
  populationLabel: "Population under your influence",
  wealthLabel: "Dynasty wealth",
  citiesLabel: "Cities",
  systemsLabel: "Star systems",
  loading: "Generating the sector...",
};
