/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Append Explicit Execution Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Encoded dokime.client.v1.ExecutionsCreateRequest")]
    request: PathBuf,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    if std::fs::metadata(&args.request)?.len() > settings.retained_bytes.get() as u64 {
        return Err(crate::error::Error::Capacity("Append Executions: Select a Smaller Request".into()));
    }
    let request = client::ExecutionsCreateRequest::decode(std::fs::read(args.request)?.as_slice())?;
    let client = settings.sdk(endpoint, false).await?;
    let value = client.executions.create(settings.request(request)).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
