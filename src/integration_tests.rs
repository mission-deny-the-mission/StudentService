use super::*;
use actix_web::{test, web, App};
use actix_web::dev::{Response, Service, ServiceResponse};
use askama::DynTemplate;
use table_extract::Table;
use migration;
use migration::MigratorTrait;
use scraper::{Html, Selector};

// creates and configured a database and injects sample data to test with
async fn setup_data_and_db() -> DatabaseConnection {
    // create an in memory database for us to use
    let db = Database::connect("sqlite::memory:").await.expect("");
    // setups up the database schema for the in memory database
    // this uses a database migration tool that is part of sea-orm
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

    let course_1 = course::ActiveModel {
        id: Set(1),
        name: Set("Trigonometry".to_string()),
        subject: Set("Mathematics".to_string()),
        leader: Set("Robert Pearson".to_string()),
        tuition_cost: Set(300 as f64),
    };
    let course_2 = course::ActiveModel {
        id: Set(2),
        name: Set("Kinematics".to_string()),
        subject: Set("Physics".to_string()),
        leader: Set("Paul Suttersby".to_string()),
        tuition_cost: Set(400.45),
    };

    course::Entity::insert(course_1).exec(&db).await.expect("");
    course::Entity::insert(course_2).exec(&db).await.expect("");

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
    // when calling it from each test. I did something similar to that for ASE module in C#

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

    // here we make the request for the student list page
    let req = test::TestRequest::get().uri("/StudentList").to_request();
    let resp = test::call_and_read_body(&app, req).await;
    // The response is actually given as a binary data type defined in the actix crate.
    // In order to deal with this
    let text = std::str::from_utf8(&resp).unwrap();

    // in order to make sure we got the correct result we need to compare it to the students that
    // are actually in the database
    // the first step here is to fetch said list of students from the database using the database
    // model
    let studentList: Vec<student::Model> = Student::find().all(&db)
        .await.expect("Could not fetch records from database.");
    // Then we take the response from the service and extract the student table
    // this uses a crate (package) called table-extract that deals with parsing HTML tables
    // honestly it's kind of odd there is a module just for that but it works quite well
    // aside from not being able to parse links
    let table = Table::find_first(text).unwrap();
    // we keep a count of how many student entries there are in the table and compare it against the
    // we got from the database later on
    let mut counter: usize = 0;
    for row in &table {
        // Unfortunately because table-extract dosen't do anything with links we have to use
        // another package to deal with this part. I choose to use scraper here
        // This is because the StudentID fields has hyperlinks to the student profile pages for
        // each student
        let selector = Selector::parse("a").unwrap();
        let id_row = row.get("Student ID").unwrap();
        let fragment = Html::parse_fragment(id_row);
        let id = fragment.select(&selector).next().unwrap().text().next().unwrap();
        // This next part is based on a linear search algorithm. We essentially need to find
        // the corresponding student record for each entry in the HTML table, and this does the
        // finding part. We keep a note of it's been found or not using this variable so that we
        // can trigger an assertion if an entry has been created where it shouldn't exist
        let mut student_record_found: bool = false;
        for student in &studentList {
            if student.student_id.eq(id) {
                // if we find the student a bunch of assertions are made to make sure the details
                // match between the database record and the student list
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
        assert!(student_record_found);
        counter += 1;
    }
    // here is the assertion I was talking about earlier that checks the number of courses
    // and makes sure non are missing
    assert_eq!(counter, studentList.len())
}

#[actix_web::test]
async fn test_list_courses() {
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
        .service(fetch_courses)).await;

    // send request to service
    let req = test::TestRequest::get().uri("/FetchCourses").to_request();
    let resp = test::call_and_read_body(&app, req).await;
    // standard trick of converting the response into text so we can parse it, the function
    // above this one does the same thing
    let text = std::str::from_utf8(&resp).unwrap();

    // this fetches the course list from the database to compare against
    let courseList = Course::find().all(&db)
        .await.expect("Could not fetch records from database.");
    // parses the table in the response HTML
    let table = Table::find_first(text).unwrap();
    let mut counter: usize = 0;
    for row in &table {
        let name = row.get("Name").unwrap();
        // linear search similar to the last function
        // searches for a matching course in the records retrieved from the database earlier
        let mut course_found: bool = false;
        for course in &courseList {
            if course.name.eq(name) {
                // assertions to make sure that the course details match
                assert!(course.subject.eq(row.get("Subject").unwrap()));
                assert!(course.leader.eq(row.get("Leader").unwrap()));
                assert_eq!(course.tuition_cost,
                           row.get("Tuition fee").expect("Missing field in response")
                               .parse::<f64>().unwrap());
                course_found = true;
                break;
            }
        }
        assert!(course_found);
        counter += 1;
    }
    // this makes sure we actually got the right number of courses from the courses list
    // ensures none are missing
    assert_eq!(counter, courseList.len());
}

#[actix_web::test]
async fn test_create_student() {
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
        .service(register_student_submit)).await;

    // In order to create a student a form is required.
    // This stands in place of the real HTML form
    let form = student_form_input {
        student_id: "c1359".to_string(),
        name: "Félix McCracken".to_string(),
        email: "f.mccracken@student.riversdale-academy.ac.uk".to_string(),
        phone_number: None,
        address: "".to_string(),
    };

    // We pass the form when we make the post request
    let req = test::TestRequest::post().set_form(&form).uri("/RegisterStudentSubmit").to_request();
    // we don't actually need to do anything with the response body for this test since we are
    // mainly measuring what records it creates and how it effects the other microservice
    test::call_service(&app, req).await;

    // Here we attempt to fetch the newly created student from the database
    let result = Student::find_by_id(&form.student_id).one(&db)
        .await.expect("Database error occured during testing");
    if let Some(student) = result {
        // check to see if the details of the created student matches those set above
        assert!(form.name.eq(&student.name));
        assert!(form.email.eq(&student.email));
        assert_eq!(form.phone_number, student.phone_number);
        assert!(form.address.eq(&student.address));
    } else {
        assert!(false);
    }

    // we also check if the service successfully created a finance account for the student
    if finance_client.checkFinanceAccount(&form.student_id)
        .await.expect("Error using finance application")
    {
        // for cleanup we have to delete the finance account if it has been created
        finance_client.deleteFinanceAccount(&form.student_id);
    } else {
        assert!(false);
    }

    // Make sure the finance account was deleted successfully
    assert!(!finance_client.checkFinanceAccount(&form.student_id)
        .await.expect("Error using finance application"))
}