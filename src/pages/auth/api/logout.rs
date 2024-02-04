use rocket::{delete, http::{CookieJar, Status}, response::status};

use crate::{auth_uri, errors::error::{Error, ErrorReason}, utils};

#[delete("/logout")]
pub fn logout_user(cookies: &CookieJar<'_>) -> Result<utils::custom_redirect::Redirect, status::Custom<String>> {
    let user_info = cookies.get_private("user_info");

    match user_info {
        Some(_) => {
            cookies.remove_private(cookies.get_private("user_info").unwrap());
            Ok(utils::custom_redirect::Redirect::to(auth_uri!(super::super::login::page)))
        }
        None => Err(status::Custom(
            Status::NotFound,
            Error::to_string(Error::new(ErrorReason::InvalidRequest, "You are not logged in. Please try refreshing the page.".to_string())
        )))
    }
}