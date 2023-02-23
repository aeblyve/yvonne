use crate::rocket::futures::{TryFutureExt, TryStreamExt};
use std::io::Cursor;
use rocket::http::ContentType;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;
use rocket::State;

use crate::Db;
use crate::AppState;

use lazy_static::lazy_static;

use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

//use image::Luma;
//use image::ImageBuffer;
//use image::{GrayImage};

use imageproc::drawing::draw_text;

use qrcode_generator::{QrCodeEcc, QRCodeError};

use rusttype::{Font, Scale};
extern crate printpdf;

// imports the `image` library with the exact version that we are using
use printpdf::*;

use std::convert::From;
use std::convert::TryFrom;
use std::fs::File;

use printpdf::image_crate::ImageOutputFormat;
use printpdf::image_crate::Luma;
use printpdf::image_crate::ImageBuffer;
use printpdf::image_crate::{GrayImage};
use crate::QR_CODE_DIMENSION;

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

#[post("/item", data = "<item>")]
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

#[get("/item/<id>")]
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

#[get("/item/qr/<id>")]
pub async fn read_qr(mut db: Connection<Db>, state: &State<AppState>, id:i64) -> (ContentType, Vec<u8>) {
    let foo = sqlx::query!(
        "SELECT id, name FROM item WHERE id = ?",
        id
    )
    .fetch_one(&mut *db)
    .map_ok(|r| {
        //generate_item_qr_label(state, r.id, r.name)
        crate::util::generate_qr_label(state, r.id, r.name, "item")
    }).await.expect("Got image okay");
    let mut bytes: Vec<u8> = Vec::new();
    foo.write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png).expect("Saved as PNG okay");
    (ContentType::PNG, bytes)
}

#[delete("/item/<id>")]
pub async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM item WHERE id = ?", id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}

#[get("/item")]
pub async fn list(mut db: Connection<Db>) -> Result<Json<Vec<i64>>> {
    let ids = sqlx::query!("SELECT id FROM item")
        .fetch(&mut *db)
        .map_ok(|r| r.id.unwrap())
        .try_collect::<Vec<_>>()
        .await?;

    Ok(Json(ids))
}

#[get("/item/qr")]
pub async fn list_qr(state: &State<AppState>, mut db: Connection<Db>) -> (ContentType, Vec<u8>) {
    let items = sqlx::query!("SELECT id, name FROM item")
        .fetch(&mut *db)
        .map_ok(|r| (r.id.unwrap(), r.name))
        .try_collect::<Vec<_>>()
        .await.unwrap();
    (ContentType::PDF, crate::util::generate_qr_pdf(state, items, "item"))
}
