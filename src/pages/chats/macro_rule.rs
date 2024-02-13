/// This is a custom uri! macro to make the uri! redirect prepend the /auth path
#[macro_export]
macro_rules! chats_uri {
    ($($t:tt)*) => {
        rocket::uri!("/chats", $($t)*)
    }
}
