use reqwest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
//use std::fmt::Error;
use serde_json;
//use std::io::Error;

type JsonLinks = HashMap<String, HashMap<String, String>>;

#[derive(Serialize, Deserialize)]
pub struct account {
    id: i32,
    studentId: String,
    pub hasOutstandingBalance: bool,
    _links: JsonLinks
}

#[derive(Serialize, Deserialize)]
struct accountListEmbed {
    accountList: Vec<account>,
}

#[derive(Serialize, Deserialize)]
struct resultType{
    _embedded: accountListEmbed,
    _links: JsonLinks,
}

pub async fn fetch_finance_account(StudentID: &String) -> Result<Option<account>, std::fmt::Error> {
    let result = reqwest::get(format!("http://localhost:8081/accounts/student/{}", StudentID))
        .await.expect("Error executing request")
        .text()
        .await.expect("Request response does not have text segment");
    if result.contains("Could not find account for student ID") {
        return Ok(None);
    }
    let possible_finance_account = serde_json::from_str(&result);
    if let Ok(finance_account) = possible_finance_account {
        Ok(Some(finance_account))
    } else {
        panic!("Error")
    }
}

// returns true if account exits, returns false otherwise
pub async fn check_for_finance_account(StudentID: &String) -> Result<bool, std::fmt::Error>
{
    match fetch_finance_account(StudentID).await? {
        Option::None => Ok(false),
        Option::Some(x) => Ok(true),
    }
}

#[derive(Serialize, Deserialize)]
struct registerStudentJson {
    studentId: String,
}

// returns false if account already exists
// otherwise attempts account creation and returns true if successful
pub async fn register_finance_account(StudentID: &String) -> Result<bool, reqwest::Error> {
    match fetch_finance_account(StudentID).await.expect("Could not complete get request") {
        Option::Some(account) => Ok(false),
        Option::None => {
            let studentSubmission = registerStudentJson {
                studentId: StudentID.to_owned(),
            };
            reqwest::Client::new()
                .post("http://localhost:8081/accounts")
                .json(&studentSubmission)
                .send()
                .await?;
            Ok(true)
        }
    }
}