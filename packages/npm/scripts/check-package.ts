import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
const root = join(__dirname, '..');
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
const required = ['dist/index.js', 'dist/index.d.ts', 'native.js', 'native.d.ts', 'README.md', 'LICENSE'];
for (const file of required) {
  if (!existsSync(join(root, file))) throw new Error(`Missing ${file}; run npm run build first`);
}
const binaries = readdirSync(root).filter((name) => name.endsWith('.node'));
if (binaries.length === 0) throw new Error('No native binaries; run npm run build first');
if (process.argv.includes('--all')) {
  for (const platform of ['win32-x64-msvc', 'linux-x64-gnu', 'linux-arm64-gnu', 'darwin-x64', 'darwin-arm64']) {
    const file = `qqbot-sdk.${platform}.node`;
    if (!binaries.includes(file)) throw new Error(`Missing release binary: ${file}`);
  }
}
if (process.env.RELEASE_TAG && process.env.RELEASE_TAG !== `qqbot-sdk-rs-v${pkg.version}`) {
  throw new Error(`Release tag ${process.env.RELEASE_TAG} does not match qqbot-sdk-rs-v${pkg.version}`);
}
console.log(`Package ${pkg.name}@${pkg.version}: ${binaries.length} native binaries verified`);
