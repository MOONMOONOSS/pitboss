use super::schema::pitboss;
use serde::{Deserialize, Serialize};

// Diesel Models
#[derive(Queryable)]
pub struct User {
  pub id: u64,
  pub banned: bool,
  pub pitted: bool,
  pub moderator: u64,
}

#[derive(Insertable)]
#[table_name = "pitboss"]
pub struct NewUserBan {
  pub id: u64,
  pub banned: bool,
  pub moderator: u64,
}

#[derive(Insertable)]
#[table_name = "pitboss"]
pub struct NewUserPit {
  pub id: u64,
  pub pitted: bool,
  pub moderator: u64,
}

// SerdeYAML Models
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandSchema {
  pub enable_pitboss: bool,
  pub enable_banboss: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSchema {
  pub discord: DiscordConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscordConfig {
  pub guild_id: u64,
  pub admin_roles: Vec<u64>,
  pub admin_users: Vec<u64>,
  pub report_channel: u64,
  pub pit_role: u64,
  pub commands: CommandSchema,
  pub token: String,
  pub ban_evade_msg: Embed,
  pub ban_msg: Embed,
  pub pit_evade_msg: Embed,
  pub pit_msg: Embed,
  pub unpit_msg: Embed,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Embed {
  pub title: String,
  pub subtitle: String,
  pub color: u32,
  pub attract: String,
  pub warning: String,
}
