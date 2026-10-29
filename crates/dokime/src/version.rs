/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, help = "Emit CLI version metadata as JSON.")]
    json: bool,
}

pub fn run(args: Args) {
    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
            })
        );
    } else {
        println!("{}", env!("CARGO_PKG_VERSION"));
    }
}
