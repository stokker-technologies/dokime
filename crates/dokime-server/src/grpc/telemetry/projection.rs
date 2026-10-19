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

// Third Party Imports
use tonic::Status;

// Local Imports
use crate::grpc::value::{self, Budget, canonical};
use dokime_runtime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn memory(sample: &TelemetrySample) -> Result<usize, Status> {
    let mut bytes = std::mem::size_of::<dokime_domain::model::telemetry::wire::TelemetrySample>();
    for text in [sample.sample_id.as_str(), sample.stream.as_str(), &sample.source] {
        value::text(text)?;
        bytes = bytes.saturating_add(text.len());
    }
    for fields in [&sample.values, &sample.summary, &sample.metadata] {
        bytes = bytes.saturating_add(canonical::object_memory(fields, &mut Budget::new())?);
    }
    if let Some(scope) = &sample.scope {
        bytes = bytes.saturating_add(canonical::scope_memory(scope)?);
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted("sample construction size overflow"));
    }
    Ok(bytes)
}

pub(in crate::grpc) fn sample(sample: &TelemetrySample) -> Result<dokime_domain::model::telemetry::wire::TelemetrySample, Status> {
    memory(sample)?;
    Ok(dokime_domain::model::telemetry::wire::TelemetrySample {
        sample_id: sample.sample_id.to_string(),
        stream: sample.stream.to_string(),
        schema_version: sample.schema_version,
        sequence: sample.sequence,
        timestamp: Some(canonical::timestamp(&sample.timestamp)?),
        received_at: Some(canonical::timestamp(&sample.received_at)?),
        source: sample.source.clone(),
        scope: sample.scope.as_ref().map(canonical::scope).transpose()?,
        values: Some(canonical::object(&sample.values, &mut Budget::new())?),
        summary: Some(canonical::object(&sample.summary, &mut Budget::new())?),
        metadata: Some(canonical::object(&sample.metadata, &mut Budget::new())?),
    })
}

/* === Private Functions === */

/* === Tests === */
