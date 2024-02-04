use rocket::{get, http::CookieJar, response::Redirect};
use rocket_dyn_templates::{context, Template};

use crate::{auth_uri as uri, cookies::settings::{self, Language, Theme}, user::user_struct::User};

#[get("/")]
pub fn page(
    user: User,
    cookies: &CookieJar<'_>,
) -> Template {
    let preferred_theme = Theme::as_str(
        &settings::get_default_theme(cookies)
    );
    let language = Language::as_str(
        &settings::get_default_language(cookies)
    );

    Template::render(
        "auth",
        context! { user, theme: preferred_theme, lang: language }
    )
}

#[get("/", rank = 2)]
pub fn redirect_if_logged_out() -> Redirect {
    Redirect::to(uri!(super::login::page))
}
