use std::{fs::File, io::Read};

use cloud_storage::Client;
use rocket::{http::{ContentType, CookieJar, Status}, post, response::status, Data};
use rocket_db_pools::Connection;
use rocket_multipart_form_data::{MultipartFormData, MultipartFormDataField, MultipartFormDataOptions};

use crate::{db::Db, errors::error::{Error, ErrorReason}, pages::auth::_utils::{self, create_user}, utils};

#[post("/register", data = "<data>")]
pub async fn register_user(
    mut db: Connection<Db>,
    cookies: &CookieJar<'_>,
    content_type: &ContentType,
    data: Data<'_>
) -> Result<utils::custom_redirect::Redirect, status::Custom<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec![
            MultipartFormDataField::file("display_image"),
            MultipartFormDataField::text("display_name"),
            MultipartFormDataField::text("password"),
            MultipartFormDataField::text("gender")
        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;

    match multipart_form_data {
        Ok(data) => {
            let display_name = data.texts.get("display_name");
            let password = data.texts.get("password");
            let display_image = data.files.get("display_image");
            let gender = data.texts.get("gender");

            match display_name {
                Some(display_name) => {
                    if display_name.len() != 1 {
                        return Err(status::Custom(
                            Status::BadRequest,
                            Error::to_string(Error::new(ErrorReason::Invalid, "Only one instance of display name is allowed.".to_string()))
                        ));
                    }

                    let display_name = &display_name.first().unwrap().text;
                    let is_name_taken = _utils::does_display_name_exist(&mut db, display_name).await;

                    if is_name_taken {
                        return Err(status::Custom(
                            Status::NotAcceptable,
                            Error::to_string(Error::new(ErrorReason::Invalid, "Display name is already taken.".to_string()))
                        ));
                    }

                    match password {
                        Some(password) => {
                            if password.len() != 1 {
                                return Err(status::Custom(
                                    Status::NotAcceptable,
                                    Error::to_string(Error::new(ErrorReason::Invalid, "Only one instance of password is allowed.".to_string()))
                                ));
                            }

                            let password = &password.first().unwrap().text;

                            if password.len() < 8 {
                                return Err(status::Custom(
                                    Status::NotAcceptable,
                                    Error::to_string(Error::new(ErrorReason::WeakPassword, "Password must be at least 8 characters long.".to_string()))
                                ));
                            }

                            match gender {
                                Some(gender) => {
                                    if gender.len() != 1 {
                                        return Err(status::Custom(
                                            Status::NotAcceptable,
                                            Error::to_string(Error::new(ErrorReason::Invalid, "Only one instance of gender is allowed".to_string())))
                                        );
                                    }

                                    let gender = &gender.first().unwrap().text;

                                    match display_image {
                                        Some(display_image) => {
                                            if display_image.len() != 1 {
                                                return Err(status::Custom(
                                                    Status::NotAcceptable,
                                                    Error::to_string(Error::new(ErrorReason::Invalid, "Only one instance of display image is allowed.".to_string()))
                                                ));
                                            }

                                            let display_image = display_image.first().unwrap();
                                            let mime_type = &display_image.content_type;

                                            match mime_type {
                                                Some(mime_type) => {
                                                    let mime_type = mime_type.essence_str();

                                                    match mime_type {
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
                                                                                    }
                                                                                    Err(err) => {
                                                                                        println!("Error: {:?}", err);

                                                                                        return Err(status::Custom(
                                                                                            Status::InternalServerError,
                                                                                            Error::to_string(Error::new(ErrorReason::SomethingWentWrong, "Failed to process image. File might be corrupted.".to_string()))
                                                                                        ));
                                                                                    }
                                                                                }
                                                                            }

                                                                            let random_name = random_string::generate(12,random_string::charsets::ALPHANUMERIC);
                                                                            let name = format!("{:?}-chat_server-{:?}", random_name, file_name);
                                                                            let result = Client::default().object().create("chat_server_local_development", bytes, &name, mime_type).await;

                                                                            match result {
                                                                                Ok(_obj) => {
                                                                                    let display_image = format!("https://storage.cloud.google.com/chat_server_local_development/{}", name);
                                                                                    
                                                                                    return create_user(&mut db, cookies, display_name, &display_image, password, &gender.clone().into()).await;
                                                                                }
                                                                                Err(err) => {
                                                                                    println!("Error: {:?}", err);

                                                                                    return Err(status::Custom(
                                                                                        Status::InternalServerError,
                                                                                        Error::to_string(Error::new(ErrorReason::SomethingWentWrong, "Failed to process image.".to_string()))
                                                                                    ));
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(err) => {
                                                                            println!("Error: {:?}", err);

                                                                            return Err(status::Custom(
                                                                                Status::InternalServerError,
                                                                                Error::to_string(Error::new(ErrorReason::SomethingWentWrong, "Failed to process image.".to_string()))
                                                                            ));
                                                                        }
                                                                    }
                                                                }
                                                                None => {
                                                                    return Err(status::Custom(
                                                                        Status::NotAcceptable,
                                                                        Error::to_string(Error::new(ErrorReason::Invalid, "File name is required.".to_string()))
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            return Err(status::Custom(
                                                                Status::NotAcceptable,
                                                                Error::to_string(Error::new(ErrorReason::InvalidMimeType, "Only Jpeg, Webp, Png, and Avif images are allowed.".to_string()))
                                                            ));
                                                        }
                                                    }
                                                }
                                                None => {
                                                    return Err(status::Custom(
                                                        Status::NotAcceptable,
                                                        Error::to_string(Error::new(ErrorReason::InvalidMimeType, "Only Jpeg, Webp, Png, and Avif images are allowed.".to_string()))
                                                    ));
                                                }
                                            }
                                        }
                                        None => {
                                            return create_user(&mut db, cookies, display_name, &String::new(), password, &gender.clone().into()).await;
                                        }
                                    }
                                }
                                None => {
                                    return Err(status::Custom(
                                        Status::NotAcceptable,
                                        Error::to_string(Error::new(ErrorReason::Invalid, "Gender is required".to_string())))
                                    )
                                }
                            }
                        }
                        None => {
                            return Err(status::Custom(
                                Status::NotAcceptable,
                                Error::to_string(Error::new(ErrorReason::IncompleteData, "Password is required".to_string()))
                            ));
                        }
                    }
                }
                None => {
                    return Err(status::Custom(
                        Status::NotAcceptable,
                        Error::to_string(Error::new(ErrorReason::IncompleteData, "Display name is required".to_string()))
                    ));
                }
            }
        }
        Err(err) => {
            println!("Error: {:?}", err);

            return Err(status::Custom(
                Status::InternalServerError,
                Error::to_string(Error::new(ErrorReason::SomethingWentWrong, "Failed to process information".to_string()))
            ));
        }
    }
}
