#[macro_use] extern crate rocket;

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

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![homepage])
        .mount("/session", session::routes())
        .attach(Template::fairing())
        .attach(db::stage())
        .mount("/assets", FileServer::from(relative!("assets")))
}
