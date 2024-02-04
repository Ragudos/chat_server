CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE TYPE gender AS ENUM ('male', 'female', 'other');
CREATE TYPE user_status AS ENUM ('active', 'suspended', 'deleted');

ALTER TABLE users
ADD COLUMN biography text,
ADD COLUMN creation_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN last_login_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN status user_status NOT NULL DEFAULT 'active',
ADD COLUMN gender gender NOT NULL DEFAULT 'other',
ADD COLUMN role user_role NOT NULL DEFAULT 'user';

CREATE INDEX IF NOT EXISTS creation_date_idx ON "users" (creation_date);
CREATE INDEX IF NOT EXISTS gender_idx ON "users" (gender);
CREATE INDEX IF NOT EXISTS role_idx ON "users" (role);
CREATE INDEX IF NOT EXISTS status_idx ON "users" (status);
CREATE INDEX IF NOT EXISTS last_login_date_idx ON "users" (last_login_date);
