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
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

    execution: String,

    #[arg(long)]
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint, false).await?;
    let request = client::ReportsCreateRequest {
        subject: Some(client::ReportSubject {
            kind: Some(client::report_subject::Kind::Execution(args.execution)),
        }),
        request_id: settings.request_id(args.request_id)?,
        ..Default::default()
    };
    let report = client.reports.create(settings.request(request)).await?.into_inner();
    let reference = report.file.ok_or("Fetch Report: Missing File Reference")?;
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    let mut offset = 0;
    let mut hash = sha2::Sha256::new();
    let mut at = report.resolved_at;
    while offset < reference.length {
        let max_bytes = (settings.message_bytes.get().saturating_sub(1024) as u64).min(reference.length - offset);
        if max_bytes == 0 {
            return Err(crate::error::Error::Capacity("Fetch Report: Increase message_bytes".into()));
        }
        let request = dokime::client::v1::StorageGetFileBytesRequest {
            file: Some(reference.clone()),
            offset,
            max_bytes,
            at: at.clone(),
        };
        let page = client.storage.get_file_bytes(settings.request(request)).await?.into_inner();
        if page.total_length != reference.length
            || page.sha256 != reference.sha256
            || page.data.is_empty()
            || page.data.len() as u64 > max_bytes
            || (at.is_some() && page.resolved_at != at)
        {
            return Err("Fetch Report: File Identity, Range or Retained Cut Changed".into());
        }
        file.write_all(&page.data)?;
        offset += page.data.len() as u64;
        hash.update(&page.data);
        at = page.resolved_at;
    }
    if hash.finalize().as_slice() != reference.sha256 {
        return Err("Fetch Report: SHA-256 Does Not Match".into());
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(&args.out)?;
    println!("{}", args.out.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
