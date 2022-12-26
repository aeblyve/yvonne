use crate::rocket::futures::TryFutureExt;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;

use crate::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Site {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    name: String,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    photo: Option<Vec<u8>>,
}

#[post("/", data = "<site>")]
pub async fn create(mut db: Connection<Db>, site: Json<Site>) -> Result<Created<Json<Site>>> {
    // how to handle nullables in the query?
    sqlx::query!(
        "INSERT INTO site (name, note, photo) VALUES (?, ?, ?)",
        site.name,
        site.note,
        site.photo
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(site))
}

#[get("/<id>")]
pub async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Site>> {
    sqlx::query!("SELECT id, name, note, photo FROM site WHERE id = ?", id)
        .fetch_one(&mut *db)
        .map_ok(|r| {
            Json(Site {
                id: Some(r.id),
                name: r.name,
                note: r.note,
                photo: r.photo,
            })
        })
        .await
        .ok()
}
