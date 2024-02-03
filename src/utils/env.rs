use dotenv;

pub fn load_db_url() -> String {
    dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn load_cloudinary_api_key() -> String {
    dotenv::var("CLOUDINARY_API_KEY").expect("CLOUDINARY_API_KEY must be set")
}

pub fn load_cloudinary_api_secret() -> String {
    dotenv::var("CLOUDINARY_API_SECRET").expect("CLOUDINARY_API_SECRET must be set")
}

pub fn load_cloudinary_cloud_name() -> String {
    dotenv::var("CLOUDINARY_CLOUD_NAME").expect("CLOUDINARY_CLOUD_NAME must be set")
}


