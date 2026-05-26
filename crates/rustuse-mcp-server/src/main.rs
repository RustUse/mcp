#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod prompts;
mod resources;
mod server;
mod tools;

use rmcp::{ServiceExt, transport::stdio};
use server::RustUseMcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = RustUseMcpServer::new()?.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
