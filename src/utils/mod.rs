use crate::{consts::PLACEHOLDER_IMAGES, user::user_struct::Gender};

pub mod env;
pub mod custom_redirect;

pub fn get_placeholder_display_image(
    display_image: Option<&String>,
    gender: &Gender
) -> String {
    match display_image {
        Some(display_image ) => {
            if display_image.is_empty() {
                match gender {
                    Gender::Male => PLACEHOLDER_IMAGES.first().unwrap().to_string(),
                    Gender::Female => PLACEHOLDER_IMAGES.get(1).unwrap().to_string(),
                    Gender::Other => PLACEHOLDER_IMAGES.get(2).unwrap().to_string()
                }
            } else {
                display_image.clone()
            }
        },
        None => {
            match gender {
                Gender::Male => PLACEHOLDER_IMAGES.first().unwrap().to_string(),
                Gender::Female => PLACEHOLDER_IMAGES.get(1).unwrap().to_string(),
                Gender::Other => PLACEHOLDER_IMAGES.get(2).unwrap().to_string()
            }
        }
    }
}