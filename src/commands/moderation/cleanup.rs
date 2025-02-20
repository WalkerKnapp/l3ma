use chrono::{TimeDelta, Utc};
use crate::context::Context;
use serenity::model::channel::Message;
use poise::futures_util::StreamExt;

/// Delete recent messages in the channel
#[poise::command(
slash_command,
required_permissions = "MANAGE_MESSAGES",
on_error = "crate::commands::error_handler",
ephemeral = true
)]
pub async fn cleanup(
    ctx: Context<'_>,
    #[description = "Number of messages to delete"] #[min = 2] #[max = 1000] messages: u32,
) -> anyhow::Result<()> {
    ctx.defer_ephemeral().await?;

    let mut total_deleted = 0;
    let mut to_bulk_delete = Vec::new();

    let mut message_stream = ctx.channel_id().messages_iter(ctx.http()).boxed();
    while let Some(message) = message_stream.next().await {
        let message: Message = message?;
        // Don't delete pinned messages
        if !message.pinned {
            // Messages can only be bulk deleted if they are younger than 2 weeks old,
            // so if they're older, individually delete
            if Utc::now().signed_duration_since(&*message.timestamp) < TimeDelta::weeks(2) {
                to_bulk_delete.push(message.id);
            } else {
                message.delete(ctx.http()).await?;
            }
            total_deleted += 1;
        }

        if total_deleted >= messages {
            break;
        }
    }

    // Bulk delete messages that can be bulk deleted
    for bulk_chunk in to_bulk_delete.chunks(100) {
        ctx.channel_id().delete_messages(ctx.http(), bulk_chunk).await?;
    }

    ctx.reply(format!("Removed {} messages", total_deleted)).await?
        .delete(ctx).await?;

    Ok(())
}

