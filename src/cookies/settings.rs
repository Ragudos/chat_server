use rocket::http::CookieJar;

pub enum Theme {
    Light,
    Dark,
    System
}

pub enum Language {
    English,
    Tagalog
}

pub struct Settings {
    pub theme: Theme,
    pub custom_theme: String,
    pub language: Language,
}

impl<'a> Theme {
    pub fn as_str(&self) -> &'a str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(str: &'a str) -> Theme {
        match str {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        }
    }
}

impl<'a> Language {
    pub fn as_str(&self) -> &'a str {
        match self {
            Language::English => "en",
            Language::Tagalog => "ph",
        }
    }

    pub fn from_str(str: &'a str) -> Language {
        match str {
            "en" => Language::English,
            "ph" => Language::Tagalog,
            _ => Language::English,
        }
    }
}

pub fn get_default_theme(cookie: &CookieJar<'_>) -> Theme {
    if let Some(theme) = cookie.get("theme") {
        let theme = theme.value_trimmed();

        Theme::from_str(theme)
    } else {
        Theme::Light
    }
}

pub fn get_default_language(cookie: &CookieJar<'_>) -> Language {
    if let Some(lang) = cookie.get("lang") {
        let lang = lang.value_trimmed();

        Language::from_str(lang)
    } else {
        Language::English
    }
}
