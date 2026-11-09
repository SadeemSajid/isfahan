import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const rawArg = process.argv[2]?.trim().replace(/^v/, '');

if (!rawArg || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(rawArg)) {
	console.error('Usage: bun run version:bump <major.minor.patch>');
	console.error('Example: bun run version:bump 0.2.0');
	process.exit(1);
}

const root = resolve(import.meta.dir, '..');

// 1. Update package.json
const pkgPath = resolve(root, 'package.json');
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
pkg.version = rawArg;
writeFileSync(pkgPath, JSON.stringify(pkg, null, '\t') + '\n');

// 2. Update src-tauri/tauri.conf.json
const tauriConfPath = resolve(root, 'src-tauri/tauri.conf.json');
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = rawArg;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');

// 3. Update src-tauri/Cargo.toml
const cargoPath = resolve(root, 'src-tauri/Cargo.toml');
const cargoContent = readFileSync(cargoPath, 'utf8');
const updatedCargo = cargoContent.replace(
	/^version\s*=\s*"[^"]+"/m,
	`version = "${rawArg}"`
);
writeFileSync(cargoPath, updatedCargo);

console.log(`Updated version to v${rawArg} in:`);
console.log('  - package.json');
console.log('  - src-tauri/tauri.conf.json');
console.log('  - src-tauri/Cargo.toml');
console.log('\nTo cut a GitHub Release with Windows & macOS installers, run:');
console.log(`  git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml`);
console.log(`  git commit -m "chore(release): v${rawArg}"`);
console.log(`  git tag v${rawArg}`);
console.log(`  git push origin main --tags`);
