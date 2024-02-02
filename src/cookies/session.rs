use rocket::{form::Form, get, http::CookieJar, post, request::FlashMessage, response::{status, Redirect}, routes, FromForm, Route};
use rocket_dyn_templates::{context, Template};

use rocket_db_pools::Connection;

use bcrypt;

use crate::{cookies, user::user_struct::User, db::Db};

#[derive(FromForm)]
struct Login<'r> {
    display_name: &'r str,
    password: &'r str
}

#[macro_export]
macro_rules! session_uri {
    ($($t:tt)*) => {
        rocket::uri!("/session", $($t)*)
    }
}

pub use session_uri as uri;

// Since Rocket goes to another matching rout if parameter types
// does not exist or does not match what we have, we can use the same route for both authorized and unauthorized
#[get("/")]
fn index(user: User, cookies: &CookieJar<'_>) -> Template {
    let theme = cookies::settings::get_default_theme(cookies);

    Template::render("session", context! {
        user,
        theme
    })
}

#[get("/", rank = 2)]
fn unauthorized() -> Redirect {
    Redirect::to(uri!(login_page))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login", rank = 2)]
fn login_page(cookies: &CookieJar<'_>, flash: Option<FlashMessage<'_>>) -> Template {
    let theme = cookies::settings::get_default_theme(cookies);

    Template::render("login", context! {
        lang: "en",
        theme,
        flash: &flash
    })
}

#[post("/login", data = "<login_info>")]
async fn post_login<'a>(mut db: Connection<Db>, cookies: &CookieJar<'_>, login_info: Form<Login<'_>>) -> Result<Redirect, status::Unauthorized<&'a str>> {
    let user = User::get_by_display_name(&mut db, login_info.display_name).await;

    match user {
        Some(user) => {
            let user_credentials = User::get_user_credentials(&mut db, &user.id).await;

            match user_credentials {
                Some(user_credentials) => {
                    if bcrypt::verify(&login_info.password, &user_credentials.password).unwrap() {
                        cookies.add_private(("user_info", serde_json::to_string(&user).unwrap()));

                        Ok(Redirect::to(uri!(index)))
                    } else {
                        Err(status::Unauthorized("Invalid credentials"))
                    }
                },
                None => Err(status::Unauthorized("Invalid credentials"))
            }
        },
        None=> Err(status::Unauthorized("Invalid credentials"))
    }
}

pub fn routes() -> Vec<Route> {
    routes![index, unauthorized, login, login_page, post_login]
}
