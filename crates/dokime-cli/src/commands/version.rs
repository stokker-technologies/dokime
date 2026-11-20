/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report the Built Command Identity

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env::consts;

// Third Party Imports
use clap::CommandFactory;

// Local Imports
use crate::{Result, cli::Cli};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn run(args: Args) -> Result<()> {
    let revision = env!("DOKIME_SOURCE_REVISION");
    let channel = env!("DOKIME_BUILD_CHANNEL");
    let revision = (revision != "unknown").then_some(revision);
    let channel = (!channel.is_empty()).then_some(channel);
    let metadata = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"), "revision": revision, "channel": channel,
        "target_os": consts::OS, "target_arch": consts::ARCH,
    });
    if args.json {
        println!("{}", serde_json::to_string(&metadata)?);
    } else {
        print!("{}", Cli::command().render_version());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
