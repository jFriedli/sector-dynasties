import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { assetReferences, verifyStaticBuild } from "../scripts/verify-static-build.mjs";

const temporaryDirectories = [];

async function fixture(indexHtml, assets = []) {
  const directory = await mkdtemp(path.join(tmpdir(), "sector-dynasties-static-"));
  temporaryDirectories.push(directory);
  await writeFile(path.join(directory, "index.html"), indexHtml);

  for (const asset of assets) {
    const assetPath = path.join(directory, asset);
    await mkdir(path.dirname(assetPath), { recursive: true });
    await writeFile(assetPath, "fixture");
  }

  return directory;
}

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

describe("static build verification", () => {
  it("finds script and stylesheet references", () => {
    expect(
      assetReferences(
        '<link rel="stylesheet" href="./assets/app.css"><script src="./assets/app.js"></script>',
      ),
    ).toEqual(["./assets/app.css", "./assets/app.js"]);
  });

  it("accepts a bundle whose referenced assets stay inside dist", async () => {
    const directory = await fixture('<script type="module" src="./assets/app.js"></script>', [
      "assets/app.js",
    ]);

    await expect(verifyStaticBuild(directory)).resolves.toEqual(["./assets/app.js"]);
  });

  it("rejects root-relative assets that break under a subpath", async () => {
    const directory = await fixture('<script type="module" src="/assets/app.js"></script>', [
      "assets/app.js",
    ]);

    await expect(verifyStaticBuild(directory)).rejects.toThrow(
      "root-relative asset reference is not relocatable",
    );
  });

  it("rejects missing build assets", async () => {
    const directory = await fixture('<script type="module" src="./assets/missing.js"></script>');

    await expect(verifyStaticBuild(directory)).rejects.toThrow();
  });
});
