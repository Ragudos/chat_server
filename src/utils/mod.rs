use crate::{consts::PLACEHOLDER_IMAGES, user::user_struct::{Gender, User}};

pub mod env;
pub mod custom_redirect;

pub fn get_placeholder_display_image(
    user: &Option<User>
) -> Option<&'static str> {
    match user {
        Some(user) => {
            let gender = &user.gender;

            match gender {
                Gender::Male => PLACEHOLDER_IMAGES.get(0).copied(),
                Gender::Female => PLACEHOLDER_IMAGES.get(1).copied(),
                Gender::Other => PLACEHOLDER_IMAGES.get(2).copied()
            }
        }
        None => None
    }
}