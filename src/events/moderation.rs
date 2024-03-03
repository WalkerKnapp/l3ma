use serenity::all::Member;
use serenity::prelude::Context as SerenityContext;
use crate::context::{Data, P2SR_SERVER, P2SR_DUNCE_ROLE};

use luma1_data::entity::prelude::*;
use luma1_data::sea_orm::EntityTrait;

pub async fn ensure_dunced(ctx: SerenityContext, data: &Data, member: Member) -> anyhow::Result<()> {
    // We only perform checks in P2SR
    if member.guild_id != P2SR_SERVER {
        return Ok(());
    }

    // Check to see if the member is dunced
    if DunceInstants::find_by_id(member.user.id.get() as i64)
        .one(&data.db).await?.is_some() {
        // Give the user back their dunce role
        member.add_role(&ctx.http, P2SR_DUNCE_ROLE).await?;
    }

    Ok(())
}