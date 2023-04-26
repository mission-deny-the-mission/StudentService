use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct register_account_body {
    studentId: String,
}

pub async fn register_account(StudentID: &String) -> Option<reqwest::Error> {
    let body = register_account_body { studentId: StudentID.to_owned() };
    reqwest::Client::new()
        .post("http://localhost/api/register")
        .json(&body)
        .send()
        .await.ok()?;
    Option::None
}