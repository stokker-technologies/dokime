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
use std::collections::BTreeMap;
// Third Party Imports
use tonic::Status;
// Local Imports
use super::rejection;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;
use dokime_runtime::definition::parameters::limits;
use dokime_runtime::definition::{ParameterDefinition, TestCase, TestCaseTemplate};

/* === Constants === */
// Conservative native allocation estimate, shared by expression intermediates
// and generated cases across the entire plan. This is not an RSS measurement.
pub(super) const BUDGET: usize = 8 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn limit() -> Status {
    rejection(
        "planning_payload_limit",
        "plan exceeds the conservative 8 MiB materialization budget",
    )
}

pub(super) fn charge(remaining: &mut usize, amount: usize) -> Result<(), Status> {
    limits::charge(remaining, amount).map_err(super::bounds::status)
}

pub(super) fn registered_case(case: &TestCase, remaining: &mut usize) -> Result<(), Status> {
    charge(remaining, 512)?;
    charge(remaining, case.suite_id.len().saturating_mul(8))?;
    retained_metadata(&case.metadata, remaining)?;
    if let Some(suite) = &case.suite {
        retained_metadata(suite, remaining)?;
    }
    for (name, value) in &case.parameters {
        charge(remaining, name.len().saturating_mul(8))?;
        retained_value(value, remaining, 0)?;
    }
    Ok(())
}

fn retained_metadata(metadata: &Metadata, remaining: &mut usize) -> Result<(), Status> {
    charge(remaining, metadata_cost(metadata))?;
    for tag in &metadata.tags {
        charge(remaining, tag.len().saturating_mul(8).saturating_add(256))?;
    }
    for (name, value) in &metadata.attributes {
        charge(remaining, name.len().saturating_mul(8))?;
        retained_value(value, remaining, 0)?;
    }
    Ok(())
}

fn retained_value(value: &Value, remaining: &mut usize, depth: usize) -> Result<(), Status> {
    if depth > 32 {
        return Err(limit());
    }
    charge(remaining, 256)?;
    match value {
        Value::Text(text) => charge(remaining, text.len().saturating_mul(8))?,
        Value::Array(values) => {
            for value in values {
                retained_value(value, remaining, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                charge(remaining, name.len().saturating_mul(8))?;
                retained_value(value, remaining, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn case_cost(
    template: &TestCaseTemplate,
    suite: &Metadata,
    suite_id: &str,
    parameters: &BTreeMap<String, Vec<Value>>,
    files: &BTreeMap<String, Option<String>>,
    remaining: &mut usize,
) -> Result<usize, Status> {
    let mut cost = metadata_cost(&template.metadata)
        .saturating_add(metadata_cost(suite))
        .saturating_add(suite_id.len().saturating_mul(8))
        .saturating_add(512);
    for definition in &template.parameters {
        let shape = parameter_shape(definition, &template.parameters, parameters, 0, remaining)?;
        cost = cost.saturating_add(shape.cost).saturating_add(
            definition
                .metadata
                .identifier
                .as_str()
                .len()
                .saturating_mul(8),
        );
    }
    for file in &template.files {
        let selected = files
            .get(file.metadata.identifier.as_str())
            .unwrap_or(&file.default);
        if let Some(selected) = selected {
            cost = cost
                .saturating_add(selected.len().saturating_mul(8))
                .saturating_add(file.metadata.identifier.as_str().len().saturating_mul(8))
                .saturating_add(256);
        }
    }
    if cost > *remaining {
        return Err(limit());
    }
    Ok(cost)
}

/* === Private Functions === */
fn metadata_cost(value: &Metadata) -> usize {
    value
        .identifier
        .as_str()
        .len()
        .saturating_add(value.label.len())
        .saturating_add(value.description.as_ref().map_or(0, String::len))
        .saturating_mul(8)
        .saturating_add(512)
}

fn parameter_shape(
    definition: &ParameterDefinition,
    definitions: &[ParameterDefinition],
    overrides: &BTreeMap<String, Vec<Value>>,
    depth: usize,
    remaining: &mut usize,
) -> Result<limits::Shape, Status> {
    limits::parameter_shape(definition, definitions, overrides, depth, remaining)
        .map_err(super::bounds::status)
}

/* === Tests === */
