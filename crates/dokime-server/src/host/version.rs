/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::domain::DEV_PROTOCOL;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, help = "Emit stable host identity for development tools.")]
    json: bool,
}

pub fn run(args: Args) {
    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "sdk_revision": serde_json::Value::Null,
                "dev_protocol": DEV_PROTOCOL,
            })
        );
    } else {
        println!("{}", env!("CARGO_PKG_VERSION"));
    }
}
