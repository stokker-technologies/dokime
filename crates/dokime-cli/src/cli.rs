/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line Arguments

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env::consts, num::NonZeroU64, path::PathBuf};

// Third Party Imports
use clap::Parser;

// Local Imports
use crate::{Result, commands::Command};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Parser)]
#[command(
    name = "dokime",
    display_name = "Dokime",
    about = "Author Plugins and Control a Dokime Host",
    disable_version_flag = true,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    #[arg(long, short = 'V', help = "Print Build Identity")]
    pub version: bool,

    #[arg(long, requires = "version", help = "Print Build Identity as JSON")]
    pub json: bool,

    #[arg(long, global = true, help = "Host URL for Remote Commands, or HOST:PORT for Hosting")]
    pub endpoint: Option<String>,

    #[arg(long, global = true, conflicts_with = "endpoint", help = "Portable .pair Pairing File")]
    pub remote: Option<PathBuf>,

    #[arg(long, global = true, help = "TOML Configuration File")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, value_name = "seconds", help = "Connection Timeout in Seconds, Overriding connect_timeout_s")]
    pub connect_timeout: Option<NonZeroU64>,

    #[arg(long, global = true, help = "Native Credential Namespace, Overriding credential_namespace")]
    pub credential_namespace: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn version(json: bool) -> Result<String> {
    if !json {
        return Ok(env!("DOKIME_BUILD_VERSION").to_owned());
    }
    let revision = env!("DOKIME_SOURCE_REVISION");
    let channel = env!("DOKIME_BUILD_CHANNEL");
    let revision = (revision != "unknown").then_some(revision);
    let channel = (!channel.is_empty()).then_some(channel);
    let metadata = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"), "revision": revision, "channel": channel,
        "target_os": consts::OS, "target_arch": consts::ARCH,
    });
    Ok(serde_json::to_string(&metadata)?)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use clap::{Parser, error::ErrorKind};

    use super::Cli;

    #[test]
    fn parse_build_identity_flags() {
        for arguments in [["dokime", "--version"], ["dokime", "-V"]] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            assert!(cli.version);
            assert!(!cli.json);
            assert!(cli.command.is_none());
        }
        let cli = Cli::try_parse_from(["dokime", "--version", "--json"]).unwrap();
        assert!(cli.version && cli.json);
        assert!(cli.command.is_none());
        let error = Cli::try_parse_from(["dokime", "--json"]).err().unwrap();
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }
}
