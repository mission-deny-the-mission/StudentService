use super::*;
use actix_web::{test, web, App};
use actix_web::dev::{Response, Service, ServiceResponse};
use askama::DynTemplate;
use table_extract::Table;
use migration;
use migration::MigratorTrait;

// function to create database and setup database schema using a migration
// it then adds some sample data to the database
async fn setup_data_and_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("");
    migration::Migrator::up(&db, None).await.expect("");

    let student_1 = student::ActiveModel {
        student_id: Set("c1234".to_string()),
        name: Set("Millie Baxter".to_string()),
        email: Set("m.baxter@student.riversdale-academy.ac.uk".to_string()),
        phone_number: Set(Some("0151351515".to_string())),
        address: Set("72 Grape Lane, Riversdale town".to_string()),
    };
    let student_2 = student::ActiveModel {
        student_id: Set("c4321".to_string()),
        name: Set("David Samson".to_string()),
        email: Set("d.samson@student.riversdale-academy.ac.uk".to_string()),
        phone_number: Set(Some("0195879835".to_string())),
        address: Set("2 Park Avenue, Riversdale town".to_string()),
    };
    student::Entity::insert(student_1).exec(&db).await.expect("");
    student::Entity::insert(student_2).exec(&db).await.expect("");

    db.clone()
}

#[actix_web::test]
async fn test_index_get() {
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let finance_client = ReqwestFinanceClient {
        BaseURL: config.finance_url,
    };
    let db = setup_data_and_db().await;
    let app = test::init_service(App::new()
        .app_data(Data::new(ProgramState {
            finance_client: finance_client.clone(),
            db: db.clone(),
        }))
        .service(index)).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());



}

#[actix_web::test]
async fn test_list_student_account() {
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let finance_client = ReqwestFinanceClient {
        BaseURL: config.finance_url,
    };
    let db = setup_data_and_db().await;
    let app = test::init_service(App::new()
        .app_data(Data::new(ProgramState {
            finance_client: finance_client.clone(),
            db: db.clone(),
        }))
        .service(student_list)).await;

    let req = test::TestRequest::get().uri("/StudentList").to_request();
    let resp = test::call_and_read_body(&app, req).await;
    let text = std::str::from_utf8(&resp).unwrap();

    let studentList: Vec<student::Model> = Student::find().all(&db)
        .await.expect("Could not fetch records from database.");
    let table = Table::find_first(text).unwrap();
    let mut counter: usize = 0;
    for row in &table {
        let id = row.get("Student ID").unwrap();
        let mut student_record_found: bool = false;
        for student in &studentList {
            if student.student_id.eq(id) {
                assert!(student.name.eq(row.get("Name").unwrap()));
                assert!(student.email.eq(row.get("Email").unwrap()));
                if let Some(number) = &student.phone_number {
                    assert!(number.eq(row.get("Phone number").unwrap()))
                }
                assert!(student.address.eq(row.get("Address").unwrap()));
                student_record_found = true;
                break;
            }
        }
        assert!(!student_record_found);
        counter += 1;
    }
    assert_eq!(counter, studentList.len())
}