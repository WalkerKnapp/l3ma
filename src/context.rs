use anyhow::Context as _;
use serenity::all::{ChannelId, ForumTagId, GuildId, RoleId};

pub const P2SR_SERVER: GuildId = GuildId::new(146404426746167296);
pub const P2SR_NOTIFICATIONS_CHANNEL: ChannelId = ChannelId::new(432229671711670272);
pub const P2SR_DUNCE_ROLE: RoleId = RoleId::new(146404426746167296);

#[derive(Clone, Copy)]
pub struct ForumAutoCloseConfig {
    pub forum_channel_id: ChannelId,
    pub close_tag_id: ForumTagId,
    pub lock_on_close: bool,
}

pub struct Data {
    pub db: luma1_data::sea_orm::DatabaseConnection,
    pub forum_auto_close: Option<ForumAutoCloseConfig>,
}
pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

impl Data {
    /// Create a new context for the bot's execution
    /// !!!! This will be called multiple times for different shards!
    pub async fn new() -> anyhow::Result<Self> {
        let database_url =
            std::env::var("DATABASE_URL").expect("No DATABASE_URL environment variable set");

        println!("Connecting to database at {}", database_url);
        let forum_auto_close = load_forum_auto_close_config()?;
        Ok(Data {
            db: luma1_data::sea_orm::Database::connect(database_url).await?,
            forum_auto_close,
        })
    }
}

fn load_forum_auto_close_config() -> anyhow::Result<Option<ForumAutoCloseConfig>> {
    let channel_id = match std::env::var("FORUM_AUTO_CLOSE_CHANNEL_ID") {
        Ok(val) => Some(val),
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => return Err(e.into()),
    };
    let tag_id = match std::env::var("FORUM_AUTO_CLOSE_TAG_ID") {
        Ok(val) => Some(val),
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => return Err(e.into()),
    };
    let lock_flag = match std::env::var("FORUM_AUTO_CLOSE_LOCK") {
        Ok(val) => Some(val),
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => return Err(e.into()),
    };

    match (channel_id, tag_id, lock_flag) {
        (None, None, None) => Ok(None),
        (None, None, Some(_)) => anyhow::bail!(
            "FORUM_AUTO_CLOSE_LOCK requires FORUM_AUTO_CLOSE_CHANNEL_ID and FORUM_AUTO_CLOSE_TAG_ID"
        ),
        (Some(channel_id), Some(tag_id), lock_flag) => {
            let channel_id = channel_id
                .parse::<u64>()
                .context("FORUM_AUTO_CLOSE_CHANNEL_ID must be an integer Discord channel id")?;
            let tag_id = tag_id
                .parse::<u64>()
                .context("FORUM_AUTO_CLOSE_TAG_ID must be an integer Discord forum tag id")?;
            let lock_on_close = match lock_flag {
                Some(value) => value
                    .parse::<bool>()
                    .context("FORUM_AUTO_CLOSE_LOCK must be either true or false")?,
                None => false,
            };
            Ok(Some(ForumAutoCloseConfig {
                forum_channel_id: ChannelId::new(channel_id),
                close_tag_id: ForumTagId::new(tag_id),
                lock_on_close,
            }))
        }
        _ => anyhow::bail!(
            "FORUM_AUTO_CLOSE_CHANNEL_ID and FORUM_AUTO_CLOSE_TAG_ID must both be provided when enabling forum auto-close"
        ),
    }
}
