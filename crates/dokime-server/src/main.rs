/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Standalone server process entry point

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{path::Path, process::ExitCode};

// Third Party Imports
use clap::Parser;

// Local Imports
use dokime_server::host::{self, Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, Parser)]
#[command(name = "dokime-server", about = "Run The Dokime Backend.", version)]
struct Cli {
    #[arg(long, global = true, default_value = "auto", value_parser = clap::value_parser!(dokime_terminal::ColorMode), help = "Color output: auto, always, or never.")]
    color: dokime_terminal::ColorMode,
    #[command(flatten)]
    args: host::Args,
    #[command(subcommand)]
    command: Option<Command>,
}

/* === Enums === */
#[derive(Debug, clap::Subcommand)]
enum Command {
    Version(host::version::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() -> ExitCode {
    let cli = Cli::parse();
    dokime_terminal::set_color_mode(cli.color);
    if let Some(Command::Version(args)) = cli.command {
        host::version::run(args);
        return ExitCode::SUCCESS;
    }
    let terminal = dokime_terminal::Terminal::new(cli.color);
    let result = cli.args.network.config().and_then(|config| {
        load_environment(cli.args.env_file.as_deref())?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        runtime.block_on(host::run(cli.args, config, terminal.clone()))
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            terminal.failure(dokime::plugin::Diagnostic::capture(&error));
            error.exit_code()
        }
    }
}

fn load_environment(env_file: Option<&Path>) -> Result<()> {
    if let Some(path) = env_file {
        return dotenv::from_path(path)
            .map_err(|_| Error::Input(format!("cannot load environment file {}", path.display())));
    }
    match dotenv::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenv::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(Error::Input("cannot load project .env file".into())),
    }
}

/* === Tests === */
