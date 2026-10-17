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
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};
// Third Party Imports
use harmos::operations::Context;
// Local Imports
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use crate::{
    runtime::Error, runtime::Result, RuntimeHandle,
    plot::{PlotPayload, PlotPayloadCursor, PlotPayloadRequest},
};
use dokime_domain::model::identity::Identifier;
use crate::domains::ActionDispatcher;
use crate::telemetry::TelemetryContract;
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub(crate) struct Resources {
    pub(crate) dispatchers: Arc<RwLock<BTreeMap<Identifier, Arc<dyn ActionDispatcher>>>>,
    pub(crate) unavailable: Arc<RwLock<BTreeSet<String>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Resources {
    pub(crate) fn new() -> Self {
        Self {
            dispatchers: Arc::new(RwLock::new(BTreeMap::new())),
            unavailable: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    pub(crate) fn from_runtime(runtime: &RuntimeHandle) -> Self {
        Self {
            dispatchers: runtime.action_dispatchers.clone(),
            unavailable: runtime.unavailable_plugins.clone(),
        }
    }
    pub(crate) fn available(&self, source: &str) -> bool {
        !self
            .unavailable
            .read()
            .expect("plugin availability lock poisoned")
            .contains(source)
    }
    pub(crate) fn dispatcher(&self, source: &Identifier) -> Option<Arc<dyn ActionDispatcher>> {
        self.dispatchers
            .read()
            .expect("analysis dispatcher lock poisoned")
            .get(source)
            .cloned()
    }
    pub(crate) async fn invoke(
        &self,
        source: &Identifier,
        expected: &Arc<dyn ActionDispatcher>,
        operation: &str,
        request: crate::analysis::Request,
    ) -> Result<crate::analysis::Response> {
        let invocation = {
            let dispatchers = self
                .dispatchers
                .read()
                .expect("analysis dispatcher lock poisoned");
            if !self.available(source.as_str())
                || !dispatchers
                    .get(source)
                    .is_some_and(|current| Arc::ptr_eq(current, expected))
            {
                return Err(Error::Admission(AdmissionFailure::Unavailable {
                    code: "analysis_owner_changed".into(),
                    message: "analysis owner changed while its evidence was being selected".into(),
                }));
            }
            expected.analyze(operation, request)
        };
        invocation
            .await
            .map_err(|error| Error::Operation(error.to_string()))
    }
}

/* === Public Functions === */
pub(crate) async fn materialize(
    ctx: &Context<'_>,
    request: &PlotPayloadRequest,
    telemetry: &[TelemetrySample],
    payload: PlotPayload,
    contract: Option<&TelemetryContract>,
) -> Result<PlotPayload> {
    let resources = &ctx.scope().resources::<crate::app::Dokime>().analysis;
    let state = ctx.scope().state::<crate::app::Dokime>().state();
    let prepared = state
        .read(|state| {
            crate::runtime::query::plots::analysis::Prepared::resolve_with(
                resources, state, request,
            )
            .map(|prepared| (prepared, state.process_epoch().to_string()))
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
        .value?;
    let (Some(prepared), epoch) = prepared else {
        return Ok(payload);
    };
    prepared.check_evidence(telemetry, request)?;
    let mut cursor = PlotPayloadCursor::default();
    for sample in telemetry.iter().filter(|sample| {
        sample.stream.as_str() == request.stream
            && crate::plot::sample_matches_scope(sample.scope.as_ref(), request.scope.as_ref())
    }) {
        let stream =
            crate::domains::telemetry::telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
        let ordinal = cursor.telemetry_streams.entry(stream).or_default();
        *ordinal = (*ordinal).max(sample.sequence);
    }
    prepared
        .execute_in(
            crate::runtime::query::plots::analysis::Environment::Report {
                ctx,
                resources,
                epoch: &epoch,
            },
            request,
            &cursor,
            payload,
            contract,
        )
        .await
}

/* === Private Functions === */

/* === Tests === */
