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
use harmos_protocol::v1::OperationContract;

// Local Imports
use super::ClientSettings;
use crate::{Error, Result, client::Context};
use dokime_protocol::client::EvidenceArchiveRequest;
use dokime_protocol::client::EvidenceArchiveResponse;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Evidence/Archive";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<dokime_protocol::client::EvidenceArchiveRequest, dokime_protocol::client::EvidenceArchiveResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(context: &Context, request: EvidenceArchiveRequest) -> Result<EvidenceArchiveResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let key =
            format!("archive/{}", super::views::hex(&request.subject.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default()));
        let (mut reply, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
            let (store, _read_cancellation) = context.host.store().read_session()?;
            let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
            match subject.kind.as_ref() {
                Some(dokime_protocol::client::evidence_subject::Kind::Execution(execution)) => {
                    store.write_execution_archive(execution, &mut file)?
                }
                Some(dokime_protocol::client::evidence_subject::Kind::Cycle(cycle)) => {
                    store.write_cycle_archive(cycle, std::num::NonZeroUsize::new(context.settings.page_items).unwrap(), &mut file)?;
                }
                None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
            }
            let (file, length, sha256) = super::content_transfer::temporary(file)?;
            let digest =
                (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let reference = harmos_protocol::storage::FileReference {
                metadata: Some(harmos_protocol::storage::FileMetadata {
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
            Ok(dokime_protocol::client::EvidenceArchiveResponse { file: Some(reference), resolved_at: None })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
