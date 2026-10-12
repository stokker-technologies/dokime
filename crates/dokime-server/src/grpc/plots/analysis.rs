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
use super::super::{
    catalog,
    value::{self, Budget},
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn selection(
    value: v1::AnalysisSelection,
) -> Result<dokime::analysis::Selection, Status> {
    value::text(&value.id)?;
    if value.id.is_empty() || value.version == 0 {
        return Err(Status::invalid_argument(
            "analysis requires an ID and positive version",
        ));
    }
    let parameters = value::decode_object(value.parameters)?;
    let selection = dokime::analysis::Selection {
        id: value.id,
        version: value.version,
        parameters,
    };
    if serde_json::to_vec(&selection.parameters)
        .map_err(|error| Status::invalid_argument(error.to_string()))?
        .len()
        > dokime::analysis::MAX_PARAMETER_BYTES
    {
        return Err(Status::resource_exhausted(
            "analysis configuration exceeds its budget",
        ));
    }
    Ok(selection)
}

pub(crate) fn selection_wire(
    selection: &dokime::analysis::Selection,
) -> Result<v1::AnalysisSelection, Status> {
    Ok(v1::AnalysisSelection {
        id: selection.id.clone(),
        version: selection.version,
        parameters: Some(super::wire::values(&selection.parameters)?),
    })
}

pub(super) fn definition(
    definition: &dokime::analysis::Definition,
) -> Result<v1::AnalysisDefinition, Status> {
    definition.check().map_err(Status::internal)?;
    let mut budget = Budget::new();
    Ok(v1::AnalysisDefinition {
        id: definition.id.clone(),
        version: definition.version,
        label: definition.label.clone(),
        description: definition.description.clone(),
        parameters: catalog::parameters(&definition.parameters, &mut budget)?,
        outputs: definition
            .outputs
            .iter()
            .map(|output| v1::AnalysisOutput {
                key: output.key.clone(),
                label: output.label.clone(),
                unit: output.unit.clone(),
                axis: output.axis.as_ref().map(|axis| v1::AnalysisAxis {
                    key: axis.key.clone(),
                    unit: axis.unit.clone(),
                }),
            })
            .collect(),
        max_samples: definition.limits.max_samples,
        max_output_values: definition.limits.max_output_values,
    })
}

pub(super) fn computation(
    computed: dokime_app::runtime::query::PlotComputation,
) -> Result<v1::AnalysisComputation, Status> {
    Ok(v1::AnalysisComputation {
        definition: Some(definition(&computed.definition)?),
        selection: Some(selection_wire(&computed.selection)?),
        source: Some(v1::AnalysisEvidence {
            epoch: computed.source.epoch,
            stream: computed.source.stream,
            scope: computed.source.scope.as_ref().map(value::scope),
            telemetry_streams: computed
                .source
                .cursor
                .telemetry_streams
                .into_iter()
                .collect(),
            record_sequence: computed.source.cursor.record_sequence,
            window_s: computed.source.window_s,
            fields: computed.source.fields,
            x_key: computed.source.x_key,
        }),
        series: computed
            .series
            .into_iter()
            .map(|series| v1::AnalysisSeries {
                key: series.key,
                label: series.label,
                origin: series.origin,
                input_samples: series.input_samples as u64,
                first_axis: series.first_axis,
                last_axis: series.last_axis,
                outputs: series
                    .output
                    .outputs
                    .into_iter()
                    .map(|output| v1::AnalysisValues {
                        display_indices: series
                            .display_indices
                            .get(&output.key)
                            .cloned()
                            .unwrap_or_default(),
                        key: output.key,
                        unit: output.unit,
                        axis_unit: output.axis_unit,
                        axis: output.axis,
                        values: output.values,
                    })
                    .collect(),
            })
            .collect(),
    })
}

/* === Private Functions === */

/* === Tests === */
