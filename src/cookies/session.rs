use std::{fs::File, io::Read};

use cloud_storage::Client;
use rocket::{form::Form, get, http::{ContentType, Cookie, CookieJar, Status}, post, request::FlashMessage, response::{status, Redirect}, routes, Data, FromForm, Route};
use rocket_dyn_templates::{context, Template};

use rocket_db_pools::Connection;

use bcrypt;
use rocket_multipart_form_data::{mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, TextField};
use serde::{Deserialize, Serialize};

use crate::{consts, cookies, db::Db, user::user_struct::User, utils};

#[derive(Debug, Deserialize, Serialize)]
struct ImageFile {
    name: String,
    data: Vec<u8>,
    size: u64,
    mimetype: String,
}

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

#[get("/register")]
fn register(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/register", rank = 2)]
fn register_page(cookies: &CookieJar<'_>, flash: Option<FlashMessage<'_>>) -> Template {
    let theme = cookies::settings::get_default_theme(cookies);

    Template::render("register", context! {
        lang: "en",
        theme,
        flash: &flash
    })
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
async fn post_login<'a>(mut db: Connection<Db>, cookies: &CookieJar<'_>, login_info: Form<Login<'_>>) -> Result<utils::custom_redirect::Redirect, status::Custom<&'a str>> {
    let user = User::get_by_display_name(&mut db, login_info.display_name).await;

    match user {
        Some(user) => {
            let user_credentials = User::get_user_credentials(&mut db, &user.id).await;

            match user_credentials {
                Some(user_credentials) => {
                    let is_allowed = bcrypt::verify(login_info.password, &user_credentials.password);

                    match is_allowed {
                        Ok(true) => {
                            let stringified_user = serde_json::to_string(&user);

                            match stringified_user {
                                Ok(stringified_user) => {
                                    cookies.add_private(Cookie::new("user_info", stringified_user));

                                    Ok(utils::custom_redirect::Redirect::to(uri!(index)))
                                },
                                Err(_) => {
                                    Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                                }
                            }
                        },
                        Ok(false) => Err(status::Custom(Status::Unauthorized, consts::ERRORS.invalid_credentials)),
                        Err(_err) => {
                            Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                        }
                    }
                },
                None => {
                    println!("User exists in \"users\" table but not in \"user_credentials\" table. Display name: {}", login_info.display_name);

                    Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                }
            }
        },
        None => {
            Err(status::Custom(Status::Unauthorized, consts::ERRORS.invalid_credentials))
        }
    }
}

async fn create_user<'a>(db: &mut Connection<Db>, cookies: &CookieJar<'_>, display_name: &str, display_image: &str, password: &str) -> Result<utils::custom_redirect::Redirect, status::Custom<&'a str>> {
    let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST);

    match hashed_password {
        Ok(hashed_password) => {
            let created_user = User::create_user(db, display_name, display_image, &hashed_password).await;

            match created_user {
                Ok(created_user) => {
                    let stringified_user = serde_json::to_string(&created_user);

                    match stringified_user {
                        Ok(stringified_user) => {
                            cookies.add_private(Cookie::new("user_info", stringified_user));

                            Ok(utils::custom_redirect::Redirect::to(uri!(index)))
                        },
                        Err(_err) => {
                            Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                        }
                    }
                },
                Err(_err) => {
                    Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                }
            }
        },
        Err(_err) => {
            Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
        }
    }
}

async fn parse_register_info_and_create_user<'a> (
    db: &mut Connection<Db>,
    cookies: &CookieJar<'_>,
    display_name: Option<&Vec<TextField>>,
    password: Option<&Vec<TextField>>,
    display_image: &str
) -> Result<utils::custom_redirect::Redirect, status::Custom<&'a str>> {
    match display_name {
        Some(display_name) => {
            if display_name.len() > 1 {
                return Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_data));
            }

            if let Some(display_name) = display_name.first() {
                match password {
                    Some(password) => {
                        if password.len() > 1 {
                            return Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_data));
                        }

                        if let Some(password) = password.first() {
                            create_user(db, cookies, display_name.text.as_str(), display_image, password.text.as_str()).await
                        } else {
                            Err(status::Custom(Status::NotAcceptable, consts::ERRORS.incomplete_data))
                        }
                    },
                    None => {
                        Err(status::Custom(Status::NotAcceptable, consts::ERRORS.incomplete_data))
                    }
                }
            } else {
                Err(status::Custom(Status::NotAcceptable, consts::ERRORS.incomplete_data))
            }
            
        },
        None => {
            Err(status::Custom(Status::NotAcceptable, consts::ERRORS.incomplete_data))
        }
    }
}

#[post("/register", data = "<data>")]
async fn post_register<'a>(mut db: Connection<Db>, cookies: &CookieJar<'_>, content_type: &ContentType, data: Data<'_>) -> Result<utils::custom_redirect::Redirect, status::Custom<&'a str>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec![
            MultipartFormDataField::file("display_image").content_type_by_string(Some(mime::IMAGE_STAR)).unwrap(),
            MultipartFormDataField::text("display_name"),
            MultipartFormDataField::text("password")
        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;

    match multipart_form_data {
        Ok(data) => {
            let display_image = data.files.get("display_image");
            let display_name = data.texts.get("display_name");
            let password = data.texts.get("password");            

            match display_image {
                Some(display_image) => {
                    if display_image.len() > 1 {
                        return Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_data));
                    }

                    if let Some(display_image) = &display_image.first() {
                        let mime_type = &display_image.content_type;

                        match mime_type {
                            Some(mime_type) => {
                                match mime_type.essence_str() {
                                    "image/jpg" | "image/jpeg" | "image/png" | "image/webp" | "image/avif" => {
                                        let file_name = &display_image.file_name;
                                        let file_path = &display_image.path;

                                        match file_name {
                                            Some(file_name) => {
                                                let read_file = File::open(file_path);

                                                match read_file {
                                                    Ok(buf) => {
                                                        let mut bytes: Vec<u8> = Vec::new();

                                                        for byte in buf.bytes() {
                                                            match byte {
                                                                Ok(byte) => {
                                                                    bytes.push(byte);
                                                                },
                                                                Err(err) => {
                                                                    println!("File byte read err: {:?}. File might be corrupted", err);

                                                                    return Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong));
                                                                }
                                                            }
                                                        }

                                                        let result = Client::default().object().create("chat_server_local_development", bytes, file_name, mime_type.essence_str()).await;

                                                        match result {
                                                            Ok(_obj) => {
                                                                let display_image = format!("https://storage.googleapis.com/chat_server_local_development/{}", file_name);

                                                                parse_register_info_and_create_user(&mut db, cookies, display_name, password, &display_image).await
                                                            }
                                                            Err(err) => {
                                                                println!("Google Cloud err: {:?}", err);

                                                                Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        println!("File read err: {:?}", err);

                                                        return Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong));
                                                    }
                                                }
                                            }
                                            None => {
                                                Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_file_name))
                                            }
                                        }
                                    }
                                    _ => {
                                        Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_file_mime_type))
                                    }
                                }
                            }
                            None => {
                                Err(status::Custom(Status::NotAcceptable, consts::ERRORS.invalid_file_mime_type))
                            }
                        }
                    } else {
                        Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
                    }
                }
                None => {
                    let placeholder = "/assets/placeholder/profile_picture.jpg";

                    parse_register_info_and_create_user(&mut db, cookies, display_name, password, placeholder).await
                }
            }
        }
        Err(_) => {
            Err(status::Custom(Status::InternalServerError, consts::ERRORS.something_went_wrong))
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![index, unauthorized, register, register_page, login, login_page, post_login, post_register]
}
