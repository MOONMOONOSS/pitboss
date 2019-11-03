table! {
  pitboss (id) {
    id -> Unsigned<Bigint>,
    banned -> Bool,
    pitted -> Bool,
    moderator -> Unsigned<Bigint>,
  }
}

allow_tables_to_appear_in_same_query!(pitboss,);
