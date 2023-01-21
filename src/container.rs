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

use image::Luma;
use image::ImageBuffer;
use image::{GrayImage};

use imageproc::drawing::draw_text;

use qrcode_generator::{QrCodeEcc, QRCodeError};

use rusttype::{Font, Scale};

lazy_static! {
    static ref FONT: Font<'static> = {
        let font_data: &[u8] = include_bytes!("../assets/iosevka-regular.ttf");
        Font::try_from_bytes(font_data).expect("Decoded font okay")
    };
}





use crate::QR_CODE_DIMENSION;

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

#[get("/container/qr/<id>")]
pub async fn read_qr(mut db: Connection<Db>, state: &State<AppState>, id:i64) -> (ContentType, Vec<u8>) {
    let foo = sqlx::query!(
        "SELECT id, name FROM container WHERE id = ?",
        id
    )
    .fetch_one(&mut *db)
    .map_ok(|r| {
        generate_container_qr_label(state, r.id, r.name)
    }).await.expect("Got image okay");
    let mut bytes: Vec<u8> = Vec::new();
    foo.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png).expect("Saved as PNG okay");
    (ContentType::PNG, bytes)
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
    }

    Ok(())
}

fn generate_container_qr_code(state: &State<AppState>, id: i64) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, QRCodeError> {
    let url = format!("{}/container/{}", state.root_url, id);
    qrcode_generator::to_image_buffer(url, QrCodeEcc::Low, QR_CODE_DIMENSION)
}

fn generate_container_qr_label(state: &State<AppState>, id: i64, name: String) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut code = generate_container_qr_code(state, id).unwrap(); // make space on the left for X chars, limit names to X chars, imageproc::drawing imageops overlay
    let mut label = GrayImage::new((QR_CODE_DIMENSION * 2) as u32, QR_CODE_DIMENSION as u32); // make this a white background?

    image::imageops::overlay(&mut label, &code, QR_CODE_DIMENSION as i64, 0);

    draw_text(&mut label, Luma { 0: [255] }, 0, 0, Scale { x: 100.0, y: 100.0 }, &FONT, name.as_str())
}

/// Get a pdf containing printable qr coded labels for all containers
#[get("/container/qr")]
pub async fn read_qr_pdf(state: &State<AppState>) -> (ContentType, Vec<u8>) {
    (ContentType::PDF, state.pdf.clone())
}
