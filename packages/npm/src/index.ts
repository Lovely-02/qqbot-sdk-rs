import { EventEmitter } from 'node:events';
import { NativeBot } from '../native.js';
import { eventTopics, camelAlias, type SubscriptionEvents } from './event-topics';

export interface BotOptions {
  appId: string;
  appSecret: string;
  mode?: 'public' | 'private';
  intents?: number;
}
/** QQ MessageRequest 字段保留官方的下划线命名方式。 */
export interface MessageRequest {
  content?: string;
  msg_type?: number;
  msg_id?: string;
  msg_seq?: number;
  event_id?: string;
  image?: string;
  markdown?: Record<string, unknown>;
  keyboard?: Record<string, unknown>;
  media?: Record<string, unknown>;
  embed?: Record<string, unknown>;
  ark?: Record<string, unknown>;
  [field: string]: unknown;
}
export type Message = string | MessageRequest;
export type MessageResponse = Record<string, unknown>;
export interface Target {
  id: string;
  send(message: Message): Promise<MessageResponse>;
}
export interface EventData extends Record<string, unknown> {
  id?: string;
  content?: string;
  group_openid?: string;
  channel_id?: string;
  guild_id?: string;
  author?: { user_openid?: string; [field: string]: unknown };
}
export interface BotEvent {
  id: string | null;
  name: string;
  /** 使用十进制字符串，避免跨越原生模块边界时丢失数值精度。 */
  sequence: string | null;
  data: EventData;
  /** 仅受支持的消息事件提供此方法。 */
  reply?: (message: Message) => Promise<MessageResponse>;
}
export interface MessageEvent extends BotEvent {
  reply(message: Message): Promise<MessageResponse>;
}
export interface BotEvents extends SubscriptionEvents {
  event: [BotEvent];
  error: [Error];
}
export const Intents = Object.freeze({
  GUILDS: 1, GUILD_MEMBERS: 1 << 1, GUILD_MESSAGES: 1 << 9,
  GUILD_MESSAGE_REACTIONS: 1 << 10, DIRECT_MESSAGE: 1 << 12,
  GROUP_MEMBER_EVENT: 1 << 24, GROUP_AND_C2C_EVENT: 1 << 25,
  INTERACTION: 1 << 26, MESSAGE_AUDIT: 1 << 27,
  FORUMS_EVENT: 1 << 28, AUDIO_ACTION: 1 << 29,
  PUBLIC_GUILD_MESSAGES: 1 << 30,
});
type TargetKind = 'user' | 'group' | 'channel' | 'direct';
export class Bot extends EventEmitter<BotEvents> {
  #native: NativeBot;
  #running?: Promise<void>;
  #stopped = false;
  constructor(options: BotOptions) {
    super({ captureRejections: true });
    this.#native = new NativeBot(options);
  }
  override on<K extends keyof BotEvents>(name: K, listener: (...args: BotEvents[K]) => void): this;
  override on(name: string | symbol, listener: (...args: unknown[]) => void): this;
  // 此实现边界沿用 EventEmitter 接受不同事件参数列表的约定。
  override on(name: string | symbol, listener: (...args: any[]) => void): this {
    return super.on(name as keyof BotEvents, listener);
  }
  override once<K extends keyof BotEvents>(name: K, listener: (...args: BotEvents[K]) => void): this;
  override once(name: string | symbol, listener: (...args: unknown[]) => void): this;
  override once(name: string | symbol, listener: (...args: any[]) => void): this {
    return super.once(name as keyof BotEvents, listener);
  }
  user(id: string): Target { return this.#target('user', id); }
  group(id: string): Target { return this.#target('group', id); }
  channel(id: string): Target { return this.#target('channel', id); }
  direct(id: string): Target { return this.#target('direct', id); }
  #target(kind: TargetKind, id: string): Target {
    return { id, send: (message) => this.#native.send(kind, id, message) as Promise<MessageResponse> };
  }
  #event(payload: string): void {
    const event = JSON.parse(payload) as BotEvent;
    const data = event.data;
    let target: Target | undefined;
    switch (event.name) {
      case 'GROUP_AT_MESSAGE_CREATE':
      case 'GROUP_MESSAGE_CREATE': target = this.group(data.group_openid ?? ''); break;
      case 'C2C_MESSAGE_CREATE': target = this.user(data.author?.user_openid ?? ''); break;
      case 'MESSAGE_CREATE':
      case 'AT_MESSAGE_CREATE': target = this.channel(data.channel_id ?? ''); break;
      case 'DIRECT_MESSAGE_CREATE': target = this.direct(data.guild_id ?? ''); break;
    }
    if (target) {
      const destination = target;
      event.reply = (message) => {
        if (!data.id) return Promise.reject(new Error('Message event has no message id'));
        const body = typeof message === 'string' ? { content: message } : { ...message };
        return destination.send({ ...body, msg_id: data.id, event_id: undefined });
      };
    }
    this.emit('event', event);
    this.emit(event.name as keyof BotEvents, event);
    let topic: string | undefined = Object.hasOwn(eventTopics, event.name)
      ? eventTopics[event.name as keyof typeof eventTopics] : undefined;
    if (topic) this.emit(camelAlias(event.name) as keyof BotEvents, event);
    while (topic) {
      this.emit(topic as keyof BotEvents, event);
      const separator = topic.lastIndexOf('.');
      if (separator === -1) break;
      topic = topic.slice(0, separator);
    }
  }
  /** 停止时兑现 Promise，发生不可恢复的错误时拒绝 Promise；每个 Bot 实例只能启动一次。 */
  start(): Promise<void> {
    if (this.#stopped) return Promise.reject(new Error('Create a new Bot after stop'));
    if (this.#running) return this.#running;
    this.#running = this.#native.run((error, payload) => {
      // 捕获 JavaScript 异常，避免异常穿过原生回调边界。
      try {
        if (error) throw error;
        this.#event(payload);
      } catch (cause) {
        const error = cause instanceof Error ? cause : new Error(String(cause));
        queueMicrotask(() => this.emit('error', error));
      }
    });
    return this.#running;
  }
  async stop(): Promise<void> {
    this.#stopped = true;
    this.#native.stop();
    await this.#running;
  }
}
