#!/usr/bin/env node
/* eslint-disable no-console */

import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const docsRoot = path.join(root, "docs");
const configPath = path.join(docsRoot, ".vitepress", "config.ts");

const config = await readFile(configPath, "utf8");
const links = [...config.matchAll(/\blink:\s*["'](\/[^"']*)["']/g)];

async function exists(target) {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}

async function resolves(link) {
  const clean = link.split(/[?#]/, 1)[0].replace(/^\/+|\/+$/g, "");
  const candidates =
    clean === ""
      ? [path.join(docsRoot, "index.md")]
      : [path.join(docsRoot, `${clean}.md`), path.join(docsRoot, clean, "index.md")];
  const results = await Promise.all(candidates.map(exists));
  return results.some(Boolean);
}

const missing = [];
for (const [, link] of links) {
  if (!(await resolves(link))) missing.push(link);
}

if (missing.length > 0) {
  console.error(`[docs] ${missing.length} nav/sidebar link(s) point to a missing page:`);
  for (const link of missing) console.error(`  - ${link}`);
  console.error("Each target must exist as <link>.md or <link>/index.md under docs/.");
  process.exit(1);
}

console.log(`[docs] ${links.length} nav/sidebar link(s) resolve.`);
