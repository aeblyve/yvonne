use crate::rocket::futures::{TryFutureExt, TryStreamExt};
use rocket::http::ContentType;
use rocket::State;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;
use std::io::Cursor;

use crate::AppState;
use crate::Db;

use lazy_static::lazy_static;

use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

//use image::Luma;
//use image::ImageBuffer;
//use image::{GrayImage};

use imageproc::drawing::draw_text;

use qrcode_generator::{QRCodeError, QrCodeEcc};

use rusttype::{Font, Scale};
extern crate printpdf;

// imports the `image` library with the exact version that we are using
use printpdf::*;

use std::convert::From;
use std::convert::TryFrom;
use std::fs::File;

use crate::QR_CODE_DIMENSION;
use printpdf::image_crate::GrayImage;
use printpdf::image_crate::ImageBuffer;
use printpdf::image_crate::ImageOutputFormat;
use printpdf::image_crate::Luma;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PutContainer {
    pub parent_container_id: Option<i64>,
    pub name: String,
    pub note: Option<String>,
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

#[get("/container/qr/<id>")]
pub async fn read_qr(
    mut db: Connection<Db>,
    state: &State<AppState>,
    id: i64,
) -> (ContentType, Vec<u8>) {
    let foo = sqlx::query!("SELECT id, name FROM container WHERE id = ?", id)
        .fetch_one(&mut *db)
        .map_ok(|r| {
            //generate_container_qr_label(state, r.id, r.name)
            crate::util::generate_qr_label(state, r.id, r.name, "container")
        })
        .await
        .expect("Got image okay");
    let mut bytes: Vec<u8> = Vec::new();
    foo.write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)
        .expect("Saved as PNG okay");
    (ContentType::PNG, bytes)
}

#[put("/container/<id>", data = "<container>")]
pub async fn full_update(
    mut db: Connection<Db>,
    id: i64,
    container: Json<PutContainer>,
) -> Result<Created<Json<Container>>> {
    sqlx::query!(
        "UPDATE container SET parent_container_id=?, name=?, note=?, photo=? WHERE id = ?",
        container.parent_container_id,
        container.name,
        container.note,
        container.photo,
        id
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/")) // TODO revisit this return
}

#[delete("/container/<id>")]
pub async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM container WHERE id = ?", id) // container has ON DELETE CASCADE
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
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

#[get("/container/qr")]
pub async fn list_qr(state: &State<AppState>, mut db: Connection<Db>) -> (ContentType, Vec<u8>) {
    let containers = sqlx::query!("SELECT id, name FROM container")
        .fetch(&mut *db)
        .map_ok(|r| (r.id.unwrap(), r.name))
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    (
        ContentType::PDF,
        crate::util::generate_qr_pdf(state, containers, "container"),
    )
}
