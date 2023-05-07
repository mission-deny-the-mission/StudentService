use migration::async_trait::async_trait;
use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;

// a trait is used here so that a different library client or even library microservice could be used
#[async_trait]
pub trait LibraryClient {
    async fn registerAccount(&self, StudentID: &String) -> Option<reqwest::Error>;
}

// this houses the required fields needed to create a library account
// as you can see it's just the student ID
// It derives Serialize and Deserialize so that it can be encoded and decoded into JSON
#[derive(Serialize, Deserialize)]
struct register_account_body {
    studentId: String,
}

// Struct that represents a configured library client
// It stores the Base URL for the library microservice being used
#[derive(Clone)]
pub struct ReqwestLibraryClient {
    pub BaseURL: String,
}

#[async_trait]
impl LibraryClient for ReqwestLibraryClient {
    async fn registerAccount(&self, StudentID: &String) -> Option<reqwest::Error> {
        let body = register_account_body { studentId: StudentID.to_owned() };
        reqwest::Client::new()
            .post(format!("{}/api/register", self.BaseURL))
            .json(&body)
            .send()
            .await.ok()?;
        Option::None
    }
}