import { access, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const REFERENCE_PATTERN = /\b(?:href|src)=["']([^"']+)["']/gu;
const EXTERNAL_REFERENCE_PATTERN = /^(?:[a-z][a-z\d+.-]*:|\/\/|#)/iu;

export function assetReferences(html) {
  return [...html.matchAll(REFERENCE_PATTERN)].map((match) => match[1]);
}

export async function verifyStaticBuild(distDirectory) {
  const indexPath = path.join(distDirectory, "index.html");
  const html = await readFile(indexPath, "utf8");
  const references = assetReferences(html);
  const localReferences = references.filter(
    (reference) => !EXTERNAL_REFERENCE_PATTERN.test(reference),
  );

  if (localReferences.length === 0) {
    throw new Error("dist/index.html does not reference any local assets");
  }

  for (const reference of localReferences) {
    if (reference.startsWith("/")) {
      throw new Error(`root-relative asset reference is not relocatable: ${reference}`);
    }

    const assetPath = reference.split(/[?#]/u, 1)[0];
    const resolvedPath = path.resolve(distDirectory, assetPath);
    const relativePath = path.relative(distDirectory, resolvedPath);

    if (relativePath.startsWith("..") || path.isAbsolute(relativePath)) {
      throw new Error(`asset reference escapes dist: ${reference}`);
    }

    await access(resolvedPath);
  }

  return localReferences;
}

const scriptPath = process.argv[1] ? path.resolve(process.argv[1]) : null;

if (scriptPath === fileURLToPath(import.meta.url)) {
  const distDirectory = path.resolve(process.cwd(), "dist");
  const references = await verifyStaticBuild(distDirectory);
  process.stdout.write(`Verified ${references.length} relocatable static asset references.\n`);
}
