import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
const root = resolve(__dirname, '..');
const args = process.argv.slice(2);
const debug = args[0] === '--debug';
if (debug) args.shift();
const cli = join(dirname(require.resolve('@napi-rs/cli/package.json')), 'dist/cli.js');
for (const [program, options] of [
  [cli, ['build', '--platform', ...(debug ? [] : ['--release']), '--js', 'native.js', '--dts', 'native.d.ts', ...args]],
  [join(dirname(require.resolve('typescript/package.json')), 'bin/tsc'), ['-p', 'tsconfig.json']],
] as const) {
  const result = spawnSync(process.execPath, [program, ...options], { cwd: root, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
