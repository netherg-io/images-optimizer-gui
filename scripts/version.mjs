import fs from 'node:fs';

const packagePath = new URL('../package.json', import.meta.url);
const cargoPath = new URL('../src-tauri/Cargo.toml', import.meta.url);
const tauriPath = new URL('../src-tauri/tauri.conf.json', import.meta.url);
const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
const tauriJson = JSON.parse(fs.readFileSync(tauriPath, 'utf8'));
const cargo = fs.readFileSync(cargoPath, 'utf8');
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1];

if (process.argv[2] === 'check') {
  const versions = [packageJson.version, tauriJson.version, cargoVersion];
  if (new Set(versions).size !== 1) {
    throw new Error(
      `Version mismatch: package=${versions[0]}, tauri=${versions[1]}, cargo=${versions[2]}`,
    );
  }
  console.log(versions[0]);
} else if (process.argv[2] === 'set') {
  const version = process.argv[3];
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version || '')) {
    throw new Error('Usage: yarn version:set <semver>');
  }
  packageJson.version = version;
  tauriJson.version = version;
  fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);
  fs.writeFileSync(tauriPath, `${JSON.stringify(tauriJson, null, 2)}\n`);
  fs.writeFileSync(
    cargoPath,
    cargo.replace(/^version = "[^"]+"/m, `version = "${version}"`),
  );
  console.log(version);
} else {
  throw new Error('Use check or set');
}
