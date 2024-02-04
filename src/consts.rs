<<<<<<< HEAD

pub struct Errors {
    pub invalid_credentials: &'static str,
    pub invalid_file_mime_type: &'static str,
    pub invalid_file_name: &'static str,
    pub weak_password: &'static str,
    pub incomplete_data: &'static str,
    pub invalid_data: &'static str,
    pub something_went_wrong: &'static str,
}

pub const ERRORS: Errors = Errors {
    invalid_credentials: "Invalid credentials",
    invalid_file_mime_type: "Invalid file mime type",
    invalid_file_name: "Invalid file name",
    weak_password: "Weak password",
    incomplete_data: "Incomplete data",
    invalid_data: "Invalid data",
    something_went_wrong: "Something went wrong",
};
=======
/// Placeholder images for the app
/// 1. Placeholder profile picture for male
/// 2. Placeholder profile picture for female
/// 3. Placeholder profile picture for other
pub const PLACEHOLDER_IMAGES: [&str; 3] = [
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/male.jpg",
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/female.jpg",
    "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/other.png"
];
>>>>>>> 8275b32 (refreshes last_login_date on every login)
