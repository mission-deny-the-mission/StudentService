use askama::Template;
use crate::entity::*;
use super::finance_client::account;

#[derive(Template)]
#[template(path = "Courses.html")]
struct CourseListTemplate<'a> {
    courses: &'a Vec<course::Model>,
}

pub fn CourseListView(records: Vec<course::Model>) -> String {
    let template = CourseListTemplate { courses: &records };
    template.render().unwrap()
}

#[derive(Template)]
#[template(path = "Student.html")]
struct StudentListTemplate {
    student: student::Model,
    finance: Option<account>
}

pub fn StudentListView(student_record: student::Model, finance_account: Option<account>) -> String {
    let template = StudentListTemplate {
        student: student_record,
        finance: finance_account,
    };
    template.render().unwrap()
}