/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read or Update the Host Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::SuitesListOptions;

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

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    let value = client
        .suites
        .list_with(
            "",
            SuitesListOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    if args.json {
        return crate::output::print(&value, true);
    }
    for suite in value.values {
        let metadata = suite.metadata.ok_or("List Suites: Read a Suite With Metadata")?;
        println!("{}\t{}\t{} Templates", metadata.identifier, metadata.label, suite.templates.len());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
