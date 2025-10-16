use serenity::all::{Channel, ChannelType, GuildChannel, Member, Reaction, ReactionType};
use serenity::builder::EditThread;
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

    async fn thread_update(
        &self,
        ctx: SerenityContext,
        old: Option<GuildChannel>,
        new: GuildChannel,
    ) {
        let Some(config) = &self.data.forum_auto_close else {
            return;
        };

        if new.parent_id != Some(config.forum_channel_id) {
            return;
        }

        if !matches!(
            new.kind,
            ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
        ) {
            return;
        }

        if !new.applied_tags.iter().any(|tag| *tag == config.close_tag_id) {
            return;
        }

        if old
            .as_ref()
            .is_some_and(|previous| previous.applied_tags.iter().any(|tag| *tag == config.close_tag_id))
        {
            return;
        }

        if new
            .thread_metadata
            .as_ref()
            .map(|metadata| metadata.archived)
            .unwrap_or(false)
        {
            return;
        }

        if old
            .as_ref()
            .and_then(|thread| thread.thread_metadata.as_ref())
            .is_some_and(|metadata| metadata.archived)
        {
            return;
        }

        if let Err(err) = new
            .id
            .edit_thread(&ctx.http, EditThread::new().archived(true))
            .await
        {
            eprintln!(
                "Failed to auto-close thread {} after close tag applied: {:?}",
                new.id, err
            );
        }
    }

    async fn reaction_add(&self, ctx: SerenityContext, reaction: Reaction) {
        let Some(config) = &self.data.forum_auto_close else {
            return;
        };

        if !matches!(&reaction.emoji, ReactionType::Unicode(emoji) if emoji == "✅") {
            return;
        }

        let Some(user_id) = reaction.user_id else {
            return;
        };

        let channel = match reaction.channel(&ctx.http).await {
            Ok(Channel::Guild(channel)) => channel,
            Ok(_) => return,
            Err(err) => {
                eprintln!(
                    "Failed to fetch channel {} for reaction {}: {:?}",
                    reaction.channel_id,
                    reaction.message_id,
                    err
                );
                return;
            }
        };

        if channel.parent_id != Some(config.forum_channel_id) {
            return;
        }

        if !matches!(
            channel.kind,
            ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
        ) {
            return;
        }

        if channel.owner_id != Some(user_id) {
            return;
        }

        if channel
            .thread_metadata
            .as_ref()
            .map(|metadata| metadata.archived)
            .unwrap_or(false)
        {
            return;
        }

        if let Err(err) = channel
            .id
            .edit_thread(&ctx.http, EditThread::new().archived(true))
            .await
        {
            eprintln!(
                "Failed to auto-close thread {} after owner reaction: {:?}",
                channel.id, err
            );
        }
    }
}
