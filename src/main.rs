mod entity;
mod view;

#[macro_use] extern crate actix_web;

use std::sync::mpsc::SendError;
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

#[get("/StudentProfile/{id}")]
async fn student_profile(path: web::Path<i32>) -> Result<impl Responder> {
    let id = path.into_inner();
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let query_result = Student::find_by_id(id).one(&db)
        .await.expect("Could not get record from database.");
    if let Some(student) = query_result {
        Ok(HttpResponse::Ok().body(StudentListView(student)))
    } else {
        Ok(HttpResponse::NotFound().body("Could not find student with that ID"))
    }
}

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
            .service(student_profile)
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
