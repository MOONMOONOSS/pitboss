use serde::{
  Deserialize,
  Serialize,
};
use super::schema::pitboss;

// Diesel Models
#[derive(Queryable)]
pub struct User {
  pub id: u64,
  pub banned: bool,
  pub pitted: bool,
  pub moderator: u64,
}

#[derive(Insertable)]
#[table_name="pitboss"]
pub struct NewUserBan {
  pub id: u64,
  pub banned: bool,
  pub moderator: u64,
}

#[derive(Insertable)]
#[table_name="pitboss"]
pub struct NewUserPit {
  pub id: u64,
  pub pitted: bool,
  pub moderator: u64,
}

// SerdeYAML Models
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSchema {
  pub discord: DiscordConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscordConfig {
  pub guild_id: u64,
  pub admin_roles: Vec<u64>,
  pub admin_users: Vec<u64>,
  pub token: String,
  pub ban_msg: String,
  pub pit_msg: String,
}
