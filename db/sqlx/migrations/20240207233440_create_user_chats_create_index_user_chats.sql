CREATE TABLE IF NOT EXISTS user_chats (
    id SERIAL PRIMARY KEY,
    message TEXT NOT NULL,
    -- Recipient of this message
    receiver_id INTEGER NOT NULL,
    -- Since I don't know how to implement fuzzy search efficiently without this, I just add this.
    receiver_display_name text NOT NULL,
    -- Owner of this message
    owner_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (receiver_display_name) REFERENCES users(display_name) ON UPDATE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS user_chats_owner_id_receiver_id_unique ON user_chats (owner_id, receiver_id);
CREATE INDEX IF NOT EXISTS user_chats_messages ON user_chats (message);
CREATE INDEX IF NOT EXISTS user_chats_created_at_owner_id ON user_chats (created_at, owner_id);
