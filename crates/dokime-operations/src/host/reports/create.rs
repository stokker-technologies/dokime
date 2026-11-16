/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::{FileMetadata, OperationContract, ReadCut};

// Local Imports
use super::PreparedReport;
use crate::host::ClientSettings;
use crate::host::files::uploads::{append, create, delete, publish};
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ReportsCreateRequest, ReportsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: ReportsCreateRequest) -> Result<ReportsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let subject = request.subject.as_ref().ok_or_else(|| Error::Invalid("Create Report: Select Its Execution or Cycle".into()))?;
        let (file, metadata) = context.host.render_report(subject, &request.document_id, request.at.as_ref())?;
        let (retained, length, digest) = crate::host::content::transfer::temporary(
            file.try_clone().map_err(|error| Error::Invalid(format!("Create Report: {error}")))?,
        )?;
        let digest =
            (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
        let name = format!("prepared-report/{}", request.request_id);
        let created = create::handle(
            context,
            UploadsCreateRequest {
                metadata: Some(FileMetadata {
                    id: name.clone(),
                    scope: metadata.test_execution_id.clone().or(Some(metadata.test_cycle_id.clone())),
                    media_type: "text/html".into(),
                    expected_length: Some(length),
                    expected_sha256: Some(digest.clone()),
                }),
                request_id: format!("{ID}/{name}/create"),
            },
        )
        .await?;
        let upload = created.upload.ok_or_else(|| Error::Precondition("Create Report: Create Its Native Upload".into()))?;
        let result = async {
            let mut offset = 0;
            while offset < length {
                let count = (length - offset).min(created.chunk_bytes) as usize;
                let mut data = vec![0; count];
                crate::host::content::transfer::read_temporary(&retained, offset, &mut data)?;
                append::handle(
                    context,
                    UploadsAppendRequest {
                        upload: Some(upload.clone()),
                        offset,
                        data,
                        request_id: format!("{ID}/{name}/{offset}"),
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
                    request_id: format!("{ID}/{name}/publish"),
                },
            )
            .await?;
            published.upload.ok_or_else(|| Error::Precondition("Create Report: Publish Its Native File".into()))
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
                    return Err(Error::Invalid(format!("Create Report: {source}; Remove Its Failed Upload: {cleanup}")));
                }
                return Err(source);
            }
        };

        if let Some(execution) = &metadata.test_execution_id {
            let scope = DocumentScope {
                cycle_id: metadata.test_cycle_id.clone(),
                execution_id: execution.clone(),
            };
            let (document, cut) = context.host.current_document(&scope, &request.document_id)?;
            let prepared = PreparedReport {
                execution: execution.clone(),
                execution_seal: cut.seal,
                document: Some(document),
                file: Some(reference.clone()),
            };
            administration::retain(context, &name, &prepared).await?;
        }
        context.host.retain_report(reference.clone())?;
        let position = context.host.store().evidence_position(None)?;
        Ok(ReportsCreateResponse {
            report: Some(metadata),
            file: Some(reference),
            resolved_at: Some(ReadCut {
                positions: vec![position],
                revisions: Vec::new(),
            }),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ReportsCreateRequest, ReportsCreateResponse, context, "dokime.host.v1", "Reports", "Create");
