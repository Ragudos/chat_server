#[macro_use] extern crate rocket;

use chat_server::{catchers, db, pages::{auth, chats, homepage}};
use rocket::fs::{FileServer, relative};
use rocket_csrf_token::{CsrfConfig, Fairing};
use rocket_dyn_templates::{handlebars::handlebars_helper, Template};

handlebars_helper!(eq_str: |first_arg: String, second_arg: String| first_arg == second_arg);
handlebars_helper!(eq_num: |first_arg: isize, second_arg: isize| first_arg == second_arg);

#[launch]
fn rocket() -> _  {
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
        .mount("/chats", routes! [
            chats::api::chats_of_user::chats_of_user,
            chats::index::page,
            chats::api::chats_of_user::error_if_logged_out,
            chats::index::rederirect_if_logged_out
        ])
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("eq_str", Box::new(eq_str));

            engines
                .handlebars
                .register_helper("eq_num", Box::new(eq_num));

            engines
                .handlebars
                .register_partial("header", r#""#)
                .unwrap();
        }))
        .attach(Fairing::new(CsrfConfig::default()))
        .attach(db::stage())
        .register("/", catchers![catchers::internal_error, catchers::not_found, catchers::unauthorized])
        .mount("/assets", FileServer::from(relative!("assets")))
}
