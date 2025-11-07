//! The `get` subcommand.

use clap::Parser;
use eyre::Result;

use ntf_api::ApiClient;

/// Arguments for `ntf-cli get`.
#[derive(Debug, Parser)]
pub struct Get {
    /// ID of the notification to get.
    id: usize,
    /// The API base URL.
    #[arg(long = "url", default_value = "http://localhost:3000")]
    base_url: String,
}

impl super::Command for Get {
    #[tracing::instrument(name = "get", level = "trace", skip_all)]
    async fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running get");

        let Self { id, base_url } = self;

        let api = ApiClient::new(base_url);
        let notification = api.get_notification(*id).await?;

        println!("{notification:?}");

        Ok(())
    }
}
