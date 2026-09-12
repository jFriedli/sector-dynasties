import type {
  DynastyMemberSummary,
  RngDomainSummary,
  SimStateSnapshot,
  StateSummary,
} from "./simTypes";

// Data shaping for the state inspector dev panel (issue #93). Flattens the
// whole SimState hierarchy - systems, planets, countries, cities, dynasty
// members, and active RNG domain streams - into one searchable list of rows
// so a developer can find an arbitrary entity by type or name without
// reading raw JSON. Extends the debug overlay (issue #35) rather than being
// a wholly separate surface; see DebugOverlay.tsx / StateInspector.tsx.
//
// Dev-only tooling: never scanned by the copy lint (see docs/CONTENT_GUIDE.md)
// because nothing here is imported from src/content.

export type EntityType = "system" | "planet" | "country" | "city" | "dynasty_member" | "rng_domain";

export interface EntityRow {
  /** Unique within the row list: `${type}:${id}`, since ids are only unique
   * within their own entity type (a system and a city can share an id). */
  key: string;
  type: EntityType;
  id: string;
  name: string;
  /** Human-readable breadcrumb through the hierarchy, e.g.
   * "Aster > Aster Prime > North Compact", empty for top-level entities. */
  path: string;
  /** Extra detail shown alongside the name, kept short (a specialization,
   * a role, a fingerprint) rather than a full field dump. */
  detail: string;
}

/**
 * Flattens a `SimStateSnapshot` and its paired `StateSummary` into one list
 * of searchable entity rows, in a stable order: systems and their
 * descendants depth-first, then dynasty members, then RNG domains.
 */
export function entityRowsFromState(
  snapshot: SimStateSnapshot,
  summary: StateSummary,
): EntityRow[] {
  const rows: EntityRow[] = [];

  for (const system of snapshot.sector.systems) {
    rows.push({
      key: `system:${system.id}`,
      type: "system",
      id: String(system.id),
      name: system.name,
      path: "",
      detail: `${system.planets.length} planet(s)`,
    });

    for (const planet of system.planets) {
      rows.push({
        key: `planet:${planet.id}`,
        type: "planet",
        id: String(planet.id),
        name: planet.name,
        path: system.name,
        detail: planet.resource_tags.join(", "),
      });

      for (const country of planet.countries) {
        rows.push({
          key: `country:${country.id}`,
          type: "country",
          id: String(country.id),
          name: country.name,
          path: `${system.name} > ${planet.name}`,
          detail: `${country.cities.length} city/cities`,
        });

        for (const city of country.cities) {
          rows.push({
            key: `city:${city.id}`,
            type: "city",
            id: String(city.id),
            name: city.name,
            path: `${system.name} > ${planet.name} > ${country.name}`,
            detail: city.specialization,
          });
        }
      }
    }
  }

  for (const member of summary.dynasty_members) {
    rows.push(dynastyMemberRow(member));
  }

  for (const domain of summary.rng_domains) {
    rows.push(rngDomainRow(domain));
  }

  return rows;
}

function dynastyMemberRow(member: DynastyMemberSummary): EntityRow {
  return {
    key: `dynasty_member:${member.id}`,
    type: "dynasty_member",
    id: String(member.id),
    name: member.name,
    path: "",
    detail: `${member.role}, age ${member.age_years}${member.alive ? "" : " (deceased)"}`,
  };
}

function rngDomainRow(domain: RngDomainSummary): EntityRow {
  return {
    key: `rng_domain:${domain.domain}`,
    type: "rng_domain",
    id: domain.domain,
    name: domain.domain,
    path: "",
    detail: domain.fingerprint,
  };
}

/** Every entity type, in the display order used by the type filter dropdown. */
export const ENTITY_TYPES: readonly EntityType[] = [
  "system",
  "planet",
  "country",
  "city",
  "dynasty_member",
  "rng_domain",
];

/** Short, stable label for an entity type, used in the type filter and row list. */
export function entityTypeLabel(type: EntityType): string {
  switch (type) {
    case "system":
      return "System";
    case "planet":
      return "Planet";
    case "country":
      return "Country";
    case "city":
      return "City";
    case "dynasty_member":
      return "Dynasty member";
    case "rng_domain":
      return "RNG domain";
  }
}

/**
 * Filters rows by a case-insensitive name substring match and/or an entity
 * type. Either filter may be empty/null to mean "no restriction". A plain
 * array scan is intentional: the bootstrap slice's data sizes (dozens of
 * rows, not thousands) don't justify a search index.
 */
export function filterEntityRows(
  rows: readonly EntityRow[],
  query: string,
  type: EntityType | null,
): EntityRow[] {
  const needle = query.trim().toLowerCase();
  return rows.filter((row) => {
    if (type !== null && row.type !== type) return false;
    if (needle.length > 0 && !row.name.toLowerCase().includes(needle)) return false;
    return true;
  });
}
