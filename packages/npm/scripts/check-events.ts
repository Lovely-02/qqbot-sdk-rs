import assert from 'node:assert/strict';
import type { Bot as BotType, MessageEvent } from '../src/index';
import { eventTopics, camelAlias } from '../src/event-topics';

let receive: (error: Error | null, payload: string) => void;
const sent: unknown[][] = [];
class NativeBotMock {
  run(callback: typeof receive): Promise<void> {
    receive = callback;
    return Promise.resolve();
  }
  send(...args: unknown[]): Promise<Record<string, unknown>> {
    sent.push(args);
    return Promise.resolve({ id: 'reply' });
  }
  stop(): void {}
}
// 在加载包装层前替换原生边界，验证真实分发路径而不连接 QQ。
const nativePath = require.resolve('../native.js');
require.cache[nativePath] = { exports: { NativeBot: NativeBotMock } } as NodeModule;
const { Bot } = require('../src/index') as typeof import('../src/index');

async function main(): Promise<void> {
  const bot: BotType = new Bot({ appId: 'test', appSecret: 'test' });
  const errors: Error[] = [];
  bot.on('error', (error) => errors.push(error));
  await bot.start();
  const cases = [
    ['GROUP_AT_MESSAGE_CREATE', 'groupAtMessageCreate', 'message.group', 'message.group'],
    ['GROUP_MESSAGE_CREATE', 'groupMessageCreate', 'message.group', 'message.group'],
    ['C2C_MESSAGE_CREATE', 'c2cMessageCreate', 'message.private.friend', 'message.private'],
    ['DIRECT_MESSAGE_CREATE', 'directMessageCreate', 'message.private.guild', 'message.private'],
    ['AT_MESSAGE_CREATE', 'atMessageCreate', 'message.guild.at', 'message.guild'],
    ['MESSAGE_CREATE', 'messageCreate', 'message.guild.normal', 'message.guild'],
  ] as const;
  for (const [name, alias, topic, parent] of cases) {
    const order: string[] = [];
    const events: MessageEvent[] = [];
    const keys = [...new Set(['event', name, alias, topic, parent, 'message'])];
    for (const key of keys) {
      bot.on(key, (event) => { order.push(key); events.push(event as MessageEvent); });
    }
    receive(null, JSON.stringify({ name, id: 'event-id', sequence: '9007199254740993', data: {
      id: 'msg-id', group_openid: 'group', guild_id: 'guild', channel_id: 'channel',
      author: { user_openid: 'user' },
    } }));
    assert.deepEqual(order, keys);
    assert.ok(events.every((event) => event === events[0]));
    assert.equal(events[0]!.sequence, '9007199254740993');
    await events[0]!.reply('hello');
    assert.deepEqual(sent.at(-1), [
      name.startsWith('GROUP_') ? 'group' : name === 'C2C_MESSAGE_CREATE' ? 'user'
        : name === 'DIRECT_MESSAGE_CREATE' ? 'direct' : 'channel',
      name.startsWith('GROUP_') ? 'group' : name === 'C2C_MESSAGE_CREATE' ? 'user'
        : name === 'DIRECT_MESSAGE_CREATE' ? 'guild' : 'channel',
      { content: 'hello', msg_id: 'msg-id', event_id: undefined },
    ]);
    for (const key of keys) bot.removeAllListeners(key);
  }
  assert.equal(eventTopics.GROUP_AT_MESSAGE_CREATE, 'message.group');
  assert.equal(eventTopics.GROUP_MESSAGE_CREATE, 'message.group');
  assert.equal(eventTopics.FRIEND_ADD, 'notice.friend.increase');
  assert.equal(eventTopics.GROUP_JOIN_REQUEST, 'request.group.add');
  assert.equal(eventTopics.MESSAGE_DELETE, 'notice.guild.recall');
  for (const [name, leaf] of Object.entries(eventTopics)) {
    const topics: string[] = [];
    let topic = leaf as string;
    while (true) {
      topics.push(topic);
      const separator = topic.lastIndexOf('.');
      if (separator === -1) break;
      topic = topic.slice(0, separator);
    }
    const expected = ['event', name, camelAlias(name), ...topics];
    const observed: string[] = [];
    for (const key of expected) bot.on(key, () => { observed.push(key); });
    receive(null, JSON.stringify({ name, data: {} }));
    assert.deepEqual(observed, expected, name);
    for (const key of expected) bot.removeAllListeners(key);
  }
  // 确认父级通知和请求具备类型提示，且不会串到 message。
  bot.once('notice', (event) => { assert.equal(event.name, 'FRIEND_ADD'); });
  bot.once('request.group', (event) => { assert.equal(event.name, 'GROUP_JOIN_REQUEST'); });
  let unexpectedMessages = 0;
  bot.on('message', () => { unexpectedMessages++; });
  for (const name of ['FRIEND_ADD', 'GROUP_JOIN_REQUEST', 'UNKNOWN_EVENT', 'constructor', '__proto__']) {
    receive(null, JSON.stringify({ name, data: {} }));
  }
  assert.equal(unexpectedMessages, 0);
  bot.removeAllListeners('message');
  let count = 0;
  const listener = () => { count++; };
  bot.once('message', listener);
  bot.on('message.group', listener).off('message.group', listener);
  for (const name of ['READY', 'UNKNOWN_EVENT', 'MESSAGE_CREATE', 'MESSAGE_CREATE']) {
    receive(null, JSON.stringify({ name, data: {} }));
  }
  assert.equal(count, 1);
  bot.on('message', async () => { throw new Error('handler failed'); });
  receive(null, JSON.stringify({ name: 'MESSAGE_CREATE', data: {} }));
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(errors[0]?.message, 'handler failed');
  await bot.stop();
  console.log(`${Object.keys(eventTopics).length} event subscriptions verified`);
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
