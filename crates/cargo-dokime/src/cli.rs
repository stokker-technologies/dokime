/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Parser)]
#[command(name = "cargo dokime", version, about = "Author Dokime Sidecars And Native Installers")]
pub struct Cli {
    #[arg(long, global = true, default_value = "auto", value_parser = clap::value_parser!(dokime::terminal::ColorMode), help = "Color output: auto, always, or never.")]
    pub color: dokime::terminal::ColorMode,

    #[command(subcommand)]
    pub command: crate::commands::Command,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
