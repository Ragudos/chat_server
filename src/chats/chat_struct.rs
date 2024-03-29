use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use time::{Date, OffsetDateTime};

use crate::{db::Db, user::user_struct::Gender, utils::get_placeholder_display_image};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chat {
    pub id: i32,
    #[serde(rename = "receiverName")]
    pub receiver_name: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: i32,
    #[serde(rename = "receiverAvatar")]
    pub receiver_avatar: String,
    #[serde(rename = "senderName")]
    pub sender_name: String,
    #[serde(rename = "senderId")]
    pub sender_id: i32,
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: String,
    #[serde(rename = "created_at")]
    pub created_at: OffsetDateTime,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    #[serde(rename = "isReceiverMessage")]
    pub is_receiver_message: bool,
    pub message: String,
    #[serde(rename = "senderId")]
    pub sender_id: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagesInChat {
    /// For ex. "sender_id=1&receiver_id=2"
    pub id: String,
    #[serde(rename = "receiverName")]
    pub receiver_name: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: i32,
    #[serde(rename = "receiverAvatar")]
    pub receiver_avatar: String,
    #[serde(rename = "senderName")]
    pub sender_name: String,
    #[serde(rename = "senderId")]
    pub sender_id: i32,
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: String,
    pub messages: Vec<ChatMessage>
}

impl MessagesInChat {
    pub fn new(
        id: String,
        receiver_name: String,
        receiver_id: i32,
        receiver_avatar: String,
        sender_name: String,
        sender_id: i32,
        sender_avatar: String,
        messages: Vec<ChatMessage>
    ) -> Self {
        Self {
            id,
            receiver_name,
            receiver_id,
            receiver_avatar,
            sender_name,
            sender_id,
            sender_avatar,
            messages
        }
    }
}

impl Chat {
    pub fn new(
        id: i32,
        receiver_name: String,
        receiver_id: i32,
        receiver_avatar: String,
        sender_name: String,
        sender_id: i32,
        sender_avatar: String,
        created_at: OffsetDateTime,
        message: String,
    ) -> Self {
        Self {
            id,
            receiver_name,
            receiver_id,
            receiver_avatar,
            sender_name,
            sender_id,
            sender_avatar,
            created_at,
            message,
        }
    }

    pub async fn save_chat(
        db: &mut Connection<Db>,
        sender_id: &i32,
        receiver_id: &i32,
        receiver_display_name: &String,
        message: &String,
    ) -> Result<OffsetDateTime, sqlx::Error> {
        let record = sqlx::query!(
            r#"
            INSERT INTO user_chats (owner_id, receiver_id, message, receiver_display_name)
            VALUES ($1, $2, $3, $4)
            RETURNING created_at;
            "#,
            sender_id,
            receiver_id,
            message,
            receiver_display_name
        ).fetch_one(&mut ***db).await?;

        Ok(record.created_at)
    }

    pub async fn get_messages(
        db: &mut Connection<Db>,
        owner_id: &i32,
        receiver_id: &i32,
    ) -> Result<MessagesInChat, sqlx::Error> {
        let user_chats = sqlx::query! (
            r#"
            SELECT * FROM user_chats
            WHERE (owner_id = $1 AND receiver_id = $2)
            OR (owner_id = $2 AND receiver_id = $1)
            ORDER BY created_at ASC
            "#,
            owner_id,
            receiver_id
        ).fetch_all(&mut ***db).await?;
        
        let user_chat_owner = sqlx::query! (
            r#"
                SELECT
                id,
                display_name,
                display_image,
                gender as "gender: Gender"
                FROM users
                WHERE id = $1
            "#,
            owner_id
        ).fetch_one(&mut ***db).await?;

        let user_chat_receiver = sqlx::query! (
            r#"
                SELECT
                id,
                display_name,
                display_image,
                gender as "gender: Gender"
                FROM users
                WHERE id = $1
            "#,
            receiver_id
        ).fetch_one(&mut ***db).await?;

        let mut messages = Vec::new();

        for chat in user_chats {
            let date = chat.created_at.date();

            messages.push(ChatMessage {
                is_receiver_message: chat.receiver_id != *receiver_id,
                message: chat.message,
                sender_id: chat.owner_id,
                created_at: format!("{}-{}-{} at {}:{}:{}", date.year(), date.month(), date.day(), chat.created_at.hour(), chat.created_at.minute(), chat.created_at.second()),
                receiver_id: chat.receiver_id,
            });
        }

        let user_chat_user_display_image = get_placeholder_display_image(user_chat_owner.display_image.as_ref(), &user_chat_owner.gender);
        let user_chat_receiver_display_image = get_placeholder_display_image(user_chat_receiver.display_image.as_ref(), &user_chat_receiver.gender);

        let messages_in_chat = MessagesInChat::new(
            format!("sender_id={}&receiver_id={}", owner_id, receiver_id),
            user_chat_receiver.display_name,
            user_chat_receiver.id,
            user_chat_receiver_display_image,
            user_chat_owner.display_name,
            user_chat_owner.id,
            user_chat_user_display_image,
            messages,
        );

        Ok(messages_in_chat)
    }

    pub async fn get_user_chats(
        db: &mut Connection<Db>,
        user_id: &i32,
        search: &String,
    ) -> Result<Vec<Chat>, sqlx::Error> {
        if search.is_empty() {
            let chats = sqlx::query!(
                r#"
                WITH LatestChats AS (
                    SELECT
                    id,
                    message,
                    receiver_id,
                    owner_id,
                    created_at,
                    receiver_display_name,
                    ROW_NUMBER() OVER (
                        PARTITION BY GREATEST(receiver_id, owner_id),
                        LEAST(receiver_id, owner_id)
                        ORDER BY created_at DESC
                    ) AS rn
                    FROM user_chats
                    WHERE owner_id = $1 OR receiver_id = $1
                )
                SELECT *
                FROM LatestChats
                WHERE rn = 1
                ORDER BY created_at DESC;
                "#,
                user_id,
            ).fetch_all(&mut ***db).await?;

            let mut user_chats = Vec::new();

            for chat in chats {
                let user_chat_owner = sqlx::query! (
                    r#"
                        SELECT
                        id,
                        display_name,
                        display_image,
                        gender as "gender: Gender"
                        FROM users
                        WHERE id = $1
                    "#,
                    chat.owner_id
                ).fetch_one(&mut ***db).await?;

                let user_chat_receiver = sqlx::query! (
                    r#"
                        SELECT
                        id,
                        display_name,
                        display_image,
                        gender as "gender: Gender"
                        FROM users
                        WHERE id = $1
                    "#,
                    chat.receiver_id
                ).fetch_one(&mut ***db).await?;

                let user_chat_owner_display_image = get_placeholder_display_image(user_chat_owner.display_image.as_ref(), &user_chat_owner.gender);
                let user_chat_receiver_display_image = get_placeholder_display_image(user_chat_receiver.display_image.as_ref(), &user_chat_receiver.gender);

                user_chats.push(Chat::new(
                    chat.id,
                    user_chat_receiver.display_name,
                    user_chat_receiver.id,
                    user_chat_receiver_display_image,
                    user_chat_owner.display_name,
                    user_chat_owner.id,
                    user_chat_owner_display_image,
                    chat.created_at,
                    chat.message,
                ));
            }

            Ok(user_chats)
        } else {
            let chats = sqlx::query!(
                r#"
                WITH LatestChats AS (
                    SELECT
                    id,
                    message,
                    receiver_id,
                    owner_id,
                    created_at,
                    receiver_display_name,
                    ROW_NUMBER() OVER (
                        PARTITION BY GREATEST(receiver_id, owner_id),
                        LEAST(receiver_id, owner_id)
                        ORDER BY created_at DESC
                    ) AS rn
                    FROM user_chats
                    WHERE owner_id = $1 OR receiver_id = $1 AND
                    similarity(receiver_display_name, $2) > 0.2
                )
                SELECT *
                FROM LatestChats
                WHERE rn = 1
                ORDER BY similarity(receiver_display_name, $2) DESC;
                "#,
                user_id,
                search
            ).fetch_all(&mut ***db).await?;

            let mut user_chats = Vec::new();

            for chat in chats {
                let user_chat_owner = sqlx::query! (
                    r#"
                        SELECT
                        id,
                        display_name,
                        display_image,
                        gender as "gender: Gender"
                        FROM users
                        WHERE id = $1
                    "#,
                    chat.owner_id
                ).fetch_one(&mut ***db).await?;

                let user_chat_receiver = sqlx::query! (
                    r#"
                        SELECT
                        id,
                        display_name,
                        display_image,
                        gender as "gender: Gender"
                        FROM users
                        WHERE id = $1
                    "#,
                    chat.receiver_id
                ).fetch_one(&mut ***db).await?;

                let user_chat_owner_display_image = get_placeholder_display_image(user_chat_owner.display_image.as_ref(), &user_chat_owner.gender);
                let user_chat_receiver_display_image = get_placeholder_display_image(user_chat_receiver.display_image.as_ref(), &user_chat_receiver.gender);

                user_chats.push(Chat::new(
                    chat.id,
                    user_chat_receiver.display_name,
                    user_chat_receiver.id,
                    user_chat_receiver_display_image,
                    user_chat_owner.display_name,
                    user_chat_owner.id,
                    user_chat_owner_display_image,
                    chat.created_at,
                    chat.message,
                ));
            }

            Ok(user_chats)
        }

        
    }

    pub async fn get_latest_chat(
        db: &mut Connection<Db>,
        user_id: &i32,
        receiver_id: &i32,
    ) -> Result<Chat, sqlx::Error> {
        let mut transaction = db.begin().await?;

        let latest_chat = sqlx::query!(
            r#"
            SELECT * FROM user_chats
            WHERE (owner_id = $1 AND receiver_id = $2)
            OR (owner_id = $2 AND receiver_id = $1)
            AND created_at = (
                SELECT MAX(created_at) FROM user_chats
                WHERE (owner_id = $1 AND receiver_id = $2)
                OR (owner_id = $2 AND receiver_id = $1)
            )
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id,
            receiver_id
        ).fetch_one(&mut *transaction).await?;
        
        let user_chat_owner = sqlx::query! (
            r#"
                SELECT
                id,
                display_name,
                display_image,
                gender as "gender: Gender"
                FROM users
                WHERE id = $1
            "#,
            latest_chat.owner_id
        ).fetch_one(&mut *transaction).await?;

        let user_chat_receiver = sqlx::query! (
            r#"
                SELECT
                id,
                display_name,
                display_image,
                gender as "gender: Gender"
                FROM users
                WHERE id = $1  
            "#,
            latest_chat.receiver_id
        ).fetch_one(&mut * transaction).await?;

        transaction.commit().await?;

        let user_chat_owner_display_image = match user_chat_owner.display_image {
            Some(display_image) => display_image,
            None => {
                get_placeholder_display_image(None, &user_chat_owner.gender).to_string()
            }
        };

        let user_chat_receiver_display_image = match user_chat_receiver.display_image {
            Some(display_image) => display_image,
            None => {
                get_placeholder_display_image(None, &user_chat_receiver.gender).to_string()
            }
        };

        Ok(Chat::new(
            latest_chat.id,
            user_chat_receiver.display_name,
            user_chat_receiver.id,
            user_chat_receiver_display_image,
            user_chat_owner.display_name,
            user_chat_owner.id,
            user_chat_owner_display_image,
            latest_chat.created_at,
            latest_chat.message,
        ))
        
    }
}
