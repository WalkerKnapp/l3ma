mod moderation;
mod development;

use poise::serenity_prelude as serenity;

use std::ops::Sub;
use crate::context::*;

/// pong :3
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    let ms = serenity::Timestamp::now().timestamp_millis()
        - ctx.created_at().timestamp_millis();
    ctx.say(format!("Pong! ({} ms)", ms)).await?;
    Ok(())
}

pub fn generate_commands() -> Vec<poise::Command<Data, anyhow::Error>> {
    let commands = vec![
        ping(),
        moderation::ban(),
        development::register_commands()
    ];

    return commands
}
