import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const versionFile = path.join(root, 'target/site-dev/pkg/inspace-dev-version.txt');
const databaseUrl = process.env.DATABASE_URL
  || 'postgres://instant_space:instant_space_pass@127.0.0.1:5432/instant_space_rust';
const jobs = process.env.CARGO_BUILD_JOBS || '3';
const rustSysroot = spawnSync('rustc', ['--print', 'sysroot'], { encoding: 'utf8' }).stdout.trim();
const lldDir = path.join(rustSysroot, 'lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld');
let server = null;
let stopping = false;

const rustRoots = [
  'app/src',
  'crates/auth/src',
  'crates/db/src',
  'crates/domain/src',
  'crates/map-ui/src',
];
const staticRoots = ['app/style', 'app/vendor'];
const manifests = ['Cargo.toml', 'Cargo.lock', 'app/Cargo.toml'];

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: root, stdio: 'inherit', ...options });
    child.on('exit', (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} exited with ${code ?? signal}`));
    });
  });
}

function walk(relative, files = new Map()) {
  const absolute = path.join(root, relative);
  if (!fs.existsSync(absolute)) return files;
  const stat = fs.statSync(absolute);
  if (stat.isFile()) {
    files.set(relative, `${stat.mtimeMs}:${stat.size}`);
    return files;
  }
  for (const entry of fs.readdirSync(absolute, { withFileTypes: true })) {
    if (entry.name.startsWith('.') || entry.name === 'target') continue;
    walk(path.join(relative, entry.name), files);
  }
  return files;
}

function snapshot() {
  const files = new Map();
  for (const item of [...rustRoots, ...staticRoots, ...manifests]) walk(item, files);
  return files;
}

function changes(before, after) {
  const changed = [];
  for (const [file, stamp] of after) if (before.get(file) !== stamp) changed.push(file);
  for (const file of before.keys()) if (!after.has(file)) changed.push(file);
  return changed;
}

function buildScope(changed) {
  if (changed.every((file) => staticRoots.some((dir) => file.startsWith(`${dir}/`)))) {
    return 'static';
  }
  if (changed.every((file) => (
    file.endsWith('.js')
    || file === 'app/src/main.rs'
    || file.startsWith('app/src/server/')
    || file.startsWith('crates/auth/src/')
    || file.startsWith('crates/db/src/')
  ))) {
    return 'server';
  }
  return 'full';
}

async function buildWasm() {
  console.log('\n[dev] 构建浏览器端（debug 增量）');
  await run('node', ['scripts/build-wasm.mjs', '--dev'], {
    env: { ...process.env, CARGO_BUILD_JOBS: jobs },
  });
}

async function buildServer() {
  console.log('\n[dev] 构建服务端（debug 增量）');
  await run('cargo', ['build', '--profile', 'dev-fast', '-p', 'instant-space-app', '--bin', 'instant-space-app'], {
    env: {
      ...process.env,
      CARGO_BUILD_JOBS: jobs,
      PATH: `${lldDir}:${process.env.PATH}`,
      RUSTFLAGS: `${process.env.RUSTFLAGS || ''} -C link-arg=-fuse-ld=lld`.trim(),
    },
  });
}

function startServer() {
  server = spawn(path.join(root, 'target/dev-fast/instant-space-app'), [], {
    cwd: root,
    stdio: 'inherit',
    env: {
      ...process.env,
      DATABASE_URL: databaseUrl,
      INSTANT_SPACE_ADDR: '127.0.0.1:3001',
      INSTANT_SPACE_SITE_ROOT: 'target/site-dev',
      INSPACE_DEV_RELOAD: '1',
      RUST_LOG: process.env.RUST_LOG || 'info',
    },
  });
  server.on('exit', (code) => {
    if (!stopping && code !== null) console.error(`[dev] 服务退出：${code}`);
  });
}

async function restartServer() {
  if (server && server.exitCode === null) {
    server.kill('SIGTERM');
    await new Promise((resolve) => server.once('exit', resolve));
  }
  startServer();
}

function notifyBrowser() {
  fs.mkdirSync(path.dirname(versionFile), { recursive: true });
  fs.writeFileSync(versionFile, `${Date.now()}\n`);
}

async function stop() {
  if (stopping) return;
  stopping = true;
  console.log('\n[dev] 退出开发模式，恢复正式服务…');
  if (server && server.exitCode === null) server.kill('SIGTERM');
  if (!process.env.INSPACE_DEV_MANAGED) {
    spawnSync('systemctl', ['start', 'instant-space-rust.service'], { stdio: 'inherit' });
  }
  process.exit(0);
}

process.on('SIGINT', stop);
process.on('SIGTERM', stop);

console.log('[dev] 首次准备 debug 增量产物；完成后接管 http://127.0.0.1:3001');
try {
  await buildWasm();
  await buildServer();
} catch (error) {
  console.error(`[dev] 首次构建失败，正式服务保持不变：${error.message}`);
  process.exit(1);
}
notifyBrowser();
spawnSync('systemctl', ['stop', 'instant-space-rust.service'], { stdio: 'inherit' });
startServer();
console.log('[dev] 已启动：http://127.0.0.1:3001（线上反代也会看到开发版本）');
console.log('[dev] CSS/图片直接刷新；Rust/内联 JS 增量构建。Ctrl+C 恢复正式服务。');

let previous = snapshot();
while (!stopping) {
  await new Promise((resolve) => setTimeout(resolve, 800));
  const next = snapshot();
  const changed = changes(previous, next);
  previous = next;
  if (!changed.length) continue;

  const scope = buildScope(changed);
  console.log(`\n[dev] 发现变更：${changed.slice(0, 6).join(', ')}${changed.length > 6 ? '…' : ''}`);
  try {
    if (scope !== 'static') {
      if (scope === 'full') await buildWasm();
      await buildServer();
      await restartServer();
    }
    notifyBrowser();
    console.log(scope === 'static' ? '[dev] 静态资源已刷新' : '[dev] 增量构建完成，浏览器正在刷新');
  } catch (error) {
    console.error(`[dev] 构建失败，继续保留上一个可运行版本：${error.message}`);
  }
}
