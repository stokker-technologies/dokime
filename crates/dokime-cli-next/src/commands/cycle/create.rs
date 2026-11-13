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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
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
        return Err(crate::error::Error::Capacity("Start Cycle: Select a Smaller Case File or Increase retained_bytes".into()));
    }
    let cases = v1::PreviewSelectionReply::decode(std::fs::read(args.cases)?.as_slice())?;
    let channel = settings.channel(endpoint, false).await?;
    let mut client = v1::execution_client::ExecutionClient::with_interceptor(channel, settings.interceptor());
    let origins = cases.cases.into_iter().map(|case| case.origin.ok_or("Start Cycle: Case Has No Frozen Origin"))
        .collect::<Result<Vec<_>, _>>()?;
    let request = v1::ExecutionCreateCycleRequest { origins, continue_automatically: true, ..Default::default() };
    let value = client.create_cycle(request).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
