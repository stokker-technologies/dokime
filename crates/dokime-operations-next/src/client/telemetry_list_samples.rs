/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.telemetry.list_samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::TelemetryListSamplesRequest, v1::TelemetrySamplesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::TelemetryListSamplesRequest) -> Result<v1::TelemetrySamplesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        if !request.schema_versions.is_empty()
            || !request.sources.is_empty()
            || !request.signals.is_empty()
            || !request.after_stream_cursors.is_empty()
            || request.start.is_some()
            || request.end.is_some()
        {
            return Err(Error::Invalid(
                "ListSamples Schema, Source, Signal, Cursor and Time Filters Are Not Yet Served by the New Host".into(),
            ));
        }
        let identifier = super::projection::execution_scope(request.scope.as_ref())?;
        let snapshot = context.host.store().read(identifier)?;
        super::projection::check_scope(request.scope.as_ref().unwrap(), &snapshot)?;
        let mut samples =
            super::projection::samples(&snapshot, &request.streams, request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()))?;
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        if request.latest {
            samples = samples.split_off(samples.len().saturating_sub(limit));
        } else {
            super::projection::bounded(samples.len(), limit)?;
        }
        Ok(v1::TelemetrySamplesReply {
            samples,
            complete: true,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
