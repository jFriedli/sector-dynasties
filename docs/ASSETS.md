# Assets: licensing and workflow

## Rules

- Agents may source icons, textures, and images when licensing clearly permits
  redistribution in this project (MIT-licensed code, public repository).
- Prefer permissively licensed assets (CC0, CC-BY with attribution, MIT/Apache-2.0
  for any bundled code-like assets such as fonts).
- Record license and attribution for every non-original asset in
  `ASSET_CREDITS.md` at the repo root (create it in the same PR as the first
  imported asset). Include: file path, source URL, license, and attribution text
  if the license requires it.
- Never hotlink production assets from an external URL. Download and commit them
  (or generate them proceduraly) so the game works offline and isn't at the mercy
  of a third party removing a file.
- Do not commit large or dubiously-licensed binaries "temporarily." If in doubt,
  don't commit it.

## When an asset can't be sourced

If a needed asset cannot legally or practically be obtained or generated, open an
issue labeled `needs-user-asset` that states exactly what is needed (dimensions,
format, style direction, where it should be placed in the repo) rather than
inventing a placeholder that looks finished. Do not block unrelated development on
this; use a clearly-labeled procedural or placeholder asset (a solid-color
rectangle, a generated shape) in the meantime and reference the issue in a code
comment near where the placeholder is used.

## Character portraits

The simulation should store character appearance as descriptors (a portrait
abstraction), not depend directly on one image generation provider or a single
hardcoded image path. This lets the eventual implementation be procedural/layered,
a pre-generated library, or externally generated and cached, without changing
`sim-core`. The core simulation must remain playable offline; it must not require
an online AI image service at runtime. See the portrait prototype and portrait
pipeline issues under the `portraits` label.

## Fonts and UI assets

Same rules as above. Prefer well-known open licenses (SIL OFL for fonts, for
example) and record them in `ASSET_CREDITS.md`.
