use rocket::{form::Form, post, response::content::RawHtml, FromForm};
use rocket_db_pools::Connection;

use crate::{db::Db, user::user_struct::User};

#[derive(FromForm)]
pub struct SearchData {
    search: String
}

#[post("/", data = "<data>")]
pub async fn search(
    mut db: Connection<Db>,
    data: Form<SearchData>,
    user: User
) -> RawHtml<String> {
    if data.search.is_empty() {
        return RawHtml("".to_string());
    }

    let users = User::search_for_users_with_display_name(&mut db, &user.display_name, &data.search).await;

    match users {
        Ok(users) => {
            if users.is_empty() {
                return RawHtml("<ul><li><p>No users found</p></li></ul>".to_string());
            }

            let mut html = String::new();

            html.push_str("<ul>");

            for user in users {
                html.push_str(&format!(
                    "
                    <li>
                        <button
                            type=\"button\"
                            title=\"Chat with {}\"
                            hx-get=\"/chats?receiver_id={}&is_htmx=true\"
                            hx-trigger=\"click\"
                            hx-target=\"#chat_container\"
                        >
                            <img
                                src=\"{}\"
                                alt=\"{}'s Profile Picture\"
                                width=\"32\"
                                height=\"32\"
                                loading=\"lazy\"
                                class=\"profile\"
                            />
                            {}
                        </button>
                    </li>
                    ",
                    user.display_name,
                    user.id,
                    user.display_image.unwrap_or_else(|| "https://via.placeholder.com/40".to_string()),
                    user.display_name,
                    user.display_name
                ));
            }

            html.push_str("</ul>");

            RawHtml(html)
        },
        Err(_) => RawHtml("<ul><li><p>No users found</p></li></ul>".to_string())
    
    }
}

#[post("/", data = "<_data>", rank = 2)]
pub fn unauthorized_search(
    _data: Form<SearchData>,
) -> String {
    "You are not authorized to search without being logged in.".to_string()
}
