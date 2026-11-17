import assert from 'node:assert/strict';
import { createPrivateKey, sign, verify, createPublicKey } from 'node:crypto';
import { Bot } from '../src/index';
import { NativeBot } from '../native.js';
import { createApi } from '../src/api';

async function main() {
  const calls: unknown[] = [];
  const api = createApi(async (operation, args) => { calls.push([operation, args]); return null; });
  await api.groups.batchRemoveMembers('group', { member_openids: ['member'] });
  await api.messages.deleteC2c('user', 'message');
  await api.guilds.membersPage('guild', undefined, 20);
  assert.deepEqual(calls, [
    ['groups.batchRemoveMembers', ['group', { member_openids: ['member'] }]],
    ['messages.deleteC2c', ['user', 'message']],
    ['guilds.membersPage', ['guild', null, 20]],
  ]);
  // Every generated JS method must have a matching native dispatch arm.
  const { readFileSync } = await import('node:fs');
  const nativeSource = readFileSync(require.resolve('../src/api.rs'), 'utf8');
  const nativeOperations = new Set([...nativeSource.matchAll(/"([a-z]+\.[A-Za-z0-9]+)" =>/g)].map(match => match[1]));
  const exposedOperations = new Set<string>();
  for (const [group, methods] of Object.entries(api)) {
    for (const [method, call] of Object.entries(methods)) {
      assert.equal(typeof call, 'function');
      await (call as () => Promise<unknown>)();
      const operation = `${group}.${method}`;
      exposedOperations.add(operation);
      assert.deepEqual(calls.at(-1), [operation, []]);
    }
  }
  assert.deepEqual(exposedOperations, nativeOperations);
  assert.equal(exposedOperations.size, 129);
  const native = new NativeBot({ appId: 'test', appSecret: 'test' });
  await assert.rejects(native.apiCall('groups.membersPage', [{}, null]), /invalid type/);
  await assert.rejects(native.apiCall('unknown.operation', []), /unknown API operation/);
  await assert.rejects(native.apiCall('messages.uploadMedia', [{ kind: 'invalid', id: 'g' }, 1, [], false]), /media target kind/);
  const secret = 'test-secret';
  const seed = Buffer.from(secret.repeat(4)).subarray(0, 32);
  const key = createPrivateKey({ key: Buffer.concat([Buffer.from('302e020100300506032b657004220420', 'hex'), seed]), format: 'der', type: 'pkcs8' });
  const timestamp = '123456789';
  const signature = (body: Buffer) => sign(null, Buffer.concat([Buffer.from(timestamp), body]), key).toString('hex');
  const bot = new Bot({ appId: 'test', appSecret: secret, mode: 'webhook' });
  let count = 0;
  bot.on('message.group', () => { count++; });
  const body = Buffer.from(JSON.stringify({ op: 0, s: 42, t: 'GROUP_AT_MESSAGE_CREATE', d: { id: 'm', group_openid: 'g' } }));
  assert.deepEqual(bot.handleWebhook(timestamp, signature(body), body), { op: 12 });
  assert.equal(count, 1);
  assert.throws(() => bot.handleWebhook(timestamp, '00'.repeat(64), body));
  assert.equal(count, 1);
  assert.throws(() => bot.handleWebhook(timestamp, signature(body), Buffer.concat([body, Buffer.from(' ')])));
  assert.throws(() => new Bot({ appId: 'test', appSecret: secret }).handleWebhook(timestamp, signature(body), body), /webhook mode/);
  const malformed = Buffer.from('{');
  assert.throws(() => bot.handleWebhook(timestamp, signature(malformed), malformed));
  const precise = Buffer.from('{"op":0,"s":9007199254740993,"t":"READY","d":{}}');
  bot.once('READY', event => assert.equal(event.sequence, '9007199254740993'));
  bot.handleWebhook(timestamp, signature(precise), precise);
  const challenge = Buffer.from(JSON.stringify({ op: 13, d: { plain_token: 'token', event_ts: timestamp } }));
  const response = bot.handleWebhook(timestamp, signature(challenge), challenge);
  assert.equal(response.plain_token, 'token');
  assert(verify(null, Buffer.from(timestamp + 'token'), createPublicKey(key), Buffer.from(response.signature as string, 'hex')));
  await assert.rejects(bot.start(), /webhook/i);
  console.log('API routing and native webhook checks passed');
}
main().catch(error => { console.error(error); process.exitCode = 1; });
