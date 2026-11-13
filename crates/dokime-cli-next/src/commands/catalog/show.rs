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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    suite: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let mut client = dokime::v1::catalog_client::CatalogClient::with_interceptor(channel, settings.interceptor());
    let value = client.get_suite(dokime::v1::CatalogGetSuiteRequest { suite_id: args.suite }).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
