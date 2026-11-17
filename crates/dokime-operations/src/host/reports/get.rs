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
use std::io::Write;

// Third Party Imports
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::OperationContract;
use harmos_runtime::Identifier;

// Local Imports
use crate::host::ClientSettings;
use crate::host::files::uploads::{append, create, delete, publish};
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportOrigin;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};
use dokime_protocol::host::report_subject::Kind;
use dokime_storage::persisted::Scope;

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
        let temporary = context.host.temporary_report(&request.name)?;
        let scope = match temporary.as_ref().and_then(|file| file.metadata.as_ref()).and_then(|metadata| metadata.scope.as_ref()) {
            Some(scope) => Identifier::new(scope.clone())
                .map_err(|source| Error::Invalid(format!("Get Report: Restore Its Storage Scope: {source}")))?,
            None => match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                Some(Kind::Execution(execution)) => Scope::Execution(execution).identifier()?,
                Some(Kind::Cycle(cycle)) => Scope::Cycle(cycle).identifier()?,
                None => return Err(Error::Invalid("Get Report: Select Its Published Subject or Prepared Report".into())),
            },
        };
        let resource = format!(
            "report/{}/{}",
            request.name,
            crate::host::views::hex(&crate::host::paging::bytes(&ReportsGetRequest { at: None, ..request.clone() }))
        );
        let (mut reply, cut) = crate::host::views::read(context, scope, &resource, request.at.as_ref(), async {
            if let Some(reference) = temporary {
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
            file.write_all(&report.html).map_err(|error| Error::Invalid(format!("Get Report: {error}")))?;
            let (file, length, digest) = crate::host::content::transfer::temporary(file)?;
            let digest =
                (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let created = create::handle(
                context,
                UploadsCreateRequest {
                    metadata: Some(FileMetadata {
                        id: report.identifier.clone(),
                        scope: Some(Scope::Execution(execution).identifier()?.to_string()),
                        media_type: "text/html".into(),
                        expected_length: Some(length),
                        expected_sha256: Some(digest.clone()),
                    }),
                    request_id: format!("{ID}/{}/create", report.identifier),
                },
            )
            .await?;
            let upload = created.upload.ok_or_else(|| Error::Precondition("Get Report: Create Its Native Upload".into()))?;
            let result = async {
                let mut offset = 0;
                while offset < length {
                    let count = (length - offset).min(created.chunk_bytes) as usize;
                    let mut data = vec![0; count];
                    crate::host::content::transfer::read_temporary(&file, offset, &mut data)?;
                    append::handle(
                        context,
                        UploadsAppendRequest {
                            upload: Some(upload.clone()),
                            offset,
                            data,
                            request_id: format!("{ID}/{}/{offset}", report.identifier),
                        },
                    )
                    .await?;
                    offset += count as u64;
                }
                let published = publish::handle(
                    context,
                    UploadsPublishRequest {
                        upload: Some(upload.clone()),
                        length,
                        sha256: digest,
                        request_id: format!("{ID}/{}/publish", report.identifier),
                    },
                )
                .await?;
                published.upload.ok_or_else(|| Error::Precondition("Get Report: Publish Its Native File".into()))
            }
            .await;
            let reference = match result {
                Ok(reference) => reference,
                Err(source) => {
                    if let Err(cleanup) = delete::handle(
                        context,
                        UploadsDeleteRequest {
                            upload: Some(upload),
                            request_id: uuid::Uuid::now_v7().to_string(),
                        },
                    )
                    .await
                    {
                        return Err(Error::Invalid(format!("Get Report: {source}; Remove Its Failed Upload: {cleanup}")));
                    }
                    return Err(source);
                }
            };
            context.host.retain_report(reference.clone())?;
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
