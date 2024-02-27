use poise::serenity_prelude::*;
use crate::context::{Context, P2SR_SERVER};


async fn send_dm_notification(user: &User, reason: Option<&String>, ctx: &Context<'_>) -> anyhow::Result<()> {
    let embed = CreateEmbed::new()
        .color(Color::from_rgb(179, 38, 255))
        .title("You have been banned by a moderator")
        .field("Reason", reason.map(|s| s.as_str()).unwrap_or("*No reason specified, contact moderators for more information*"), false)
        .field("Appeal", "https://s.portal2.sr/appeal", false)
        .footer(CreateEmbedFooter::new("Portal 2 Speedrun Server"))
        .timestamp(Timestamp::now());

    let channel = user.create_dm_channel(ctx.http()).await?;
    channel.send_message(ctx.http(), CreateMessage::new().embed(embed)).await?;

    Ok(())
}

/// Ban a user and DMs them a reason
#[poise::command(
slash_command,
required_permissions = "BAN_MEMBERS",
on_error = "crate::commands::error_handler"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: User,
    #[description = "Reason to record/DM"] reason: Option<String>,
    #[description = "Delete messages from the last X days"] #[max = 7] cleanup: Option<u8>
) -> anyhow::Result<()> {

    // Try to notify mod actions
    super::send_mod_action_log(ctx.http(), ctx.author().clone(), |embed| {
        embed.description(format!("Banned {} ({})", user.mention(), user.id))
            .field("Reason", reason.clone().unwrap_or("*No reason specified*".to_string()), false)
    }).await?;

    // Try to DM the user the result
    let dm_result = send_dm_notification(&user, reason.as_ref(), &ctx).await;

    // Try to ban the user
    ctx.http().ban_user(P2SR_SERVER, user.id, cleanup.unwrap_or(0), reason.as_deref()).await
        .map_err(|e| anyhow::Error::new(e).context("Could not ban user"))?;

    // Send followup
    if dm_result.is_ok() {
        ctx.say(format!("Banned user {} ({})", user.mention(), user.id)).await?;
    } else {
        ctx.say(format!("Banned user {} ({})\n```diff\n- Unable to DM User\n```", user.mention(), user.id)).await?;
    }
    Ok(())
}
