use rocket::{get, http::CookieJar, response::Redirect};
use rocket_dyn_templates::{context, Template};
use time::{Duration, OffsetDateTime};

use crate::{auth_uri as uri, consts::{self, TemplateOrRedirect}, cookies::settings::{self, Language, Theme}, user::user_struct::User};

#[get("/")]
pub fn page(
    user: User,
    cookies: &CookieJar<'_>,
) -> TemplateOrRedirect {
    if user.last_login_date < OffsetDateTime::now_utc().checked_sub(Duration::seconds(30).abs()).unwrap() {
        return TemplateOrRedirect::Redirect(Redirect::to("/chats"));
    }

    let preferred_theme = Theme::as_str(
        &settings::get_default_theme(cookies)
    );
    let language = Language::as_str(
        &settings::get_default_language(cookies)
    );

    TemplateOrRedirect::Template(Template::render(
        "auth",
        context! {
            user,
            theme: preferred_theme,
            lang: language,
            metadata: consts::METADATA
        }
    ))
}

#[get("/", rank = 2)]
pub fn redirect_if_logged_out() -> Redirect {
    Redirect::to(uri!(super::login::page))
}
