#![allow(non_snake_case)]
pub mod ops;

use std::collections::HashSet;
use std::env::var;

use std::fs::{read_dir, remove_file};
use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::Mutex;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::http::StatusCode;

use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{delete, error, get, post, web, App, HttpServer, Responder};
use actix_web::{HttpResponse, Result};

use futures_util::stream::StreamExt as _;

use image::{DynamicImage, ImageOutputFormat};

use std::fs::OpenOptions;

use crate::ops::Ops;

struct MyData {
    filesSet: HashSet<String>,
    imagesSet: HashSet<String>,
}
impl MyData {
    pub fn new_loaded() -> Self {
        let filesSet = read_dir("./files")
            .unwrap()
            .into_iter()
            .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
            .collect();
        let imagesSet = read_dir("./imgs")
            .unwrap()
            .into_iter()
            .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
            .collect();

        Self {
            filesSet,
            imagesSet,
        }
    }
}

fn image_resp(img: DynamicImage, extension: &str) -> HttpResponse {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, ImageOutputFormat::Png).unwrap();
    HttpResponse::build(StatusCode::OK)
        .content_type("image/".to_owned() + extension)
        .body(w.into_inner())
}

#[get("/files/{filename}")]
async fn files(data: Data<Mutex<MyData>>, filename: web::Path<String>) -> Result<NamedFile> {
    println!("GET /files/{}", filename);
    if data.lock().unwrap().filesSet.contains(&*filename) {
        if let Ok(file) = NamedFile::open(format!("./files/{}", filename)) {
            return Ok(file);
        }
    }

    return Err(error::ErrorNotFound(filename));
}

#[get("/imgs/{filename}")]
async fn imgs(
    data: Data<Mutex<MyData>>,
    filename: web::Path<String>,
    ops: web::Query<Ops>,
) -> HttpResponse {
    println!("GET /imgs/{}", filename);
    if data.lock().unwrap().imagesSet.contains(&*filename) {
        if let Ok(img) = image::open(format!("./imgs/{}", &filename)) {
            return image_resp(
                ops.exec(img),
                Path::new(&*filename).extension().unwrap().to_str().unwrap(),
            );
        }
    }
    HttpResponse::build(StatusCode::NOT_FOUND).finish()
}

#[get("/list/imgs/{key}")]
async fn list_imgs(data: Data<Mutex<MyData>>, key: web::Path<String>) -> impl Responder {
    if &*key != &var("UPLOAD_TOKEN").unwrap() {
        return web::Json::<Vec<String>>(Vec::new());
    }

    web::Json::<Vec<String>>(
        data.lock()
            .unwrap()
            .imagesSet
            .iter()
            .map(|s| s.clone())
            .collect(),
    )
}

#[delete("/imgs/{filename}/{key}")]
async fn delete_img(data: Data<Mutex<MyData>>, info: web::Path<(String, String)>) -> HttpResponse {
    let (filename, key) = info.into_inner();
    println!("POST /imgs/{} - WITH KEY: {}", filename, key);
    if &*key != &var("UPLOAD_TOKEN").unwrap() {
        return HttpResponse::build(StatusCode::UNAUTHORIZED).finish();
    }

    let mut my_data = data.lock().unwrap();
    if my_data.imagesSet.contains(&*filename) {
        if let Ok(_) = remove_file(format!("./imgs/{}", filename)) {
            my_data.imagesSet.remove(&filename);
            return HttpResponse::build(StatusCode::OK).finish();
        }
    }

    HttpResponse::build(StatusCode::NOT_FOUND).finish()
}

#[post("/imgs/{filename}/{key}")]
async fn upload_img(
    data: Data<Mutex<MyData>>,
    mut payload: Multipart,
    info: web::Path<(String, String)>,
) -> HttpResponse {
    let (filename, key) = info.into_inner();
    println!("POST /imgs/{} - WITH KEY: {}", filename, key);
    if &*key != &var("UPLOAD_TOKEN").unwrap() {
        return HttpResponse::build(StatusCode::UNAUTHORIZED).finish();
    }

    if data.lock().unwrap().imagesSet.contains(&*filename) {
        return HttpResponse::build(StatusCode::CONFLICT).finish();
    }

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("./imgs/{}", filename))
    {
        payload.next().await;
        if let Some(item) = payload.next().await {
            let mut field = item.unwrap();

            let mut bytes = web::BytesMut::new();
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                bytes.extend_from_slice(&chunk.unwrap());
            }

            match file.write_all(&bytes) {
                Ok(_) => {
                    data.lock().unwrap().imagesSet.insert(filename);
                    HttpResponse::build(StatusCode::CREATED).finish()
                }
                Err(_) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish(),
            }
        } else {
            HttpResponse::build(StatusCode::SEE_OTHER).finish()
        }
    } else {
        HttpResponse::build(StatusCode::CREATED).finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Data::new(Mutex::new(MyData::new_loaded()));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"]);

        App::new()
            .app_data(data.clone())
            .wrap(cors)
            .service(files)
            .service(imgs)
            .service(upload_img)
            .service(delete_img)
            .service(list_imgs)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
