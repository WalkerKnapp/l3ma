mod commands;
pub mod context;

use poise::serenity_prelude as serenity;

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
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(context::Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
