/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use clap::Parser;

use crate::commands::Command;

#[derive(Debug, Parser)]
#[command(name = "dokime", about = "Connect To Dokime Backends", version)]
pub struct Cli {
    #[arg(long, global = true, default_value = "auto", value_parser = clap::value_parser!(crate::terminal::ColorMode), help = "Color output: auto, always, or never.")]
    pub color: crate::terminal::ColorMode,

    #[command(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    pub fn configuration(&self) -> crate::Result<dokime::config::Config> {
        crate::commands::configuration(&self.command)
    }

    pub async fn run(self, configuration: dokime::config::Config) -> crate::Result<()> {
        crate::terminal::set_color_mode(self.color);
        Box::pin(crate::commands::run(self.command, configuration)).await
    }
}
