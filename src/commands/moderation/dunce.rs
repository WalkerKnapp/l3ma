use anyhow::anyhow;
use chrono::{DateTime, Months, TimeDelta, Utc};
use poise::serenity_prelude::User;
use poise::serenity_prelude::Mentionable;

use luma1_data::entity::prelude::*;
use luma1_data::sea_orm::{EntityTrait, Set, sea_query};
use serenity::all::{RoleId};

use crate::context::Context;

#[macro_export]
macro_rules! ignore {
    ($x:expr) => {
        ();
    }
}

#[macro_export]
macro_rules! join_and_accumulate_errors {
    ($x:expr, $( $y:expr ),*) => {
        let errors = tokio::join!($($y),*);
        $(
            ignore!($y);
            if let Err(e) = errors.${index()} {
               $x.push(format!("- {:#}", e));
            }
        )*
    }
}

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

    // Check for an existing dunce
    let currently_dunced = DunceInstants::find_by_id(user.id.get() as i64)
        .one(&ctx.data().db).await?.is_some();

    let mut error_messages: Vec<String> = vec![];

    // Manage the user's roles if they are:
    // - Not already dunced
    // - In the server
    if !currently_dunced && let Some(member) = ctx.author_member().await {
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

        join_and_accumulate_errors!(error_messages,
                // Queue recording user's roles in DB
                DunceStoredRoles::insert_many(role_active_models)
                .on_empty_do_nothing()
                .exec(&ctx.data().db),

                // Queue removing user's roles
                member.remove_roles(ctx.http(), &roles_to_remove),

                // Queue adding the dunce role
                member.add_role(ctx.http(), P2SR_DUNCE_ROLE)
            );
    }

    // Update/insert undunce time in DB and send a report in the action log
    join_and_accumulate_errors!(error_messages,
        // Queue inserting undunce time in DB (updating if it already exists)
        DunceInstants::insert(luma1_data::entity::dunce_instants::ActiveModel {
            user_id: Set(user.id.into()),
            undunce_instant: Set(undunce_time)
        }).on_conflict(
            sea_query::OnConflict::column(luma1_data::entity::dunce_instants::Column::UserId)
                .update_column(luma1_data::entity::dunce_instants::Column::UndunceInstant)
                .to_owned()
        ).exec(&ctx.data().db),

        // Queue mod actions notification
        super::send_mod_action_log(ctx.http(), ctx.author().clone(), move |embed| {
            embed.description(
                format!("{} {} ({})",
                    if currently_dunced { "Updated dunce time for" } else { "Dunced" },
                    user.mention(),
                    &user.id
                )
            )
            .field("Remaining", format!("Undunce <t:{}:R>", undunce_time.timestamp()), true)
            .field("Expires", format!("<t:{}:f>", undunce_time.timestamp()), true)
        })
    );

    if !error_messages.is_empty() {
        ctx.say(format!("Failed to dunce user:```diff\n{}\n```", error_messages.join("\n"))).await?;
    } else {
        if currently_dunced {
            ctx.say(format!("Updated dunce time for user {} ({}), they will be undunced <t:{}:R>", user_mention, user_id, undunce_time.timestamp())).await?;
        } else {
            ctx.say(format!("Dunced user {} ({}), they will be undunced <t:{}:R>", user_mention, user_id, undunce_time.timestamp())).await?;
        }
    }

    Ok(())
}
