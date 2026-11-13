/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start a Cycle From Frozen Cases

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
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

    #[arg(long)]
    cases: PathBuf,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    if std::fs::metadata(&args.cases)?.len() > settings.retained_bytes.get() as u64 {
        return Err(crate::error::Error::Capacity("Start Cycle: Select a Smaller Case File".into()));
    }
    let cases = client::CasesCreateResponse::decode(std::fs::read(args.cases)?.as_slice())?;
    let origins =
        cases.cases.into_iter().map(|case| case.origin.ok_or("Start Cycle: Case Has No Frozen Origin")).collect::<Result<Vec<_>, _>>()?;
    let client = settings.sdk(endpoint, false).await?;
    let request = client::CyclesCreateRequest {
        origins,
        continue_automatically: true,
        request_id: settings.request_id(args.request_id)?,
        ..Default::default()
    };
    let value = client.cycles.create(settings.request(request)).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
