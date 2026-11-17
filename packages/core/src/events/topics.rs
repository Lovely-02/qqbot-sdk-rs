/// 官方事件与分层订阅主题的映射。与 Node.js 包保持一致。
pub const EVENT_TOPICS: &[(&str, &str)] = &[
    ("GROUP_AT_MESSAGE_CREATE", "message.group"),
    ("GROUP_MESSAGE_CREATE", "message.group"),
    ("C2C_MESSAGE_CREATE", "message.private.friend"),
    ("DIRECT_MESSAGE_CREATE", "message.private.guild"),
    ("AT_MESSAGE_CREATE", "message.guild.at"),
    ("MESSAGE_CREATE", "message.guild.normal"),
    ("FRIEND_ADD", "notice.friend.increase"),
    ("FRIEND_DEL", "notice.friend.decrease"),
    ("C2C_MSG_RECEIVE", "notice.friend.message.receive"),
    ("C2C_MSG_REJECT", "notice.friend.message.reject"),
    ("GROUP_MEMBER_ADD", "notice.group.increase"),
    ("GROUP_MEMBER_REMOVE", "notice.group.decrease"),
    ("GROUP_ADD_ROBOT", "notice.group.bot.add"),
    ("GROUP_DEL_ROBOT", "notice.group.bot.remove"),
    ("GROUP_MSG_RECEIVE", "notice.group.message.receive"),
    ("GROUP_MSG_REJECT", "notice.group.message.reject"),
    ("GROUP_JOIN_REQUEST", "request.group.add"),
    ("SUBSCRIBE_MESSAGE_STATUS", "notice.subscription.status"),
    ("GUILD_CREATE", "notice.guild.create"),
    ("GUILD_UPDATE", "notice.guild.update"),
    ("GUILD_DELETE", "notice.guild.delete"),
    ("CHANNEL_CREATE", "notice.channel.create"),
    ("CHANNEL_UPDATE", "notice.channel.update"),
    ("CHANNEL_DELETE", "notice.channel.delete"),
    ("GUILD_MEMBER_ADD", "notice.guild.member.increase"),
    ("GUILD_MEMBER_UPDATE", "notice.guild.member.update"),
    ("GUILD_MEMBER_REMOVE", "notice.guild.member.decrease"),
    ("MESSAGE_DELETE", "notice.guild.recall"),
    ("PUBLIC_MESSAGE_DELETE", "notice.guild.recall"),
    ("DIRECT_MESSAGE_DELETE", "notice.private.guild.recall"),
    ("MESSAGE_REACTION_ADD", "notice.guild.reaction.add"),
    ("MESSAGE_REACTION_REMOVE", "notice.guild.reaction.remove"),
    ("MESSAGE_AUDIT_PASS", "notice.message.audit.pass"),
    ("MESSAGE_AUDIT_REJECT", "notice.message.audit.reject"),
    ("INTERACTION_CREATE", "interaction.create"),
    ("FORUM_THREAD_CREATE", "notice.forum.thread.create"),
    ("FORUM_THREAD_UPDATE", "notice.forum.thread.update"),
    ("FORUM_THREAD_DELETE", "notice.forum.thread.delete"),
    ("FORUM_POST_CREATE", "notice.forum.post.create"),
    ("FORUM_POST_DELETE", "notice.forum.post.delete"),
    ("FORUM_REPLY_CREATE", "notice.forum.reply.create"),
    ("FORUM_REPLY_DELETE", "notice.forum.reply.delete"),
    ("FORUM_PUBLISH_AUDIT_RESULT", "notice.forum.audit"),
    (
        "OPEN_FORUM_THREAD_CREATE",
        "notice.forum.open.thread.create",
    ),
    (
        "OPEN_FORUM_THREAD_UPDATE",
        "notice.forum.open.thread.update",
    ),
    (
        "OPEN_FORUM_THREAD_DELETE",
        "notice.forum.open.thread.delete",
    ),
    ("OPEN_FORUM_POST_CREATE", "notice.forum.open.post.create"),
    ("OPEN_FORUM_POST_DELETE", "notice.forum.open.post.delete"),
    ("OPEN_FORUM_REPLY_CREATE", "notice.forum.open.reply.create"),
    ("OPEN_FORUM_REPLY_DELETE", "notice.forum.open.reply.delete"),
    ("AUDIO_START", "notice.audio.start"),
    ("AUDIO_FINISH", "notice.audio.finish"),
    ("AUDIO_ON_MIC", "notice.audio.mic.on"),
    ("AUDIO_OFF_MIC", "notice.audio.mic.off"),
    (
        "AUDIO_OR_LIVE_CHANNEL_MEMBER_ENTER",
        "notice.audio.member.enter",
    ),
    (
        "AUDIO_OR_LIVE_CHANNEL_MEMBER_EXIT",
        "notice.audio.member.exit",
    ),
    ("READY", "system.online"),
    ("RESUMED", "system.resumed"),
];

pub(crate) fn subscription_names(name: &str) -> Vec<String> {
    let mut names = vec!["event".to_owned(), name.to_owned()];
    if let Some((_, topic)) = EVENT_TOPICS.iter().find(|(official, _)| *official == name) {
        let mut alias = String::new();
        for (index, part) in name.to_ascii_lowercase().split('_').enumerate() {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                alias.push(if index == 0 {
                    first
                } else {
                    first.to_ascii_uppercase()
                });
                alias.extend(chars);
            }
        }
        names.push(alias);
        let mut topic = *topic;
        loop {
            names.push(topic.to_owned());
            let Some((parent, _)) = topic.rsplit_once('.') else {
                break;
            };
            topic = parent;
        }
    }
    names
}
