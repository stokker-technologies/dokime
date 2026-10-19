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
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use super::metadata;
use crate::grpc::transcode::Transcode;
use crate::grpc::{error::detailed, value};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::operations::content as wire;
use dokime_runtime::{RuntimeHandle, artifacts::ArtifactRecord};

/* === Constants === */

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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListArtifacts, _: &mut (),
    ) -> Result<Response<dokime_domain::model::content::wire::ListArtifactsReply>, Status> {
        if request.limit > 100 {
            return Err(Status::invalid_argument("page limit exceeds 100"));
        }
        value::text(&request.after_artifact_id)?;
        let epoch = self.resources.runtime.process_epoch().to_string();
        let records = self.resources.runtime.artifacts().await.map_err(inventory_failure)?;
        let start = if request.after_artifact_id.is_empty() {
            0
        } else {
            records
                .iter()
                .position(|record| record.artifact_id.as_str() == request.after_artifact_id)
                .map(|index| index + 1)
                .unwrap_or(records.len())
        };
        let limit = if request.limit == 0 { 50 } else { request.limit as usize };
        let count = records.len().saturating_sub(start).min(limit);
        let more = records.len().saturating_sub(start) > count;
        project(&records, request, &epoch, start, count, more, self.resources.maximum).map(Into::into)
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn inventory_failure(error: dokime_runtime::runtime::Error) -> Status {
    let status = match &error {
        dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted("artifact inventory exceeds its read budget")
        }
        dokime_runtime::runtime::Error::Admission(AdmissionFailure::Overload { .. }) => {
            Status::resource_exhausted("artifact inventory is busy")
        }
        dokime_runtime::runtime::Error::Admission(AdmissionFailure::Unavailable { .. })
        | dokime_runtime::runtime::Error::ShuttingDown
        | dokime_runtime::runtime::Error::Interrupted => Status::unavailable("artifact inventory is unavailable"),
        _ => Status::internal("artifact inventory could not be read"),
    };
    detailed(status, &error)
}

fn project(
    records: &[ArtifactRecord], request: &wire::ListArtifacts, epoch: &str, start: usize, count: usize, more: bool, maximum: usize,
) -> Result<dokime_domain::model::content::wire::ListArtifactsReply, Status> {
    let mut selected = value::canonical::slots(count)?;
    let mut digest = Sha256::new();
    for (index, record) in records.iter().enumerate() {
        let artifact = metadata(record)?;
        let encoded = artifact.wire_bytes()?;
        let mut prefix = Vec::with_capacity(10);
        prost::encoding::encode_varint(encoded.len() as u64, &mut prefix);
        digest.update(&prefix);
        digest.update(&encoded);
        if (start..start + count).contains(&index) {
            selected.push(artifact);
        }
    }
    let fingerprint = digest.finalize().to_vec();
    if request.cursor.as_ref().is_some_and(|cursor| cursor.epoch != epoch) {
        return Err(Status::failed_precondition("artifact cursor epoch mismatch; resynchronize without cursor"));
    }

    if request.cursor.as_ref().is_some_and(|cursor| cursor.metadata_sha256 != fingerprint) {
        return Err(Status::aborted("artifact listing changed; restart pagination"));
    }
    if !request.after_artifact_id.is_empty()
        && start == records.len()
        && !records.iter().any(|record| record.artifact_id.as_str() == request.after_artifact_id)
    {
        return Err(Status::invalid_argument("after artifact ID not found"));
    }
    let next_after_artifact_id = if more {
        Some(selected.last().ok_or_else(|| Status::internal("empty continued artifact page"))?.artifact_id.clone())
    } else {
        None
    };
    let reply = dokime_domain::model::content::wire::ListArtifactsReply {
        artifacts: selected,
        next_after_artifact_id,
        cursor: Some(dokime_domain::model::content::wire::ArtifactCursor { epoch: epoch.to_owned(), metadata_sha256: fingerprint }),
    };
    if reply.wire_len()? > maximum {
        return Err(Status::resource_exhausted("artifact page exceeds configured message limit"));
    }
    Ok(reply)
}

/* === Tests === */
