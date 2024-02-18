use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use rocket::{get, http::{CookieJar, Status}, response::{content::RawHtml, status}};

use crate::{auth_uri, chats::chat_struct::Chat, consts::{self, TemplateOrHtml}, cookies::settings::{self, Language, Theme}, db::Db, pages::auth::login, user::user_struct::User, utils};

#[get("/?<sender_id>&<receiver_id>&<is_htmx>")]
pub async fn page(
    mut db: Connection<Db>,
    user: User,
    cookies: &CookieJar<'_>,
    sender_id: Option<i32>,
    receiver_id: Option<i32>,
    is_htmx: Option<bool>
) -> Result<TemplateOrHtml, status::Custom<String>> {
    let sender_id_mut: Option<i32>;

    match sender_id {
        Some(sender_id) => {
            if sender_id != user.id && receiver_id != Some(user.id){
                return Err(status::Custom(
                    Status::Unauthorized,
                    "You are not authorized to view this chat.".to_string()
                ));
            }
            
            sender_id_mut = Some(sender_id);
        }
        None => {
            sender_id_mut = Some(user.id);
        }
    }

    let sender_id_mut = sender_id_mut.unwrap();

    match is_htmx {
        Some(true) => {
            if receiver_id.is_none() {
                return Ok(TemplateOrHtml::Html(
                    RawHtml("<div><p>Select a chat to start chatting.</p></div>".to_string())
                ));  
            }

            let user_chats = Chat::get_messages(&mut db, &sender_id_mut, &receiver_id.unwrap()).await;

            match user_chats {
                Ok(user_chats) => {
                    let receiver_name = if user_chats.receiver_id == user.id {
                        &user_chats.sender_name
                    } else {
                        &user_chats.receiver_name
                    };
                    let receiver_avatar = if user_chats.receiver_id == user.id {
                        &user_chats.sender_avatar
                    } else {
                        &user_chats.receiver_avatar
                    };

                    let upper_html = format!(
                        "
                        <div class=\"chats__container\" hx-swap=\"beforeend scroll:down\" hx-target=\"#chat_info_container\" sse-swap=\"message\" sse-connect=\"/events/chats?{}\">
                            <nav class=\"chats__header\">
                                <div>
                                    <img
                                        src=\"{}\"
                                        alt=\"{}'s Profile picture\"
                                        width=\"40\"
                                        height=\"40\"
                                        loading=\"lazy\"
                                        class=\"profile\"
                                    />
                                    <span>{}</span>
                                </div>
                            </nav>
                            <ul id=\"chat_info_container\">
                        ",
                        user_chats.id,
                        receiver_avatar,
                        receiver_name,
                        receiver_name,
                    );
                    let mut messages_html = String::new();

                    for chat in user_chats.messages {
                        let is_receiver = chat.is_receiver_message;
                        let display_image = if is_receiver {
                            &user_chats.receiver_avatar
                        } else {
                            &user_chats.sender_avatar
                        };
                        let display_name = if is_receiver {
                            &user_chats.receiver_name
                        } else {
                            &user_chats.sender_name
                        };

                        if is_receiver {
                            messages_html.push_str(
                                format!(
                                    "
                                    <li data-isreceiver=\"{}\">
                                        <div class=\"chats__message\">
                                            <div>
                                                <img
                                                    src=\"{}\"
                                                    alt=\"{}'s Profile picture\"
                                                    width=\"28\"
                                                    height=\"28\"
                                                    loading=\"lazy\"
                                                    class=\"profile\"
                                                />
                                                <div>
                                                    <small>{}</small>
                                                    <p>{}</p>
                                                </div>
                                            </div>
                                            <time>{}</time>
                                        </div>
                                    </li>
                                    ",
                                    is_receiver,
                                    display_image,
                                    display_name,
                                    display_name,
                                    chat.message,
                                    chat.created_at
                                ).as_str()
                            );
                        } else {
                            messages_html.push_str(
                                format!(
                                    "
                                    <li data-isreceiver=\"{}\">
                                        <div class=\"chats__message\">
                                            <div>
                                                <p>{}</p>
                                            </div>
                                            <time>{}</time>
                                        </div>
                                    </li>
                                    ",
                                    is_receiver,
                                    chat.message,
                                    chat.created_at
                                ).as_str()
                            );
                        }
                    }

                    let end_html = format!(
                        "   </ul>
                        <div>
                            <form
                                id=\"chats__form\"
                                hx-post=\"/chats/send\"
                                hx-trigger=\"submit\"
                                hx-swap=\"outerHTML\"
                                hx-target=\"#message_input\"
                            >
                                <input name=\"receiver_id\" value=\"{}\" hidden>
                                <input name=\"sender_id\" value=\"{}\" hidden>
                                <input id=\"message_input\" name=\"message\" placeholder=\"Type a message...\" required>
                                <button type=\"submit\" title=\"Send Message\">Send</button>
                            </form>
                        </div>
                        </div>
                        ",
                        user_chats.receiver_id,
                        user_chats.sender_id
                    );

                    Ok(TemplateOrHtml::Html(RawHtml(format!("{}{}{}", upper_html, messages_html, end_html))))
                },
                Err(err) => {
                    println!("Error: {:?}", err);

                    Ok(TemplateOrHtml::Html(
                        RawHtml(format!("<div><p>{:?}</p></div>", err))
                    ))
                }
            }
        },
        Some(false) | None => {
            let preferred_theme = Theme::as_str(
                &settings::get_default_theme(cookies)
            );
            let language = Language::as_str(
                &settings::get_default_language(cookies)
            );
            let user_chats = Chat::get_user_chats(&mut db, &sender_id_mut, &String::new()).await.ok();
            let placeholder_display_image = utils::get_placeholder_display_image(user.display_image.as_ref(), &user.gender);

            if receiver_id.is_none() {
                return Ok(TemplateOrHtml::Template(
                    Template::render(
                        "chats",
                        context! {
                            user,
                            theme: preferred_theme,
                            lang: language,
                            chats: user_chats,
                            metadata: consts::METADATA,
                            placeholder_display_image
                        }
                    )
                ));
            }

            let current_chat = Chat::get_messages(&mut db, &sender_id_mut, &receiver_id.unwrap()).await;
            
            match current_chat {
                Ok(current_chat) => {
                    Ok(TemplateOrHtml::Template(Template::render(
                        "chats",
                        context! {
                            user,
                            theme: preferred_theme,
                            lang: language,
                            chats: user_chats,
                            current_chat,
                            receiver_id,
                            metadata: consts::METADATA,
                            placeholder_display_image
                        }
                    )))
                },
                Err(err) => {
                    println!("Error: {:?}", err);

                    Ok(TemplateOrHtml::Template(
                        Template::render(
                            "chats",
                            context! {
                                user,
                                theme: preferred_theme,
                                lang: language,
                                chats: user_chats,
                                receiver_id,
                                error: "Failed to get chats.",
                                metadata: consts::METADATA,
                                placeholder_display_image
                            }
                        ) 
                    ))
                }
            }
        }
    }
}

#[get("/", rank = 2)]
pub fn rederirect_if_logged_out() -> rocket::response::Redirect {
    rocket::response::Redirect::to(auth_uri!(login::page))
}
