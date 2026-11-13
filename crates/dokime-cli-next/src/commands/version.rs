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

// Third Party Imports

// Local Imports

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

pub(crate) fn run(args: Args) -> crate::Result<()> {
    let revision = env!("DOKIME_SOURCE_REVISION");
    let channel = env!("DOKIME_BUILD_CHANNEL");
    let revision = (revision != "unknown").then_some(revision);
    let channel = (!channel.is_empty()).then_some(channel);
    let metadata = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"), "revision": revision, "channel": channel,
        "target_os": std::env::consts::OS, "target_arch": std::env::consts::ARCH,
    });
    if args.json {
        println!("{}", serde_json::to_string(&metadata)?);
    } else {
        println!("Dokime {} ({}, {})", env!("CARGO_PKG_VERSION"), revision.unwrap_or("unknown"), channel.unwrap_or("local"));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
