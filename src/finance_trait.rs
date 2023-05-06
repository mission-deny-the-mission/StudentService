use async_trait::async_trait;

pub struct FinanceAccount{
    pub id: i32,
    pub studentId: String,
    pub hasOutstandingBalance: bool,
}

#[async_trait]
pub trait FinanceClient {
    async fn getFinanceAccount(&self, StudentID: &String) -> Result<Option<FinanceAccount>, std::fmt::Error>;
    async fn deleteFinanceAccount(&self, StudentID: &String) -> Option<std::fmt::Error>;
    async fn checkFinanceAccount(&self, StudentID: &String) -> Result<bool, std::fmt::Error>;
    async fn registerFinanceClient(&self, StudentID: &String) -> Result<bool, std::fmt::Error>;
    async fn createInvoice(&self, StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String);
}