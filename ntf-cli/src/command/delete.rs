//! The `delete` subcommand.

use clap::Parser;
use eyre::Result;

use ntf_api::ApiClient;

/// Arguments for `ntf-cli delete`.
#[derive(Debug, Parser)]
pub struct Delete {
    /// ID of the notification to delete.
    id: usize,
    /// The API base URL.
    #[arg(long = "url", default_value = "http://localhost:3000")]
    base_url: String,
}

impl super::Command for Delete {
    #[tracing::instrument(name = "delete", level = "trace", skip_all)]
    async fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running delete");

        let Self { id, base_url } = self;

        let api = ApiClient::new(base_url);
        let notification = api.delete_notification(*id).await?;

        println!("deleted: {notification:?}");

        Ok(())
    }
}
