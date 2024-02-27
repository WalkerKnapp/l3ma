use anyhow::anyhow;
use chrono::{DateTime, Months, TimeDelta, Utc};
use poise::serenity_prelude::User;
use poise::serenity_prelude::Mentionable;

use luma1_data::entity::prelude::*;
use luma1_data::sea_orm::{EntityTrait, Set, ActiveModelTrait};
use serenity::all::{RoleId};

use crate::context::Context;

pub const P2SR_DUNCE_ROLE: RoleId =
    RoleId::new(146404426746167296);

#[derive(poise::ChoiceParameter)]
pub enum TimeUnits {
    Minutes,
    Hours,
    Days,
    Weeks,
    Months
}

impl TimeUnits {
    fn apply_delta(&self, time: DateTime<Utc>, amount: u32) -> anyhow::Result<DateTime<Utc>> {
        let res = match &self {
            TimeUnits::Minutes => time.checked_add_signed(TimeDelta::minutes(amount as i64)),
            TimeUnits::Hours => time.checked_add_signed(TimeDelta::hours(amount as i64)),
            TimeUnits::Days => time.checked_add_signed(TimeDelta::days(amount as i64)),
            TimeUnits::Weeks => time.checked_add_signed(TimeDelta::weeks(amount as i64)),
            TimeUnits::Months => time.checked_add_months(Months::new(amount))
        };

        res.ok_or(anyhow!("Dunce time out of maximum range"))
    }
}

/// Dunce a user
#[poise::command(
slash_command,
required_permissions = "MODERATE_MEMBERS",
on_error = "crate::commands::error_handler"
)]
pub async fn dunce(
    ctx: Context<'_>,
    #[description = "User to dunce"] user: User,
    #[description = "Time to dunce"] time: u32,
    #[description = "Time units"] time_units: TimeUnits
) -> anyhow::Result<()> {
    let user_mention = user.mention();
    let user_id = user.id;

    // Calculate when to undunce
    let undunce_time = time_units.apply_delta(Utc::now(), time)?;

    let mut error_messages: Vec<String> = vec![];

    // Check for an existing dunce
    let success_message = if DunceInstants::find_by_id(user.id.get() as i64)
        .one(&ctx.data().db).await?.is_some() {

        let (r1, r2) = tokio::join!(
            // Queue setting undunce time in DB
            luma1_data::entity::dunce_instants::ActiveModel {
                user_id: Set(user.id.into()),
                undunce_instant: Set(undunce_time)
            }.update(&ctx.data().db),
            // Queue mod actions notification
            super::send_mod_action_log(ctx.http(), ctx.author().clone(), move |embed| {
                embed.description(format!("Updating dunce time for {} ({})", user.mention(), &user.id))
                .field("Remaining", format!("Undunce <t:{}:R>", undunce_time.timestamp()), true)
                .field("Expires", format!("<t:{}:f>", undunce_time.timestamp()), true)
            })
        );
        if let Err(e) = r1 {
            error_messages.push(format!("- {:#}", e))
        }
        if let Err(e) = r2 {
            error_messages.push(format!("- {:#}", e))
        }

        // Send followup
        ctx.say(format!("Updated dunce time for user {} ({}), they will be undunced <t:{}:R>", user_mention, user_id, undunce_time.timestamp()))
    } else {
        if let Some(member) = ctx.author_member().await {

            let roles_to_remove: Vec<RoleId> = member.roles.clone().into_iter()
                .filter(|role_id| *role_id != P2SR_DUNCE_ROLE)
                .collect();

            let role_active_models: Vec<luma1_data::entity::dunce_stored_roles::ActiveModel> =
                roles_to_remove.iter().map(|role_id| {
                    luma1_data::entity::dunce_stored_roles::ActiveModel {
                        user_id: Set(ctx.author().id.into()),
                        role_id: Set((*role_id).into())
                    }
                }).collect();

            let (r1, r2) = tokio::join!(
                DunceStoredRoles::insert_many(role_active_models)
                .on_empty_do_nothing()
                .exec(&ctx.data().db),
                member.remove_roles(ctx.http(), &roles_to_remove)
            );
            if let Err(e) = r1 {
                error_messages.push(format!("- {:#}", e))
            }
            if let Err(e) = r2 {
                error_messages.push(format!("- {:#}", e))
            }
        }

        let (r1, r2) = tokio::join!(
            // Queue inserting undunce time in DB
            luma1_data::entity::dunce_instants::ActiveModel {
                user_id: Set(user.id.into()),
                undunce_instant: Set(undunce_time)
            }.insert(&ctx.data().db),
            // Queue mod actions notification
            super::send_mod_action_log(ctx.http(), ctx.author().clone(), move |embed| {
                embed.description(format!("Dunced {} ({})", user.mention(), &user.id))
                .field("Remaining", format!("Undunce <t:{}:R>", undunce_time.timestamp()), true)
                .field("Expires", format!("<t:{}:f>", undunce_time.timestamp()), true)
            })
        );
        if let Err(e) = r1 {
            error_messages.push(format!("- {:#}", e))
        }
        if let Err(e) = r2 {
            error_messages.push(format!("- {:#}", e))
        }

        // Send followup
        ctx.say(format!("Dunced user {} ({}), they will be undunced <t:{}:R>", user_mention, user_id, undunce_time.timestamp()))
    };

    // Wait for all of our tasks to execute
    if !error_messages.is_empty() {
        ctx.say(format!("Failed to dunce user:```diff\n{}\n```", error_messages.join("\n"))).await?;
    } else {
        success_message.await?;
    }

    Ok(())
}
