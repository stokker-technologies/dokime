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
use harmos::Message;

// Local Imports
use crate::analysis::{Definition, Request, Samples, Selection};
use crate::domains::ActionDispatcher;
use crate::plot::{PlotComputation, PlotComputedSeries, PlotEvidence, PlotPayload, PlotPayloadRequest};
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::{TelemetryContract, TelemetrySample};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Prepared {
    definition: Definition,
    selection: Selection,
    target: Target,
}

/* === Enums === */

enum Target {
    Builtin,
    Plugin { source: Identifier, dispatcher: Arc<dyn ActionDispatcher> },
}

pub(crate) enum Environment<'a> {
    Live(&'a RuntimeHandle),
    Report(&'a crate::report::analysis::Resources),
}

/* === Traits === */

/* === Implementations === */

impl Prepared {
    pub(super) fn resolve(
        runtime: &RuntimeHandle, state: &crate::runtime::state::RuntimeState, request: &PlotPayloadRequest,
    ) -> crate::runtime::Result<Option<Self>> {
        Self::resolve_with(&crate::report::analysis::Resources::from_runtime(runtime), state, request)
    }

    pub(crate) fn resolve_with(
        resources: &crate::report::analysis::Resources, state: &crate::runtime::state::RuntimeState, request: &PlotPayloadRequest,
    ) -> crate::runtime::Result<Option<Self>> {
        let Some(selection) = &request.analysis else {
            return Ok(None);
        };
        if request.y_keys.is_empty()
            || request.y_keys.len() as u64 > dokime_domain::model::analysis::MAX_OUTPUTS
            || request.series_by.is_some()
            || request.series_by_expression.is_some()
            || request.x_expression.is_some()
            || !request.y_expressions.is_empty()
            || request.z_expression.is_some()
            || request.z_field.is_some()
            || request.z_stream.is_some()
            || !request.filters.is_empty()
        {
            return Err(invalid("analysis requires one to 32 raw telemetry fields, without derived expressions or grouping"));
        }
        let builtin = crate::plot::builtins::definitions()
            .map_err(|error| invalid(error.to_string()))?
            .into_iter()
            .find(|definition| definition.id == selection.id);
        let (mut definition, target) = if let Some(definition) = builtin {
            (definition, Target::Builtin)
        } else {
            let (artifact, definition) = state
                .installed_plugins
                .values()
                .find_map(|artifact| {
                    artifact
                        .analyses
                        .iter()
                        .find(|definition| definition.id == selection.id)
                        .map(|definition| (artifact, definition.clone()))
                })
                .ok_or_else(|| unavailable("selected analysis is unavailable in this build or installation"))?;
            if !resources.available(&artifact.source) {
                return Err(unavailable("analysis owner is unavailable"));
            }
            let source = Identifier::new(&artifact.source).map_err(|error| invalid(error.to_string()))?;
            let dispatcher = resources.dispatcher(&source).ok_or_else(|| unavailable("analysis owner is not admitted"))?;
            (definition, Target::Plugin { source, dispatcher })
        };
        definition.check_limits().map_err(invalid)?;
        definition.check_selection_limits(selection).map_err(invalid)?;
        definition.limits = definition.limits.constrained();
        let parameters = definition.resolve(selection).map_err(invalid)?;
        Ok(Some(Self {
            definition,
            selection: Selection { parameters, ..selection.clone() },
            target,
        }))
    }

    pub(crate) fn check_evidence(&self, telemetry: &[TelemetrySample], request: &PlotPayloadRequest) -> crate::runtime::Result<()> {
        let samples = telemetry
            .iter()
            .filter(|sample| sample.stream.as_str() == request.stream && crate::plot::sample_matches_scope(sample, request.scope.as_ref()));
        let first = samples.clone().next().ok_or_else(|| invalid("analysis has no retained raw samples"))?;
        if samples.clone().any(|sample| evidence_owner(sample.scope.as_ref()) != evidence_owner(first.scope.as_ref())) {
            return Err(invalid("analysis requires one evidence scope; narrow the selection"));
        }
        let window_start = if let Some(window) = request.window_s {
            if !window.is_finite() || window <= 0.0 {
                return Err(invalid("analysis window must be finite and positive"));
            }
            let last = samples.clone().next_back().expect("nonempty matching evidence");
            let last = crate::plot::parse_timestamp(last.timestamp.as_str())
                .ok_or_else(|| invalid("analysis requires valid sample timestamps"))?;
            Some((last, window))
        } else {
            None
        };
        let selected = samples
            .filter(|sample| {
                window_start.is_none_or(|(last, window)| {
                    crate::plot::parse_timestamp(sample.timestamp.as_str()).is_some_and(|time| (last - time).as_seconds_f64() <= window)
                })
            })
            .count();
        let values = selected.checked_mul(request.y_keys.len()).ok_or_else(|| invalid("analysis input sample count overflows"))?;
        if values > crate::analysis::MAX_SAMPLES as usize {
            return Err(invalid("analysis input exceeds the aggregate sample budget"));
        }
        Ok(())
    }

    pub(super) async fn execute(
        self, runtime: &RuntimeHandle, request: &PlotPayloadRequest, position: &harmos::storage::StorageCursor,
        payload: harmos::storage::ReadBatch<PlotPayload>, contract: Option<&TelemetryContract>,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<PlotPayload>> {
        self.execute_in(Environment::Live(runtime), request, position, payload, contract).await
    }

    pub(crate) async fn execute_in(
        self, environment: Environment<'_>, request: &PlotPayloadRequest, position: &harmos::storage::StorageCursor,
        payload: harmos::storage::ReadBatch<PlotPayload>, contract: Option<&TelemetryContract>,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<PlotPayload>> {
        let total: usize = payload.series.iter().map(|series| series.points.len()).sum();
        if total > crate::analysis::MAX_SAMPLES as usize {
            return Err(invalid("analysis input exceeds the aggregate sample budget"));
        }
        let mut computed = Vec::with_capacity(payload.series.len());
        let mut output_count = 0usize;
        for series in &payload.series {
            let origin = series.points.first().and_then(|point| point.x.as_str()).map(str::to_owned);
            let origin_time = origin.as_deref().and_then(crate::plot::parse_timestamp);
            let axis = series
                .points
                .iter()
                .map(|point| {
                    if let Some(origin) = origin_time {
                        point.x.as_str().and_then(crate::plot::parse_timestamp).map(|time| (time - origin).as_seconds_f64())
                    } else {
                        point.x.as_f64()
                    }
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| invalid("analysis requires a numeric or timestamp axis"))?;
            let values = series
                .points
                .iter()
                .map(|point| point.y)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| invalid("analysis cannot omit missing values"))?;
            let column_unit = |key: &str| {
                contract
                    .and_then(|contract| contract.columns.iter().find(|column| column.metadata.identifier.as_str() == key))
                    .and_then(|column| column.display_unit.clone().or_else(|| column.unit.as_ref().map(|unit| unit.symbol.clone())))
                    .unwrap_or_default()
            };
            let samples = Samples {
                axis,
                values,
                axis_unit: if origin.is_some() { "s".into() } else { column_unit(&payload.x_key) },
                unit: series.unit.clone().unwrap_or_else(|| column_unit(&series.key)),
            };
            samples.check(self.definition.limits).map_err(|error| invalid(error.to_string()))?;
            let count = samples.values.len();
            let first_axis = *samples.axis.first().ok_or_else(|| invalid("analysis series contains no finite points"))?;
            let last_axis = *samples.axis.last().unwrap();
            let call = Request {
                samples: Some(samples),
                parameters: serde_json::to_vec(&self.selection.parameters).map_err(|error| invalid(error.to_string()))?,
            };
            let output = environment.invoke(&self.target, &self.definition.id, call).await?;
            output.check(&self.definition).map_err(|error| invalid(error.to_string()))?;
            output.check_limits().map_err(|error| invalid(error.to_string()))?;
            output_count += output.outputs.iter().map(|output| output.values.len()).sum::<usize>();
            if output_count > crate::analysis::MAX_SAMPLES as usize {
                return Err(invalid("analysis output exceeds the aggregate value budget"));
            }
            let mut display_indices = std::collections::BTreeMap::new();
            for output in &output.outputs {
                let budget = payload.max_points;
                if output.axis.is_empty() || budget == 0 || output.values.len() <= budget {
                    continue;
                }
                let indices = match budget {
                    1 => vec![0],
                    2 => vec![0, output.values.len() - 1],
                    _ => harmos::signal::lttb(&output.axis, &output.values, budget),
                };
                display_indices.insert(output.key.clone(), indices.into_iter().map(|index| index as u32).collect());
            }
            computed.push(PlotComputedSeries {
                key: series.key.clone(),
                label: series.label.clone(),
                origin,
                input_samples: count,
                first_axis,
                last_axis,
                output,
                display_indices,
            });
        }
        let computed = PlotComputation {
            definition: self.definition,
            selection: self.selection,
            source: PlotEvidence {
                stream: request.stream.clone(),
                scope: request.scope.clone(),
                position: position.clone(),
                window_s: request.window_s,
                fields: request.y_keys.clone(),
                x_key: payload.x_key.clone(),
            },
            series: computed,
        };
        // Dispatcher/Builtin Result Construction Still Requires Its Own Admission.
        // This Projection Only Moves The Completed Result And Existing Payload.
        payload.try_map(|mut payload| {
            payload.computed = Some(computed);
            payload.series.clear();
            payload.analysis = None;
            payload.cartesian = None;
            Ok(payload)
        })
    }
}

impl Environment<'_> {
    async fn invoke(&self, target: &Target, operation: &str, request: Request) -> crate::runtime::Result<crate::analysis::Response> {
        match (self, target) {
            (Self::Report(_), Target::Builtin) => {
                let operation = operation.to_owned();
                tokio::task::spawn_blocking(move || crate::plot::builtins::evaluate(&operation, request))
                    .await?
                    .map_err(|error| invalid(error.to_string()))
            }
            (Self::Report(resources), Target::Plugin { source, dispatcher }) => {
                let current = resources
                    .dispatcher(source)
                    .filter(|current| Arc::ptr_eq(current, dispatcher))
                    .ok_or_else(|| unavailable("analysis owner changed while its preview was selected"))?;
                current.analyze(operation, request).await.map_err(|error| invalid(error.to_string()))
            }
            // The runtime serves its built-in analyses as the host.
            (Self::Live(runtime), Target::Builtin) => {
                let bytes = runtime.operations.call_encoded(&format!("host.{operation}"), request.encode()).await?;
                crate::analysis::Response::decode(&bytes).map_err(|error| invalid(error.to_string()))
            }
            (Self::Live(runtime), Target::Plugin { source, dispatcher }) => {
                runtime.analyze_with_dispatcher(source, dispatcher, operation, request).await
            }
        }
    }
}

/* === Public Functions === */

pub(crate) fn definitions(
    runtime: &RuntimeHandle, state: &crate::queries::RuntimeStateSnapshot,
) -> crate::runtime::Result<Vec<Definition>> {
    let mut definitions = crate::plot::builtins::definitions().map_err(|error| invalid(error.to_string()))?;
    for artifact in state.state().installed_plugins.values() {
        let source = Identifier::new(&artifact.source).map_err(|error| invalid(error.to_string()))?;
        if runtime.plugin_available(&artifact.source) && runtime.action_dispatcher(&source).is_some() {
            definitions.extend(artifact.analyses.clone());
        }
    }
    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(definitions)
}

/* === Private Functions === */

// Step And Criterion Attribution Do Not Start A New Telemetry Sequence.
fn evidence_owner(
    scope: Option<&dokime_domain::model::execution::RecordScope>,
) -> (Option<&dokime_domain::model::identity::TestCycleId>, Option<&dokime_domain::model::identity::TestExecutionId>) {
    (scope.and_then(|scope| scope.test_cycle_id.as_ref()), scope.and_then(|scope| scope.test_execution_id.as_ref()))
}

fn invalid(message: impl Into<String>) -> crate::runtime::Error {
    crate::runtime::Error::Admission(AdmissionFailure::Validation {
        code: "analysis_input".into(),
        message: message.into(),
        issues: vec![],
    })
}

fn unavailable(message: impl Into<String>) -> crate::runtime::Error {
    crate::runtime::Error::Admission(AdmissionFailure::Unavailable {
        code: "analysis_unavailable".into(),
        message: message.into(),
    })
}

/* === Tests === */
