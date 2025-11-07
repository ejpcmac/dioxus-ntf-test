//! A CLI client for the notification API.

use eyre::Result;

use ntf_cli::NtfCli;

fn main() -> Result<()> {
    color_eyre::install()?;
    NtfCli::run()
}
