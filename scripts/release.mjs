#!/usr/bin/env node
/* eslint-disable no-console */

import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));

const packageJsonPath = path.join(root, "package.json");
const tauriConfigPath = path.join(root, "src-tauri", "tauri.conf.json");

const args = process.argv.slice(2);
const checkOnly = args.includes("--check");
const rawArg = args.find((a) => a !== "--check");

if (!checkOnly && !rawArg) {
  console.error("Usage: bun run scripts/release.mjs <version> (e.g. 0.2.0)");
  console.error("       bun run scripts/release.mjs --check");
  process.exit(1);
}

const version = rawArg ? rawArg.replace(/^v/, "") : null;

if (version !== null && !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`Invalid semver version: "${version}"`);
  process.exit(1);
}

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

async function writeJson(filePath, data) {
  await writeFile(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

const packageJson = await readJson(packageJsonPath);
const tauriConfig = await readJson(tauriConfigPath);

if (checkOnly) {
  if (packageJson.version !== tauriConfig.version) {
    console.error(
      `Version drift: package.json=${packageJson.version}, tauri.conf.json=${tauriConfig.version}. ` +
        "Run `bun run release:version <version>` to sync.",
    );
    process.exit(1);
  }
  console.log(`Versions in sync: ${packageJson.version}`);
  process.exit(0);
}

packageJson.version = version;
tauriConfig.version = version;

await writeJson(packageJsonPath, packageJson);
await writeJson(tauriConfigPath, tauriConfig);

if (packageJson.version !== tauriConfig.version) {
  console.error(
    `Version drift after sync: package.json=${packageJson.version}, tauri.conf.json=${tauriConfig.version}`,
  );
  process.exit(1);
}

console.log(`Version synced to ${version} (package.json + tauri.conf.json)`);
