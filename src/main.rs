use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serenity::{
  client::Client,
  framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
  },
  model::{
    channel::Message,
    guild::Member,
    id::{
      GuildId,
      UserId,
      RoleId,
    },
    user::User,
  },
  prelude::{Context, EventHandler},
};
use std::fs::File;

group!({
  name: "general",
  options: {},
  commands: [
    banonjoin,
    cancelban,
    pit,
    pitonjoin,
    unpit,
  ],
});

struct Handler;

impl EventHandler for Handler {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ConfigSchema {
  discord: DiscordConfig,
  mysql: SqlConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DiscordConfig {
  guild_id: u64,
  admin_roles: Vec<u64>,
  admin_users: Vec<u64>,
  token: String,
  ban_msg: String,
  pit_msg: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SqlConfig {
  username: String,
  password: String,
  endpoint: String,
  port: u16,
  database: String,
}

lazy_static!{
  static ref CONFIG: ConfigSchema = get_config();
}

fn get_config() -> ConfigSchema {
  let f = File::open("./config.yaml").unwrap();

  serde_yaml::from_reader(&f).unwrap()
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
fn banonjoin(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn cancelban(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn pit(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn pitonjoin(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn unpit(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}