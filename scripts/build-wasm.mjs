import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const pkgDir = join(root, "target", "site", "pkg");
const wasmInput = join(
  root,
  "target",
  "wasm32-unknown-unknown",
  "release",
  "instant_space_web.wasm",
);

function executable(name) {
  return process.platform === "win32" ? `${name}.exe` : name;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    ...options,
  });

  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed`);
  }
}

function output(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
  });

  if (result.status !== 0) {
    return null;
  }

  return result.stdout.trim();
}

function lockedWasmBindgenVersion() {
  const lock = readFileSync(join(root, "Cargo.lock"), "utf8");
  const match = lock.match(
    /\[\[package\]\]\r?\nname = "wasm-bindgen"\r?\nversion = "([^"]+)"/,
  );
  if (!match) {
    throw new Error("Could not find wasm-bindgen version in Cargo.lock");
  }
  return match[1];
}

function wasmBindgenCommand() {
  const executable = process.platform === "win32" ? "wasm-bindgen.exe" : "wasm-bindgen";
  const cargoHome = process.env.CARGO_HOME ?? join(homedir(), ".cargo");
  return join(cargoHome, "bin", executable);
}

function ensureWasmBindgen() {
  const version = lockedWasmBindgenVersion();
  const command = wasmBindgenCommand();
  const current = existsSync(command) ? output(command, ["--version"]) : null;

  if (current?.includes(version)) {
    return command;
  }

  run(executable("cargo"), ["install", "wasm-bindgen-cli", "--version", version, "--locked"]);
  return command;
}

run(executable("rustup"), ["target", "add", "wasm32-unknown-unknown"]);
run(executable("cargo"), [
  "build",
  "-p",
  "instant-space-app",
  "--lib",
  "--release",
  "--target",
  "wasm32-unknown-unknown",
  "--no-default-features",
  "--features",
  "hydrate",
]);

rmSync(pkgDir, { recursive: true, force: true });
mkdirSync(pkgDir, { recursive: true });
mkdirSync(dirname(wasmInput), { recursive: true });

const wasmBindgen = ensureWasmBindgen();
run(wasmBindgen, [
  wasmInput,
  "--out-dir",
  pkgDir,
  "--target",
  "web",
  "--out-name",
  "instant_space_app",
  "--no-typescript",
]);

const jsOutput = join(pkgDir, "instant_space_app.js");
let js = readFileSync(jsOutput, "utf8");
const shimImport = js.match(
  /from ['"](\.\/snippets\/instant-map-ui-[^'"]+\/src\/maplibre_shim\.js)['"]/,
);

if (shimImport) {
  const shimPath = join(pkgDir, shimImport[1]);
  const shimHash = createHash("sha256")
    .update(readFileSync(shimPath))
    .digest("hex")
    .slice(0, 12);

  js = js.replaceAll(`${shimImport[1]}'`, `${shimImport[1]}?v=${shimHash}'`);
  js = js.replaceAll(`${shimImport[1]}"`, `${shimImport[1]}?v=${shimHash}"`);
  writeFileSync(jsOutput, js);
}
