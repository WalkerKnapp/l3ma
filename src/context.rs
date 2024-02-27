use serenity::all::{ChannelId, GuildId};

pub const P2SR_SERVER: GuildId =
    GuildId::new(146404426746167296);
pub const P2SR_NOTIFICATIONS_CHANNEL: ChannelId =
    ChannelId::new(432229671711670272);

pub struct Data {
    pub db: luma1_data::sea_orm::DatabaseConnection
}
pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

impl Data {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .expect("No DATABASE_URL environment variable set");

        Ok(Data {
            db: luma1_data::sea_orm::Database::connect(database_url).await?
        })
    }
}