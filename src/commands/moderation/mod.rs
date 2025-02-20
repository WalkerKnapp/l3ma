mod ban;
mod dunce;
mod cleanup;

pub use ban::ban as ban;
pub use dunce::dunce as dunce;
pub use dunce::undunce as undunce;
pub use cleanup::cleanup as cleanup;
use crate::context::{P2SR_NOTIFICATIONS_CHANNEL};

use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
use serenity::all::{CacheHttp, Timestamp, User};

pub async fn send_mod_action_log(
    http: impl CacheHttp,
    author: User,
    embed_builder: impl Fn(CreateEmbed) -> CreateEmbed
) -> anyhow::Result<()> {

    let mut notif_author = CreateEmbedAuthor::new("");
    if let Some(url) = author.avatar_url() {
        notif_author = notif_author.icon_url(url);
    }
    notif_author = notif_author.name(author.name);

    let embed = CreateEmbed::new().author(notif_author).timestamp(Timestamp::now());
    let embed = embed_builder(embed);

    P2SR_NOTIFICATIONS_CHANNEL.send_message(
        http, CreateMessage::new().embed(embed)
    ).await.map_err(|e| anyhow::Error::new(e).context("Could not send notification message"))?;

    Ok(())
}
