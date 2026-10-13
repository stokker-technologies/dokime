/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos::Protobuf;
use harmos::operations::{Context, Operation, Response};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use super::{metadata, metadata_memory};
use crate::grpc::{error::detailed, value};
use dokime_app::{RuntimeHandle, artifacts::ArtifactRecord};
use dokime_contract::operations::content as wire;

/* === Constants === */
const SCAN_LIMIT: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct List {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for List {
    type Declaration = wire::ListArtifacts;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListArtifactsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ListArtifactsReply>, Status> {
        if request.limit > 100 {
            return Err(Status::invalid_argument("page limit exceeds 100"));
        }
        value::text(&request.after_artifact_id)?;
        let epoch = self.resources.runtime.process_epoch().to_string();
        let records = self
            .resources
            .runtime
            .artifacts_limited(SCAN_LIMIT)
            .await
            .map_err(|source| {
                detailed(
                    Status::resource_exhausted("artifact inventory exceeds limit"),
                    &source,
                )
            })?;
        let start = if request.after_artifact_id.is_empty() {
            0
        } else {
            records
                .iter()
                .position(|record| record.artifact_id.as_str() == request.after_artifact_id)
                .map(|index| index + 1)
                .unwrap_or(records.len())
        };
        let limit = if request.limit == 0 {
            50
        } else {
            request.limit as usize
        };
        let count = records.len().saturating_sub(start).min(limit);
        let more = records.len().saturating_sub(start) > count;
        let mut required = std::mem::size_of::<wire::ListArtifactsReply>()
            .saturating_add(epoch.len())
            .saturating_add(32)
            .saturating_add(count.saturating_mul(std::mem::size_of::<wire::ArtifactMetadata>()));
        let mut scratch = 0usize;
        for (index, record) in records.iter().enumerate() {
            let bytes = metadata_memory(record)?;
            scratch = scratch.max(bytes.saturating_mul(2).saturating_add(16));
            if (start..start + count).contains(&index) {
                required = required.saturating_add(
                    bytes.saturating_sub(std::mem::size_of::<wire::ArtifactMetadata>()),
                );
            }
        }
        if more {
            required =
                required.saturating_add(records[start + count - 1].artifact_id.as_str().len());
        }
        required = required.saturating_add(scratch);
        if required > self.resources.maximum {
            return Err(Status::resource_exhausted(
                "artifact inventory projection exceeds budget",
            ));
        }
        let allowance = NonZeroUsize::new(required)
            .ok_or_else(|| Status::resource_exhausted("invalid artifact page allowance"))?;
        let mut refusal = None;
        let projected = records.try_project(allowance, |records, _| {
            match project(
                records,
                request,
                &epoch,
                start,
                count,
                more,
                self.resources.maximum,
            ) {
                Ok(page) => Ok(page),
                Err(source) => {
                    refusal = Some(source);
                    Err(harmos::Error::InvalidInput {
                        field: "artifact listing",
                        reason: "canonical projection refused",
                    })
                }
            }
        });
        if let Some(source) = refusal {
            return Err(source);
        }
        let projected = projected.map_err(memory_failure)?;
        projected.try_into_response().map_err(memory_failure)
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn project(
    records: &[ArtifactRecord],
    request: &wire::ListArtifactsRequest,
    epoch: &str,
    start: usize,
    count: usize,
    more: bool,
    maximum: usize,
) -> Result<wire::ListArtifactsReply, Status> {
    let mut selected = value::canonical::slots(count)?;
    let mut digest = Sha256::new();
    for (index, record) in records.iter().enumerate() {
        let artifact = metadata(record)?;
        let encoded = artifact.encode_to_vec();
        let mut prefix = Vec::with_capacity(10);
        prost::encoding::encode_varint(encoded.len() as u64, &mut prefix);
        digest.update(&prefix);
        digest.update(&encoded);
        if (start..start + count).contains(&index) {
            selected.push(artifact);
        }
    }
    let fingerprint = digest.finalize().to_vec();
    if request
        .cursor
        .as_ref()
        .is_some_and(|cursor| cursor.epoch != epoch)
    {
        return Err(Status::failed_precondition(
            "artifact cursor epoch mismatch; resynchronize without cursor",
        ));
    }

    if request
        .cursor
        .as_ref()
        .is_some_and(|cursor| cursor.metadata_sha256 != fingerprint)
    {
        return Err(Status::aborted(
            "artifact listing changed; restart pagination",
        ));
    }
    if !request.after_artifact_id.is_empty()
        && start == records.len()
        && !records
            .iter()
            .any(|record| record.artifact_id.as_str() == request.after_artifact_id)
    {
        return Err(Status::invalid_argument("after artifact ID not found"));
    }
    let next_after_artifact_id = if more {
        Some(
            selected
                .last()
                .ok_or_else(|| Status::internal("empty continued artifact page"))?
                .artifact_id
                .clone(),
        )
    } else {
        None
    };
    let reply = wire::ListArtifactsReply {
        artifacts: selected,
        next_after_artifact_id,
        cursor: Some(wire::ArtifactCursor {
            epoch: epoch.to_owned(),
            metadata_sha256: fingerprint,
        }),
    };
    if reply.encoded_len() > maximum {
        return Err(Status::resource_exhausted(
            "artifact page exceeds configured message limit",
        ));
    }
    Ok(reply)
}
fn memory_failure(source: harmos::Error) -> Status {
    let source = dokime_app::Error::from(source);
    detailed(
        Status::resource_exhausted("artifact page exceeds original storage budget"),
        &source,
    )
}

/* === Tests === */
