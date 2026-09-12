import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const __dirname = dirname(fileURLToPath(import.meta.url));
const styles = readFileSync(resolve(__dirname, "../src/styles.css"), "utf8");

function cssVariable(name) {
  const match = styles.match(new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{6})`));
  if (!match) throw new Error(`Missing CSS variable --${name}`);
  return match[1];
}

function rgb(hex) {
  return [
    Number.parseInt(hex.slice(1, 3), 16),
    Number.parseInt(hex.slice(3, 5), 16),
    Number.parseInt(hex.slice(5, 7), 16),
  ];
}

function linear(channel) {
  const normalized = channel / 255;
  return normalized <= 0.03928 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
}

function luminance(color) {
  return 0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2]);
}

function contrastRatio(foreground, background) {
  const a = luminance(rgb(foreground));
  const b = luminance(rgb(background));
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

function deuteranopia(color) {
  return [
    Math.round(0.367 * color[0] + 0.861 * color[1] - 0.228 * color[2]),
    Math.round(0.28 * color[0] + 0.673 * color[1] + 0.047 * color[2]),
    Math.round(-0.012 * color[0] + 0.043 * color[1] + 0.969 * color[2]),
  ].map((channel) => Math.min(255, Math.max(0, channel)));
}

function distance(a, b) {
  return Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
}

describe("accessibility palette", () => {
  it("meets WCAG AA contrast for main text surfaces and controls", () => {
    const pairs = [
      ["text", "page"],
      ["text", "surface"],
      ["text", "surface-muted"],
      ["text-muted", "page"],
      ["text-muted", "surface"],
      ["text", "surface-muted"],
      ["primary-text", "primary"],
    ];

    for (const [foreground, background] of pairs) {
      expect(
        contrastRatio(cssVariable(`color-${foreground}`), cssVariable(`color-${background}`)),
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("keeps stat accent colors distinguishable under deuteranopia simulation", () => {
    const accents = ["accent-population", "accent-city", "accent-system"].map((name) =>
      deuteranopia(rgb(cssVariable(`color-${name}`))),
    );

    for (let a = 0; a < accents.length; a += 1) {
      for (let b = a + 1; b < accents.length; b += 1) {
        expect(distance(accents[a], accents[b])).toBeGreaterThanOrEqual(60);
      }
    }
  });
});
