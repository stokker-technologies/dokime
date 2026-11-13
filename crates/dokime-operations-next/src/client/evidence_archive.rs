/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Evidence/Archive";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::EvidenceArchiveRequest, v1::EvidenceArchiveResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::EvidenceArchiveRequest) -> Result<v1::EvidenceArchiveResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let key=format!("archive/{}",super::views::hex(&request.subject.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default()));
let (mut reply,cut)=super::views::read(context,&key,request.at.as_ref(),async { 
        let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
        let (store, _read_cancellation) = context.host.store().read_session()?;
        let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
        match subject.kind.as_ref() {
            Some(v1::evidence_subject::Kind::Execution(execution)) => store.write_execution_archive(execution, &mut file)?,
            Some(v1::evidence_subject::Kind::Cycle(cycle)) => {
                let mut count = 0;
                let mut after = String::new();
                loop {
                    let ids =
                        store.cycle_execution_page(cycle, &after, std::num::NonZeroUsize::new(context.settings.page_items).unwrap())?;
                    if ids.is_empty() {
                        break;
                    }
                    for id in ids {
                        after = id.clone();
                        if store.metadata(&id)?.execution.cycle != *cycle {
                            continue;
                        }
                        let mut execution =
                            tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Execution Export: {error}")))?;
                        store.write_execution_archive(&id, &mut execution)?;
                        use std::io::{Seek, Write};
                        let length = execution.stream_position().map_err(|error| Error::Invalid(error.to_string()))?;
                        let mut prefix = Vec::new();
                        prost::encoding::encode_key(1, prost::encoding::WireType::LengthDelimited, &mut prefix);
                        prost::encoding::encode_varint(length, &mut prefix);
                        file.write_all(&prefix).map_err(|error| Error::Invalid(error.to_string()))?;
                        execution.rewind().map_err(|error| Error::Invalid(error.to_string()))?;
                        std::io::copy(&mut execution, &mut file).map_err(|error| Error::Invalid(error.to_string()))?;
                        count += 1;
                    }
                }
                if count == 0 {
                    return Err(Error::Invalid("Download Evidence: Select an Existing Cycle".into()));
                }
            }
            None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
        }
        let (file, length, sha256) = super::content_transfer::temporary(file)?;
let digest=(0..sha256.len()).step_by(2).map(|index|u8::from_str_radix(&sha256[index..index+2],16).unwrap()).collect::<Vec<_>>();
let reference=harmos_protocol::storage::FileReference {metadata:Some(harmos_protocol::storage::FileMetadata {id:format!("temporary-evidence/{}",uuid::Uuid::now_v7()),scope:None,media_type:"application/protobuf".into(),expected_length:Some(length),expected_sha256:Some(digest.clone())}),length,sha256:digest,..Default::default()};
context.host.retain_report(file,reference.clone())?;
Ok(v1::EvidenceArchiveResponse {file:Some(reference),resolved_at:None}) }).await?;
reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
