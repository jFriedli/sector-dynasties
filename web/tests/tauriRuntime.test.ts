import { afterEach, describe, expect, it } from "vitest";
import { isTauriRuntime } from "../src/tauriRuntime";

describe("isTauriRuntime", () => {
  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("is false in a plain browser/jsdom window", () => {
    expect(isTauriRuntime()).toBe(false);
  });

  it("is true once Tauri's webview global is present", () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};

    expect(isTauriRuntime()).toBe(true);
  });
});
