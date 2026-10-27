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
use std::collections::BTreeSet;

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::definition::ParameterDefinition;
use crate::model::definition::ParameterExpr;
use crate::model::definition::parameter_name;
use crate::model::definition::resolve_parameter_spaces;
use crate::model::values::Value;

/* === Constants === */

pub const MAX_OUTPUTS: u64 = 32;
pub const MAX_PARAMETERS: u64 = 64;
pub const MAX_UNIT_BYTES: u64 = 128;

pub const MAX_SAMPLES: u32 = 262_144;
pub const MAX_PARAMETER_BYTES: usize = 65_536;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    pub id: String,
    pub version: u32,
    pub label: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub outputs: Vec<Output>,
    pub limits: Limits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub max_samples: u32,
    pub max_output_values: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub axis: Option<Axis>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Axis {
    pub key: String,
    pub unit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    pub id: String,
    pub version: u32,

    #[serde(default)]
    pub parameters: BTreeMap<String, Value>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for Selection {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.id.capacity()).saturating_add(
            harmos_domain::MemorySize::memory_size(&self.parameters).saturating_sub(std::mem::size_of_val(&self.parameters)),
        )
    }
}

impl Definition {
    pub fn check_limits(&self) -> Result<(), String> {
        if self.outputs.len() as u64 > MAX_OUTPUTS || self.parameters.len() as u64 > MAX_PARAMETERS {
            return Err("analysis declaration exceeds configured output or parameter counts".into());
        }
        self.check()?;
        let parameters = serde_json::to_vec(&self.parameters).map_err(|error| error.to_string())?;
        if parameters.len() as u64 > MAX_PARAMETER_BYTES as u64 {
            return Err("analysis parameter declarations exceed their configured byte budget".into());
        }
        let units = self.outputs.iter().flat_map(|output| output.unit.iter().chain(output.axis.iter().flat_map(|axis| axis.unit.iter())));
        if units.into_iter().any(|unit| unit.len() as u64 > MAX_UNIT_BYTES) {
            return Err("analysis output units exceed their configured byte budget".into());
        }
        Ok(())
    }

    pub fn check_selection_limits(&self, selection: &Selection) -> Result<(), String> {
        let parameters = serde_json::to_vec(&selection.parameters).map_err(|error| error.to_string())?;
        if parameters.len() as u64 > MAX_PARAMETER_BYTES as u64 {
            return Err("analysis parameters exceed their configured byte budget".into());
        }
        Ok(())
    }

    pub fn check(&self) -> Result<(), String> {
        if self.id.is_empty() || self.version == 0 || self.label.is_empty() {
            return Err("analysis requires an identity, positive version and label".into());
        }
        self.limits.check()?;
        if self.outputs.is_empty() {
            return Err("analysis must declare at least one output".into());
        }
        let mut keys = BTreeSet::new();
        for output in &self.outputs {
            if output.key.is_empty() || output.label.is_empty() || !keys.insert(&output.key) {
                return Err("analysis output keys must be nonempty and unique".into());
            }
            if output.axis.as_ref().is_some_and(|axis| axis.key.is_empty()) {
                return Err("analysis output axis requires a key".into());
            }
        }
        let mut parameters = BTreeSet::new();
        for parameter in &self.parameters {
            if !matches!(parameter.expression, ParameterExpr::Literal { .. }) {
                return Err("analysis parameters require literal defaults, without sweep expansion".into());
            }
            if !parameters.insert(parameter_name(parameter)) {
                return Err("analysis parameter names must be unique".into());
            }
        }
        Ok(())
    }

    pub fn resolve(&self, selection: &Selection) -> Result<BTreeMap<String, Value>, String> {
        self.check()?;
        if selection.id != self.id || selection.version != self.version {
            return Err("analysis identity or version is unavailable".into());
        }
        if selection.parameters.keys().any(|key| !self.parameters.iter().any(|definition| parameter_name(definition) == key)) {
            return Err("analysis received an undeclared parameter".into());
        }
        let mut definitions = self.parameters.clone();
        for definition in &mut definitions {
            if let Some(value) = selection.parameters.get(parameter_name(definition)) {
                definition.expression = ParameterExpr::Literal { value: value.clone() };
            }
        }
        let resolved = resolve_parameter_spaces(&definitions, &|_| None).map_err(|error| error.to_string())?;
        resolved
            .into_iter()
            .map(|(key, space)| {
                if space.values.len() != 1 {
                    return Err(format!("analysis parameter {key} requires exactly one value"));
                }
                Ok((key, space.values.into_iter().next().expect("one resolved value")))
            })
            .collect()
    }
}

impl Limits {
    pub fn constrained(self) -> Self {
        Self {
            max_samples: u64::from(self.max_samples).min(u64::from(MAX_SAMPLES)) as u32,
            max_output_values: u64::from(self.max_output_values).min(u64::from(MAX_SAMPLES)) as u32,
        }
    }

    pub fn check(self) -> Result<(), String> {
        if self.max_samples == 0 || self.max_output_values == 0 {
            return Err("analysis sample and output budgets must be positive".into());
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::identity::{Identifier, Metadata, ParameterId};
    use crate::model::values::ValueType;

    #[test]
    fn analysis_refuses_sweeps() {
        let parameter = ParameterDefinition {
            expansion: Default::default(),
            parameter_id: ParameterId::from_seed("analysis.factor"),
            metadata: Metadata::new(Identifier::new("factor").unwrap(), "Factor", None),
            group_id: None,
            value_type: ValueType::Number,
            unit: None,
            min: Some(0.0),
            max: Some(10.0),
            required: true,
            expression: ParameterExpr::Literal { value: Value::Number(2.0) },
            runtime_writable: false,
            selection: None,
            list: None,
            extensions: BTreeMap::new(),
        };
        let mut definition = Definition {
            id: "analysis-fixture.scale".into(),
            version: 1,
            label: "Scale Signal".into(),
            description: "Multiply Original Samples By A Typed Factor".into(),
            parameters: vec![parameter],
            outputs: vec![Output {
                key: "scaled".into(),
                label: "Scaled Signal".into(),
                unit: Some("V".into()),
                axis: Some(Axis { key: "time".into(), unit: Some("s".into()) }),
            }],
            limits: Limits { max_samples: 64, max_output_values: 64 },
        };
        assert!(definition.check().is_ok());

        definition.parameters[0].expression = ParameterExpr::Linspace { start: 0.0, stop: 1.0, count: usize::MAX };
        assert!(definition.check().unwrap_err().contains("literal defaults"));
    }
}
