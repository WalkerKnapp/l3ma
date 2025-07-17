#![feature(macro_metavar_expr)]

mod commands;
pub mod context;
mod events;

use std::collections::HashSet;
use poise::serenity_prelude as serenity;
use crate::context::Data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::fs::exists(".env").unwrap_or(false) {
        dotenv::dotenv()
            .expect("Failed to load .env");
    }
    for (key, val) in std::env::vars() {
        if let Some(stripped_key) = key.strip_suffix("_FILE") {
            match std::fs::read_to_string(&val) {
                Ok(res) => {
                    // SAFETY: This code is only run before the program has forked into multiple threads
                    unsafe { std::env::set_var(stripped_key, res); }
                }
                Err(e) => {
                    eprintln!("Failed to read environment variable value at {key}={val}: {e}");
                }
            }
        }
    }

    let token = std::env::var("DISCORD_TOKEN")
        .expect("No DISCORD_TOKEN environment variable set");

    let command_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::generate_commands(),
            owners: HashSet::from([152559951930327040.into()]),
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data::new().await.expect("Failed to create state data"))
            })
        })
        .build();

    let event_handler = events::Handler {
        data: Data::new().await.expect("Failed to create state data")
    };

    let mut client = serenity::ClientBuilder::new(
        token,
        serenity::GatewayIntents::non_privileged()
            .union(serenity::GatewayIntents::GUILD_MEMBERS)
    )
        .framework(command_framework)
        .event_handler(event_handler)
        .await?;

    println!("Starting discord bot!");
    client.start().await?;

    Ok(())
}
