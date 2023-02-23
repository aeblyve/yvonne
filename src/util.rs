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

const QR_CODE_DIMENSION: usize = 300;

lazy_static! {
    static ref FONT: Font<'static> = {
        let font_data: &[u8] = include_bytes!("../assets/iosevka-regular.ttf");
        Font::try_from_bytes(font_data).expect("Failed to decode font!")
    };
}

pub fn generate_qr_code(state: &State<AppState>, id: i64, model_route: &str) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, QRCodeError> {
    let url = format!("{}/{}/{}", state.root_url, model_route, id);
    qrcode_generator::to_image_buffer(url, QrCodeEcc::Low, QR_CODE_DIMENSION)
}

pub fn generate_qr_label(state: &State<AppState>, id: i64, name: String, model_route: &str) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut code = generate_qr_code(state, id, model_route).unwrap();
    let mut label = GrayImage::new((QR_CODE_DIMENSION) as u32, (QR_CODE_DIMENSION as f32 * 1.25) as u32);

    printpdf::image_crate::imageops::overlay(&mut label, &code, 0, 0);

    let x_scale = 6.0 / name.len() as f32 * 125.0;

    draw_text(&mut label, Luma { 0: [255] }, 0, QR_CODE_DIMENSION as i32 - 5, Scale { x: 84.0, y: 84.0 }, &FONT, name.as_str())
}

pub fn generate_qr_pdf(state: &State<AppState>, model_info: Vec<(i64, String)>, model_name: &str) -> Vec<u8> {
    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let mut imx = Mm(0.76);
    let mut imy = Mm(297.0 - 32.0);

    let mut count = 0;
    for (id, name) in model_info {
        let label = crate::util::generate_qr_label(state, id, name, model_name);
        let img = Image::from_dynamic_image(&label.into());
        let transform = ImageTransform {
            translate_x : Some(imx),
            translate_y : Some(imy),
            rotate : None,
            scale_x : None,
            scale_y : None,
            dpi : Some(300.0)
        };
        img.add_to_layer(current_layer.clone(), transform);

        imx += Mm(25.4);
        imx += Mm(0.76);

        count += 1;
        if count % 8 == 0 {
            imy -= Mm(32.0);
            imy -= Mm(1.11);

            imx = Mm(0.76);
        }
    }

    doc.save_to_bytes().unwrap()
}
