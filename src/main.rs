mod entity;
//mod view;
mod finance_client;

#[macro_use] extern crate actix_web;

use std::sync::mpsc::SendError;
use std::path::PathBuf;
use actix_files::NamedFile;
use actix_web::{HttpServer, App, web::Data, web, middleware::Logger, Responder, HttpResponse, Result};
use entity::{course, student, prelude};
use entity::prelude::{Course, Student};
use sea_orm::prelude::*;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use crate::finance_client::fetch_finance_account;

use askama::Template;
use crate::entity::*;
//use super::finance_client::account;

const DATABASE_URI: &str = "sqlite://student.db";

#[derive(Template)]
#[template(path = "Courses.html")]
struct CourseListTemplate<'a> {
    courses: &'a Vec<course::Model>,
}



#[get("/FetchCourses")]
async fn fetch_courses() -> Result<impl Responder> {
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let records = Course::find().all(&db)
        .await.expect("Could not fetch course records from database");
    let template = CourseListTemplate { courses: &records };
    let html = template.render().unwrap();

    Ok(HttpResponse::Ok().body(html))
}

#[derive(Template)]
#[template(path = "StudentList.html")]
struct StudentListTemplate {
    student_finance_array: Vec<(student::Model, Option<finance_client::account>)>,
}

#[get("/StudentList")]
async fn student_list() -> Result<impl Responder> {
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let studentList: Vec<student::Model> = Student::find().all(&db)
        .await.expect("Could not fetch records from database.");
    let mut student_finance_list:  Vec<(student::Model, Option<finance_client::account>)> = Vec::with_capacity(studentList.len());
    for student in studentList {
        let finance_account_option = fetch_finance_account(&student.student_id.to_owned())
            .await.expect("Error occurred while fetching finance account");
        student_finance_list.push((student, finance_account_option));
    }
    let template = StudentListTemplate { student_finance_array: student_finance_list };
    let html = template.render().unwrap();
    Ok(HttpResponse::Ok().body(html))
}


#[derive(Template)]
#[template(path = "Student.html")]
struct StudentProfileTemplate {
    student: student::Model,
    finance: Option<finance_client::account>
}

#[get("/StudentProfile/{id}")]
async fn student_profile(path: web::Path<i32>) -> Result<impl Responder> {
    let id = path.into_inner();
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let query_result = Student::find_by_id(id).one(&db)
        .await.expect("Could not get record from database.");
    if let Some(student) = query_result {
        let finance_account = fetch_finance_account(&student.student_id.to_owned())
            .await.expect("Error occured while trying to fetch finance account");
        let template = StudentProfileTemplate {
            student: student,
            finance: finance_account,
        };
        let html = template.render().unwrap();
        Ok(HttpResponse::Ok().body(html))
    } else {
        Ok(HttpResponse::NotFound().body("Could not find student with that ID"))
    }
}

#[get("/RegisterStudentForm.html")]
async fn student_form() -> Result<impl Responder> {
    let path: PathBuf = "./static/RegisterStudent.html".parse().unwrap();
    Ok(NamedFile::open(path))
}

#[derive(Deserialize)]
struct student_form_input {
    student_id: String,
    name: String,
    email: String,
    phone_number: Option<String>,
    address: String,
}
#[post("/RegisterStudentSubmit")]
async fn register_student_submit(form: web::Form<student_form_input>) -> Result<impl Responder> {
    let student_entry = student::ActiveModel {
        id: NotSet,
        name: Set(form.name.to_owned()),
        email: Set(form.email.to_owned()),
        student_id: Set(form.student_id.to_owned()),
        phone_number: Set(form.phone_number.to_owned()),
        address: Set(form.address.to_owned()),
    };
    finance_client::register_finance_account(&form.student_id.to_owned())
        .await.expect("Could not register student in finance application.");
    let db = Database::connect(DATABASE_URI)
        .await.expect("Could not connect to database");
    let student_record = student::Entity::insert(student_entry).exec(&db)
        .await.expect("Could not insert record");
    let success_path: PathBuf = "./static/RegisterSuccess.html".parse().unwrap();
    Ok(NamedFile::open(success_path))
}

#[get("/")]
async fn index() -> Result<impl Responder> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let logger = Logger::default();
        App::new()
            .service(index)
            .service(fetch_courses)
            .service(student_list)
            .service(student_profile)
            .service(student_form)
            .service(register_student_submit)
    })
        .bind(("0.0.0.0", 8085))?
        .run()
        .await
}
