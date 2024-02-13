use rocket::{response::{content::RawHtml, Redirect}, Responder};
use serde::{Deserialize, Serialize};
use rocket_dyn_templates::Template;

use crate::utils::custom_redirect::Redirect as HtmxRedirect;

#[derive(Responder)]
pub enum TemplateOrHtml {
    Template(Template),
    Html(RawHtml<String>)
}

#[derive(Responder)]
pub enum TemplateOrRedirect {
    Template(Template),
    Redirect(Redirect),
    HtmxRedirect(HtmxRedirect)
}

/// Placeholder images for the app
/// 1. Placeholder profile picture for male
/// 2. Placeholder profile picture for female
/// 3. Placeholder profile picture for other
pub const PLACEHOLDER_IMAGES: [&str; 3] = [
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/male.jpg",
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/female.jpg",
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/other.png"
];

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub title: &'static str,
    pub description: &'static str,
}

pub const METADATA: Metadata = Metadata {
    title: "Chat App",
    description: "Chat app while practicing Rocket",
};
