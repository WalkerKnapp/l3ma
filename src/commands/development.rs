use crate::context::*;

async fn development_check(ctx: Context<'_>) -> anyhow::Result<bool> {
    Ok(ctx.author().id == 152559951930327040)
}

/// Register commands
#[poise::command(prefix_command, check = "development_check")]
pub async fn register_commands(ctx: Context<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
