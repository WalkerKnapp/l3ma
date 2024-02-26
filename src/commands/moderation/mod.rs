mod ban;
mod dunce;

pub use ban::ban as ban;
use crate::context::{Context, P2SR_NOTIFICATIONS_CHANNEL};

use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateMessage};

pub async fn send_mod_action_log(
    ctx: &Context<'_>,
    embed_builder: impl Fn(CreateEmbed) -> CreateEmbed
) -> anyhow::Result<()> {

    let mut notif_author = CreateEmbedAuthor::new(&ctx.author().name);
    if let Some(url) = ctx.author().avatar_url() {
        notif_author = notif_author.icon_url(url);
    }

    let embed = CreateEmbed::new().author(notif_author);
    let embed = embed_builder(embed);

    P2SR_NOTIFICATIONS_CHANNEL.send_message(
        ctx.http(), CreateMessage::new().embed(embed)
    ).await.map_err(|e| anyhow::Error::new(e).context("Could not send notification message"))?;

    Ok(())
}
