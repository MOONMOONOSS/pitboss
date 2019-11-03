#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use diesel::{
  prelude::*,
  mysql::MysqlConnection,
  r2d2::{ConnectionManager, Pool},
  RunQueryDsl,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use self::models::{
  ConfigSchema,
  NewUserBan,
  NewUserPit,
  User as UserModel,
};
use serenity::{
  client::Client,
  framework::standard::{
    macros::{command, check, group},
    Args, CommandResult, Check, CheckResult, CommandOptions, StandardFramework,
  },
  model::{
    channel::Message,
    gateway::Ready,
    guild::Member,
    id::{GuildId, UserId, RoleId},
    user::User,
  },
  prelude::{Context, EventHandler},
  utils::Colour,
};
use std::{
  env,
  fs::File,
  result::Result,
};

group!({
  name: "general",
  options: {},
  commands: [
    ban,
    unban,
    pit,
    unpit,
  ],
});

struct Handler;

impl EventHandler for Handler {
  fn guild_member_addition(&self, context: Context, guild_id: GuildId, new_member: Member) {
    use self::schema::pitboss::dsl::*;

    // Stop execution if the user isn't joining the target guild
    if guild_id != GuildId(CONFIG.discord.guild_id) { return }

    println!("User {} has joined Guild {}", new_member.user_id(), guild_id);

    let conn = POOL.get().unwrap();
    let user_id = *new_member
      .user_id()
      .as_u64();
    let res = pitboss
      .filter(id.eq(user_id))
      .load::<UserModel>(&conn)
      .expect("Error loading user info");
    
    match res.len() {
      0 => println!("No records found for {}", user_id),
      _ => {
        println!("HEY! Record found for {}!", user_id);
      }
    }
  }

  fn ready(&self, _: Context, ready: Ready) {
    println!("{} reporting for duty!", ready.user.name);
  }
}

lazy_static!{
  static ref CONFIG: ConfigSchema = get_config();
  static ref POOL: Pool<ConnectionManager<MysqlConnection>> = establish_connection();
}

const EMBED_FOOTER: &str = "This is an automated message | Bot by Dunkel#0001";

fn add_ban(id: u64, moderator: u64) -> Result<UserModel, diesel::result::Error> {
  use schema::pitboss;

  let new_usr = NewUserBan {
    id,
    banned: true,
    moderator,
  };
  let conn = POOL.get().unwrap();

  r#try!(diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn));
  
  pitboss::table
    .order(pitboss::id.desc())
    .first(&conn)
}

fn add_pit(id: u64, moderator: u64) -> Result<UserModel, diesel::result::Error> {
  use schema::pitboss;

  let new_usr = NewUserPit {
    id,
    pitted: true,
    moderator,
  };
  let conn = POOL.get().unwrap();

  r#try!(diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn));
  
  pitboss::table
    .order(pitboss::id.desc())
    .first(&conn)
}

fn establish_connection() -> Pool<ConnectionManager<MysqlConnection>> {
  dotenv().ok();

  let db_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL env var must be set");
  let manager = ConnectionManager::<MysqlConnection>::new(db_url);
  Pool::builder()
    .build(manager)
    .expect("Failed to create pool")
}

fn get_config() -> ConfigSchema {
  let f = File::open("./config.yaml").unwrap();

  serde_yaml::from_reader(&f).unwrap()
}

fn rem_usr(id: u64) -> Result<(), diesel::result::Error> {
  use self::schema::pitboss::dsl::*;

  let conn = POOL.get().unwrap();
  r#try!(diesel::delete(pitboss.filter(id.eq(id)))
    .execute(&conn));

  Ok(())
}

#[check]
#[name = "Admin"]
fn is_authorized_usr(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
  let mut is_admin = false;
  // Checks if the issuing user has one of the admin roles defined in the config
  'role_check: for role in &CONFIG.discord.admin_roles {
    if msg.member(&ctx.cache)
      .unwrap()
      .roles
      .contains(&RoleId(*role)) {
        is_admin = true;

        break 'role_check
      }
  }
  // Don't perform this admin check if we know they are admin already
  if !is_admin {
    // Checks if the issuing user is one of the authorized users as defined in the config
    'user_check: for user in &CONFIG.discord.admin_users {
      if msg.author.id == UserId(*user) {
        is_admin = true;

        break 'user_check
      }
    }
  }
  // Checks if the issuing user issued the command from the correct guild
  if msg.guild_id.unwrap() != GuildId(CONFIG.discord.guild_id) {
    return CheckResult::new_log("User issued command from wrong guild")
  }

  match is_admin {
    true => return true.into(),
    false => return CheckResult::new_log("User lacked permission")
  }
}

#[check]
#[name = "UserMention"]
fn is_usr_mention(_: &mut Context, msg: &Message, args: &mut Args, _: &CommandOptions) -> CheckResult {
  let usr = args
    .single_quoted::<String>()
    .unwrap();
  let prefix = usr
    .get(0..=1)
    .unwrap()
    .to_string();
  let postfix = usr
    .chars()
    .last()
    .unwrap()
    .to_string();
  // Rewind so other functions can access the args after we finish
  args.rewind();
  // Parse the user string into a UserId
  let usr = mention_to_user_id(args);
  
  // Is the argument a valid @ mention and not a self @ mention?
  match prefix == *"<@" && postfix == *">" && msg.author.id != usr {
    true => return true.into(),
    false => return CheckResult::new_log("Supplied arguments doesn't include a mentioned user")
  }
}

fn mention_to_user_id(args: &mut Args) -> UserId {
  let mut usr = args
  .single_quoted::<String>()
  .unwrap();

  args.rewind();
  usr.retain(|c| c.to_string().parse::<i8>().is_ok());

  UserId(usr.parse::<u64>().unwrap())
}

fn main() {
  // Bot login
  let mut client: Client =
    Client::new(&CONFIG.discord.token, Handler).expect("Error creating client");

  client.with_framework(
    StandardFramework::new()
      .configure(|c| c.prefix("!"))
      .group(&GENERAL_GROUP),
  );

  // Start listening for events, single shard. Shouldn't need more than one shard
  if let Err(why) = client.start() {
    println!("An error occurred while running the client: {:?}", why);
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, UserMention)]
fn ban(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, UserMention)]
fn unban(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, UserMention)]
fn pit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match add_pit(*usr.as_u64(), *msg.author.id.as_u64()) {
    Ok(v) => {
      let member = GuildId(CONFIG.discord.guild_id).member(&ctx, *usr.as_u64());

      match member {
        Ok(mut me) => {
          // Add pit role to user
          match me.add_role(&ctx, CONFIG.discord.pit_role) {
            Ok(_) => (),
            Err(e) => {
              msg.channel_id.send_message(&ctx, |m| {
                m.content(format!("**TRACE LOG**\n```{:?}```", e));
                m.embed(|e| {
                  e.title("Pitting failed!");
                  e.description(format!("<@{}> has NOT been pitted.\nPlease try again later", *usr.as_u64()));
                  e.color(Colour::new(0xFF0000));
                  e.footer(|f| {
                    f.text(EMBED_FOOTER)
                  })
                })
              })?;

              rem_usr(*usr.as_u64())?;
              
              return Ok(())
            },
          }

          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been pitted.", *usr.as_u64()));
              e.color(Colour::new(0x00960C));
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          // Direct message user to explain they have been pitted.
          let usr_obj = me
            .user_id()
            .to_user(&ctx)?;
          usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.pit_msg.title);
              e.description(&CONFIG.discord.pit_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.pit_msg.color));
              e.field(&CONFIG.discord.pit_msg.attract, &CONFIG.discord.pit_msg.warning, true);
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          return Ok(())
        },
        Err(e) => {
          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Pitboss Success");
              e.description(format!("<@{}> has been added to the Pitboss watchlist.", *usr.as_u64()));
              e.field("Pitboss checks users joining the server and takes action if a match is found.", "You will be alerted if this user joins the server.", true);
              e.color(Colour::new(0xE79900));
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          return Ok(())
        },
      }
    },
    Err(e) => {
      println!("Error adding pit: {:?}", e);

      msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("**TRACE LOG**\n```{:?}```", e));
        m.embed(|e| {
          e.title("Pitting failed!");
          e.description(format!("<@{}> has NOT been pitted.\nPlease try again later", *usr.as_u64()));
          e.color(Colour::new(0xFF0000));
          e.footer(|f| {
            f.text(EMBED_FOOTER)
          })
        })
      })?;

      return Ok(())
    }
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, UserMention)]
fn unpit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match rem_usr(*usr.as_u64()) {
    Ok(v) => {
      let member = GuildId(CONFIG.discord.guild_id).member(&ctx, *usr.as_u64());

      match member {
        Ok(mut me) => {

          // Remove pit role from user
          match me.remove_role(&ctx, CONFIG.discord.pit_role) {
            Ok(_) => (),
            Err(e) => {
              msg.channel_id.send_message(&ctx, |m| {
                m.content(format!("**TRACE LOG**\n```{:?}```", e));
                m.embed(|e| {
                  e.title("Pit removal failed!");
                  e.field("User may still be pitted", "Please try again later", true);
                  e.color(Colour::new(0xFF0000));
                  e.footer(|f| {
                    f.text(EMBED_FOOTER)
                  })
                })
              })?;

              add_pit(*usr.as_u64(), *msg.author.id.as_u64())?;
              
              return Ok(())
            },
          }

          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been un-pitted.", *usr.as_u64()));
              e.color(Colour::new(0x00960C));
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          // Direct message user to explain they have been released from the pit.
          let usr_obj = me
            .user_id()
            .to_user(&ctx)?;
          usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.unpit_msg.title);
              e.description(&CONFIG.discord.unpit_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.unpit_msg.color));
              e.field(&CONFIG.discord.unpit_msg.attract, &CONFIG.discord.unpit_msg.warning, true);
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          return Ok(())
        },
        Err(e) => {
          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been removed from the Pitboss watchlist.", *usr.as_u64()));
              e.color(Colour::new(0x00960C));
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          return Ok(())
        },
      }
    },
    Err(e) => {
      println!("Error removing pit: {:?}", e);

      msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("**TRACE LOG**\n```{:?}```", e));
        m.embed(|e| {
          e.title("Pit removal failed!");
          e.field("User may still be pitted", "Please try again later", true);
          e.color(Colour::new(0xFF0000));
          e.footer(|f| {
            f.text(EMBED_FOOTER)
          })
        })
      })?;

      return Ok(())
    },
  }
}
