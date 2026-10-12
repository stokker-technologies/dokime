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
use crate::commands::Command;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, Parser)]
#[command(name = "dokime", about = "Dokime Rust", version)]
pub struct Cli {
    #[arg(long, global = true, default_value = "auto", value_parser = clap::value_parser!(dokime_terminal::ColorMode), help = "Color output: auto, always, or never.")]
    pub color: dokime_terminal::ColorMode,
    #[command(subcommand)]
    pub(crate) command: Command,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Cli {
    pub async fn run(self) -> crate::Result<()> {
        dokime_terminal::set_color_mode(self.color);
        crate::commands::run(self.command, dokime_terminal::Terminal::new(self.color)).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
