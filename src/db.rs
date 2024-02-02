use rocket::{fairing::{self, AdHoc}, Build, Rocket};

use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("sqlx")]
pub struct Db(sqlx::PgPool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(_err) => {
                Err(rocket)
            }
        }
        None => Err(rocket)
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Database startup", |rocket| async {
        rocket.attach(Db::init())
            .attach(AdHoc::try_on_ignite("Database migration", run_migrations))
    })
}
