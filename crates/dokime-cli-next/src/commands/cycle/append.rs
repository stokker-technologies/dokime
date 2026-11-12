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

use prost::Message;
use std::path::PathBuf;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Encoded dokime.v1.ExecutionAppendExecutionsRequest")]
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
        return Err("Append Executions: Select a Smaller Request".into());
    }
    let request = dokime::v1::ExecutionAppendExecutionsRequest::decode(std::fs::read(args.request)?.as_slice())?;
    let channel = settings.channel(endpoint, false).await?;
    let value = dokime::v1::execution_client::ExecutionClient::new(channel).append_executions(request).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
