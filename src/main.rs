#![allow(non_snake_case)]
pub mod ops;
pub mod routes;

use crate::routes::*;

use std::collections::HashSet;
use std::fs::read_dir;

use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};

struct MyData {
    filesSet: HashSet<String>,
    imagesSet: HashSet<String>,
}
impl MyData {
    pub fn new_loaded() -> Self {
        let mut filesSet: HashSet<String> = read_dir("./files")
            .unwrap()
            .into_iter()
            .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
            .collect();
        let mut imagesSet: HashSet<String> = read_dir("./imgs")
            .unwrap()
            .into_iter()
            .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
            .collect();

        imagesSet.remove(".keep");
        filesSet.remove(".keep");

        Self {
            filesSet,
            imagesSet,
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Data::new(Mutex::new(MyData::new_loaded()));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE", "PATCH"]);

        App::new()
            .app_data(data.clone())
            .wrap(cors)
            .service(files)
            .service(imgs)
            .service(upload_img)
            .service(delete_img)
            .service(rename_img)
            .service(list_imgs)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
