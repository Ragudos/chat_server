use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest, Request}, serde::{Deserialize, Serialize}, time::OffsetDateTime};
use serde_json;
use rocket_db_pools::{sqlx, Connection};
use sqlx::Acquire;

use crate::{db::Db, utils};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "gender", rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Other
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "displayImage")]
    pub display_image: Option<String>,
    pub biography: Option<String>,
    #[serde(rename = "creationDate")]
    pub creation_date: OffsetDateTime,
    #[serde(rename = "lastLoginDate")]
    pub last_login_date: OffsetDateTime,
    pub status: UserStatus,
    pub gender: Gender,
    pub role: UserRole,
    #[serde(rename = "profilePictures")]
    pub profile_pictures: Vec<String>
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PartialUser {
    pub id: i32,
    pub display_name: String,
    pub display_image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserCredentials {
    pub id: i32,
    pub user_id: i32,
    pub email: Option<String>,
    /// Hashed password
    pub password_hash: String,
}

impl From<Gender> for String {
    fn from(value: Gender) -> Self {
        match value {
            Gender::Male => "male".to_string(),
            Gender::Female => "female".to_string(),
            Gender::Other => "other".to_string()
        }
    }
}

impl From<UserStatus> for String {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Active => "active".to_string(),
            UserStatus::Suspended => "suspended".to_string(),
            UserStatus::Deleted => "deleted".to_string()
        }
    }
}

impl From<UserRole> for String {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Admin => "admin".to_string(),
            UserRole::User => "user".to_string()
        }
    }
}

impl From<String> for Gender {
    fn from(val: String) -> Self {
        match val.to_lowercase().as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            "other" => Gender::Other,
            _ => Gender::Other
        }
    }
}

impl From<String> for UserStatus {
    fn from(val: String) -> Self {
        match val.to_lowercase().as_str() {
            "active" => UserStatus::Active,
            "suspended" => UserStatus::Suspended,
            "deleted" => UserStatus::Deleted,
            _ => UserStatus::Active
        }
    }
}

impl From<String> for UserRole {
    fn from(val: String) -> Self {
        match val.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "user" => UserRole::User,
            _ => UserRole::User
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        request.cookies()
            .get_private("user_info")
            .and_then(|cookie| {
                let user_info = cookie.value_trimmed();
                
                serde_json::from_str(user_info).ok()
           }).or_forward(Status::Unauthorized)
    }
}

impl User {
    pub fn new(
        id: i32,
        display_name: String,
        display_image: Option<String>,
        biography: Option<String>,
        creation_date: OffsetDateTime,
        last_login_date: OffsetDateTime,
        status: UserStatus,
        gender: Gender,
        role: UserRole,
        profile_pictures: Vec<String>
    ) -> User {
        User {
            id,
            display_name,
            display_image,
            biography,
            creation_date,
            last_login_date,
            gender,
            status,
            role,
            profile_pictures
        }
    }

    pub async fn get_display_name(db: &mut Connection<Db>, user_id: &i32) -> Option<String> {
        sqlx::query!(
            "SELECT display_name FROM users WHERE id = $1", user_id
        )
        .fetch_one(&mut ***db).await.ok().map(|user| user.display_name)
    }

    pub async fn get_by_id(db: &mut Connection<Db>, id: &i32) -> Option<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
            id,
            display_name,
            display_image,
            role as "role: UserRole",
            biography,
            creation_date,
            last_login_date,
            status as "status: UserStatus",
            gender as "gender: Gender",
            profile_pictures
            FROM users WHERE id = $1
            "#, id
        )
        .fetch_one(&mut ***db).await.ok()
    }

    pub async fn get_by_display_name(db: &mut Connection<Db>, display_name: &String) -> Option<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
            id,
            display_name,
            display_image,
            role as "role: UserRole",
            biography,
            creation_date,
            last_login_date,
            status as "status: UserStatus",
            gender as "gender: Gender",
            profile_pictures
            FROM users WHERE display_name = $1
            "#, display_name
        )
        .fetch_one(&mut ***db).await.ok()
    }

    /// @param user_id: The user's id
    pub async fn get_user_credentials(db: &mut Connection<Db>, user_id: &i32) -> Option<UserCredentials> {
        sqlx::query_as!(
            UserCredentials,
            "SELECT * FROM user_credentials WHERE user_id = $1", user_id
        )
        .fetch_one(&mut ***db).await.ok()
    }

    pub async fn create_user(db: &mut Connection<Db>, display_name: &String, display_image: &String, hashed_password: &String, gender: &Gender) -> Result<User, sqlx::Error> {
        let mut transaction = db.begin().await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (display_name, display_image, gender)
            VALUES ($1, $2, $3)
            RETURNING
            id,
            gender as "gender: Gender",
            status as "status: UserStatus",
            display_name,
            display_image,
            role as "role: UserRole",
            biography,
            creation_date,
            last_login_date,
            profile_pictures
            "#,
            display_name,
            display_image,
            gender as &Gender
        )
        .fetch_one(&mut *transaction).await?;

        sqlx::query!(
            "INSERT INTO user_credentials (user_id, password_hash) VALUES ($1, $2) RETURNING *", user.id, hashed_password
        ).fetch_one(&mut *transaction).await?;

        transaction.commit().await?;

        Ok(user)
    }

    pub async fn search_for_users_with_display_name(
        db: &mut Connection<Db>,
        display_name_of_current_user: &String,
        search: &String
    ) -> Result<Vec<PartialUser>, sqlx::Error> {
        let users = sqlx::query!(
            r#"
            SELECT
            id,
            display_name,
            display_image,
            gender as "gender: Gender"
            FROM users
            WHERE display_name != $2
            AND similarity(display_name, $1) > 0.2
            ORDER BY similarity(display_name, $1) DESC;
            "#,
            search,
            display_name_of_current_user
        ).fetch_all(&mut ***db).await?;

        let mut users_vec = Vec::new();

        for user in users {
            let display_image = Some(utils::get_placeholder_display_image(user.display_image.as_ref(), &user.gender));

            users_vec.push(PartialUser {
                id: user.id,
                display_name: user.display_name,
                display_image
            });
        }

        Ok(users_vec)
    }
}

