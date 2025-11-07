//! A CLI client for the notification API.

use eyre::Result;

use ntf_cli::NtfCli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    NtfCli::run().await
}
