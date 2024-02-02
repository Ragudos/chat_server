CREATE TABLE IF NOT EXISTS "users" (
    id serial PRIMARY KEY,
    display_name text NOT NULL,
    display_image text NOT NULL
);