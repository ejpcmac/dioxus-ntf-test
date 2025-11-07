//! The `hello` subcommand.

use clap::Parser;
use eyre::Result;

/// Arguments for `ntf-cli hello`.
#[derive(Debug, Parser)]
pub struct Hello {
    /// Who to say hello to.
    name: Option<String>,
}

impl super::Command for Hello {
    #[tracing::instrument(name = "hello", level = "trace", skip_all)]
    fn run(&self) -> Result<()> {
        tracing::info!(params = ?self, "running hello");

        let Self { name } = self;

        let name = name.as_deref().unwrap_or("world");
        println!("Hello, {name}!");

        Ok(())
    }
}
