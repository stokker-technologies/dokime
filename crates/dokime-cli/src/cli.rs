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
use std::path::PathBuf;

// Third Party Imports
use clap::Parser;

// Local Imports
use crate::commands::Command;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Parser)]
#[command(name = "dokime", about = "Author Plugins and Control a Dokime Host", version)]
pub(crate) struct Cli {
    #[arg(long, global = true, help = "Host URL for Remote Commands, or HOST:PORT for Hosting")]
    pub endpoint: Option<String>,

    #[arg(long, global = true, conflicts_with = "endpoint", help = "Portable .dokime-remote Pairing File")]
    pub remote: Option<PathBuf>,

    #[arg(long, global = true, help = "TOML Configuration File")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
