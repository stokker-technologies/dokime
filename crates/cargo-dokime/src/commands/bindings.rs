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
use std::path::PathBuf;
// Third Party Imports
use harmos_packaging::proto;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
pub struct ProtoArgs {
    #[arg(long)]
    output: PathBuf,
}

/* === Enums === */

#[derive(clap::Subcommand)]
enum Command {
    #[command(about = "Export Protobuf Contracts")]
    Proto(ProtoArgs),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(args: Args) -> Result<(), crate::error::Error> {
    match args.command {
        Command::Proto(args) => {
            let contracts = dokime::harmos::operations::Contracts::of::<dokime::operations::Operations>()?;
            let output = proto::generate(&contracts, &args.output)?;
            println!("{}", output.display());
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
