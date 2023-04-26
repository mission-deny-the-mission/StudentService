use reqwest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json;

type JsonLinks = HashMap<String, HashMap<String, String>>;

// Used for fetching an existing finance account from the finance microservice
// it represents fields in the JSON coming from the microservice that represents an individual
// finance account
#[derive(Serialize, Deserialize)]
pub struct account {
    id: i32,
    studentId: String,
    pub hasOutstandingBalance: bool,
    _links: JsonLinks
}

// function which fetches an account from the finance service using a student id
pub async fn fetch_finance_account(StudentID: &String) -> Result<Option<account>, std::fmt::Error> {
    // this executes the get request to the finance service
    let result = reqwest::get(format!("http://localhost:8081/accounts/student/{}", StudentID))
        .await.expect("Error executing request")
        .text()
        .await.expect("Request response does not have text segment");
    // If the finance account does not exist we expect an error message from the finance service
    // this if statement catches that error message and returns the appropriate response
    if result.contains("Could not find account for student ID") {
        return Ok(None);
    }
    // attempts to decode the finance account into the account struct
    let possible_finance_account = serde_json::from_str(&result);
    // this if statement does error handling for if this process fails.
    if let Ok(finance_account) = possible_finance_account {
        Ok(Some(finance_account))
    } else {
        panic!("Error")
    }
}

// returns true if account exits, returns false otherwise
// internally it uses the fetch_finance_account function defined above
pub async fn check_for_finance_account(StudentID: &String) -> Result<bool, std::fmt::Error>
{
    match fetch_finance_account(StudentID).await? {
        Option::None => Ok(false),
        Option::Some(x) => Ok(true),
    }
}

// Struct used when attempting to register an account with the finance service,
// it only contains the StudentID
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
            // creates an instance of the struct used to construct the JSON for the post request
            let studentSubmission = registerStudentJson { studentId: StudentID.to_owned(), };
            // sends the post request to register the account
            reqwest::Client::new()
                .post("http://localhost:8081/accounts")
                .json(&studentSubmission)
                .send()
                .await?;
            // returns true if the process was successful
            Ok(true)
        }
    }
}

#[derive(Serialize)]
struct invoiceAccountInput {
    studentId: String,
}

// Struct used to submit an invoice to the finance service
#[derive(Serialize)]
struct invoiceInput {
    amount: f64,
    dueDate: String,
    // since type is a keyword in rust we have to use this decorator to work around it
    #[serde(rename = "type")]
    invoiceType: String,
    account: invoiceAccountInput,
}

// Used for decoding the JSON recieved from the finance account when creating an invoice
#[derive(Deserialize)]
struct createInvoiceResult {
    id: i64,
    reference: String,
    amount: f64,
    dueDate: String,
    #[serde(rename = "type")]
    invoiceType: String,
    status: String,
    studentId: String,
    _links: JsonLinks,
}

// Function to register an invoice with the finance service
async fn createInvoice(StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String)
    -> Result<Result<createInvoiceResult, serde_json::Error>, reqwest::Error> {
    // creates an instance of the invoiceInput struct that is then used to make the JSON for the
    // POST request
    let input = invoiceInput {
        amount,
        dueDate: dueDate.to_owned(),
        invoiceType: invoiceType.to_owned(),
        account: invoiceAccountInput {
            studentId: StudentID.to_owned(),
        },
    };
    // submits the POST reqeust to the service
    let response = reqwest::Client::new()
        .post("http://localhost:8081/invoices/")
        .json(&input)
        .send()
        .await?
        .text()
        .await?;
    // attempts to decode and then return the result of the invoice creation
    Ok(serde_json::from_str(&response))
}


pub async fn createInvoiceExternal(StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String) {
    let returnValue = createInvoice(StudentID, invoiceType, amount, dueDate);
    returnValue.await.expect("Could not create invoice");
}