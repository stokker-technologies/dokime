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
    let client = dokime::client::HostClient::new(channel, settings.client())?;
    let cycle = client.cycles.start(cases.cases).await?;
    let value = v1::CreateCycleReply {
        cycle_id: cycle.identifier,
        execution_ids: cycle.executions,
        ..Default::default()
    };
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
