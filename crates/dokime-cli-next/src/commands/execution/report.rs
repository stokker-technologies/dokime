/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fetch and Verify a Report Before Publishing the Output File

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Write, path::PathBuf};

// Third Party Imports
use sha2::Digest;

// Local Imports
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    execution: String,

    #[arg(long)]
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, true).await?;
    let mut client = dokime::v1::reports_client::ReportsClient::with_interceptor(channel, settings.interceptor());
    let mut stream = client.download_execution_report(v1::ReportsDownloadExecutionReportRequest { execution_id: args.execution }).await?.into_inner();
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    let mut offset = 0;
    let mut hash = sha2::Sha256::new();
    let mut metadata = false;
    let mut complete = false;
    let mut terminal = false;
    while let Some(frame) = stream.message().await? {
        if terminal {
            return Err("Fetch Report: Data Follows the Terminal Response".into());
        }
        match frame.value.ok_or("Fetch Report: Missing Envelope Value")? {
            v1::reports_download_execution_report_output::Value::Item(frame) => {
                if complete {
                    return Err("Fetch Report: Data Follows Completion".into());
                }
                match frame.payload.and_then(|value| value.kind).ok_or("Fetch Report: Missing Payload")? {
                    v1::report_html_frame_payload::Kind::Report(_) => {
                        if metadata || offset != 0 {
                            return Err("Fetch Report: Duplicate or Late Metadata".into());
                        }
                        metadata = true;
                    }
                    v1::report_html_frame_payload::Kind::Chunk(chunk) => {
                        if !metadata || chunk.offset != offset || chunk.data.is_empty() {
                            return Err("Fetch Report: Missing or Out-of-Order Chunk".into());
                        }
                        file.write_all(&chunk.data)?;
                        offset += chunk.data.len() as u64;
                        hash.update(&chunk.data);
                    }
                    v1::report_html_frame_payload::Kind::Complete(value) => {
                        let digest = hash.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                        if !metadata || value.size_bytes != offset || value.sha256 != digest {
                            return Err("Fetch Report: Size or SHA-256 Does Not Match".into());
                        }
                        complete = true;
                    }
                }
            }
            v1::reports_download_execution_report_output::Value::Response(_) => {
                if !complete {
                    return Err("Fetch Report: Terminal Response Precedes Completion".into());
                }
                terminal = true;
            }
        }
    }
    if !terminal {
        return Err("Fetch Report: Missing Terminal Response".into());
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(&args.out)?;
    println!("{}", args.out.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
