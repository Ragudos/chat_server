use std::borrow::Borrow;

use rocket::{form::Form, http::{Cookie, CookieJar, Status}, post, response::status};
use rocket_db_pools::Connection;

use crate::{auth_uri, db::Db, errors::error::{Error, ErrorReason}, pages::auth::login::Login, user::{actions::UserActions, user_struct::User}, utils};

use super::super::index;

#[post("/login", data = "<login_info>")]
pub async fn login_user(
    mut db: Connection<Db>,
    cookies: &CookieJar<'_>,
    login_info: Form<Login<'_>>
) -> Result<utils::custom_redirect::Redirect, status::Custom<String>> {
    let user = User::get_by_display_name(&mut db, login_info.display_name.to_string().borrow()).await;

    match user {
        Some(user) => {
            let user_credentials = User::get_user_credentials(&mut db, &user.id).await;

            match user_credentials {
                Some(user_credentials) => {
                    let is_same_password = bcrypt::verify(login_info.password, user_credentials.password_hash.as_str());

                    match is_same_password  {
                        Ok(true) => {
                            let updated_user = UserActions::update_last_login_date(&mut db, &user.id).await;

                            match updated_user {
                                Ok(user) => {
                                    let stringified_user = serde_json::to_string(&user);

                                    match stringified_user {
                                        Ok(stringified_user) => {
                                            cookies.add_private(Cookie::new("user_info", stringified_user));

                                            Ok(utils::custom_redirect::Redirect::to(auth_uri!(index::page)))
                                        }
                                        Err(err) => {
                                            println!("Error: {:?}", err);

                                            Err(status::Custom(
                                                Status::InternalServerError,
                                                Error::to_string(Error::new(
                                                    ErrorReason::SomethingWentWrong,
                                                    "Credentials are valid, but something went wrong in parsing user information.".to_string()
                                                )),
                                            ))
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("Error: {:?}", err);

                                    Err(status::Custom(
                                        Status::InternalServerError,
                                        Error::to_string(Error::new(
                                            ErrorReason::SomethingWentWrong,
                                            "Credentials are valid, but something went wrong.".to_string()
                                        )),
                                    ))
                                }
                            }
                        }
                        Ok(false) => {
                            Err(status::Custom(
                                Status::Unauthorized,
                                Error::to_string(Error::new(
                                    ErrorReason::InvalidCredentials,
                                    "Invalid credentials".to_string()
                                )),
                            ))
                        }
                        Err(err) => {
                            println!("Error: {:?}", err);

                            Err(status::Custom(
                                Status::InternalServerError,
                                Error::to_string(Error::new(
                                    ErrorReason::SomethingWentWrong,
                                    "Unable to check if credentials are valid.".to_string()
                                )),
                            ))
                        }
                    }
                }
                None => {
                    println!("User exists in \"users\" table but not in \"user_credentials\" table. Display name: {}", login_info.display_name);

                    Err(status::Custom(
                        Status::InternalServerError,
                        Error::to_string(Error::new(
                            ErrorReason::SomethingWentWrong,
                            "Unable to check if credentials are valid.".to_string()
                        )),
                    ))
                }
            }
        }
        None => {
            Err(status::Custom(
                Status::Unauthorized,
                Error::to_string(Error::new(
                    ErrorReason::InvalidCredentials,
                    "Invalid credentials".to_string()
                )),
            ))
        }
    }
}
