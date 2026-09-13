import { useMemo, useState } from "react";
import { copy } from "./content/copy";
import { searchGlobally, type SearchResult } from "./globalSearchLogic";
import type { DynastyMemberSummary, Sector } from "./simTypes";

interface GlobalSearchProps {
  sector: Sector;
  members: readonly DynastyMemberSummary[];
  onSelectCity: (cityId: number) => void;
  onSelectCharacter: (characterId: number) => void;
}

function resultDetail(result: SearchResult): string {
  if (result.kind === "city") {
    return copy.globalSearchCityLocation(result.planetName, result.countryName);
  }
  const roleLabel = result.role === "Head" ? copy.dynastyRoleHead : copy.dynastyRoleMember;
  const statusLabel = result.alive ? copy.dynastyStatusAlive : copy.dynastyStatusDeceased;
  return `${roleLabel}, ${statusLabel}`;
}

function resultKindLabel(result: SearchResult): string {
  return result.kind === "city"
    ? copy.globalSearchCityKindLabel
    : copy.globalSearchCharacterKindLabel;
}

/** A single search field that jumps to a matching character or city (issue
 * #88): matching itself lives in `globalSearchLogic.ts` so it can be unit
 * tested without rendering, matching this codebase's split between data
 * shaping and components (see `sectorBrowserLogic.ts` / `SectorBrowser.tsx`). */
export function GlobalSearch({
  sector,
  members,
  onSelectCity,
  onSelectCharacter,
}: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const trimmedQuery = query.trim();
  const results = useMemo(() => searchGlobally(sector, members, query), [sector, members, query]);

  const handleSelect = (result: SearchResult) => {
    if (result.kind === "city") {
      onSelectCity(result.id);
    } else {
      onSelectCharacter(result.id);
    }
    setQuery("");
  };

  return (
    <section className="global-search" aria-labelledby="global-search-heading">
      <label className="global-search-field" htmlFor="global-search-input">
        <span id="global-search-heading">{copy.globalSearchLabel}</span>
        <input
          id="global-search-input"
          type="search"
          value={query}
          placeholder={copy.globalSearchPlaceholder}
          onChange={(event) => setQuery(event.target.value)}
        />
      </label>
      {trimmedQuery.length > 0 &&
        (results.length > 0 ? (
          <ul className="global-search-results" aria-label={copy.globalSearchResultsLabel}>
            {results.map((result) => (
              <li key={`${result.kind}-${result.id}`}>
                <button
                  className="global-search-result"
                  type="button"
                  onClick={() => handleSelect(result)}
                >
                  <span className="global-search-result-kind">{resultKindLabel(result)}</span>
                  <span className="global-search-result-name">{result.name}</span>
                  <span className="global-search-result-detail">{resultDetail(result)}</span>
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="global-search-empty">{copy.globalSearchNoResults(trimmedQuery)}</p>
        ))}
    </section>
  );
}
