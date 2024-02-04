use rocket::{response::Responder, Response};
use serde::{Deserialize, Serialize};

pub enum ErrorReason {
    Invalid,
    Required,
    Unauthorized,
    AlreadyExists,
    SomethingWentWrong,
    WeakPassword,
    InvalidCredentials,
    InvalidMimeType,
    InvalidFileName,
    IncompleteData,
    ALreadyLoggedIn,
    InvalidRequest
}

pub struct Error {
    pub reason: ErrorReason,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorJson {
    pub code: u16,
    pub message: String,
    pub reason: String,
}

impl Error {
    pub fn new(reason: ErrorReason, message: String) -> Self {
        Self {
            reason,
            message,
        }
    }

    pub fn from_str(str: &str) -> Result<Self, &'static str> {
        let mut parts = str.splitn(2, ':');

        if let Some(reason) = parts.next() {
            if let Some(message) = parts.next() {
                Ok(Self {
                    reason: ErrorReason::from_str(reason),
                    message: message.to_string(),
                })
            } else {
                Err("Invalid error string")
            }
        } else {
            Err("Invalid error string")
        }
    }

    pub fn to_string(self) -> String {
        format!("{}: {}", self.reason.as_str(), self.message)
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build().ok()
    }
}

impl ErrorReason {
    pub fn from_str(str: &str) -> Self {
        match str {
            "invalid" => ErrorReason::Invalid,
            "required" => ErrorReason::Required,
            "unauthorized" => ErrorReason::Unauthorized,
            "already_exists" => ErrorReason::AlreadyExists,
            "something_went_wrong" => ErrorReason::SomethingWentWrong,
            "weak_password" => ErrorReason::WeakPassword,
            "invalid_credentials" => ErrorReason::InvalidCredentials,
            "invalid_mime_type" => ErrorReason::InvalidMimeType,
            "invalid_file_name" => ErrorReason::InvalidFileName,
            "incomplete_data" => ErrorReason::IncompleteData,
            "already_logged_in" => ErrorReason::ALreadyLoggedIn,
            "invalid_request" => ErrorReason::InvalidRequest,
            _ => ErrorReason::SomethingWentWrong,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorReason::Invalid => "invalid",
            ErrorReason::Required => "required",
            ErrorReason::Unauthorized => "unauthorized",
            ErrorReason::AlreadyExists => "already_exists",
            ErrorReason::SomethingWentWrong => "something_went_wrong",
            ErrorReason::WeakPassword => "weak_password",
            ErrorReason::InvalidCredentials => "invalid_credentials",
            ErrorReason::InvalidMimeType => "invalid_mime_type",
            ErrorReason::InvalidFileName => "invalid_file_name",
            ErrorReason::IncompleteData => "incomplete_data",
            ErrorReason::ALreadyLoggedIn => "already_logged_in",
            ErrorReason::InvalidRequest => "invalid_request",
        }
    }
}
