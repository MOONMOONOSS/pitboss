#[derive(Queryable)]
pub struct User {
  pub id: u64,
  pub banned: bool,
  pub pitted: bool,
  pub moderator: u64,
}
