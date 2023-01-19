use crate::rocket::futures::{TryFutureExt, TryStreamExt};
use rocket::http::ContentType;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;
use rocket::State;

use crate::Db;
use crate::AppState;

use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

/// A container of arbitrarily size, potentially contained by another container
/// - e.g. a particular building contains a particular toolchest contains a
/// particular cabinet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Container {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub parent_container_id: Option<i64>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<u8>>,
}

#[post("/container", data = "<container>")]
pub async fn create(
    mut db: Connection<Db>,
    container: Json<Container>,
) -> Result<Created<Json<Container>>> {
    sqlx::query!(
        "INSERT INTO container (parent_container_id, name, note, photo) VALUES (?, ?, ?, ?)",
        container.parent_container_id,
        container.name,
        container.note,
        container.photo
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(container))
}

#[get("/container/<id>")]
pub async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Container>> {
    sqlx::query!(
        "SELECT id, parent_container_id, name, note, photo FROM container WHERE id = ?",
        id
    )
    .fetch_one(&mut *db)
    .map_ok(|r| {
        Json(Container {
            id: Some(r.id),
            parent_container_id: r.parent_container_id,
            name: r.name,
            note: r.note,
            photo: r.photo,
        })
    })
    .await
    .ok()
}

#[get("/container")]
pub async fn list(mut db: Connection<Db>) -> Result<Json<Vec<i64>>> {
    let ids = sqlx::query!("SELECT id FROM container")
        .fetch(&mut *db)
        .map_ok(|r| r.id.unwrap())
        .try_collect::<Vec<_>>()
        .await?;

    Ok(Json(ids))
}


#[delete("/container/<id>")]
pub async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM container WHERE id = ?", id) // container has ON DELETE CASCADE
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}

/// Generate a PDF containing QR coded labels for each container
async fn generate_qr_pdf(state: &State<AppState>, mut db:Connection<Db>) -> Result<()> {
    let containers = sqlx::query!("SELECT id, name FROM container")
        .fetch(&mut *db)
        .map_ok(|r| (r.id.unwrap(), r.name))
        .try_collect::<Vec<_>>()
        .await?;

    for (id, name) in containers {
        // TODO
    }

    Ok(())
}

/// Get a pdf containing printable qr coded labels for all containers
#[get("/container/qr")]
pub async fn read_qr_pdf(state: &State<AppState>) -> (ContentType, Vec<u8>) {
    (ContentType::PDF, state.pdf.clone())
}
