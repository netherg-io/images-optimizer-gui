import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

const root = fileURLToPath(new URL('..', import.meta.url));
const versionFiles = [
  'package.json',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
  'src-tauri/tauri.conf.json',
];

export function nextVersion(version, kind) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match || !['patch', 'minor', 'major'].includes(kind)) {
    throw new Error('Usage: yarn release patch|minor|major');
  }

  let [, major, minor, patch] = match.map(Number);
  if (kind === 'patch') patch += 1;
  if (kind === 'minor') [minor, patch] = [minor + 1, 0];
  if (kind === 'major') [major, minor, patch] = [major + 1, 0, 0];
  return `${major}.${minor}.${patch}`;
}

function output(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: 'utf8' }).trim();
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: 'inherit' });
}

function release(kind) {
  if (output('git', ['status', '--porcelain'])) {
    throw new Error('Commit or discard local changes before releasing.');
  }
  if (output('git', ['branch', '--show-current']) !== 'main') {
    throw new Error('Releases must be created from main.');
  }

  run('git', ['fetch', 'origin', 'main', '--tags']);
  if (
    output('git', ['rev-parse', 'HEAD']) !==
    output('git', ['rev-parse', 'origin/main'])
  ) {
    throw new Error('Local main must match origin/main.');
  }

  const current = JSON.parse(
    fs.readFileSync(new URL('../package.json', import.meta.url)),
  ).version;
  const version = nextVersion(current, kind);
  const tag = `v${version}`;
  if (output('git', ['tag', '--list', tag]))
    throw new Error(`${tag} already exists.`);

  run(process.execPath, ['scripts/version.mjs', 'set', version]);
  run('cargo', ['check', '--manifest-path', 'src-tauri/Cargo.toml']);
  run(process.execPath, ['scripts/version.mjs', 'check']);
  run('git', ['add', ...versionFiles]);
  run('git', ['commit', '-m', `release: prepare ${version}`]);
  run('git', ['tag', '-a', tag, '-m', tag]);
  run('git', ['push', '--atomic', 'origin', 'main', tag]);
  console.log(`Release ${tag} started.`);
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    release(process.argv[2]);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
