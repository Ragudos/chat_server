use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest, Request}, serde::{Deserialize, Serialize}, time::OffsetDateTime};
use serde_json;
use rocket_db_pools::{sqlx, Connection};

use crate::db::Db;

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

impl Into<Gender> for String {
    fn into(self) -> Gender {
        match self.to_lowercase().as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            "other" => Gender::Other,
            _ => Gender::Other
        }
    }
}

impl Into<UserStatus> for String {
    fn into(self) -> UserStatus {
        match self.to_lowercase().as_str() {
            "active" => UserStatus::Active,
            "suspended" => UserStatus::Suspended,
            "deleted" => UserStatus::Deleted,
            _ => UserStatus::Active
        }
    }
}

impl Into<UserRole> for String {
    fn into(self) -> UserRole {
        match self.to_lowercase().as_str() {
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
                let user  = serde_json::from_str(user_info).ok();
                user
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

    pub async fn get_by_display_name(db: &mut Connection<Db>, display_name: &String) -> Option<User> {
        let record = sqlx::query!(
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
        .fetch_one(&mut ***db).await.ok();
    
        match record {
            Some(record) => {
                Some(User::new(record.id, record.display_name, record.display_image, record.biography, record.creation_date, record.last_login_date, record.status, record.gender, record.role, record.profile_pictures))
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
                    password_hash: record.password_hash
                })
            },
            None => None
        }
    }

    pub async fn create_user(db: &mut Connection<Db>, display_name: &String, display_image: &String, hashed_password: &String, gender: &Gender) -> Result<User, sqlx::Error> {
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
        .fetch_one(&mut ***db).await;

        match user {
            Ok(user) => {
                let user = User::new(user.id, user.display_name, user.display_image, user.biography, user.creation_date, user.last_login_date, user.status, user.gender, user.role, user.profile_pictures);

                let user_credentials = sqlx::query!(
                    "INSERT INTO user_credentials (user_id, password_hash) VALUES ($1, $2) RETURNING *", user.id, hashed_password
                ).fetch_one(&mut ***db).await;

                match user_credentials {
                    Ok(_) => Ok(user),
                    Err(err) => Err(err)
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}

