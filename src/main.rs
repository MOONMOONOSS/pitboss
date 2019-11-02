#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use diesel::{
  mysql::MysqlConnection,
  r2d2::{
    ConnectionManager,
    Pool,
  },
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use self::models::{
  ConfigSchema,
  DiscordConfig,
};
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
use std::{
  env,
  fs::File,
};

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

lazy_static!{
  static ref CONFIG: ConfigSchema = get_config();
  static ref CONN: Pool<ConnectionManager<MysqlConnection>> = establish_connection();
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
