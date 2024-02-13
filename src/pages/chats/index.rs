use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use rocket::{get, http::CookieJar, response::{content::RawHtml, status}};

use crate::{auth_uri, chats::chat_struct::Chat, consts::{self, TemplateOrHtml}, cookies::settings::{self, Language, Theme}, db::Db, pages::auth::login, user::user_struct::User, utils};

#[get("/?<receiver_id>&<is_htmx>")]
pub async fn page(
    mut db: Connection<Db>,
    user: User,
    cookies: &CookieJar<'_>,
    receiver_id: Option<i32>,
    is_htmx: Option<bool>
) -> Result<TemplateOrHtml, status::Custom<String>> {
    match is_htmx {
        Some(true) => {
            if receiver_id.is_none() {
                return Ok(TemplateOrHtml::Html(
                    RawHtml("<div><p>Select a chat to start chatting.</p></div>".to_string())
                ));  
            }

            let user_chats = Chat::get_messages(&mut db, &user.id, &receiver_id.unwrap()).await;

            match user_chats {
                Ok(user_chats) =>{
                    let upper_html = format!(
                        "
                        <div class=\"chats__container\">
                            <nav class=\"chats__header\">
                                <div>
                                    <img
                                        src=\"{}\"
                                        alt=\"{}'s Profile picture\"
                                        width=\"40\"
                                        height=\"40\"
                                        loading=\"lazy\"
                                    />
                                    <span>{}</span>
                                </div>
                            <div class=\"chats__content\">
                                <ul id=\"chat_info_container\" hx-swap-oob=\"beforeend\">
                        ",
                        user_chats.receiver_avatar,
                        user_chats.receiver_name,
                        user_chats.receiver_name
                    );
                    let mut messages_html = String::new();

                    for chat in user_chats.messages {
                        let display_image = match chat.is_receiver_message {
                            true => &user_chats.receiver_avatar,
                            false => &user_chats.sender_avatar
                        };
                        let display_name = match chat.is_receiver_message {
                            true => &user_chats.receiver_name,
                            false => &user_chats.sender_name
                        };

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
                                            />
                                            <small>{}</small>
                                        </div>
                                        <p>{}</p>
                                    </div>
                                </li>
                                ",
                                chat.is_receiver_message,
                                display_image,
                                display_name,
                                display_name,
                                chat.message
                            ).as_str()
                        );
                    }

                    let end_html = format!(
                        "
                            </ul>
                        </div>
                        <form
                            id=\"chats__form\"
                            ws-send
                            hx-trigger=\"submit\"
                        >
                            <input name=\"receiver_id\" value=\"{}\" hidden>
                            <input name=\"sender_id\" value=\"{}\" hidden>
                            <input name=\"message\" placeholder=\"Type a message...\" required>
                        </form>
                        ",
                        user_chats.receiver_id,
                        user_chats.sender_id
                    );

                    Ok(TemplateOrHtml::Html(RawHtml(format!("{}{}{}", upper_html, messages_html, end_html))))
                },
                Err(err) => {
                    println!("Error: {:?}", err);

                    return Ok(TemplateOrHtml::Html(
                        RawHtml(format!("<div><p>{:?}</p></div>", err))
                    ));
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
            let user_chats = Chat::get_user_chats(&mut db, &user.id, &String::new()).await.ok();
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

            let current_chat = Chat::get_messages(&mut db, &user.id, &receiver_id.unwrap()).await;
            
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

                    return Ok(TemplateOrHtml::Template(
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
                    ));
                }
            }
        }
    }
}

#[get("/", rank = 2)]
pub fn rederirect_if_logged_out() -> rocket::response::Redirect {
    rocket::response::Redirect::to(auth_uri!(login::page))
}
