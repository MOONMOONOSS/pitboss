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
    macros::{command, group},
    Args, CommandResult, StandardFramework,
  },
  model::{
    channel::Message,
    guild::Member,
    id::{GuildId, UserId, RoleId},
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
}

lazy_static!{
  static ref CONFIG: ConfigSchema = get_config();
  static ref POOL: Pool<ConnectionManager<MysqlConnection>> = establish_connection();
}

fn add_ban(id: u64, moderator: u64) -> UserModel {
  use schema::pitboss;

  let new_usr = NewUserBan {
    id,
    banned: true,
    moderator,
  };
  let conn = POOL.get().unwrap();

  diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn)
    .expect("Error saving user ban.");
  
  pitboss::table
    .order(pitboss::id.desc())
    .first(&conn)
    .unwrap()
}

fn add_pit(id: u64, moderator: u64) -> UserModel {
  use schema::pitboss;

  let new_usr = NewUserPit {
    id,
    pitted: true,
    moderator,
  };
  let conn = POOL.get().unwrap();

  diesel::insert_into(pitboss::table)
    .values(&new_usr)
    .execute(&conn)
    .expect("Error saving user ban.");
  
  pitboss::table
    .order(pitboss::id.desc())
    .first(&conn)
    .unwrap()
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

fn rem_usr(id: u64) {
  use self::schema::pitboss::dsl::*;

  let conn = POOL.get().unwrap();
  let num_deleted = diesel::delete(pitboss.filter(id.eq(id)))
    .execute(&conn)
    .expect("Error removing ban/pit");
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
fn ban(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn unban(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn pit(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}

#[command]
fn unpit(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
  Ok(())
}
