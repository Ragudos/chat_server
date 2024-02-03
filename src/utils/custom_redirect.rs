use rocket::{http::{uri::Reference, Status}, response::Responder, Response};

/// Since we are using HTMX
/// we need HX-Redirect instead of the usual 303 status code
/// when submitting a form since we use 
/// hx-post
pub struct Redirect(Status, Option<Reference<'static>>);

impl Redirect {
    pub fn to<U: TryInto<Reference<'static>>>(uri: U) -> Redirect {
        Redirect(Status::Ok, uri.try_into().ok())
    }
}

impl<'r> Responder<'r, 'static> for Redirect {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        if let Some(uri) = self.1 {
            Response::build()
                .status(self.0)
                .raw_header("Hx-Redirect", uri.to_string())
                .ok()
        } else {
            println!("Invalid URI used for redirect.");
            Err(Status::InternalServerError)
        }
    }
}

