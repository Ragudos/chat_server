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
            if user_chats.is_empty() {
                return Ok(RawHtml("<li><p>No chats found</p></li>".to_string()));
            }

            let mut html = String::new();

            for chat in user_chats {
                let receiver_name = if chat.sender_id == user_id {
                    &chat.receiver_name
                } else {
                    &chat.sender_name
                };
                let receiver_avatar = if chat.sender_id == user_id {
                    &chat.receiver_avatar
                } else {
                    &chat.sender_avatar
                };
                let receiver_id = if chat.sender_id == user_id {
                    &chat.receiver_id
                } else {
                    &chat.sender_id
                };
                let sender_id = if chat.sender_id == user_id {
                    &chat.sender_id
                } else {
                    &chat.receiver_id
                };

                html.push_str(&format!(
                    "
                    <li data-iscurrent=\"\">
                        <button
                            type=\"button\"
                            title=\"Chat with {}\"
                            hx-get=\"/chats?sender_id={}&receiver_id={}\"
                            hx-trigger=\"click\"
                            hx-target=\"#chat_container\"
                            hx-sync=\"button[hx-target='#chat_container']:replace\"
                            class=\"ghost\"
                        >
                            <img
                                src=\"{}\"
                                alt=\"{}'s Profile Picture\"
                                width=\"32\"
                                height=\"32\"
                                loading=\"lazy\"
                                class=\"profile\"
                            />
                            <div>
                                <span>{}</span>
                                <p>{}</p>
                            </div>
                        </button>
                        <hr>
                    </li>
                    ",
                    receiver_name,
                    sender_id,
                    receiver_id,
                    receiver_avatar,
                    receiver_name,
                    receiver_name,
                    chat.message
                ));
            }

            Ok(RawHtml(html))
        }
        Err(err) => {
            println!("Error: {:?}", err);

            Ok(
                RawHtml("<li><p>Something went wrong in fetching chats.</p></li>".to_string())
            )
        }
    }
}

#[post("/chats_of_user?<user_id>", data = "<_form>", rank = 2)]
pub fn error_if_logged_out(user_id: i32, _form: Form<SeachFormData>) -> String {
    Error::to_string(Error::new(ErrorReason::Unauthorized, format!("You are not authorized to view the chats of User: {}.", user_id)))
}
