use poise::serenity_prelude::*;
use crate::context::{Context, Data};

const P2SR_SERVER: GuildId =
    GuildId::new(146404426746167296);
const P2SR_NOTIFICATIONS_CHANNEL: ChannelId =
    ChannelId::new(432229671711670272);

async fn error_handler(error: poise::FrameworkError<'_, Data, anyhow::Error>) {
    if let poise::FrameworkError::Command {error, ctx, .. } = error {
        if let Err(e) = ctx.say(format!("```diff\n- {:#}\n```", error)).await {
            eprintln!("Error in error handling: {}", e);
        }
    } else {
        eprintln!("Couldn't handle ban error: {}", error);
    }
}

/// Ban a user and DMs them a reason
#[poise::command(
    slash_command,
    required_permissions = "BAN_MEMBERS",
    on_error = "error_handler"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: User,
    #[description = "Reason to record/DM"] reason: Option<String>,
    #[description = "Delete messages from the last X days"] #[max = 7] cleanup: Option<u8>
) -> anyhow::Result<()> {

    // Try to ban the user
    ctx.http().ban_user(P2SR_SERVER, user.id, cleanup.unwrap_or(0), reason.as_deref()).await
        .map_err(|e| anyhow::Error::new(e).context("Could not ban user"))?;

    // Try to notify mod actions
    let mut notif_author = CreateEmbedAuthor::new(&ctx.author().name);
    if let Some(url) = ctx.author().avatar_url() {
        notif_author = notif_author.icon_url(url);
    }
    P2SR_NOTIFICATIONS_CHANNEL.send_message(
        ctx.http(), CreateMessage::new().embed(CreateEmbed::new()
            .author(notif_author)
            .description(format!("Banned {} ({})", user.mention(), user.id))
            .field("Reason", reason.clone().unwrap_or("*No reason specified*".to_string()), false))
    ).await.map_err(|e| anyhow::Error::new(e).context("Could not send notification message"))?;

    // Try to DM the user the result
    let dm_result: anyhow::Result<()> = {
        let embed = CreateEmbed::new()
            .color(Color::from_rgb(179, 38, 255))
            .title("You have been banned by a moderator")
            .field("Reason", reason.unwrap_or("*No reason specified, contact moderators for more information*".to_string()), false)
            .footer(CreateEmbedFooter::new("Portal 2 Speedrun Server"))
            .timestamp(Timestamp::now());

        let channel = user.create_dm_channel(ctx.http()).await?;
        channel.send_message(ctx.http(), CreateMessage::new().embed(embed)).await?;

        Ok(())
    };

    // Send followup
    if dm_result.is_ok() {
        ctx.say(format!("Banned user {} ({})", user.mention(), user.id)).await?;
    } else {
        ctx.say(format!("Banned user {} ({})\n```diff\n- Unable to DM User\n```", user.mention(), user.id)).await?;
    }
    Ok(())
}
