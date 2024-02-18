#[macro_use] extern crate rocket;

use chat_server::{api, catchers, chats::chat_struct::Chat, db::{self, Db}, pages::{auth, chats, homepage}, user::user_struct::User, utils::get_placeholder_display_image};
use rocket::{form::Form, fs::FileServer, http::Status, response::{content::RawHtml, status, stream::{Event, EventStream}}, tokio::sync::broadcast::{channel, error::RecvError, Sender}, Shutdown, State};
use rocket_csrf_token::{CsrfConfig, Fairing};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{handlebars::handlebars_helper, Template};
use serde::{Deserialize, Serialize};
use rocket::tokio::select;

handlebars_helper!(eq_str: |first_arg: String, second_arg: String| first_arg == second_arg);
handlebars_helper!(eq_num: |first_arg: isize, second_arg: isize| first_arg == second_arg);

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ChatRoomMessage {
    sender_id: i32,
    receiver_id: i32,
    message: String,
    created_at: Option<String>,
}

#[get("/events/chats?<sender_id>&<receiver_id>")]
async fn chats_sse(
    mut db: Connection<Db>,
    queue: &State<Sender<ChatRoomMessage>>,
    sender_id: i32,
    receiver_id: i32,
    user: User,
    mut end: Shutdown
) -> Result<EventStream![], status::Custom<String>> {
    if user.id != sender_id && receiver_id != user.id {
        return Err(status::Custom(Status::Unauthorized, "Unauthorized".to_string()));
    }

    let receiver: Option<User>;

    if receiver_id == user.id {
        receiver = Some(user.clone());
    } else {
        receiver = User::get_by_id(&mut db, &receiver_id).await;
    }
    
    if receiver.is_none() {
        return Err(status::Custom(Status::NotFound, "User to chat with not found.".to_string()));
    }

    let receiver = receiver.unwrap();
    let mut rx = queue.subscribe();

    Ok(EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => {
                        println!("{}: {:?}", user.id, msg);
                        msg
                    },
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue
                },
                _ = &mut end => break
            };

            if (msg.sender_id == sender_id && msg.receiver_id == receiver_id) || (msg.sender_id == receiver_id && msg.receiver_id == sender_id) {
                let name = if msg.sender_id == user.id {
                    &user.display_name
                } else {
                    &receiver.display_name
                };

                let display_image = if msg.sender_id == user.id {
                    get_placeholder_display_image(user.display_image.as_ref(), &user.gender)
                } else {
                    get_placeholder_display_image(receiver.display_image.as_ref(), &receiver.gender)
                };

                let is_receiver = msg.sender_id != user.id;
                let mut html: String;

                if is_receiver {
                    html = format!(
                        r#"<li data-isreceiver="{}">
                            <div class="chats__message">
                                <div>
                                <img
                                    src="{}"
                                    alt="{}'s Profile picture"
                                    width="28"
                                    height="28"
                                    loading="lazy"
                                    class="profile"
                                />
                                    <div>
                                        <small>{}</small>
                                        <p>{}</p>
                                    </div>
                                </div>
                                <time>{}</time>
                            </div>
                        </li>"#,
                        is_receiver,
                        display_image,
                        name,
                        name,
                        msg.message,
                        msg.created_at.unwrap()
                    );
                } else {
                    html = format!(
                        r#"<li data-isreceiver="{}">
                            <div class="chats__message">
                                <div>
                                    <p>{}</p>
                                </div>
                                <time>{}</time>
                            </div>
                        </li>"#,
                        is_receiver,
                        msg.message,
                        msg.created_at.unwrap()
                    );
                }

                let id = format!("msg_{}{}", sender_id, receiver_id);

                html.push_str(format!(
                    "
                        <p hx-swap-oob=\"true\" id=\"{}\">{}</p>
                    ",
                    id,
                    msg.message
                ).as_str());

                yield Event::data(
                    html
                ).event("message")
            }
        }
    })
}

#[post("/chats/send", data = "<data>")]
async fn send_msg(
    mut db: Connection<Db>,
    data: Form<ChatRoomMessage>,
    user: User,
    queue: &State<Sender<ChatRoomMessage>>
) -> Result<RawHtml<String>, status::Custom<String>>{
    let receiver_id = &data.receiver_id;
    let sender_id = &data.sender_id;
    let message = &data.message;

    if sender_id != &user.id {
        return Err(status::Custom(Status::Unauthorized, "Unauthorized".to_string()));
    }

    let receiver_name = User::get_display_name(&mut db, receiver_id).await;

    if receiver_name.is_none() {
        return Err(status::Custom(Status::NotFound, "User to chat with not found.".to_string()));
    }

    let res = Chat::save_chat(&mut db, sender_id, receiver_id, &receiver_name.unwrap(), message).await;

    if res.is_err() {
        return Err(status::Custom(Status::InternalServerError, "Something went wrong. Please try again.".to_string()));
    }

    let created_at = res.unwrap();

    let _res = queue.send(ChatRoomMessage {
        sender_id: *sender_id,
        receiver_id: *receiver_id,
        message: message.clone(),
        created_at: Some(format!("{}-{}-{} at {}:{}:{}", created_at.year(), created_at.month(), created_at.day(), created_at.hour(), created_at.minute(), created_at.second()))
    });

    Ok(RawHtml(r#"
        <input id="message_input" type="text" required name="message" placeholder="Type a message" />
    "#.to_string()))
}

#[launch]
fn rocket() -> _  {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![homepage::page, chats_sse, send_msg])
        .mount("/auth", routes![
            auth::login::page,
            auth::login::redirect_if_logged_in,
            auth::register::page,
            auth::register::redirect_if_logged_in,
            auth::index::page,
            auth::index::redirect_if_logged_out,
            auth::api::login::login_user,
            auth::api::register::register_user,
            auth::api::logout::logout_user,
        ])
        .mount("/chats", routes! [
            chats::api::chats_of_user::chats_of_user,
            chats::index::page,
            chats::api::chats_of_user::error_if_logged_out,
            chats::index::rederirect_if_logged_out,
        ])
        .mount("/search", routes! [
            api::search::search,
            api::search::unauthorized_search
        ])
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("eq_str", Box::new(eq_str));

            engines
                .handlebars
                .register_helper("eq_num", Box::new(eq_num));

            engines
                .handlebars
                .register_partial("header", "
                    <header>
                        <div class=\"container\">
                            <a class=\"link\" href=\"/\" title=\"{{metadata.title}} Home\">
                                {{metadata.title}}
                            </a>
                            <div class=\"header__search-container\">
                                <input hx-sync=\"this:replace\" hx-target=\"#list-of-search-results\" hx-post=\"/search\" hx-trigger=\"input changed delay:500ms, search\" type=\"search\" placeholder=\"Search\" name=\"search\" />
                                <button type=\"button\" id=\"mobile-open-search\">
                                    <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"currentColor\">
                                        <path fill-rule=\"evenodd\" d=\"M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z\" clip-rule=\"evenodd\" />
                                    </svg>
                                </button>
                                <div class=\"list-of-search-results-container\">
                                    <div id=\"list-of-search-results\"></div>
                                </div>
                            </div>
                            <div class=\"header__user\">
                                {{#if user}}
                                    <button class=\"icon ghost\" aria-controls=\"user-dropdown\" type=\"button\" id=\"user-dropdown-control\" aria-expanded=\"false\">
                                        <img
                                            src=\"{{#if (eq_str user.displayImage \"\")}}{{placeholder_display_image}}{{else}}{{user.displayImage}}{{/if}}\"
                                            alt=\"{{user.displayName}}'s Profile picture\"
                                            width=\"40\"
                                            height=\"40\"
                                            loading=\"eager\"
                                            class=\"profile ghost\"
                                        />
                                    </button>
                                    <div class=\"dropdown\" id=\"user-dropdown\">
                                        <div>
                                            <ul>
                                                <li>
                                                    <a tabindex=\"-1\" href=\"/profile?user_id={{user.id}}\" title=\"Profile\"><small>Profile</small></a>
                                                </li>
                                                <li>
                                                    <button tabindex=\"-1\" type=\"button\" hx-delete=\"/auth/logout\" title=\"Logout\"><small>Logout</small></button>
                                                </li>
                                            </ul>
                                        </div>
                                    </div>
                                {{else}}
                                    <a href=\"/auth/login\" class=\"primary\" title=\"Sign in\">Sign in</a>
                                {{/if}}
                            </div>
                        </div>
                    </header>
                ")
                .unwrap();
        }))
        .attach(Fairing::new(CsrfConfig::default()))
        .attach(db::stage())
        .register("/", catchers![catchers::internal_error, catchers::not_found, catchers::unauthorized])
        .mount("/assets", FileServer::from("assets"))
        .manage(channel::<ChatRoomMessage>(1024).0)
}
