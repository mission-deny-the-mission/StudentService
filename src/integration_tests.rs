use super::*;
use actix_web::{test, web, App};
use actix_web::dev::{Response, Service, ServiceResponse};
use askama::DynTemplate;
use table_extract::Table;
use migration;
use migration::MigratorTrait;

// creates and configured a database and injects sample data to test with
async fn setup_data_and_db() -> DatabaseConnection {
    // create an in memory database for us to use
    let db = Database::connect("sqlite::memory:").await.expect("");
    // setups up the database schema in the in memory database
    migration::Migrator::up(&db, None).await.expect("");

    // couple of sample student records
    // please note these are not real people, that would probably violate data protection
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

    // inserts the sample student records into the in memory database
    student::Entity::insert(student_1).exec(&db).await.expect("");
    student::Entity::insert(student_2).exec(&db).await.expect("");

    // we have to do this do deal with how Rust accounts for memory allocations
    // I would explain more but it's beyond my pay grade and understanding at the minute
    db.clone()
}

// test the index page, nice and simple
#[actix_web::test]
async fn test_index_get() {
    // The start of all these tests is basically the same.
    // I tried putting this in a function, didn't work out, Rust is VERY finicky with return types.
    // Compiler will literally give you the type it thinks you've returned instead of the one you
    // specified in the header, you go okay and paste it into the header, then it complains that the
    // type IT GAVE YOU doesn't exist somehow or can't be found somehow. smh.
    // If I had more time I would probably work out how to make one test higher order function
    // where you passed through code that made the requests and assertions you wanted as functions
    // when calling it from each test. I did something similar for ASE module in C#

    // start by getting the environment variables from the file if necessary
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    // we use the URL for the finance service specified in the environment
    let finance_client = ReqwestFinanceClient {
        BaseURL: config.finance_url,
    };
    let library_client = ReqwestLibraryClient {
        BaseURL: config.library_url,
    };
    // run the function above to setup the database and sample data
    let db = setup_data_and_db().await;
    // create the app
    let app = test::init_service(App::new()
        .app_data(Data::new(ProgramState {
            finance_client: finance_client.clone(),
            library_client: library_client.clone(),
            db: db.clone(),
        }))
        .service(index)).await;

    // This is the part that actually varies significantly between functions from here on.
    // Yep I know, it's a lot of boilerplate before we get here.

    // Send request for the index page to the test service
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    // make sure it gave a positive response
    // not actually much to test here as this is a static page
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_list_student_account() {
    // same stuff as before pretty much
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let finance_client = ReqwestFinanceClient {
        BaseURL: config.finance_url,
    };
    let library_client = ReqwestLibraryClient {
        BaseURL: config.library_url,
    };
    let db = setup_data_and_db().await;
    let app = test::init_service(App::new()
        .app_data(Data::new(ProgramState {
            finance_client: finance_client.clone(),
            library_client: library_client.clone(),
            db: db.clone(),
        }))
        // only difference between the functions in this whole block is this line that binds the right service
        // technically you could just bind all of them, like the main application does
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