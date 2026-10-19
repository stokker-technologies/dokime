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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[command(about = "Open The Installed Desktop GUI On A Running Backend; Arguments After -- Reach It Verbatim.", trailing_var_arg = true)]
pub struct Args {
    #[arg(long, visible_alias = "connect", help = "Native gRPC endpoint of the backend.")]
    endpoint: String,

    #[arg(long, help = "GUI executable. Defaults to the installed dokime-gui.")]
    gui: Option<PathBuf>,

    #[arg(allow_hyphen_values = true, help = "Arguments forwarded to the GUI.")]
    arguments: Vec<OsString>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(args: Args) -> crate::Result<()> {
    let gui = Program::GUI.resolve(args.gui.as_deref())?;

    let mut command = Command::new(gui);
    command.arg("--endpoint").arg(args.endpoint).args(args.arguments);
    hand_over(command)
}

/* === Private Functions === */

/* === Tests === */
