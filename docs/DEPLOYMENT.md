# Deploying the static build

`web/dist/` is a fully static, relocatable site: no server-side rendering,
no API, and no runtime dependency beyond a plain HTTP file server. This
covers the M9 browser-packaging goal in `docs/ROADMAP.md`.

## Build it

```bash
cd web
npm install
npm run build:wasm   # generates web/src/wasm/, gitignored
npm run build         # tsc -b && vite build -> web/dist/
```

`web/vite.config.ts` sets `base: "./"`, so every asset URL in
`dist/index.html` (and every chunk Vite emits) is relative rather than
rooted at `/`. That makes the same `dist/` output work whether it's served
from a domain root or from a subpath, such as a GitHub Pages project site
at `https://<user>.github.io/<repo>/`. No `--base` flag or build-time
knowledge of the final URL is required.

The WASM bridge (`web/src/wasm/sim_wasm_bg.wasm`) is fetched by the
generated bindgen glue via `new URL("sim_wasm_bg-*.wasm", import.meta.url)`,
which resolves relative to the JS module's own URL at runtime. That is
relocatable by construction and needs no extra configuration.

## Deploy anywhere that serves static files

Copy the contents of `web/dist/` to any static host: GitHub Pages, an S3
bucket with static website hosting, Netlify/Vercel's static output, or a
plain `nginx`/Apache document root. There is nothing to configure server
side beyond serving files (no rewrites, no headers, no API routes).

### GitHub Pages example

1. Build (`npm run build:wasm && npm run build`).
2. Publish `web/dist/` as the Pages source, e.g. with
   `actions/upload-pages-artifact` + `actions/deploy-pages` in a workflow,
   or by pushing `web/dist/`'s contents to a `gh-pages` branch.
3. The site works at `https://<user>.github.io/<repo>/` (a subpath) without
   any further changes, because of the relative base above.

## Verify locally before deploying

Because the dev server (`npm run dev`) always serves from its own root,
subpath relocatability has to be checked against the built output, not the
dev server. A quick local check:

```bash
npm run build:wasm && npm run build
mkdir -p /tmp/static-check/some-subpath
cp -r dist/* /tmp/static-check/some-subpath/
npx serve /tmp/static-check -l 4173
# then open http://localhost:4173/some-subpath/ and confirm the app loads
```

If the app renders sector/city data (not a blank page or 404s in the
console) when loaded from a nested path like this, the build is
relocatable.
