#[macro_use]
extern crate rocket;

use rocket_db_pools::sqlx::{self};
use rocket_db_pools::Database;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket::State;

use genpdf::Document;

mod container;
mod item;
mod item_location;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[derive(Database)]
#[database("testdb")] // an in-memory db
pub struct Db(sqlx::SqlitePool);

#[cfg(not(test))]
#[derive(Database)]
#[database("db")]
pub struct Db(sqlx::SqlitePool);


pub struct AppState {
    pub root_url: String,
    pub pdf: Vec<u8>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {

    let state = AppState {
        root_url: String::from("foobar.com"),
        pdf: [1, 2, 3, 4].to_vec()
    };

    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .manage(state)
        .mount("/", routes![index])
        .mount(
            "/",
            routes![container::create, container::read, container::delete],
        )
        .mount("/", routes![item::create, item::read, item::delete])
        .mount(
            "/",
            routes![
                item_location::create,
                item_location::read,
                item_location::delete
            ],
        )
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}
