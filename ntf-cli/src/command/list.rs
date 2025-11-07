//! The `list` subcommand.

use clap::Parser;
use eyre::Result;

use ntf_api::ApiClient;

/// Arguments for `ntf-cli list`.
#[derive(Debug, Parser)]
pub struct List {
    /// The API base URL.
    #[arg(long = "url", default_value = "http://localhost:3000")]
    base_url: String,
}

impl super::Command for List {
    #[tracing::instrument(name = "list", level = "trace", skip_all)]
    async fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running list");

        let Self { base_url } = self;

        let api = ApiClient::new(base_url);
        let notifications = api.list_notifications().await?;

        println!("notifications = {notifications:?}");

        Ok(())
    }
}
