/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Inspect a Bundle or Running Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::path::PathBuf;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, required_unless_present = "url", conflicts_with = "url")]
    bundle: Option<PathBuf>,

    #[arg(long, required_unless_present = "bundle")]
    url: Option<String>,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, _endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    if let Some(path) = args.bundle {
        let manifest = super::bundle::inspect(&path)?;
        if args.json {
            println!("{}", serde_json::to_string(&manifest)?);
        } else {
            println!("{} {}", manifest["identity"]["name"].as_str().unwrap_or(""), manifest["identity"]["version"].as_str().unwrap_or(""));
        }
        return Ok(());
    }
    // Use the Protocol Descriptor to Bind Describe Without a Second Wire Model
    let url = args.url.ok_or("Inspect Plugin: Supply --bundle or --url")?;
    let description = dokime_runtime_next::describe_plugin(&url, &settings.placement()).await?;
    crate::output::print(&description, args.json)
}

/* === Private Functions === */

/* === Tests === */
