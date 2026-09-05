//! QQ HTTP API 封装。

pub mod bot;
pub mod channel;
pub mod group;
pub mod guild;
pub mod interaction;
pub mod menu;
pub mod message;
pub mod panel;
pub mod user;
pub mod utility;

pub use bot::BotApi;
pub use channel::ChannelApi;
pub use group::GroupApi;
pub use guild::GuildApi;
pub use interaction::InteractionApi;
pub use menu::MenuApi;
pub use message::{MediaTarget, MessageApi};
pub use panel::PanelApi;
pub use user::UserApi;
pub use utility::UtilityApi;

pub(crate) fn segment(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

/// 构建查询参数列表，并省略未设置的参数。
pub(crate) fn optional_query<'a>(
    values: impl IntoIterator<Item = (&'a str, Option<String>)>,
) -> Vec<(&'a str, String)> {
    values
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
        .collect()
}
