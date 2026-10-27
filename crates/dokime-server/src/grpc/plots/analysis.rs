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
    value::{Budget, canonical},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn selection(value: dokime::domain::model::plots::wire::AnalysisSelection) -> Result<dokime::analysis::Selection, Status> {
    if value.id.is_empty() || value.version == 0 {
        return Err(Status::invalid_argument("analysis requires an ID and positive version"));
    }
    let parameters = match value.parameters.map(|value| canonical::decode(value, &mut Budget::new()?, 0)).transpose()? {
        None => Default::default(),
        Some(dokime::domain::model::values::Value::Object(fields)) => fields,
        Some(_) => {
            return Err(Status::invalid_argument("analysis parameters must be an object"));
        }
    };
    let selection = dokime::analysis::Selection { id: value.id, version: value.version, parameters };
    Ok(selection)
}

pub(crate) fn selection_wire(
    selection: &dokime::analysis::Selection,
) -> Result<dokime::domain::model::plots::wire::AnalysisSelection, Status> {
    Ok(dokime::domain::model::plots::wire::AnalysisSelection {
        id: selection.id.clone(),
        version: selection.version,
        parameters: Some(super::wire::values(&selection.parameters)?),
    })
}

pub(super) fn definition(
    definition: &dokime::analysis::Definition,
) -> Result<dokime::domain::model::plots::wire::AnalysisDefinition, Status> {
    definition.check().map_err(Status::internal)?;
    definition_checked(definition)
}

fn definition_checked(definition: &dokime::analysis::Definition) -> Result<dokime::domain::model::plots::wire::AnalysisDefinition, Status> {
    let mut budget = Budget::new()?;
    Ok(dokime::domain::model::plots::wire::AnalysisDefinition {
        id: definition.id.clone(),
        version: definition.version,
        label: definition.label.clone(),
        description: definition.description.clone(),
        parameters: catalog::parameters(&definition.parameters, &mut budget)?,
        outputs: mapped(definition.outputs.iter(), |output| {
            Ok(dokime::domain::model::plots::wire::AnalysisOutput {
                key: output.key.clone(),
                label: output.label.clone(),
                unit: output.unit.clone(),
                axis: output
                    .axis
                    .as_ref()
                    .map(|axis| dokime::domain::model::plots::wire::AnalysisAxis { key: axis.key.clone(), unit: axis.unit.clone() }),
            })
        })?,
        max_samples: definition.limits.max_samples,
        max_output_values: definition.limits.max_output_values,
    })
}

pub(super) fn computation(
    computed: dokime::runtime::query::PlotComputation,
) -> Result<dokime::domain::model::plots::wire::AnalysisComputation, Status> {
    let series = mapped(computed.series.into_iter(), |mut series| {
        let outputs = mapped(series.output.outputs.into_iter(), |output| {
            Ok(dokime::domain::model::plots::wire::AnalysisValues {
                display_indices: series.display_indices.remove(&output.key).unwrap_or_default(),
                key: output.key,
                unit: output.unit,
                axis_unit: output.axis_unit,
                axis: output.axis,
                values: output.values,
            })
        })?;
        Ok(dokime::domain::model::plots::wire::AnalysisSeries {
            key: series.key,
            label: series.label,
            origin: series.origin,
            input_samples: series.input_samples as u64,
            first_axis: series.first_axis,
            last_axis: series.last_axis,
            outputs,
        })
    })?;
    Ok(dokime::domain::model::plots::wire::AnalysisComputation {
        definition: Some(definition_checked(&computed.definition)?),
        selection: Some(selection_wire(&computed.selection)?),
        source: Some(dokime::domain::model::plots::wire::AnalysisEvidence {
            store: computed.source.position.history.store.as_str().to_owned(),
            epoch: computed.source.position.history.epoch.as_str().to_owned(),
            position: computed.source.position.sequence,
            stream: computed.source.stream,
            scope: computed.source.scope.as_ref().map(canonical::scope).transpose()?,
            window_s: computed.source.window_s,
            fields: computed.source.fields,
            x_key: computed.source.x_key,
        }),
        series,
    })
}

/* === Private Functions === */

fn mapped<S, T>(values: impl ExactSizeIterator<Item = S>, mut map: impl FnMut(S) -> Result<T, Status>) -> Result<Vec<T>, Status> {
    let mut output = canonical::slots(values.len())?;
    for value in values {
        output.push(map(value)?);
    }
    Ok(output)
}

/* === Tests === */
