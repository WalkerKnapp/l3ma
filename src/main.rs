use poise::serenity_prelude as serenity;

struct Data {}
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

/// 🏓
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv()
        .expect("Failed to load .env");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("No DISCORD_TOKEN environment variable set");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
