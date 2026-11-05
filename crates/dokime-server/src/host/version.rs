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

// Local Imports
use dokime::domain::DEV_PROTOCOL;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, help = "Emit stable host identity for development tools.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(args: Args) {
    if args.json {
        let build = dokime::version::current();
        let revision = build.revision.map(|revision| revision.full());
        println!(
            "{}",
            serde_json::json!({
                "version": build.version,
                "revision": revision,
                "channel": build.channel,
                "sdk_revision": serde_json::Value::Null,
                "dev_protocol": DEV_PROTOCOL,
            })
        );
    } else {
        println!("{}", dokime::version::label());
    }
}

/* === Private Functions === */

/* === Tests === */
