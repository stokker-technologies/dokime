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
use crate::client::ExecutionsCreateOptions;
use dokime_protocol::host::ExecutionsCreateRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Encoded ExecutionsCreateRequest")]
    request: PathBuf,

    #[arg(long)]
    json: bool,

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    if std::fs::metadata(&args.request)?.len() > settings.retained_bytes.get() as u64 {
        return Err(crate::error::Error::Capacity("Append Executions: Select a Smaller Request".into()));
    }
    let request = ExecutionsCreateRequest::decode(std::fs::read(args.request)?.as_slice())?;
    let id_file = args.id_file.open()?;
    let client = settings.sdk(endpoint).await?;
    let value = client
        .executions
        .create_with(
            request.parent,
            request.executions,
            ExecutionsCreateOptions {
                context: settings.context()?,
                request_id: (!request.request_id.is_empty()).then_some(request.request_id),
            },
        )
        .await?;
    if let Some(file) = id_file {
        file.write(value.executions.iter().filter_map(|execution| execution.summary.as_ref()).map(|summary| &summary.execution_id))?;
    }
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
