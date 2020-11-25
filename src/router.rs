use crate::types::{CmState, LanUserTable};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Router {
    async fn logout(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn devices(&mut self) -> Result<LanUserTable, Box<dyn std::error::Error>>;
    async fn temperature(&mut self) -> Result<CmState, Box<dyn std::error::Error>>;
}
