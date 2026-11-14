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
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    url: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let mut client = client::plugins_client::PluginsClient::with_interceptor(channel, settings.interceptor());
    let value = client.add(client::AddRemotePlugin { url: args.url }).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
