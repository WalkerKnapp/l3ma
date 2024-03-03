use serenity::all::Member;
use serenity::prelude::{EventHandler};
use serenity::prelude::Context as SerenityContext;
use crate::context::Data;

mod moderation;

pub struct Handler {
    pub(crate) data: Data
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: SerenityContext, new_member: Member) {
        if let Err(e) = moderation::ensure_dunced(ctx, &self.data, new_member).await {
            eprintln!("Encountered error while handling event: {:?}", e);
        }
    }
}
