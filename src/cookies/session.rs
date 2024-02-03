use cloudinary::upload::{result::UploadResult, Source, Upload, UploadOptions};
use rocket::{form::Form, get, http::{ContentType, Cookie, CookieJar, Status}, post, request::FlashMessage, response::{status, Redirect}, routes, Data, FromForm, Route};
use rocket_dyn_templates::{context, Template};

use rocket_db_pools::Connection;

use bcrypt;
use rocket_multipart_form_data::{mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, TextField};
use serde::{Deserialize, Serialize};

use crate::{cookies, user::user_struct::User, db::Db, utils::{self, env}};

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
                    let is_allowed = bcrypt::verify(&login_info.password, &user_credentials.password);

                    match is_allowed {
                        Ok(true) => {
                            let stringified_user = serde_json::to_string(&user);

                            match stringified_user {
                                Ok(stringified_user) => {
                                    cookies.add_private(Cookie::new("user_info", stringified_user));

                                    return Ok(utils::custom_redirect::Redirect::to(uri!(index)));
                                },
                                Err(_) => {
                                    return Err(status::Custom(Status::BadRequest, "Something went wrong."));
                                }
                            }
                        },
                        Ok(false) => Err(status::Custom(Status::Unauthorized, "Invalid credentials")),
                        Err(err) => {
                            println!("Failed verifying password. {:?}", err);

                            return Err(status::Custom(Status::BadRequest, "Something went wrong."));
                        }
                    }
                },
                None => {
                    println!("Failed getting user credentials.");

                    return Err(status::Custom(Status::Unauthorized, "Invalid credentials"));
                }
            }
        },
        None => {
            println!("Failed getting user.");

            return Err(status::Custom(Status::Unauthorized, "Invalid credentials"));
        }
    }
}

async fn create_user<'a>(db: &mut Connection<Db>, cookies: &CookieJar<'_>, display_name: &str, display_image: &str, password: &str) -> Result<utils::custom_redirect::Redirect, status::Custom<&'a str>> {
    let hashed_password = bcrypt::hash(&password, bcrypt::DEFAULT_COST);

    match hashed_password {
        Ok(hashed_password) => {
            let created_user = User::create_user(db, display_name, display_image, &hashed_password).await;

            match created_user {
                Ok(created_user) => {
                    let stringified_user = serde_json::to_string(&created_user);

                    match stringified_user {
                        Ok(stringified_user) => {
                            cookies.add_private(Cookie::new("user_info", stringified_user));

                            return Ok(utils::custom_redirect::Redirect::to(uri!(index)));
                        },
                        Err(err) => {
                            println!("Error deserializing User struct. {:?}", err);

                            return Err(status::Custom(Status::BadRequest, "Something went wrong."));
                        }
                    }
                },
                Err(err) => {
                    println!("Error creating user. {:?}", err);

                    return Err(status::Custom(Status::BadRequest, "Something went wrong."));
                }
            }
        },
        Err(err) => {
            println!("Error hashing password. {:?}", err);
            return Err(status::Custom(Status::BadRequest, "Something went wrong."));
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
                return Err(status::Custom(Status::NotAcceptable, "Display name must only exist once."));
            }

            if let Some(display_name) = display_name.get(0) {
                match password {
                    Some(password) => {
                        if password.len() > 1 {
                            return Err(status::Custom(Status::NotAcceptable, "Password must only exist once."));
                        }

                        if let Some(password) = password.get(0) {
                            return create_user(db, cookies, display_name.text.as_str(), display_image, &password.text.as_str()).await;
                        } else {
                            return Err(status::Custom(Status::NotAcceptable, "Password must not be empty."));
                        }
                    },
                    None => {
                        return Err(status::Custom(Status::NotAcceptable, "Password must not be empty."));
                    }
                }
            } else {
                return Err(status::Custom(Status::NotAcceptable, "Display name must not be empty."));
            }
            
        },
        None => {
            return Err(status::Custom(Status::NotAcceptable, "Username must not be empty."));
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
                        return Err(status::Custom(Status::NotAcceptable, "Only one image can be uploaded"));
                    }

                    if let Some(display_image) = &display_image.first() {
                        let mime_type = &display_image.content_type;

                        match mime_type {
                            Some(mime_type) => {
                                println!("Mime type: {:?}", mime_type.essence_str());
                                match mime_type.essence_str() {
                                    "image/jpg" | "image/jpeg" | "image/png" | "image/webp" | "image/avif" => {
                                        let file_name = &display_image.file_name;
                                        let file_path = &display_image.path;

                                        match file_name {
                                            Some(file_name) => {
                                                let options = UploadOptions::new().set_public_id(file_name.clone());
                                                let upload = Upload::new(env::load_cloudinary_api_key(), env::load_cloudinary_cloud_name(), env::load_cloudinary_api_secret());
                                                let result = upload.image(Source::Path(file_path.into()), &options).await;

                                                match result {
                                                    Ok(result) => {
                                                        match result {
                                                            UploadResult::Success(result) => {
                                                                let display_image = result.secure_url;

                                                                return parse_register_info_and_create_user(&mut db, cookies, display_name, password, &display_image).await;
                                                            }
                                                            UploadResult::Error(error) => {
                                                                println!("Error uploading image. {:?}", error);

                                                                return Err(status::Custom(Status::InternalServerError, "Something went wrong."));
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        println!("Error uploading image. {:?}", err);

                                                        return Err(status::Custom(Status::InternalServerError, "Something went wrong."));
                                                    }
                                                }
                                            }
                                            None => {
                                                return Err(status::Custom(Status::NotAcceptable, "File name must not be empty."));
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(status::Custom(Status::NotAcceptable, "Only Png, Jpeg, Webp, and Avif images are allowed."));
                                    }
                                }
                            }
                            None => {
                                return Err(status::Custom(Status::NotAcceptable, "Only images are allowed."));
                            }
                        }
                    } else {
                        return Err(status::Custom(Status::InternalServerError, "Something went wrong."));
                    }
                }
                None => {
                    let placeholder = "/assets/placeholder/profile_picture.jpg";

                    return parse_register_info_and_create_user(&mut db, cookies, display_name, password, placeholder).await;
                }
            }
        }
        Err(_) => {
            return Err(status::Custom(Status::BadRequest, "Something went wrong."));
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![index, unauthorized, register, register_page, login, login_page, post_login, post_register]
}
