import { cityDetailsFromSector } from "./sectorBrowserLogic";
import type { DynastyMemberSummary, DynastyRole, Sector } from "./simTypes";

export interface CharacterSearchResult {
  kind: "character";
  id: number;
  name: string;
  role: DynastyRole;
  alive: boolean;
}

export interface CitySearchResult {
  kind: "city";
  id: number;
  name: string;
  planetName: string;
  countryName: string;
}

export type SearchResult = CharacterSearchResult | CitySearchResult;

/** How closely a result's name matches the query: an exact match ranks
 * ahead of a prefix match, which ranks ahead of any other substring match.
 * Keeps the most likely target near the top without any indexing
 * infrastructure, which the bootstrap slice's data sizes don't need yet
 * (issue #88). */
function matchRank(name: string, normalizedQuery: string): number {
  const normalizedName = name.toLowerCase();
  if (normalizedName === normalizedQuery) return 0;
  if (normalizedName.startsWith(normalizedQuery)) return 1;
  return 2;
}

/** Searches dynasty members and sector cities by name (issue #88). A
 * case-insensitive substring match is enough for the bootstrap slice's
 * data sizes, so this stays a plain linear scan rather than building any
 * search index. Blank or whitespace-only queries return no results, so
 * an empty search field doesn't show every character and city at once. */
export function searchGlobally(
  sector: Sector,
  members: readonly DynastyMemberSummary[],
  query: string,
): SearchResult[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) return [];

  const characterResults: CharacterSearchResult[] = members
    .filter((member) => member.name.toLowerCase().includes(normalizedQuery))
    .map((member) => ({
      kind: "character",
      id: member.id,
      name: member.name,
      role: member.role,
      alive: member.alive,
    }));

  const cityResults: CitySearchResult[] = cityDetailsFromSector(sector)
    .filter((city) => city.name.toLowerCase().includes(normalizedQuery))
    .map((city) => ({
      kind: "city",
      id: city.id,
      name: city.name,
      planetName: city.planetName,
      countryName: city.countryName,
    }));

  return [...characterResults, ...cityResults].sort(
    (a, b) => matchRank(a.name, normalizedQuery) - matchRank(b.name, normalizedQuery),
  );
}
