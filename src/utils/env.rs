use dotenv;

pub fn load_db_url() -> String {
    dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
