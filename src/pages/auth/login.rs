use rocket::{get, http::CookieJar, response::Redirect, FromForm};
use rocket_dyn_templates::{context, Template};

use crate::{auth_uri as uri, cookies::settings::{self, Language, Theme}, user::user_struct::User};
use super::index;

#[derive(FromForm)]
pub struct Login<'lifetime> {
    pub display_name: &'lifetime str,
    pub password: &'lifetime str,
}

#[get("/login")]
pub fn redirect_if_logged_in(_user: User) -> Redirect {
    Redirect::to(uri!(index::page))
}

#[get("/login", rank = 2)]
pub fn page(cookies: &CookieJar<'_>) -> Template {
    let preferred_theme = Theme::as_str(
        &settings::get_default_theme(cookies)
    );
    let language = Language::as_str(
        &settings::get_default_language(cookies)
    );

    Template::render(
        "login",
        context! { theme: preferred_theme, lang: language }
    )
}
