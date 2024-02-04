use bcrypt;
use rocket::{http::{CookieJar, Status}, response::status};
use rocket_db_pools::Connection;

use crate::{db::Db, errors::error::{Error, ErrorReason}, user::user_struct::{Gender, User}, utils};

pub async fn create_user(
    db: &mut Connection<Db>,
    cookies: &CookieJar<'_>,
    display_name: &String,
    display_image: &String,
    password: &String,
    gender: &Gender
) -> Result<utils::custom_redirect::Redirect, status::Custom<String>> {
    let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST);

    match hashed_password {
        Ok(hashed_password) => {
            let new_user = User::create_user(db, display_name, display_image, &hashed_password, gender).await;

            match new_user {
                Ok(new_user) => {
                    let stringified_user = serde_json::to_string(&new_user);

                    match stringified_user {
                        Ok(stringified_user) => {
                            cookies.add_private(rocket::http::Cookie::new("user_info", stringified_user));

                            return Ok(utils::custom_redirect::Redirect::to("/"));
                        }
                        Err(err) => {
                            println!("Error: {:?}", err);

                            return Err(status::Custom(
                                Status::InternalServerError,
                                Error::to_string(Error::new(
                                    ErrorReason::SomethingWentWrong,
                                    "Failed to register".to_string()
                                )),
                            ));
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {:?}", err);

                    return Err(status::Custom(
                        Status::InternalServerError,
                        Error::to_string(Error::new(
                            ErrorReason::SomethingWentWrong,
                            "Failed to register".to_string()
                        )),
                    ));
                }
            }
        }
        Err(err) => {
            println!("Error: {:?}", err);

            return Err(status::Custom(
                Status::InternalServerError,
                Error::to_string(Error::new(
                    ErrorReason::SomethingWentWrong,
                    "Failed to process information".to_string()
                )),
            ));
        }
    }
}

pub async fn does_display_name_exist(
    db: &mut Connection<Db>,
    display_name: &String
) -> bool {
    User::get_by_display_name(db, display_name).await.is_some()
}
