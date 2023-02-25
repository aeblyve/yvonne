use crate::rocket::futures::TryFutureExt;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;

use crate::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

/// Item location - e.g. container x has 5 of part y in cubby "5A"
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ItemLocation {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub item_id: i64,
    pub container_id: i64,
    pub quantity: Option<i64>,
}

/// Create a name item location.
#[post("/itemloc", data = "<itemloc>")]
pub async fn create(
    mut db: Connection<Db>,
    itemloc: Json<ItemLocation>,
) -> Result<Created<Json<ItemLocation>>> {
    sqlx::query!(
        "INSERT INTO item_location (item_id, container_id, quantity) VALUES (?, ?, ?)",
        itemloc.item_id,
        itemloc.container_id,
        itemloc.quantity,
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(itemloc))
}

#[get("/itemloc/<id>")]
pub async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<ItemLocation>> {
    sqlx::query!(
        "SELECT id, item_id, container_id, quantity FROM item_location WHERE id = ?",
        id
    )
    .fetch_one(&mut *db)
    .map_ok(|r| {
        Json(ItemLocation {
            id: Some(r.id),
            item_id: r.item_id,
            container_id: r.container_id,
            quantity: r.quantity,
        })
    })
    .await
    .ok()
}

#[delete("/itemloc/<id>")]
pub async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM item_location where id = ?", id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}
