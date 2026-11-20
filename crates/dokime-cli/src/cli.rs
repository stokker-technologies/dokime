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
use std::{num::NonZeroU64, path::PathBuf};

// Third Party Imports
use clap::Parser;

// Local Imports
use crate::commands::Command;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Parser)]
#[command(name = "dokime", display_name = "Dokime", about = "Author Plugins and Control a Dokime Host", version = env!("DOKIME_BUILD_VERSION"))]
pub(crate) struct Cli {
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
    pub command: Command,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
