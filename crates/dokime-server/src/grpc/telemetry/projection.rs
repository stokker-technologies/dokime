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
use crate::grpc::value::{Budget, canonical};
use dokime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn sample(sample: &TelemetrySample) -> Result<dokime::domain::model::telemetry::wire::TelemetrySample, Status> {
    Ok(dokime::domain::model::telemetry::wire::TelemetrySample {
        sample_id: sample.sample_id.as_str().to_owned(),
        stream: sample.stream.as_str().to_owned(),
        schema_version: sample.schema_version,
        sequence: sample.sequence,
        timestamp: Some(canonical::timestamp(&sample.timestamp)?),
        received_at: Some(canonical::timestamp(&sample.received_at)?),
        source: sample.source.clone(),
        scope: sample.scope.as_ref().map(canonical::scope).transpose()?,
        values: Some(canonical::object(&sample.values, &mut Budget::new()?)?),
        summary: Some(canonical::object(&sample.summary, &mut Budget::new()?)?),
        metadata: Some(canonical::object(&sample.metadata, &mut Budget::new()?)?),
    })
}

/* === Private Functions === */

/* === Tests === */
