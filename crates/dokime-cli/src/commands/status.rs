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

mod build_identity {
    include!("../../build_identity.rs");
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::StatusGetOptions;

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
    let channel = settings.channel(endpoint).await?;
    let client = dokime_client_internal::Client::from_channel(channel, settings.client())?;
    let value = client.status.get_with(StatusGetOptions { context: settings.context()? }).await?;
    if !args.json {
        let identity = value.status.as_ref().and_then(|status| status.identity.as_ref());
        let version = identity.map(|identity| identity.application_version.as_str()).unwrap_or_default();
        let build = value.build.clone().unwrap_or_default();
        println!("{}", build_identity::version(version, &build.source_revision, &build.channel));
    }
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
