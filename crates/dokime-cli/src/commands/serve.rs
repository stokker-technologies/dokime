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
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

// Third Party Imports

// Local Imports
use crate::installation::{Program, hand_over};
use crate::terminal::{ColorMode, color_mode};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[command(about = "Run The Installed Dokime Server; Arguments After -- Reach It Verbatim.", trailing_var_arg = true)]
pub struct Args {
    #[arg(long, help = "Server executable. Defaults to the installed dokime-server.")]
    server: Option<PathBuf>,

    #[arg(allow_hyphen_values = true, help = "Arguments forwarded to the server.")]
    arguments: Vec<OsString>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(args: Args) -> crate::Result<()> {
    let server = Program::SERVER.resolve(args.server.as_deref())?;
    let color = match color_mode() {
        ColorMode::Auto => "auto",
        ColorMode::Always => "always",
        ColorMode::Never => "never",
    };

    let mut command = Command::new(server);
    command.arg("--color").arg(color).args(args.arguments);
    hand_over(command)
}

/* === Private Functions === */

/* === Tests === */
