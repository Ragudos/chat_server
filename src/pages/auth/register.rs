use rocket::{get, http::CookieJar, response::Redirect};
use rocket_dyn_templates::{context, Template};

use crate::{auth_uri as uri, consts, cookies::settings::{self, Language, Theme}, user::user_struct::User};

use super::index;

#[get("/register")]
pub fn redirect_if_logged_in(_user: User) -> Redirect {
    Redirect::to(uri!(index::page))
}

#[get("/register", rank = 2)]
pub fn page(cookies: &CookieJar<'_>) -> Template {
    let preferred_theme = Theme::as_str(
        &settings::get_default_theme(cookies)
    );
    let language = Language::as_str(
        &settings::get_default_language(cookies)
    );

    Template::render(
        "register",
        context! {
            theme: preferred_theme,
            lang: language,
            metadata: consts::METADATA
        }
    )
}
