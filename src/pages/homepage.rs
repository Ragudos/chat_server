use rocket::{get, http::CookieJar};
use rocket_dyn_templates::{context, Template};

use crate::{cookies::{self, settings::{Language, Theme}}, user::user_struct::User, utils::get_placeholder_display_image};

#[get("/")]
pub fn page(
    user: Option<User>,
    cookies: &CookieJar<'_>
) -> Template {
    let preferred_theme = Theme::as_str(
        &cookies::settings::get_default_theme(cookies)
    );
    let language = Language::as_str(
        &cookies::settings::get_default_language(cookies)
    );

    let placeholder_display_image = get_placeholder_display_image(&user);

    println!("User: {:?}", user);

    Template::render(
        "index",
        context! { lang: language, theme: preferred_theme, user: user, placeholder_display_image }
    )
}
