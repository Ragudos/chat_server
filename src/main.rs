#[macro_use] extern crate rocket;

<<<<<<< HEAD
<<<<<<< HEAD
use chat_server::{cookies::{self, session}, db};
use rocket::{fs::{FileServer, relative}, http::CookieJar};
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn homepage(cookies: &CookieJar<'_>) -> Template {
    let theme = cookies::settings::get_default_theme(cookies);

    Template::render("index", context! {
        lang: "en",
        theme
    })
}
=======
use chat_server::{db, pages::{auth, homepage}};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;
>>>>>>> 9369702 (cleaned up modules)
=======
use chat_server::{catchers, db, pages::{auth, homepage}};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;
use rocket_csrf_token::{CsrfConfig, Fairing};
>>>>>>> dcd12f0 (added more columns)

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![homepage::page])
        .mount("/auth", routes![
            auth::login::page,
            auth::login::redirect_if_logged_in,
            auth::register::page,
            auth::register::redirect_if_logged_in,
            auth::index::page,
            auth::index::redirect_if_logged_out,
            auth::api::login::login_user,
            auth::api::register::register_user,
            auth::api::logout::logout_user,
        ])
        .attach(Template::fairing())
        .attach(Fairing::new(CsrfConfig::default()))
        .attach(db::stage())
        .register("/", catchers![catchers::internal_error, catchers::not_found, catchers::unauthorized])
        .mount("/assets", FileServer::from(relative!("assets")))
}
