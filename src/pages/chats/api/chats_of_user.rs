use rocket::{form::Form, post, response::content::RawHtml, FromForm};
use rocket_db_pools::Connection;

use crate::{chats::chat_struct::Chat, db::Db, errors::error::{Error, ErrorReason}, user::user_struct::User};

#[derive(FromForm)]
pub struct SeachFormData {
    search: String
}

#[post("/chats_of_user?<user_id>", data = "<data>")]
pub async fn chats_of_user(
    mut db: Connection<Db>,
    user: User,
    user_id: i32,
    data: Form<SeachFormData>
) -> Result<RawHtml<String>, String> {
    if user.id != user_id {
        return Err(Error::to_string(Error::new(ErrorReason::Unauthorized, format!("You are not authorized to view the chats of User: {}.", user_id))));
    }

    let user_chats = Chat::get_user_chats(&mut db, &user_id, &data.search).await;

    match user_chats {
        Ok(user_chats) => {
            if user_chats.len() == 0 {
                return Ok(RawHtml("<li><p>No chats found</p></li>".to_string()));
            }

            let mut html = String::new();

            for chat in user_chats {
                html.push_str(&format!(
                    "
                    <li>
                        <button
                            type=\"button\"
                            title=\"Chat with {}\"
                            hx-get=\"/chats?sender_id={}&receiver_id={}\"
                            hx-trigger=\"click\"
                            hx-target=\"#chat_container\"
                        >
                            <img
                                src=\"{}\"
                                alt=\"{}'s Profile Picture\"
                                width=\"40\"
                                height=\"40\"
                                loading=\"lazy\"
                            />
                            {}
                        </button>
                    </li>
                    ",
                    chat.receiver_name,
                    chat.sender_id,
                    chat.receiver_id,
                    chat.receiver_avatar,
                    chat.receiver_name,
                    chat.receiver_name
                ));
            }

            Ok(RawHtml(html))
        }
        Err(err) => {
            println!("Error: {:?}", err);

            return Ok(
                RawHtml("<li><p>Something went wrong in fetching chats.</p></li>".to_string())
            )
        }
    }
}

#[post("/chats_of_user?<user_id>", data = "<_form>", rank = 2)]
pub fn error_if_logged_out(user_id: i32, _form: Form<SeachFormData>) -> String {
    Error::to_string(Error::new(ErrorReason::Unauthorized, format!("You are not authorized to view the chats of User: {}.", user_id)))
}
