import { existsSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";

const root = process.cwd();
const pkgDir = join(root, "target", "site", "pkg");
const buildScript = readFileSync(join(root, "scripts", "build-wasm.mjs"), "utf8");
const outputName = buildScript.match(/const OUTPUT_NAME = "([^"]+)"/)?.[1];
assert.ok(outputName, "WASM output name should be declared by the build script");
const jsPath = join(pkgDir, `${outputName}.js`);
const wasmPath = join(pkgDir, `${outputName}_bg.wasm`);

rmSync(pkgDir, { recursive: true, force: true });

const npmCli = process.env.npm_execpath;
const command = npmCli ? process.execPath : "npm";
const args = npmCli ? [npmCli, "run", "build:wasm"] : ["run", "build:wasm"];
const result = spawnSync(command, args, {
  cwd: root,
  stdio: "inherit",
});

assert.equal(result.status, 0, "npm run build:wasm should exit successfully");
assert.ok(existsSync(jsPath), "WASM JS loader should be generated");
assert.ok(existsSync(wasmPath), "WASM binary should be generated");

const loader = readFileSync(jsPath, "utf8");
assert.match(loader, new RegExp(`${outputName}_bg\\.wasm`));
assert.match(loader, /export function hydrate/);
assert.match(loader, /as default/);
