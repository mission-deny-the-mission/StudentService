use migration::async_trait::async_trait;
use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;

#[async_trait]
pub trait LibraryClient {
    async fn registerAccount(&self, StudentID: &String) -> Option<reqwest::Error>;
}

#[derive(Serialize, Deserialize)]
struct register_account_body {
    studentId: String,
}

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