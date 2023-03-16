use askama::Template;
use crate::entity::*;

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
}

pub fn StudentListView(student_record: student::Model) -> String {
    let template = StudentListTemplate { student: student_record };
    template.render().unwrap()
}