use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest, Request}, serde::{Serialize, Deserialize}};
use serde_json;
use rocket_db_pools::{sqlx, Connection};

use crate::db::Db;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: i32,
    pub display_name: String,
    pub display_image: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserCredentials {
    pub id: i32,
    pub user_id: i32,
    pub email: Option<String>,
    /// Hashed password
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        request.cookies()
            .get_private("user_info")
            .and_then(|cookie| {
                let user_info = cookie.value_trimmed();
                let user: Option<User>  = serde_json::from_str(user_info).unwrap_or(None);
                user
           }).or_forward(Status::Unauthorized)
    }
}

impl User {
    pub async fn get_by_display_name(db: &mut Connection<Db>, display_name: &str) -> Option<User> {
        let record = sqlx::query!(
            "SELECT * FROM users WHERE display_name = $1", display_name
        )
        .fetch_one(&mut ***db).await.ok();
    
        match record {
            Some(record) => {
                Some(User {
                    id: record.id,
                    display_name: record.display_name,
                    display_image: record.display_image
                })
            },
            None => None
        }
    }

    /// @param user_id: The user's id
    pub async fn get_user_credentials(db: &mut Connection<Db>, user_id: &i32) -> Option<UserCredentials> {
        let record = sqlx::query!(
            "SELECT * FROM user_credentials WHERE user_id = $1", user_id
        )
        .fetch_one(&mut ***db).await.ok();
    
        match record {
            Some(record) => {
                Some(UserCredentials {
                    id: record.id,
                    user_id: record.user_id,
                    email: record.email,
                    password: record.password
                })
            },
            None => None
        }
    }
}
