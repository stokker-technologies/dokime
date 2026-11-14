/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportOrigin;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::report_subject::Kind;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ReportsGetRequest, ReportsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ReportsGetRequest) -> Result<ReportsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let resource = format!(
            "report/{}/{}",
            request.name,
            crate::host::views::hex(&crate::host::paging::bytes(&ReportsGetRequest { at: None, ..request.clone() }))
        );
        let (mut reply, cut) = crate::host::views::read(context, &resource, request.at.as_ref(), async {
            if let Some((_, reference)) = context.host.temporary_report(&request.name)? {
                return Ok(ReportsGetResponse {
                    file: Some(reference),
                    report: None,
                    resolved_at: None,
                });
            }
            let execution = match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                Some(Kind::Execution(execution)) => execution,
                _ => return Err(Error::Invalid("Get Report: Select Its Published Execution".into())),
            };
            let report = context
                .host
                .store()
                .reports(execution)?
                .into_iter()
                .find(|report| request.name.is_empty() || report.identifier == request.name)
                .ok_or_else(|| Error::NotFound("Get Report: Publish the Selected Report First".into()))?;
            let cut = context.host.store().metadata(execution)?;
            let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Get Report: {error}")))?;
            use std::io::Write;
            file.write_all(&report.html).map_err(|error| Error::Invalid(format!("Get Report: {error}")))?;
            let (_, length, digest) = crate::host::content::transfer::temporary(
                file.try_clone().map_err(|error| Error::Invalid(format!("Get Report: {error}")))?,
            )?;
            let digest =
                (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let reference = FileReference {
                metadata: Some(FileMetadata {
                    id: report.identifier.clone(),
                    scope: Some(execution.clone()),
                    media_type: "text/html".into(),
                    expected_length: Some(length),
                    expected_sha256: Some(digest.clone()),
                }),
                length,
                sha256: digest,
                ..Default::default()
            };
            context.host.retain_report(file, reference.clone())?;
            Ok(ReportsGetResponse {
                report: Some(ReportMetadata {
                    test_cycle_id: cut.execution.cycle,
                    test_execution_id: Some(execution.clone()),
                    identifier: report.identifier,
                    metadata: report.document.and_then(|document| document.metadata),
                    origin: ReportOrigin::Sealed as i32,
                    ..Default::default()
                }),
                file: Some(reference),
                resolved_at: None,
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ReportsGetRequest, ReportsGetResponse, context, "dokime.host.v1", "Reports", "Get");
