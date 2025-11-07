//! The `ack` subcommand.

use clap::Parser;
use eyre::Result;

use ntf_api::ApiClient;

/// Arguments for `ntf-cli ack`.
#[derive(Debug, Parser)]
pub struct Ack {
    /// ID of the notification to acknowledge.
    id: usize,
    /// The API base URL.
    #[arg(long = "url", default_value = "http://localhost:3000")]
    base_url: String,
}

impl super::Command for Ack {
    #[tracing::instrument(name = "ack", level = "trace", skip_all)]
    async fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running ack");

        let Self { id, base_url } = self;

        let api = ApiClient::new(base_url);
        let notification = api.ack_notification(*id).await?;

        println!("acknowledged: {notification:?}");

        Ok(())
    }
}
