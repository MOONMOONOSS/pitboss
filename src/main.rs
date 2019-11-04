#![allow(clippy::useless_let_if_seq)]
#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use self::models::{ConfigSchema, NewUserBan, NewUserPit, User as UserModel};
use diesel::{
  mysql::MysqlConnection,
  prelude::*,
  r2d2::{ConnectionManager, Pool},
  result::{Error as DieselError, DatabaseErrorKind},
  RunQueryDsl,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::{
  client::Client,
  framework::standard::{
    macros::{check, command, group},
    Args, CheckResult, CommandOptions, CommandResult, StandardFramework,
  },
  model::{
    channel::Message,
    gateway::Ready,
    guild::Member,
    id::{ChannelId, GuildId, RoleId, UserId},
  },
  prelude::{Context, EventHandler},
  utils::Colour,
};
use std::{env, fs::File, result::Result};

group!({
  name: "general",
  options: {},
  commands: [
    ban,
    unban,
    force_unban,
    pit,
    unpit,
  ],
});

struct Handler;

impl EventHandler for Handler {
  fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
    use self::schema::pitboss::dsl::*;

    // Stop execution if the user isn't joining the target guild
    if guild_id != GuildId(CONFIG.discord.guild_id) {
      return;
    }

    println!(
      "User {} has joined Guild {}",
      new_member.user_id(),
      guild_id
    );

    let conn = POOL.get().unwrap();
    let user_id = *new_member.user_id().as_u64();
    let res = pitboss
      .filter(id.eq(user_id))
      .load::<UserModel>(&conn)
      .expect("Error loading user info");

    match res.len() {
      0 => println!("No records found for {}", user_id),
      _ => {
        println!("HEY! Record found for {}!", user_id);

        if res[0].banned {
          let usr = new_member.user_id();
          let member = GuildId(CONFIG.discord.guild_id)
            .member(&ctx, *usr.as_u64())
            .unwrap();

          let usr_obj = member.user_id().to_user(&ctx).unwrap();
          let _ = usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.ban_evade_msg.title);
              e.description(&CONFIG.discord.ban_evade_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.ban_evade_msg.color));
              e.field(
                &CONFIG.discord.ban_evade_msg.attract,
                &CONFIG.discord.ban_evade_msg.warning,
                true,
              );
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          });

          match member.ban(&ctx, &7) {
            Ok(_) => (),
            Err(e) => {
              let _ = ChannelId(CONFIG.discord.report_channel).send_message(&ctx, |m| {
                m.content(format!("**TRACE LOG**\n```{:?}```", e));
                m.embed(|e| {
                  e.title("Banning failed!");
                  e.description(format!(
                    "<@{}> is on the Banboss watchlist and wasn't banned!",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              });

              return;
            }
          }

          let _ = ChannelId(CONFIG.discord.report_channel).send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Banboss Success");
              e.description(format!(
                "<@{}> is on the Banboss watchlist and was banned.",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          });

          return;
        }
        if res[0].pitted {
          let usr = new_member.user_id();
          let mut member = GuildId(CONFIG.discord.guild_id)
            .member(&ctx, *usr.as_u64())
            .unwrap();

          // Add pit role to user
          match member.add_role(&ctx, CONFIG.discord.pit_role) {
            Ok(_) => (),
            Err(e) => {
              let _ = ChannelId(CONFIG.discord.report_channel).send_message(&ctx, |m| {
                m.content(format!("**TRACE LOG**\n```{:?}```", e));
                m.embed(|e| {
                  e.title("Pitting failed!");
                  e.description(format!(
                    "<@{}> is on the Pitboss watchlist and wasn't pitted!",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              });

              return;
            }
          }

          // Direct message user to explain they have been pitted.
          let usr_obj = member.user_id().to_user(&ctx).unwrap();
          let _ = usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.pit_evade_msg.title);
              e.description(&CONFIG.discord.pit_evade_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.pit_evade_msg.color));
              e.field(
                &CONFIG.discord.pit_evade_msg.attract,
                &CONFIG.discord.pit_evade_msg.warning,
                true,
              );
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          });
          let _ = ChannelId(CONFIG.discord.report_channel).send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Pitboss Success");
              e.description(format!(
                "<@{}> is on the Pitboss watchlist and was pitted",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          });
        }
      }
    }
  }

  fn ready(&self, _: Context, ready: Ready) {
    println!("{} reporting for duty!", ready.user.name);
  }
}

lazy_static! {
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

  diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn)?;

  pitboss::table.order(pitboss::id.desc()).first(&conn)
}

fn add_pit(id: u64, moderator: u64) -> Result<UserModel, diesel::result::Error> {
  use schema::pitboss;

  let new_usr = NewUserPit {
    id,
    pitted: true,
    moderator,
  };
  let conn = POOL.get().unwrap();

  diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn)?;

  pitboss::table.order(pitboss::id.desc()).first(&conn)
}

fn establish_connection() -> Pool<ConnectionManager<MysqlConnection>> {
  dotenv().ok();

  let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");
  let manager = ConnectionManager::<MysqlConnection>::new(db_url);
  Pool::builder()
    .build(manager)
    .expect("Failed to create pool")
}

fn get_config() -> ConfigSchema {
  let f = File::open("./config.yaml").unwrap();

  serde_yaml::from_reader(&f).unwrap()
}

fn rem_usr(_id: u64) -> Result<(), diesel::result::Error> {
  use self::schema::pitboss::dsl::*;

  let conn = POOL.get().unwrap();
  diesel::delete(pitboss.filter(id.eq(_id))).execute(&conn)?;

  Ok(())
}

#[check]
#[name = "Admin"]
fn is_authorized_usr(
  ctx: &mut Context,
  msg: &Message,
  _: &mut Args,
  _: &CommandOptions,
) -> CheckResult {
  let mut is_admin = false;
  // Checks if the issuing user has one of the admin roles defined in the config
  'role_check: for role in &CONFIG.discord.admin_roles {
    if msg
      .member(&ctx.cache)
      .unwrap()
      .roles
      .contains(&RoleId(*role))
    {
      is_admin = true;

      break 'role_check;
    }
  }
  // Don't perform this admin check if we know they are admin already
  if !is_admin {
    // Checks if the issuing user is one of the authorized users as defined in the config
    'user_check: for user in &CONFIG.discord.admin_users {
      if msg.author.id == UserId(*user) {
        is_admin = true;

        break 'user_check;
      }
    }
  }
  // Checks if the issuing user issued the command from the correct guild
  if msg.guild_id.unwrap() != GuildId(CONFIG.discord.guild_id) {
    return CheckResult::new_log("User issued command from wrong guild");
  }

  if is_admin {
    true.into()
  } else {
    CheckResult::new_log("User lacked permission")
  }
}

#[check]
#[name = "UserMention"]
fn is_usr_mention(
  _: &mut Context,
  msg: &Message,
  args: &mut Args,
  _: &CommandOptions,
) -> CheckResult {
  let usr = args.single_quoted::<String>().unwrap();
  let prefix = usr.get(0..=1).unwrap().to_string();
  let postfix = usr.chars().last().unwrap().to_string();
  // Rewind so other functions can access the args after we finish
  args.rewind();
  // Parse the user string into a UserId
  let usr = mention_to_user_id(args);

  // Is the argument a valid @ mention and not a self @ mention?
  if prefix == *"<@" && postfix == *">" && msg.author.id != usr {
    true.into()
  } else {
    CheckResult::new_log("Supplied arguments doesn't include a mentioned user")
  }
}

#[check]
#[name = "CommandEnabled"]
fn enable_check(
  ctx: &mut Context,
  msg: &Message,
  _: &mut Args,
  com_opts: &CommandOptions,
) -> CheckResult {
  let title = "boss has been disabled.";
  let mut prefix: &str = "";
  let mut should_terminate = false;

  if !CONFIG.discord.commands.enable_pitboss && com_opts.names[0].contains("pit") {
    prefix = "Pit";
    should_terminate = true;
  }
  if !CONFIG.discord.commands.enable_banboss && com_opts.names[0].contains("ban") {
    prefix = "Ban";
    should_terminate = true;
  }

  if should_terminate {
    let _ = msg.channel_id.send_message(&ctx, |m| {
      m.embed(|e| {
        e.title("Command Disabled");
        e.description(prefix.to_owned() + title);
        e.color(Colour::new(0x00FF_0000));
        e.footer(|f| f.text(EMBED_FOOTER))
      })
    });

    return CheckResult::new_log("Pitboss is disabled.");
  }

  true.into()
}

fn mention_to_user_id(args: &mut Args) -> UserId {
  let mut usr = args.single_quoted::<String>().unwrap();

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
#[checks(Admin, CommandEnabled, UserMention)]
fn ban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match add_ban(*usr.as_u64(), *msg.author.id.as_u64()) {
    Ok(_) => {
      let member = GuildId(CONFIG.discord.guild_id).member(&ctx, *usr.as_u64());

      match member {
        Ok(me) => {
          // Direct message user to explain they have been banned.
          // MUST happen before banning the user since we can't send messages to just anybody.
          let usr_obj = me.user_id().to_user(&ctx)?;
          usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.ban_msg.title);
              e.description(&CONFIG.discord.ban_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.ban_msg.color));
              e.field(
                &CONFIG.discord.ban_msg.attract,
                &CONFIG.discord.ban_msg.warning,
                true,
              );
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          match me.ban(&ctx, &7) {
            Ok(_) => {}
            Err(e) => {
              println!("Error adding ban: {:?}", e);

              msg.channel_id.send_message(&ctx, |m| {
                m.content(format!("**TRACE LOG**\n```{:?}```", e));
                m.embed(|e| {
                  e.title("Ban failed!");
                  e.description(format!(
                    "<@{}> has NOT been banned.\nPlease try again later",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              })?;

              rem_usr(*usr.as_u64())?;

              return Ok(());
            }
          }

          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been banned.", *usr.as_u64()));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          Ok(())
        }
        Err(_) => {
          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Banboss Success");
              e.description(format!("<@{}> has been added to the Banboss watchlist.", *usr.as_u64()));
              e.field("Banboss checks users joining the server and automatically bans if a match is found.", "You will be alerted if this user joins the server.", true);
              e.color(Colour::new(0x00E7_9900));
              e.footer(|f| {
                f.text(EMBED_FOOTER)
              })
            })
          })?;

          Ok(())
        }
      }
    }
    Err(e) => {
      match e {
        DieselError::DatabaseError(err_type, _) => {
          match err_type {
            DatabaseErrorKind::UniqueViolation => {
              msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                  e.title("User already banned");
                  e.description(format!(
                    "<@{}> is already banned.",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              })?;
            },
            _ => {},
          }
        },
        _ => {
          println!("Error adding ban: {:?}", e);
          msg.channel_id.send_message(&ctx, |m| {
            m.content(format!("**TRACE LOG**\n```{:?}```", e));
            m.embed(|e| {
              e.title("Banning failed!");
              e.description(format!(
                "<@{}> has NOT been banned.\nPlease try again later",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x00FF_0000));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;
        }
      }

      Ok(())
    }
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, CommandEnabled, UserMention)]
fn force_unban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match rem_usr(*usr.as_u64()) {
    Ok(_) => {
      // Reply to moderator
      msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
          e.title("Success");
          e.description(format!("<@{}> has been pardoned.", *usr.as_u64()));
          e.color(Colour::new(0x0000_960C));
          e.footer(|f| f.text(EMBED_FOOTER))
        })
      })?;

      Ok(())
    }
    Err(e) => {
      println!("Error removing ban: {:?}", e);

      msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("**TRACE LOG**\n```{:?}```", e));
        m.embed(|e| {
          e.title("Pardon failed!");
          e.description(format!(
            "<@{}> has NOT been pardoned.\nPlease try again later",
            *usr.as_u64()
          ));
          e.color(Colour::new(0x00FF_0000));
          e.footer(|f| f.text(EMBED_FOOTER))
        })
      })?;

      Ok(())
    }
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, CommandEnabled, UserMention)]
fn unban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match rem_usr(*usr.as_u64()) {
    Ok(_) => {
      let guild = GuildId(CONFIG.discord.guild_id);

      match guild.unban(&ctx, usr) {
        Ok(_) => {}
        Err(e) => {
          println!("Error removing ban: {:?}", e);

          msg.channel_id.send_message(&ctx, |m| {
            m.content(format!("**TRACE LOG**\n```{:?}```", e));
            m.embed(|e| {
              e.title("Pardon failed!");
              e.description(format!(
                "<@{}> has NOT been pardoned.\nPlease try again later",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x00FF_0000));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          add_ban(*usr.as_u64(), *msg.author.id.as_u64())?;

          return Ok(());
        }
      }

      // Reply to moderator
      msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
          e.title("Success");
          e.description(format!("<@{}> has been pardoned.", *usr.as_u64()));
          e.color(Colour::new(0x0000_960C));
          e.footer(|f| f.text(EMBED_FOOTER))
        })
      })?;

      Ok(())
    }
    Err(e) => {
      println!("Error removing ban: {:?}", e);

      msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("**TRACE LOG**\n```{:?}```", e));
        m.embed(|e| {
          e.title("Pardon failed!");
          e.description(format!(
            "<@{}> has NOT been pardoned.\nPlease try again later",
            *usr.as_u64()
          ));
          e.color(Colour::new(0x00FF_0000));
          e.footer(|f| f.text(EMBED_FOOTER))
        })
      })?;

      Ok(())
    }
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, CommandEnabled, UserMention)]
fn pit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match add_pit(*usr.as_u64(), *msg.author.id.as_u64()) {
    Ok(_) => {
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
                  e.description(format!(
                    "<@{}> has NOT been pitted.\nPlease try again later",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              })?;

              rem_usr(*usr.as_u64())?;

              return Ok(());
            }
          }

          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been pitted.", *usr.as_u64()));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          // Direct message user to explain they have been pitted.
          let usr_obj = me.user_id().to_user(&ctx)?;
          usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.pit_msg.title);
              e.description(&CONFIG.discord.pit_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.pit_msg.color));
              e.field(
                &CONFIG.discord.pit_msg.attract,
                &CONFIG.discord.pit_msg.warning,
                true,
              );
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          Ok(())
        }
        Err(_) => {
          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Pitboss Success");
              e.description(format!(
                "<@{}> has been added to the Pitboss watchlist.",
                *usr.as_u64()
              ));
              e.field(
                "Pitboss checks users joining the server and takes action if a match is found.",
                "You will be alerted if this user joins the server.",
                true,
              );
              e.color(Colour::new(0x00E7_9900));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          Ok(())
        }
      }
    }
    Err(e) => {
      match e {
        DieselError::DatabaseError(err_type, _) => {
          match err_type {
            DatabaseErrorKind::UniqueViolation => {
              msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                  e.title("User already pitted");
                  e.description(format!(
                    "<@{}> is already in THE PIT.",
                    *usr.as_u64()
                  ));
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              })?;
            },
            _ => {},
          }
        },
        _ => {
          println!("Error adding pit: {:?}", e);
          msg.channel_id.send_message(&ctx, |m| {
            m.content(format!("**TRACE LOG**\n```{:?}```", e));
            m.embed(|e| {
              e.title("Pitting failed!");
              e.description(format!(
                "<@{}> has NOT been pitted.\nPlease try again later",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x00FF_0000));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;
        }
      }

      Ok(())
    }
  }
}

#[command]
#[min_args(1)]
#[max_args(1)]
#[only_in(guilds)]
#[bucket = "pitboss"]
#[checks(Admin, CommandEnabled, UserMention)]
fn unpit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let usr = mention_to_user_id(&mut args);

  match rem_usr(*usr.as_u64()) {
    Ok(_) => {
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
                  e.color(Colour::new(0x00FF_0000));
                  e.footer(|f| f.text(EMBED_FOOTER))
                })
              })?;

              add_pit(*usr.as_u64(), *msg.author.id.as_u64())?;

              return Ok(());
            }
          }

          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!("<@{}> has been un-pitted.", *usr.as_u64()));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          // Direct message user to explain they have been released from the pit.
          let usr_obj = me.user_id().to_user(&ctx)?;
          usr_obj.direct_message(&ctx, |m| {
            m.embed(|e| {
              e.title(&CONFIG.discord.unpit_msg.title);
              e.description(&CONFIG.discord.unpit_msg.subtitle);
              e.color(Colour::new(CONFIG.discord.unpit_msg.color));
              e.field(
                &CONFIG.discord.unpit_msg.attract,
                &CONFIG.discord.unpit_msg.warning,
                true,
              );
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          Ok(())
        }
        Err(_) => {
          // Reply to moderator
          msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              e.title("Success");
              e.description(format!(
                "<@{}> has been removed from the Pitboss watchlist.",
                *usr.as_u64()
              ));
              e.color(Colour::new(0x0000_960C));
              e.footer(|f| f.text(EMBED_FOOTER))
            })
          })?;

          Ok(())
        }
      }
    }
    Err(e) => {
      println!("Error removing pit: {:?}", e);

      msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("**TRACE LOG**\n```{:?}```", e));
        m.embed(|e| {
          e.title("Pit removal failed!");
          e.field("User may still be pitted", "Please try again later", true);
          e.color(Colour::new(0x00FF_0000));
          e.footer(|f| f.text(EMBED_FOOTER))
        })
      })?;

      Ok(())
    }
  }
}
