import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "src/wasm/**",
      // Tauri Linux packaging spike (issue #102): its own Cargo build
      // output, not app source. Holds generated JS (codegen assets, the
      // injected `__global-api-script.js`) and binary blobs that aren't
      // meant to be linted.
      "src-tauri/target/**",
      "src-tauri/gen/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    plugins: { "react-hooks": reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
);
