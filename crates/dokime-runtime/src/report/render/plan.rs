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
use serde::{Serialize, Serializer, ser::SerializeSeq};

// Local Imports
use super::{ReportValue, ReportValues};
use dokime_domain::model::definition::parameters::ParameterDefinition;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::{Value, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize)]
pub(super) struct Cycle<'a> {
    pub(super) test_cycle_id: &'a str,
    pub(super) title: &'a str,
    pub(super) suite_id: &'a str,
    pub(super) suite_name: &'a str,
    pub(super) status: &'static str,
    pub(super) case_count: usize,
    pub(super) queue_count: usize,
    pub(super) current_test_execution_id: Option<&'a str>,
    pub(super) current_queue_item_id: Option<&'a str>,
    pub(super) continue_on_failure: bool,
    pub(super) started_at: &'a Timestamp,
    pub(super) finished_at: Option<&'a Timestamp>,
    pub(super) duration_s: Option<()>,
    pub(super) duration_delta_s: Option<()>,
    pub(super) metadata: &'a Metadata,
    pub(super) generated_at: &'a Timestamp,
}

#[derive(Serialize)]
pub(super) struct Case<'a> {
    pub(super) case_id: &'a str,
    pub(super) template_id: &'a str,
    pub(super) parameters: ReportValues<'a>,
    pub(super) generation_index: usize,
    pub(super) label: &'a str,
    pub(super) metadata: &'a Metadata,
    pub(super) latest_test_execution_id: Option<&'a str>,
    pub(super) status: &'static str,
}

#[derive(Serialize)]
pub(super) struct QueueItem<'a> {
    pub(super) queue_item_id: &'a str,
    pub(super) test_cycle_id: &'a str,
    pub(super) ordinal: usize,
    pub(super) template_id: &'a str,
    pub(super) case_id: &'a str,
    pub(super) repeat_index: u8,
    pub(super) status: &'static str,
    pub(super) current_test_execution_id: Option<&'a str>,
    pub(super) attempt_count: usize,
    pub(super) blocked_reason: Option<()>,
}

#[derive(Serialize)]
pub(super) struct Template<'a> {
    pub(super) template_id: &'a str,
    pub(super) name: &'a str,
    pub(super) description: Option<()>,
    pub(super) source_file: Option<()>,
    pub(super) parameters: Parameters<'a>,
    pub(super) files: &'static [()],
    pub(super) steps: &'static [()],
    pub(super) measurements: &'static [()],
    pub(super) criteria: &'static [()],
    pub(super) initialize_hooks: &'static [()],
    pub(super) finalize_hooks: &'static [()],
    pub(super) interrupt_hooks: &'static [()],
    pub(super) cycle_summary: Option<()>,
    pub(super) metadata: BTreeMap<(), ()>,
}

// The Template's Parameter Definitions In The Shape The Live Catalog Projects.
pub(super) struct Parameters<'a>(pub(super) &'a [ParameterDefinition]);

#[derive(Serialize)]
struct Parameter<'a> {
    parameter_id: &'a str,
    name: &'a str,
    values: Values<'a>,
    label: &'a str,
    description: Option<&'a str>,
    unit: Option<&'a str>,
    value_type: &'a ValueType,
    choices: Values<'a>,
    min: Option<f64>,
    max: Option<f64>,
    metadata: &'a Metadata,
}

struct Values<'a>(&'a [Value]);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serialize for Parameters<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for definition in self.0 {
            let choices = Values(definition.selection.as_ref().map_or(&[], |selection| selection.choices.as_slice()));
            rows.serialize_element(&Parameter {
                parameter_id: definition.parameter_id.as_str(),
                name: definition.metadata.identifier.as_str(),
                values: Values(choices.0),
                label: &definition.metadata.label,
                description: definition.metadata.description.as_deref(),
                unit: definition.unit.as_ref().map(|unit| unit.symbol.as_str()),
                value_type: &definition.value_type,
                choices,
                min: definition.min,
                max: definition.max,
                metadata: &definition.metadata,
            })?;
        }
        rows.end()
    }
}

impl Serialize for Values<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            rows.serialize_element(&ReportValue(value))?;
        }
        rows.end()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
