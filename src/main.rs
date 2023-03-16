mod entity;
mod view;

#[macro_use] extern crate actix_web;

use actix_web::{HttpServer, App, web::Data, web, middleware::Logger, Responder, HttpResponse, Result};
use entity::{course, student, prelude};
use entity::prelude::{Course, Student};
use sea_orm::prelude::*;
use sea_orm::*;
use serde::Serialize;
use reqwest;
use view::*;

const DATABASE_URI: &str = "sqlite://student.db";

#[get("/FetchCourses")]
async fn fetch_courses() -> Result<impl Responder> {
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let records = Course::find().all(&db)
        .await.expect("Could not fetch course records from database");
    let html = CourseListView(records);

    Ok(HttpResponse::Ok().body(html))
}

/*
#[get("/RegisterStudentPage")]
async fn register_student_page() -> Result<impl Responder> {
    Ok(())
}

#[post("/RegisterStudentSubmit")]
async fn register_student() -> Result<impl Responder> {
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");

    Ok(())
}
 */

#[get("/")]
async fn index() -> impl Responder {
    "Hello world"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let logger = Logger::default();
        App::new()
            .service(index)
            .service(fetch_courses)
    })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}
