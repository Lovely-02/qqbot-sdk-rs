import { Bot, Intents } from '../src/index';
const bot = new Bot({
  appId: process.env.QQ_APP_ID ?? '',
  appSecret: process.env.QQ_APP_SECRET ?? '',
  intents: Intents.GUILDS | Intents.PUBLIC_GUILD_MESSAGES | Intents.GROUP_AND_C2C_EVENT,
});
bot.on('error', console.error);
bot.on('ready', () => console.log('机器人已连接'));
bot.on('message.group', async (event) => {
  await event.reply('收到啦！这里是 Node.js 小助手');
});
bot.on('message.private.friend', async (event) => {
  await event.reply('你好呀！');
});
process.once('SIGINT', () => { bot.stop().catch(console.error); });
bot.start().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
