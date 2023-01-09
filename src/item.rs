use crate::rocket::futures::TryFutureExt;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;

use crate::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

/// An item type - not an individual item. e.g. M3 bolt, 20mm long
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Item {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<u8>>,
}

#[post("/", data = "<item>")]
pub async fn create(mut db: Connection<Db>, item: Json<Item>) -> Result<Created<Json<Item>>> {
    sqlx::query!(
        "INSERT INTO item (name, note, photo) VALUES (?, ?, ?)",
        item.name,
        item.note,
        item.photo
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(item))
}

#[get("/<id>")]
pub async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Item>> {
    sqlx::query!("SELECT id,name, note, photo FROM item WHERE id = ?", id)
        .fetch_one(&mut *db)
        .map_ok(|r| {
            Json(Item {
                id: Some(r.id),
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
    let result = sqlx::query!("DELETE FROM item WHERE id = ?", id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}
