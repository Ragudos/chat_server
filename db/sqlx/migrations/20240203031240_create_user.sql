CREATE TABLE IF NOT EXISTS "users" (
    id serial PRIMARY KEY,
    display_name text NOT NULL UNIQUE,
    display_image text
);