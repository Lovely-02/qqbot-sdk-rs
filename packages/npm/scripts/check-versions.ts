import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export function checkVersions(root: string, tag?: string): string {
  const read = (file: string) => readFileSync(join(root, file), 'utf8');
  const json = (file: string) => JSON.parse(read(file));
  const packageVersion = (file: string) => {
    const section = read(file).split(/^\[package\]\s*$/m)[1]?.split(/^\[/m)[0];
    const version = section?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
    if (!version) throw new Error(`Cannot read package version in ${file}`);
    return version;
  };
  const version = read('VERSION').trim();
  const lock = json('packages/npm/package-lock.json');
  const versions: Record<string, string | undefined> = {
    'packages/rust/Cargo.toml': packageVersion('packages/rust/Cargo.toml'),
    'packages/rust/Cargo.toml core dependency': read('packages/rust/Cargo.toml')
      .match(/^qqbot-sdk-core\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/m)?.[1],
    'packages/core/Cargo.toml': packageVersion('packages/core/Cargo.toml'),
    'packages/npm/Cargo.toml': packageVersion('packages/npm/Cargo.toml'),
    'packages/npm/package.json': json('packages/npm/package.json').version,
    'packages/npm/package-lock.json': lock.version,
    'packages/npm/package-lock.json packages root': lock.packages[''].version,
    '.release-please-manifest.json': json('.release-please-manifest.json')['.'],
  };
  for (const [file, names] of [
    ['Cargo.lock', ['qqbot-sdk-core', 'qqbot-sdk-rs', 'qqbot-sdk-node']],
  ] as const) {
    const packages = read(file).split(/^\[\[package\]\]\s*$/m);
    for (const name of names) {
      const entry = packages.find((item) => item.match(/^name\s*=\s*"([^"]+)"/m)?.[1] === name);
      versions[`${file} ${name}`] = entry?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
    }
  }
  for (const [file, actual] of Object.entries(versions)) {
    if (actual !== version) throw new Error(`${file}: ${actual} differs from Rust SDK ${version}`);
  }
  const expectedTag = `qqbot-sdk-rs-v${version}`;
  if (tag && tag !== expectedTag) throw new Error(`Release tag ${tag} must be ${expectedTag}`);
  return version;
}

if (require.main === module) {
  console.log(`Unified Rust/npm version: ${checkVersions(join(__dirname, '../../..'), process.env.RELEASE_TAG)}`);
}
