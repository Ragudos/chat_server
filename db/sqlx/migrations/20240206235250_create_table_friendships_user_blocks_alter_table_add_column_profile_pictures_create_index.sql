CREATE TYPE closeness AS ENUM ('close', 'acquaintance', 'friend', 'family', 'partner', 'other');

CREATE TABLE IF NOT EXISTS friendships (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    friend_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closeness closeness NOT NULL DEFAULT 'acquaintance',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (friend_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_blocks (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    blocked_user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (blocked_user_id) REFERENCES users(id) ON DELETE CASCADE
);

ALTER TABLE users
ADD COLUMN profile_pictures text[] NOT NULL DEFAULT ARRAY[]::text[];

CREATE INDEX IF NOT EXISTS friendships_user_id_friend_id_unique ON friendships (user_id, friend_id);
CREATE INDEX IF NOT EXISTS user_blocks_user_id_blocked_user_id_unique ON user_blocks (user_id, blocked_user_id);
CREATE INDEX IF NOT EXISTS friendship_closeness_user_id_friend_id_unique ON friendships (closeness, user_id, friend_id);
