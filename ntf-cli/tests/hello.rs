//! CLI tests for `ntf-cli hello`.

// NOTE: rexpect is only compatible with Unix-like systems, so let’s just not
// compile the CLI tests on Windows.
#![cfg(not(target_os = "windows"))]
#![allow(clippy::pedantic, clippy::restriction)]

use std::process::Command;

use assert_cmd::cargo_bin;
use eyre::Result;
use rexpect::session::spawn_command;

const TIMEOUT: Option<u64> = Some(1_000);

////////////////////////////////////////////////////////////////////////////////
//                                  Helpers                                   //
////////////////////////////////////////////////////////////////////////////////

fn ntf_cli_hello() -> Result<Command> {
    let mut cmd = Command::new(cargo_bin!("ntf-cli"));
    cmd.env("NO_COLOR", "true")
        // NOTE: Enable tracing to avoid missing coverage noise.
        .arg("-vvvv")
        .arg("hello");

    Ok(cmd)
}

////////////////////////////////////////////////////////////////////////////////
//                                   Hello                                    //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn says_hello_world_by_default() -> Result<()> {
    let mut process = spawn_command(ntf_cli_hello()?, TIMEOUT)?;

    process.exp_string("Hello, world!")?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn says_hello_with_name() -> Result<()> {
    let mut command = ntf_cli_hello()?;
    command.arg("Steve");

    let mut process = spawn_command(command, TIMEOUT)?;

    process.exp_string("Hello, Steve!")?;
    process.exp_eof()?;

    Ok(())
}
