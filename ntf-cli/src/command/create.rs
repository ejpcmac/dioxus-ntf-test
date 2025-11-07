//! The `create` subcommand.

use clap::Parser;
use eyre::Result;

use ntf_api::ApiClient;

/// Arguments for `ntf-cli create`.
#[derive(Debug, Parser)]
pub struct Create {
    /// The message of the notification.
    message: String,
    /// The API base URL.
    #[arg(long = "url", default_value = "http://localhost:3000")]
    base_url: String,
}

impl super::Command for Create {
    #[tracing::instrument(name = "create", level = "trace", skip_all)]
    async fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running create");

        let Self { message, base_url } = self;

        let api = ApiClient::new(base_url);
        let notification = api.create_notification(message).await?;

        println!("created: {notification:?}");

        Ok(())
    }
}
