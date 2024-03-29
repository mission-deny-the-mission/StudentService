mod entity;
mod config;
//mod view;
mod finance_trait;
mod reqwest_finance_client;
mod library_client;

#[macro_use] extern crate actix_web;

use std::fs;
use std::sync::mpsc::SendError;
use std::path::PathBuf;
use actix_files::NamedFile;
use actix_web::{HttpServer, App, web::Data, web, middleware::Logger, Responder, HttpResponse, Result, HttpRequest};
use entity::{course, student, prelude};
use entity::prelude::{Course, Student};
use sea_orm::prelude::*;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use finance_trait::{FinanceAccount, FinanceClient};
use reqwest_finance_client::ReqwestFinanceClient;
use askama::Template;
use crate::entity::*;
use chrono;
use sea_orm::TryGetError::DbErr;
use crate::entity::prelude::Enrolement;
use dotenv::dotenv;
use library_client::*;

// this is a struct used for generating a HTML page using a template with askama template library
// The fields the template needs to be filled in are specified as attributes in the struct
// This template specifically is for error pages
#[derive(Template)]
#[template(path = "Error.html")]
struct ErrorTemplate {
    title_message: String,
    body_message: String,
}

#[derive(Template)]
#[template(path = "CourseList.html")]
struct CourseListTemplate<'a> {
    courses: &'a Vec<course::Model>,
}
// This method is for the web page with the list of courses
#[get("/FetchCourses")]
async fn fetch_courses(program_state: web::Data<ProgramState>) -> Result<impl Responder> {
    let db = &program_state.db.clone();
    // This fetches all of the courses from the database
    let records = Course::find().all(db)
        .await.expect("Could not fetch course records from database");
    // this renders the HTML template for the courses page
    let template = CourseListTemplate { courses: &records };
    let html = template.render().unwrap();

    // this line controls the HTTP response including sending the HTML page to the browser
    Ok(HttpResponse::Ok().body(html))
}

// This is the form for creating a new course it is a static web page
#[get("/CreateCourse")]
async fn course_form() -> Result<impl Responder> {
    let path: PathBuf = "./static/CourseForm.html".parse().unwrap();
    Ok(NamedFile::open(path))
}

// This struct is for processing the form for creating a course. Each field of the form is
// represented here as an attribute
// Actix will parse the form for us into this struct when handling the post request
#[derive(Deserialize)]
struct course_form_input {
    name: String,
    subject: String,
    leader: String,
    tuition_cost: f64,
}
#[post("/CreateCourse")]
async fn course_submit(program_state: web::Data<ProgramState>, form: web::Form<course_form_input>)
    -> Result<impl Responder> {
    let course_entry = course::ActiveModel {
        id: NotSet,
        name: Set(form.name.to_owned()),
        subject: Set(form.subject.to_owned()),
        leader: Set(form.leader.to_owned()),
        tuition_cost: Set(form.tuition_cost.to_owned()),
    };
    let db = &program_state.db.clone();
    let student_record = course::Entity::insert(course_entry).exec(db)
        .await.expect("Could not insert record");
    let success_page_path: PathBuf = "./static/RegisterSuccess.html".parse().unwrap();
    Ok(NamedFile::open(success_page_path))
}

// Template for the list of students
// Here an std::vec is used which is a type of array data structure within rust and is short for
// vector. Here it's used to store the list of students and associated finance accounts stored
// together in a tuple.
#[derive(Template)]
#[template(path = "StudentList.html")]
struct StudentListTemplate {
    student_finance_array: Vec<(student::Model, Option<FinanceAccount>)>,
}
#[get("/StudentList")]
async fn student_list(program_state: web::Data<ProgramState>) -> Result<impl Responder> {
    let db = &program_state.db.clone();
    let finance_client = program_state.finance_client.clone();
    // We first get the student records from the database and store it in a vec
    let studentList: Vec<student::Model> = Student::find().all(db)
        .await.expect("Could not fetch records from database.");
    // then we create a larger vec to store the students with the finance accounts
    let mut student_finance_list:  Vec<(student::Model, Option<FinanceAccount>)> = Vec::with_capacity(studentList.len());
    // Then we try to retrieve the finance account associated with each student in the database
    // and store it in the second vec along with the student details
    // if we can't then we just store the student details
    for student in studentList {
        let finance_account_option = finance_client.getFinanceAccount(&student.student_id.to_owned())
            .await.expect("Error occurred while fetching finance account");
        student_finance_list.push((student, finance_account_option));
    }
    let template = StudentListTemplate { student_finance_array: student_finance_list };
    let html = template.render().unwrap();
    Ok(HttpResponse::Ok().body(html))
}

// template for the student profile page
#[derive(Template)]
#[template(path = "Student.html")]
struct StudentProfileTemplate {
    student: student::Model,
    finance: Option<FinanceAccount>
}

// function for displaying a student profile
#[get("/StudentProfile/{id}")]
async fn student_profile(program_state: web::Data<ProgramState>, path: web::Path<String>)
    -> Result<impl Responder> {
    let id = path.into_inner();
    let db = &program_state.db.clone();
    let finance_client = program_state.finance_client.clone();
    // attempt to fetch the student record
    let query_result = Student::find_by_id(id).one(db)
        .await.expect("Could not get record from database.");
    if let Some(student) = query_result {
        // this attempts to fetch the finance account for the student
        let finance_account = finance_client.getFinanceAccount(&student.student_id.to_owned())
            .await.expect("Error occurred while trying to fetch finance account");
        let template = StudentProfileTemplate {
            student: student,
            finance: finance_account,
        };
        let html = template.render().unwrap();
        Ok(HttpResponse::Ok().body(html))
    } else {
        // if the student record can't be found then give an error page
        Ok(HttpResponse::NotFound().body("Could not find student with that ID"))
    }
}

#[get("/RegisterStudentForm.html")]
async fn student_form() -> Result<impl Responder> {
    let path: PathBuf = "./static/RegisterStudent.html".parse().unwrap();
    Ok(NamedFile::open(path))
}

#[derive(Serialize, Deserialize)]
struct student_form_input {
    student_id: String,
    name: String,
    email: String,
    phone_number: Option<String>,
    address: String,
}
//function that deals with
#[post("/RegisterStudentSubmit")]
async fn register_student_submit(program_state: web::Data<ProgramState>,
                                 form: web::Form<student_form_input>)
    -> Result<impl Responder> {
    let student_entry = student::ActiveModel {
        student_id: Set(form.student_id.to_owned()),
        name: Set(form.name.to_owned()),
        email: Set(form.email.to_owned()),
        phone_number: Set(form.phone_number.to_owned()),
        address: Set(form.address.to_owned()),
    };
    let db = &program_state.db.clone();
    let finance_client = program_state.finance_client.clone();
    let library_client = program_state.library_client.clone();
    let student_record = student::Entity::insert(student_entry).exec(db)
        .await.expect("Could not insert record");
    finance_client.registerFinanceClient(&form.student_id.to_owned())
        .await.expect("Could not register student in finance application.");
    library_client.registerAccount(&form.student_id);
    let success_path: PathBuf = "./static/RegisterSuccess.html".parse().unwrap();
    Ok(NamedFile::open(success_path))
}

#[derive(Template)]
#[template(path = "EnrollForm.html")]
struct enroll_form_template {
    students: Vec<student::Model>,
    courses: Vec<course::Model>,
}
#[get("/Enroll")]
async fn enroll_form(program_state: web::Data<ProgramState>)
    -> Result<impl Responder> {
    let db = &program_state.db.clone();
    let studentList = Student::find().all(db)
        .await.expect("Could not fetch records from database.");
    let courseList = Course::find().all(db)
        .await.expect("Could not fetch course records from database");
    let template = enroll_form_template {
        students: studentList,
        courses: courseList,
    };
    let html = template.render().unwrap();
    Ok(HttpResponse::Ok().body(html))
}

// represents data from the enrollment form
#[derive(Serialize, Deserialize)]
struct enrollment_form {
    student_id: String,
    course_id: i32,
}
// handles a student enrolling in a course
#[post("/Enroll")]
async fn enroll(program_state: web::Data<ProgramState>, form: web::Form<enrollment_form>)
    -> Result<impl Responder> {
    // Get the database and finance client from the app state
    let db = &program_state.db.clone();
    let finance_client = program_state.finance_client.clone();

    // This pulls the student record from the database to make sure it exists
    let student_record = Student::find_by_id(form.student_id.clone()).one(db)
        .await.expect("Could not get record from database.");
    // This pulls the course record from the database to make sure it exists
    let course_record = Course::find_by_id(form.course_id).one(db)
        .await.expect("Could not get record from database.");

    // If both records are present we can continue
    if student_record != None && course_record != None {
        // This uses the chrono crate to get the current date, this is recorded later in the
        // database as the date of enrollment. It's formatted into a string here before it gets
        // inserted into the database.
        let date_string = format!("{}", chrono::offset::Local::now().format("%d/%m/%Y"));
        // Unwrap the student and course records from the optionals they are stored in
        let course = course_record.unwrap();
        let student = student_record.unwrap();
        // Here we setup an invoice in the finance microservice for the student's tuition fees
        finance_client.createInvoice(&student.student_id, &"TUITION".to_string(),
                                     course.tuition_cost, &date_string);
        // Here the record we put into the database for the enrolment is created
        let enrolment_record = enrolement::ActiveModel {
            student_id: Set(form.student_id.to_owned()),
            course_id: Set(form.course_id.to_owned()),
            enrolement_date: Set(date_string),
        };
        let enrolement_result = Enrolement::insert(enrolment_record).exec(db).await;
        match enrolement_result {
            Ok(Something) => {

                // if everything went okay we respond with a success page
                let success_path: PathBuf = "./static/EnrollmentSuccess.html".parse().unwrap();
                let html = fs::read_to_string(success_path).expect("Could not read file");
                Ok(HttpResponse::Ok().body(html))
            },
            Err(someerror) => {
                // If something went wrong creating the enrollment record then we give an error
                // page instead describing what went wrong
                let template = ErrorTemplate {
                    title_message: "Error occurred while trying to enroll student".to_string(),
                    body_message: "Check if the student is already enrolled.".to_string()
                };
                let html = template.render().unwrap();
                Ok(HttpResponse::Ok().body(html))
            }
        }
    }
    // If either the student or course does not exist we give an error page
    else {
        let template = ErrorTemplate {
            title_message: "Error occurred while trying to enroll student".to_string(),
            body_message: "Either the student or course you are referring to does not exist"
                .to_string()
        };
        let html = template.render().unwrap();
        Ok(HttpResponse::UnprocessableEntity().body(html))
    }
}

// index page, this page is static and is simply read verbatim from the file
#[get("/")]
async fn index() -> Result<impl Responder> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path))
}

// this structure stores the data used by different URL handlers within the service
// specifically the database connection, finance client, and library client
#[derive(Clone)]
struct ProgramState {
    finance_client: ReqwestFinanceClient,
    library_client: ReqwestLibraryClient,
    db: DatabaseConnection,
}

// main function of the program
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // stuff for reading and decoding the configuration
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    // connects to the database
    let db = Database::connect(config.database_url)
        .await.expect("Could not connect to database");
    // Sets up the finance and library client implementations
    let finance_client = ReqwestFinanceClient {
        BaseURL: config.finance_url,
    };
    let library_client = ReqwestLibraryClient {
        BaseURL: config.library_url,
    };
    // This setup up the HTTP web server and attaches all the data and services/URL handlers to it
    HttpServer::new(move || {
        App::new()
            // We inject the finance client, library client, and database into the app as data
            .app_data(Data::new(ProgramState {
                finance_client: finance_client.clone(),
                library_client: library_client.clone(),
                db: db.clone(),
            }))
            .service(index)
            .service(fetch_courses)
            .service(course_form)
            .service(course_submit)
            .service(student_list)
            .service(student_profile)
            .service(student_form)
            .service(register_student_submit)
            .service(enroll_form)
            .service(enroll)
    })
        .bind((config.server.host, config.server.port))?
        .run()
        .await
}

#[cfg(test)]
mod integration_tests;