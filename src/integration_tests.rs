use super::*;
use actix_web::{test, web, App};
use table_extract::Table;
use migration;
use migration::MigratorTrait;

// function to create database and setup database schema using a migration
async fn create_database() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await.expect("");
    migration::Migrator::up(&db, None).await.expect("");
    db
}

#[actix_web::test]
async fn test_index_get() {
    let db = create_database().await;
    let app = test::init_service(
        App::new()
            .app_data(Data::new(db.clone()))
            .service(index),
    ).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_list_student_account() {
    let db = create_database().await;
    let app = test::init_service(
        App::new()
            .app_data(Data::new(db.clone()))
            .service(student_list),
    ).await;
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