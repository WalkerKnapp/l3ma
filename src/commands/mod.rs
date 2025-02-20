mod moderation;
mod development;

use poise::serenity_prelude::*;
use crate::context::{Context, Data};

pub async fn error_handler(error: poise::FrameworkError<'_, Data, anyhow::Error>) {
    if let poise::FrameworkError::Command {error, ctx, .. } = error {
        if let Err(e) = ctx.say(format!("```diff\n- {:#}\n```", error)).await {
            eprintln!("Error in error handling: {:?}", e);
            eprintln!("Original error: {:?}", error);
        }
    } else {
        eprintln!("Couldn't handle ban error: {}", error);
    }
}

/// pong :3
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    let ms = Timestamp::now().timestamp_millis()
        - ctx.created_at().timestamp_millis();
    ctx.say(format!("Pong! ({} ms)", ms)).await?;
    Ok(())
}

pub fn generate_commands() -> Vec<poise::Command<Data, anyhow::Error>> {
    let commands = vec![
        ping(),
        moderation::ban(),
        moderation::dunce(),
        moderation::undunce(),
        moderation::cleanup(),
        development::register_commands()
    ];

    return commands
}
