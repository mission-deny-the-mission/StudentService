use reqwest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::Error;
use migration::async_trait::async_trait;
use serde_json;
use crate::finance_trait::{FinanceAccount, FinanceClient};
//use library_trait::{FinanceClient, FinanceAccount};

type JsonLinks = HashMap<String, HashMap<String, String>>;

// This is a structure that represents a finance client and stores the base URL for the finance
// serivce being used
#[derive(Clone)]
pub struct ReqwestFinanceClient {
    pub BaseURL: String,
}

// Used for fetching an existing finance account from the finance microservice
// it represents fields in the JSON coming from the microservice that represents an individual
// finance account
#[derive(Serialize, Deserialize)]
struct account {
    id: i32,
    studentId: String,
    hasOutstandingBalance: bool,
    _links: JsonLinks
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

// Struct used when attempting to register an account with the finance service,
// it only contains the StudentID
#[derive(Serialize, Deserialize)]
struct registerStudentJson {
    studentId: String,
}

// Used for decoding the JSON received from the finance account when creating an invoice
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

// Functions used in the finance client that are specific to this implementation
// these functions are mostly private for internal use only
impl ReqwestFinanceClient {
    // Function to register an invoice with the finance service
    async fn createInvoiceInternal(&self, StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String)
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
        // submits the POST reqwest to the service
        let response = reqwest::Client::new()
            .post(format!("{}/invoices/", self.BaseURL))
            .json(&input)
            .send()
            .await?
            .text()
            .await?;
        // attempts to decode and then return the result of the invoice creation
        Ok(serde_json::from_str(&response))
    }
}

// These are the functions necessary to implement the FinanceClient trait
// these will all be not be private as functions that are part of a public trait are public
#[async_trait]
impl FinanceClient for ReqwestFinanceClient {
    // function which fetches an account from the finance service using a student id
    async fn getFinanceAccount(&self, StudentID: &String) -> Result<Option<FinanceAccount>, std::fmt::Error> {
        // this executes the get request to the finance service
        let result = reqwest::get(format!("{}/accounts/student/{}", self.BaseURL, StudentID))
            .await.expect("Error executing request")
            .text()
            .await.expect("Request response does not have text segment");
        // If the finance account does not exist we expect an error message from the finance service
        // this if statement catches that error message and returns the appropriate response
        if result.contains("Could not find account for student ID") {
            return Ok(None);
        }
        // attempts to decode the finance account into the account struct
        let possible_finance_account: Result<account, serde_json::Error> = serde_json::from_str(&result);
        // this if statement does error handling for if this process fails.
        if let Ok(finance_account) = possible_finance_account {
            let output_finance_account = FinanceAccount {
                id: finance_account.id,
                studentId: finance_account.studentId,
                hasOutstandingBalance: finance_account.hasOutstandingBalance,
            };
            Ok(Some(output_finance_account))
        } else {
            panic!("Error")
        }
    }

    async fn deleteFinanceAccount(&self, StudentID: &String) -> Option<Error> {
        if let Some(account) = self.getFinanceAccount(StudentID)
            .await.expect("")
        {
            // for cleanup we have to delete the finance account if it has been created
            let client = reqwest::Client::new();
            client.delete(format!("{}/accounts/{}", self.BaseURL, account.id)).send().await.expect("");
        }
        None
    }

    // returns true if account exits, returns false otherwise
    // internally it uses the fetch_finance_account function defined above
    async fn checkFinanceAccount(&self, StudentID: &String) -> Result<bool, std::fmt::Error>
    {
        match self.getFinanceAccount(StudentID).await? {
            Option::None => Ok(false),
            Option::Some(x) => Ok(true),
        }
    }

    // returns false if account already exists
    // otherwise attempts account creation and returns true if successful
    async fn registerFinanceClient(&self, StudentID: &String) -> Result<bool, std::fmt::Error> {
        match self.getFinanceAccount(StudentID).await.expect("Could not complete get request") {
            Option::Some(account) => Ok(false),
            Option::None => {
                // creates an instance of the struct used to construct the JSON for the post request
                let studentSubmission = registerStudentJson { studentId: StudentID.to_owned(), };
                // sends the post request to register the account
                reqwest::Client::new()
                    .post(format!("{}/accounts", self.BaseURL))
                    .json(&studentSubmission)
                    .send()
                    .await
                    .expect("Error running post request to create finance account");
                // returns true if the process was successful
                Ok(true)
            }
        }
    }

    async fn createInvoice(&self, StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String) {
        let returnValue = self.createInvoiceInternal(StudentID, invoiceType, amount, dueDate);
        returnValue.await.expect("Could not create invoice");
    }
}