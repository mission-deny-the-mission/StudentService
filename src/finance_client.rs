use reqwest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json;

type JsonLink = HashMap<String, HashMap<String, String>>;

#[derive(Serialize, Deserialize)]
struct account {
    id: i32,
    studentId: String,
    hasOutstandingBalance: bool,
    _links: JsonLink
}

#[derive(Serialize, Deserialize)]
struct accountListEmbed {
    accountList: Vec<account>,
}

#[derive(Serialize, Deserialize)]
struct resultType{
    _embedded: accountListEmbed,
    _links: JsonLink,
}

pub async fn check_studentid(StudentID: String) -> Result<bool, reqwest::Error> {
    let result: resultType = reqwest::get("http://localhost:8081/accounts")
        .await?
        .json()
        .await?;
    let accounts: Vec<account> = result._embedded.accountList;
    for account in accounts {
        if account.studentId == StudentID {
            return Ok(true);
        }
    }
    return Ok(false);
}

#[derive(Serialize, Deserialize)]
struct registerStudentJson {
    studentId: String,
}

pub async fn register_student(StudentID: String) -> Result<bool, reqwest::Error> {
    let alreadyExists: bool = check_studentid(StudentID.to_owned()).await?;
    if !alreadyExists {
        let studentSubmission = registerStudentJson {
            studentId: StudentID,
        };
        reqwest::Client::new()
            .post("http://localhost:8081/accounts")
            .json(&studentSubmission)
            .send()
            .await?;
    }
    return Ok(!alreadyExists);
}