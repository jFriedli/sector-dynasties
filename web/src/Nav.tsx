import { Coins, ScrollText, Sparkles, Users } from "lucide-react";
import { copy } from "./content/copy";

export type Screen = "galaxy" | "dynasty" | "economy" | "events";

interface NavProps {
  screen: Screen;
  onSelectScreen: (screen: Screen) => void;
  pendingEventCount: number;
}

const TABS: { screen: Screen; label: string; icon: typeof Sparkles }[] = [
  { screen: "galaxy", label: copy.navGalaxy, icon: Sparkles },
  { screen: "dynasty", label: copy.navDynasty, icon: Users },
  { screen: "economy", label: copy.navEconomy, icon: Coins },
  { screen: "events", label: copy.navEvents, icon: ScrollText },
];

/** The game's primary navigation: a small set of icon tabs rather than a
 * long scrolling dashboard page, so the app reads as a game with distinct
 * screens (galaxy, dynasty, economy, events) instead of a single report. */
export function Nav({ screen, onSelectScreen, pendingEventCount }: NavProps) {
  return (
    <nav className="app-nav" aria-label="Main">
      {TABS.map((tab) => {
        const Icon = tab.icon;
        const isActive = tab.screen === screen;
        return (
          <button
            key={tab.screen}
            type="button"
            className={isActive ? "nav-tab nav-tab--active" : "nav-tab"}
            aria-current={isActive ? "page" : undefined}
            onClick={() => onSelectScreen(tab.screen)}
          >
            <Icon size={20} aria-hidden="true" />
            <span>{tab.label}</span>
            {tab.screen === "events" && pendingEventCount > 0 && (
              <span className="nav-tab-badge">{pendingEventCount}</span>
            )}
          </button>
        );
      })}
    </nav>
  );
}
