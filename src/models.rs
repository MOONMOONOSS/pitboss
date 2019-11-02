use serde::{
  Deserialize,
  Serialize,
};

#[derive(Queryable)]
pub struct User {
  pub id: u64,
  pub banned: bool,
  pub pitted: bool,
  pub moderator: u64,
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
  pub token: String,
  pub ban_msg: String,
  pub pit_msg: String,
}
