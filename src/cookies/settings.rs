use rocket::{http::CookieJar, FromFormField};

#[derive(FromFormField)]
pub enum Theme {
    #[field(value = "light")]
    Light,
    #[field(value = "dark")]
    Dark,
    #[field(value = "system")]
    System
}

#[derive(FromFormField)]
pub enum Language {
    #[field(value = "en")]
    English,
    #[field(value = "ph")]
    Tagalog
}

pub struct Settings {
    pub theme: Theme,
    pub custom_theme: String,
    pub language: Language,
}

impl From<&str> for Theme {
    fn from(theme: &str) -> Self {
        match theme {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light
        }
    }
}

pub fn get_default_theme(cookie: &CookieJar<'_>) -> String {
    if let Some(theme) = cookie.get("theme") {
        let theme = theme.value_trimmed();

        match theme.into() {
            Theme::Light => String::from("light"),
            Theme::Dark => String::from("dark"),
            Theme::System => String::from("system"),
        }
    } else {
        String::from("light")
    }
}
