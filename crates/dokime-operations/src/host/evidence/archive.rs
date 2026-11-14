/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Archive

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::EvidenceArchiveRequest;
use dokime_protocol::host::EvidenceArchiveResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Evidence/Archive";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<dokime_protocol::host::EvidenceArchiveRequest, dokime_protocol::host::EvidenceArchiveResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(context: &Context, request: EvidenceArchiveRequest) -> Result<EvidenceArchiveResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let key = format!(
            "archive/{}",
            crate::host::views::hex(&request.subject.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default())
        );
        let (mut reply, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
            let (store, _read_cancellation) = context.host.store().read_session()?;
            let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
            match subject.kind.as_ref() {
                Some(dokime_protocol::host::evidence_subject::Kind::Execution(execution)) => {
                    store.write_execution_archive(execution, &mut file)?
                }
                Some(dokime_protocol::host::evidence_subject::Kind::Cycle(cycle)) => {
                    store.write_cycle_archive(cycle, std::num::NonZeroUsize::new(context.settings.page_items).unwrap(), &mut file)?;
                }
                None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
            }
            let (file, length, sha256) = crate::host::content::transfer::temporary(file)?;
            let digest =
                (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let reference = harmos_protocol::model::FileReference {
                metadata: Some(harmos_protocol::model::FileMetadata {
                    id: format!("temporary-evidence/{}", uuid::Uuid::now_v7()),
                    scope: None,
                    media_type: "application/zip".into(),
                    expected_length: Some(length),
                    expected_sha256: Some(digest.clone()),
                }),
                length,
                sha256: digest,
                ..Default::default()
            };
            context.host.retain_report(file, reference.clone())?;
            Ok(dokime_protocol::host::EvidenceArchiveResponse { file: Some(reference), resolved_at: None })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
