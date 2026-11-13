/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.reports.materialize_report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ReportsMaterializeReportRequest, v1::MaterializedReport>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ReportsMaterializeReportRequest) -> Result<v1::MaterializedReport> {
    let context = context.clone();
    super::complete(ID, context.settings.read_timeout, async move {
        tokio::task::spawn_blocking(move || {
            let context = &context;

            let subject = request.subject.ok_or_else(|| Error::Invalid("Materialize Report: Select an Execution or Cycle".into()))?;
            let (file, metadata) = context.host.render_report(&subject, request.at.as_ref())?;
            let (file, length, sha256) = super::content_transfer::temporary(file)?;
            let digest =
                (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let identifier = format!("temporary-report/{}.html", uuid::Uuid::now_v7());
            let reference = v1::FileReference {
                position: request.at,
                metadata: Some(v1::FileMetadata {
                    id: identifier,
                    scope: metadata.test_execution_id.or(Some(metadata.test_cycle_id)),
                    media_type: "text/html".into(),
                    expected_length: Some(length),
                    expected_sha256: Some(digest.clone()),
                }),
                length,
                sha256: digest,
            };
            context.host.retain_report(file, reference.clone())?;
            Ok(v1::MaterializedReport { report: Some(reference) })
        })
        .await
        .map_err(|error| Error::Invalid(format!("Render Report: Retry After the Rendering Worker Failed: {error}")))?
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
