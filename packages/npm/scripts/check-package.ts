import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const root = join(__dirname, '..');
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
const platforms = ['win32-x64-msvc', 'linux-x64-gnu', 'linux-arm64-gnu', 'darwin-x64', 'darwin-arm64'];
const packRelease = process.argv.includes('--pack');
const required = ['dist/index.js', 'dist/index.d.ts', 'native.js', 'native.d.ts', 'README.md', 'LICENSE'];
for (const file of required) {
  if (!existsSync(join(root, file))) throw new Error(`Missing ${file}; run npm run build first`);
}
const binaries = readdirSync(root).filter((name) => name.endsWith('.node'));
if (binaries.length === 0) throw new Error('No native binaries; run npm run build first');
if (packRelease || process.argv.includes('--all')) {
  for (const platform of platforms) {
    const file = `qqbot-sdk.${platform}.node`;
    if (!binaries.includes(file)) throw new Error(`Missing release binary: ${file}`);
  }
}
if (process.env.RELEASE_TAG && ![`qqbot-sdk-rs-v${pkg.version}`, `github-packages-v${pkg.version}`].includes(process.env.RELEASE_TAG)) {
  throw new Error(`Release tag ${process.env.RELEASE_TAG} does not match qqbot-sdk-rs-v${pkg.version}`);
}
console.log(`Package ${pkg.name}@${pkg.version}: ${binaries.length} native binaries verified`);

if (packRelease) {
  const output = join(root, 'release-packages');
  const staging = mkdtempSync(join(tmpdir(), 'qqbot-sdk-npm-'));
  const { name, version, license, repository, homepage, bugs, engines, publishConfig } = pkg;
  const metadata = { version, license, repository, homepage, bugs, engines, publishConfig };
  const optionalDependencies = Object.fromEntries(platforms.map((platform) => [`${name}-${platform}`, version]));

  function pack(directory: string, manifest: object) {
    const npmCli = process.env.npm_execpath;
    if (!npmCli) throw new Error('Run npm run pack:release');
    writeFileSync(join(directory, 'package.json'), `${JSON.stringify(manifest, null, 2)}\n`);
    execFileSync(process.execPath, [npmCli, 'pack', '--ignore-scripts', '--pack-destination', output], {
      cwd: directory,
      stdio: 'inherit',
    });
  }

  try {
    mkdirSync(output, { recursive: true });
    for (const platform of platforms) {
      const [os, cpu, abi] = platform.split('-');
      const binary = `qqbot-sdk.${platform}.node`;
      const directory = join(staging, platform);
      mkdirSync(directory);
      cpSync(join(root, binary), join(directory, binary));
      cpSync(join(root, 'LICENSE'), join(directory, 'LICENSE'));
      pack(directory, {
        ...metadata,
        name: `${name}-${platform}`,
        description: `${pkg.description} for ${platform}`,
        main: binary,
        files: [binary],
        os: [os],
        cpu: [cpu],
        ...(abi === 'gnu' ? { libc: ['glibc'] } : {}),
      });
    }

    const main = join(staging, 'main');
    mkdirSync(main);
    const files = (pkg.files as string[]).filter((file) => file !== '*.node');
    for (const file of files) {
      if (existsSync(join(root, file))) cpSync(join(root, file), join(main, file), { recursive: true });
    }
    pack(main, {
      ...metadata,
      name,
      description: pkg.description,
      main: pkg.main,
      types: pkg.types,
      files,
      optionalDependencies,
    });
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}
