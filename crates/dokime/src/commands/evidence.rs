/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download And Publish Sealed Evidence Atomically

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_domain::model::exports::EvidenceSubject;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Sealed Execution Or Cycle UUID To Export.")]
    id: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Destination ZIP; Existing Files Require --overwrite.")]
    output: PathBuf,

    #[arg(long, help = "Atomically Replace An Existing Destination Only After Successful Verification.")]
    overwrite: bool,

    #[arg(long, help = "Emit The Verified Archive Path, Size And SHA-256 As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args, subject: fn(String) -> EvidenceSubject) -> crate::Result<()> {
    let id = dokime_domain::model::identity::TestCycleId::from_uuid(&args.id)
        .map_err(|_| crate::Error::Input("Evidence identity must be a UUID".into()))?;
    dokime_client::validate_download_destination(&args.output, args.overwrite)?;
    let client = args.connection.connect().await?;
    let request = dokime_client::contracts::content::DownloadEvidence { subject: Some(subject(id.to_string())) };
    let result = tokio::select! {
        result = client.download_evidence(request, args.output, args.overwrite) => result.map_err(crate::Error::from),
        signal = tokio::signal::ctrl_c() => match signal {
            Ok(()) => Err(crate::Error::Input("Evidence download cancelled. Retry the export to download from the beginning; executions are unaffected.".into())),
            Err(error) => Err(error.into()),
        },
    };
    client.close_and_wait().await;
    let downloaded = result?;
    if args.json {
        println!("{}", serde_json::json!({ "path": downloaded.path, "size_bytes": downloaded.size_bytes, "sha256": downloaded.sha256 }));
    } else {
        println!("Evidence Saved To {} ({} Bytes, SHA-256 {})", downloaded.path.display(), downloaded.size_bytes, downloaded.sha256);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
