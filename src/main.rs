#[macro_use] extern crate rocket;

// TODO Expand on post types.
// TODO // Per-post password protection is a bit excessive. What we need is a
// generalizable notion of "zones" - users with certain permissions can see
// posts by zone. These are seperate from tags.

use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};


use rocket::{Rocket, Build, futures};
use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{Serialize, Deserialize, json::Json};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("db")]
struct Db(sqlx::SqlitePool);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![index])
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        }
        None => Err(rocket),
    }
}
