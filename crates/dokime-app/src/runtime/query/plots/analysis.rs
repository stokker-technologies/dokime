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

// Local Imports
use crate::plot::{
    PlotComputation, PlotComputedSeries, PlotEvidence, PlotPayload, PlotPayloadCursor,
    PlotPayloadRequest,
};
use crate::runtime::handle::RuntimeHandle;
use dokime::analysis::{Definition, MAX_SAMPLES, Request, Samples, Selection};
use dokime::core::Identifier;
use dokime::domains::ActionDispatcher;
use dokime::telemetry::{TelemetryContract, TelemetrySample};
use harmos::Message;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Prepared {
    definition: Definition,
    selection: Selection,
    target: Target,
}

/* === Enums === */
enum Target {
    Builtin,
    Plugin {
        source: Identifier,
        dispatcher: Arc<dyn ActionDispatcher>,
    },
}

/* === Traits === */

/* === Implementations === */
impl Prepared {
    pub(super) fn resolve(
        runtime: &RuntimeHandle,
        state: &crate::runtime::state::RuntimeState,
        request: &PlotPayloadRequest,
    ) -> crate::Result<Option<Self>> {
        let Some(selection) = &request.analysis else {
            return Ok(None);
        };
        if request.y_keys.is_empty()
            || request.y_keys.len() > 32
            || request.series_by.is_some()
            || request.series_by_expression.is_some()
            || request.x_expression.is_some()
            || !request.y_expressions.is_empty()
            || request.z_expression.is_some()
            || request.z_field.is_some()
            || request.z_stream.is_some()
            || !request.filters.is_empty()
        {
            return Err(invalid(
                "analysis requires one to 32 raw telemetry fields, without derived expressions or grouping",
            ));
        }
        let builtin = crate::plot::builtins::definitions()
            .map_err(|error| invalid(error.to_string()))?
            .into_iter()
            .find(|definition| definition.id == selection.id);
        let (definition, target) = if let Some(definition) = builtin {
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
                .ok_or_else(|| {
                    unavailable("selected analysis is unavailable in this build or installation")
                })?;
            if !runtime.plugin_available(&artifact.source) {
                return Err(unavailable("analysis owner is unavailable"));
            }
            let source =
                Identifier::new(&artifact.source).map_err(|error| invalid(error.to_string()))?;
            let dispatcher = runtime
                .action_dispatcher(&source)
                .ok_or_else(|| unavailable("analysis owner is not admitted"))?;
            (definition, Target::Plugin { source, dispatcher })
        };
        let parameters = definition.resolve(selection).map_err(invalid)?;
        Ok(Some(Self {
            definition,
            selection: Selection {
                parameters,
                ..selection.clone()
            },
            target,
        }))
    }

    pub(super) fn check_evidence(
        &self,
        telemetry: &[TelemetrySample],
        request: &PlotPayloadRequest,
    ) -> crate::Result<()> {
        let samples = telemetry
            .iter()
            .filter(|sample| {
                sample.stream.as_str() == request.stream
                    && crate::plot::sample_matches_scope(
                        sample.scope.as_ref(),
                        request.scope.as_ref(),
                    )
            })
            .collect::<Vec<_>>();
        if samples.is_empty() {
            return Err(invalid("analysis has no retained raw samples"));
        }
        let first = samples[0];
        if samples.iter().any(|sample| {
            evidence_owner(sample.scope.as_ref()) != evidence_owner(first.scope.as_ref())
        }) {
            return Err(invalid(
                "analysis requires one evidence scope; narrow the selection",
            ));
        }
        let window_start = if let Some(window) = request.window_s {
            if !window.is_finite() || window <= 0.0 {
                return Err(invalid("analysis window must be finite and positive"));
            }
            let last = crate::plot::parse_timestamp(samples.last().unwrap().timestamp.as_str())
                .ok_or_else(|| invalid("analysis requires valid sample timestamps"))?;
            Some((last, window))
        } else {
            None
        };
        if first.sequence > 1 {
            let complete = window_start.is_some_and(|(last, window)| {
                crate::plot::parse_timestamp(first.timestamp.as_str())
                    .is_some_and(|first| (last - first).as_seconds_f64() >= window)
            });
            if !complete {
                return Err(invalid(
                    "requested analysis window exceeds retained raw evidence",
                ));
            }
        }
        let selected = samples
            .into_iter()
            .filter(|sample| {
                window_start.is_none_or(|(last, window)| {
                    crate::plot::parse_timestamp(sample.timestamp.as_str())
                        .is_some_and(|time| (last - time).as_seconds_f64() <= window)
                })
            })
            .collect::<Vec<_>>();
        if selected
            .windows(2)
            .any(|pair| pair[1].sequence != pair[0].sequence.saturating_add(1))
        {
            return Err(invalid("analysis source contains missing samples"));
        }
        if selected.len().saturating_mul(request.y_keys.len()) > MAX_SAMPLES as usize {
            return Err(invalid(
                "analysis input exceeds the aggregate sample budget",
            ));
        }
        Ok(())
    }

    pub(super) async fn execute(
        self,
        runtime: &RuntimeHandle,
        request: &PlotPayloadRequest,
        cursor: &PlotPayloadCursor,
        mut payload: PlotPayload,
        contract: Option<&TelemetryContract>,
    ) -> crate::Result<PlotPayload> {
        let total: usize = payload
            .series
            .iter()
            .map(|series| series.points.len())
            .sum();
        if total > MAX_SAMPLES as usize {
            return Err(invalid(
                "analysis input exceeds the aggregate sample budget",
            ));
        }
        let mut computed = Vec::with_capacity(payload.series.len());
        let mut output_count = 0usize;
        for series in &payload.series {
            let origin = series
                .points
                .first()
                .and_then(|point| point.x.as_str())
                .map(str::to_owned);
            let origin_time = origin.as_deref().and_then(crate::plot::parse_timestamp);
            let axis = series
                .points
                .iter()
                .map(|point| {
                    if let Some(origin) = origin_time {
                        point
                            .x
                            .as_str()
                            .and_then(crate::plot::parse_timestamp)
                            .map(|time| (time - origin).as_seconds_f64())
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
                    .and_then(|contract| {
                        contract
                            .columns
                            .iter()
                            .find(|column| column.metadata.identifier.as_str() == key)
                    })
                    .and_then(|column| {
                        column
                            .display_unit
                            .clone()
                            .or_else(|| column.unit.as_ref().map(|unit| unit.symbol.clone()))
                    })
                    .unwrap_or_default()
            };
            let samples = Samples {
                axis,
                values,
                axis_unit: if origin.is_some() {
                    "s".into()
                } else {
                    column_unit(&payload.x_key)
                },
                unit: series
                    .unit
                    .clone()
                    .unwrap_or_else(|| column_unit(&series.key)),
            };
            samples
                .check(self.definition.limits)
                .map_err(|error| invalid(error.to_string()))?;
            let count = samples.values.len();
            let first_axis = samples.axis[0];
            let last_axis = *samples.axis.last().unwrap();
            let call = Request {
                samples: Some(samples),
                parameters: serde_json::to_vec(&self.selection.parameters)
                    .map_err(|error| invalid(error.to_string()))?,
            };
            let output = match &self.target {
                Target::Builtin => {
                    let bytes = runtime
                        .operations
                        .call_encoded(&self.definition.id, call.encode_to_vec())
                        .await?;
                    dokime::analysis::Response::decode(bytes.as_slice())
                        .map_err(|error| invalid(error.to_string()))?
                }
                Target::Plugin { source, dispatcher } => {
                    runtime
                        .analyze_with_dispatcher(source, dispatcher, &self.definition.id, call)
                        .await?
                }
            };
            output
                .check(&self.definition)
                .map_err(|error| invalid(error.to_string()))?;
            output_count += output
                .outputs
                .iter()
                .map(|output| output.values.len())
                .sum::<usize>();
            if output_count > MAX_SAMPLES as usize {
                return Err(invalid(
                    "analysis output exceeds the aggregate value budget",
                ));
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
                    _ => harmos_signal::lttb(&output.axis, &output.values, budget),
                };
                display_indices.insert(
                    output.key.clone(),
                    indices.into_iter().map(|index| index as u32).collect(),
                );
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
        payload.computed = Some(PlotComputation {
            definition: self.definition,
            selection: self.selection,
            source: PlotEvidence {
                epoch: runtime.process_epoch().to_string(),
                stream: request.stream.clone(),
                scope: request.scope.clone(),
                cursor: cursor.clone(),
                window_s: request.window_s,
                fields: request.y_keys.clone(),
                x_key: payload.x_key.clone(),
            },
            series: computed,
        });
        payload.series.clear();
        payload.analysis = None;
        payload.cartesian = None;
        Ok(payload)
    }
}

/* === Public Functions === */
pub(crate) fn definitions(
    runtime: &RuntimeHandle,
    state: &crate::queries::RuntimeStateSnapshot,
) -> crate::Result<Vec<Definition>> {
    let mut definitions =
        crate::plot::builtins::definitions().map_err(|error| invalid(error.to_string()))?;
    for artifact in state.state().installed_plugins.values() {
        let source =
            Identifier::new(&artifact.source).map_err(|error| invalid(error.to_string()))?;
        if runtime.plugin_available(&artifact.source)
            && runtime.action_dispatcher(&source).is_some()
        {
            definitions.extend(artifact.analyses.clone());
        }
    }
    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(definitions)
}

/* === Private Functions === */
// Step And Criterion Attribution Do Not Start A New Telemetry Sequence.
fn evidence_owner(
    scope: Option<&dokime::core::RecordScope>,
) -> (
    Option<&dokime::core::TestCycleId>,
    Option<&dokime::core::TestExecutionId>,
) {
    (
        scope.and_then(|scope| scope.test_cycle_id.as_ref()),
        scope.and_then(|scope| scope.test_execution_id.as_ref()),
    )
}

fn invalid(message: impl Into<String>) -> crate::Error {
    crate::Error::Validation {
        code: "analysis_input".into(),
        message: message.into(),
        issues: vec![],
    }
}
fn unavailable(message: impl Into<String>) -> crate::Error {
    crate::Error::Unavailable {
        code: "analysis_unavailable".into(),
        message: message.into(),
    }
}

/* === Tests === */
