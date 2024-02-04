use rocket::{catch,  Request};
use rocket_dyn_templates::{context, Template};

use crate::{cookies, errors::error::ErrorJson};


#[catch(500)]
pub fn internal_error(
    request: &Request
) -> Template {
    let preferred_theme = cookies::settings::Theme::as_str(
        &cookies::settings::get_default_theme(request.cookies())
    );
    let language = cookies::settings::Language::as_str(
        &cookies::settings::get_default_language(request.cookies())
    );

    let error = ErrorJson {
        code: 500,
        message: "The server encountered an internal error. Please try again later.".to_string(),
        reason: "Internal Server Error".to_string(),
    };

    Template::render("error", context! {
        error,
        theme: preferred_theme,
        lang: language
    })
}

#[catch(404)]
pub fn not_found(
    request: &Request
) -> Template {
    let preferred_theme = cookies::settings::Theme::as_str(
        &cookies::settings::get_default_theme(request.cookies())
    );
    let language = cookies::settings::Language::as_str(
        &cookies::settings::get_default_language(request.cookies())
    );

    let error = ErrorJson {
        code: 404,
        message: "The requested resource could not be found.".to_string(),
        reason: "Not Found".to_string(),
    };

    Template::render("error", context! {
        error,
        theme: preferred_theme,
        lang: language
    })
}

#[catch(401)]
pub fn unauthorized(
    request: &Request
) -> Template {
    let preferred_theme = cookies::settings::Theme::as_str(
        &cookies::settings::get_default_theme(request.cookies())
    );
    let language = cookies::settings::Language::as_str(
        &cookies::settings::get_default_language(request.cookies())
    );

    let error = ErrorJson {
        code: 401,
        message: "You are not authorized to access this resource or do this action.".to_string(),
        reason: "Unauthorized".to_string(),
    };

    Template::render("error", context! {
        error,
        theme: preferred_theme,
        lang: language
    })
}
