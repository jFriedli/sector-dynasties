import {
  Factory,
  FlaskConical,
  Landmark,
  type LucideIcon,
  Mountain,
  Pickaxe,
  Truck,
  Wheat,
} from "lucide-react";
import type { CitySpecialization, ResourceTag } from "./simTypes";

/** One icon per city specialization, shared by every screen that shows a
 * city (the galaxy browser, the city detail panel, the economy screen) so
 * a specialization always reads as the same shape and color everywhere. */
const SPECIALIZATION_ICONS: Record<CitySpecialization, LucideIcon> = {
  Mining: Pickaxe,
  Manufacturing: Factory,
  Finance: Landmark,
  Research: FlaskConical,
  Logistics: Truck,
};

export function SpecializationIcon({
  specialization,
  size = 18,
}: {
  specialization: CitySpecialization;
  size?: number;
}) {
  const Icon = SPECIALIZATION_ICONS[specialization];
  return <Icon size={size} aria-hidden="true" />;
}

const RESOURCE_TAG_ICONS: Record<ResourceTag, LucideIcon> = {
  MetalRich: Pickaxe,
  Agricultural: Wheat,
  Arid: Mountain,
};

export function ResourceTagIcon({ tag, size = 16 }: { tag: ResourceTag; size?: number }) {
  const Icon = RESOURCE_TAG_ICONS[tag];
  return <Icon size={size} aria-hidden="true" />;
}
