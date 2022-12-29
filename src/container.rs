use crate::rocket::futures::TryFutureExt;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;

use crate::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

/// A container within a site, e.g. a particular tool chest
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Container {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub site_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<u8>>,
}

#[post("/", data = "<container>")]
pub async fn create(
    mut db: Connection<Db>,
    container: Json<Container>,
) -> Result<Created<Json<Container>>> {
    sqlx::query!(
        "INSERT INTO container (site_id, name, note, photo) VALUES (?, ?, ?, ?)",
        container.site_id,
        container.name,
        container.note,
        container.photo
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(container))
}

#[get("/<id>")]
pub async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Container>> {
    sqlx::query!(
        "SELECT id, site_id, name, note, photo FROM container WHERE id = ?",
        id
    )
    .fetch_one(&mut *db)
    .map_ok(|r| {
        Json(Container {
            id: Some(r.id),
            site_id: r.site_id,
            name: r.name,
            note: r.note,
            photo: r.photo,
        })
    })
    .await
    .ok()
}

#[delete("/<id>")]
pub async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM container WHERE id = ?", id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}
