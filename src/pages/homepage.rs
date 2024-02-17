use rocket::{get, http::CookieJar};
use rocket_dyn_templates::{context, Template};

use crate::{consts, cookies::{self, settings::{Language, Theme}}, user::user_struct::User, utils::get_placeholder_display_image};

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

    let placeholder_display_image: Option<String> = user.as_ref().map(|user| get_placeholder_display_image(user.display_image.as_ref(), &user.gender));

    Template::render(
        "index",
        context! {
            lang: language,
            theme: preferred_theme,
            user,
            placeholder_display_image,
            metadata: consts::METADATA
        }
    )
}
