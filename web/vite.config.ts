/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  // Relative base so `dist/` is relocatable: it works served from a domain
  // root or from a subpath (e.g. a GitHub Pages project site at
  // `<user>.github.io/<repo>/`) without a build-time --base flag or any
  // server-side rewrite. See docs/DEPLOYMENT.md.
  base: "./",
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts", "tests/**/*.test.tsx", "tests/**/*.test.js"],
  },
});
