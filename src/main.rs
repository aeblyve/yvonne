#[macro_use] extern crate rocket;

use rocket_db_pools::{Database};
use rocket_db_pools::sqlx::{self};

use rocket::{Rocket, Build};
use rocket::fairing::{self, AdHoc};



mod site;
#[cfg(test)] mod tests;

#[cfg(test)]
#[derive(Database)]
#[database("testdb")] // an in-memory db
pub struct Db(sqlx::SqlitePool);

#[cfg(not(test))]
#[derive(Database)]
#[database("db")]
pub struct Db(sqlx::SqlitePool);

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
        .mount("/site", routes![site::create])
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
