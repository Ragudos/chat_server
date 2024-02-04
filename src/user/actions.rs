use std::error::Error;

use rocket_db_pools::Connection;
use sqlx::Acquire;
use time::OffsetDateTime;
use crate::db::Db;

use super::user_struct::{User, UserStatus, Gender, UserRole};

pub struct UserActions {}

impl UserActions {
    pub async fn update_last_login_date(
        db: &mut Connection<Db>,
        user_id: &i32,
    ) -> Result<User, Box<dyn Error>> {
        let mut transaction = db.begin().await?;

        sqlx::query!(
            r#"
            UPDATE users
            SET last_login_date = $1
            WHERE id = $2"#,
            OffsetDateTime::now_utc(),
            user_id
        ).execute(&mut *transaction).await?;

        let user = sqlx::query!(
            r#"
            SELECT
            id,
            display_name,
            display_image,
            biography,
            creation_date,
            last_login_date,
            status as "status: UserStatus",
            gender as "gender: Gender",
            role as "role: UserRole",
            profile_pictures
            FROM users
            WHERE id = $1"#,
            user_id
        ).fetch_one(&mut *transaction).await?;

        transaction.commit().await?;
        
        Ok(User::new(
            user.id,
            user.display_name,
            user.display_image,
            user.biography,
            user.creation_date,
            user.last_login_date,
            user.status,
            user.gender,
            user.role,
            user.profile_pictures
        ))
    }

    // Provide an empty String if you want to remove the display image
    pub async fn update_display_image(
        self,
        db: &mut Connection<Db>,
        user_id: &i32,
        new_display_image: &String,
    ) -> Result<Self, Box<dyn Error>> {
        let mut transaction = db.begin().await?;
        let prev_display_image = sqlx::query!(
            r#"
            SELECT display_image
            FROM users
            WHERE id = $1"#,
            user_id
        ).fetch_one(&mut *transaction).await?.display_image;

        sqlx::query!(
            r#"
            UPDATE users
            SET display_image = $1
            WHERE id = $2"#,
            new_display_image,
            user_id
        ).execute(&mut *transaction).await?;

        match prev_display_image {
            Some(prev_display_image) => {
                if !prev_display_image.is_empty() {
                    sqlx::query!(
                        r#"
                        UPDATE users
                        SET profile_pictures = array_append(profile_pictures, $1)
                        WHERE id = $2
                        "#,
                        prev_display_image,
                        user_id
                    ).execute(&mut *transaction).await?;
                }
            },
            None => {}
        }

        transaction.commit().await?;
        
        Ok(self)
    }

    pub async fn update_biography(
        self,
        db: &mut Connection<Db>,
        user_id: &i32,
        new_biography: &String,
    ) -> Result<Self, Box<dyn Error>> {
        let mut transaction = db.begin().await?;

        sqlx::query!(
            r#"
            UPDATE users
            SET biography = $1
            WHERE id = $2"#,
            new_biography,
            user_id
        ).execute(&mut *transaction).await?;

        transaction.commit().await?;
        
        Ok(self)
    }

    pub async fn update_status(
        self,
        db: &mut Connection<Db>,
        user_id: &i32,
        new_status: &UserStatus,
    ) -> Result<Self, Box<dyn Error>> {
        let mut transaction = db.begin().await?;

        sqlx::query!(
            r#"
            UPDATE users
            SET status = $1
            WHERE id = $2"#,
            new_status as &UserStatus,
            user_id
        ).execute(&mut *transaction).await?;

        transaction.commit().await?;
        
        Ok(self)
    }

    pub async fn update_email(
        self,
        db: &mut Connection<Db>,
        user_id: &i32,
        new_email: &String,
    ) -> Result<Self, Box<dyn Error>> {
        let mut transaction = db.begin().await?;

        sqlx::query!(
            r#"
            UPDATE user_credentials
            SET email = $1
            WHERE user_id = $2"#,
            new_email,
            user_id
        ).execute(&mut *transaction).await?;

        transaction.commit().await?;
        
        Ok(self)
    }
}
