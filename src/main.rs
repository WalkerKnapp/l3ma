#![feature(macro_metavar_expr)]

mod commands;
pub mod context;

use poise::serenity_prelude as serenity;
use crate::context::Data;

#[tokio::main]
async fn main() {
    dotenv::dotenv()
        .expect("Failed to load .env");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("No DISCORD_TOKEN environment variable set");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::generate_commands(),
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data::new().await.expect("Failed to create state data"))
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
