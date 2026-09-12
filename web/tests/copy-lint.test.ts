import { describe, expect, it } from "vitest";
import { copy } from "../src/content/copy";

// Player facing text must never contain an em dash or a semicolon. See
// docs/CONTENT_GUIDE.md. This scans every string value exported from
// src/content so a future content pack only has to export its strings from
// under src/content to be covered automatically.
const FORBIDDEN = [
  { char: "—", label: "em dash" },
  { char: ";", label: "semicolon" },
];

function collectStrings(value: unknown, out: string[]) {
  if (typeof value === "string") {
    out.push(value);
  } else if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, out);
  } else if (value && typeof value === "object") {
    for (const item of Object.values(value)) collectStrings(item, out);
  }
}

describe("player-facing copy lint", () => {
  it("contains no em dash or semicolon in any exported string", () => {
    const strings: string[] = [];
    collectStrings(copy, strings);
    expect(strings.length).toBeGreaterThan(0);

    const offenders = strings.filter((s) => FORBIDDEN.some((f) => s.includes(f.char)));
    expect(offenders).toEqual([]);
  });
});
