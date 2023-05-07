use async_trait::async_trait;

// this is the structure/data type we expect a finance account client to expose
// the internal one could be more complex than this depending on the finance microservice being used
// but this all the details that are needed by this program
pub struct FinanceAccount{
    pub id: i32,
    pub studentId: String,
    pub hasOutstandingBalance: bool,
}

// This is called a trait and is equivalent to interfaces in other languages such as Java
// We make one hear so that different clients for different finance implementations can be added
// It could also be used to create a dummy client for unit testing purposes
#[async_trait]
pub trait FinanceClient {
    async fn getFinanceAccount(&self, StudentID: &String) -> Result<Option<FinanceAccount>, std::fmt::Error>;
    async fn deleteFinanceAccount(&self, StudentID: &String) -> Option<std::fmt::Error>;
    async fn checkFinanceAccount(&self, StudentID: &String) -> Result<bool, std::fmt::Error>;
    async fn registerFinanceClient(&self, StudentID: &String) -> Result<bool, std::fmt::Error>;
    async fn createInvoice(&self, StudentID: &String, invoiceType: &String, amount: f64, dueDate: &String);
}