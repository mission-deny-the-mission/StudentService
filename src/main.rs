mod entity;

#[macro_use] extern crate actix_web;

use actix_web::{HttpServer, App, web::Data, web, middleware::Logger, Responder, HttpResponse, Result};
use entity::{course, student, prelude};
use entity::prelude::{Course, Student};
use sea_orm::prelude::*;
use sea_orm::*;
use serde::Serialize;

const DATABASE_URI: &str = "sqlite://student.db";

#[get("/FetchCourses")]
async fn fetch_courses() -> Result<impl Responder> {
    let db: DatabaseConnection = Database::connect(DATABASE_URI).await.expect("Could not connect to database");
    Ok(web::Json(Course::find().into_json().all(&db)
        .await.expect("Could not fetch records from database")))
//    let courses = Course::find().into_json().all(&db)
//        .await.expect("Could not fetch records");
//    Ok(Json(courses))
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello world"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .service(index)
            .service(fetch_courses)
    })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}
