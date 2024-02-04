CREATE TABLE IF NOT EXISTS "user_credentials" (
    id serial PRIMARY KEY,
    user_id int NOT NULL REFERENCES "users" (id) ON DELETE CASCADE,
    email text,
    password_hash text NOT NULL
);