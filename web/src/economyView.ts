import type { City, CitySpecialization, Sector } from "./simTypes";

/** Every business archetype the player can currently found, and the city
 * specialization each one requires. This is a UI-only convenience for
 * filtering the "host city" dropdown to cities that will actually accept
 * the archetype. It is not authoritative: `SimHandle.foundBusiness` (the
 * wasm bridge over `sim_core::business::found_business_by_city_id`) is the
 * real rule and still validates this server-side, so a mismatch here
 * only means a worse dropdown, never a wrong game outcome. Add a new
 * archetype's entry here when `sim-core` gains one. */
export const BUSINESS_ARCHETYPES: { value: string; requiredSpecialization: CitySpecialization }[] =
  [{ value: "Mining", requiredSpecialization: "Mining" }];

export function citiesEligibleForArchetype(
  sector: Sector,
  archetype: string,
): { id: number; name: string }[] {
  const required = BUSINESS_ARCHETYPES.find(
    (entry) => entry.value === archetype,
  )?.requiredSpecialization;
  if (!required) return [];

  const cities: City[] = sector.systems.flatMap((system) =>
    system.planets.flatMap((planet) => planet.countries.flatMap((country) => country.cities)),
  );
  return cities
    .filter((city) => city.specialization === required)
    .map((city) => ({ id: city.id, name: city.name }));
}
