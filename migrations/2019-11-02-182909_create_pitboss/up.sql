CREATE TABLE pitboss (
  id SERIAL PRIMARY KEY,
  banned BOOLEAN NOT NULL DEFAULT FALSE,
  pitted BOOLEAN NOT NULL DEFAULT FALSE,
  moderator BIGINT UNSIGNED NOT NULL
)