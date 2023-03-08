mod entity;

#[macro_use] extern crate rocket;
use entity::{course, student, prelude};
use entity::prelude::{Course, Student};
use rocket::serde::json::{Json};
use sea_orm::prelude::*;
use sea_orm::*;
use rocket::http::Status;

const DATABASE_URI: &str = "sqlite://student.db";

#[get("/FetchCourses")]
async fn fetch_courses() -> Result<Json<Vec<JsonValue>>, Status> {
    let db: DatabaseConnection = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    return Ok(Json(
        Course::find().into_json().all(&db)
            .await.expect("Could not fetch records")
    ));
//    let courses = Course::find().into_json().all(&db)
//        .await.expect("Could not fetch records");
//    Ok(Json(courses))
}

#[get("/")]
fn index() -> &'static str {
    "hello!"
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/", routes![index, fetch_courses])
}
