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
use crate::model::definition::FileDefinition;
use crate::model::definition::ParameterDefinition;
use crate::model::definition::ParameterResolverError;
use crate::model::definition::TestCase;
use crate::model::definition::TestCaseTemplate;
use crate::model::definition::parameters::coerce_list_value_detail;
use crate::model::definition::parameters::coerce_parameter_value_detail;
use crate::model::definition::parameters::resolve_parameter_spaces_detail;
use crate::model::identity::CaseId;
use crate::model::identity::CommandId;
use crate::model::identity::Metadata;
use crate::model::values::Value;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */

product_error!(CaseMaterializationError, formatter {
    Self::UnknownFilterParameters(parameters) => write!(formatter, "unknown case filter parameters: {parameters}"),
    Self::UnknownOverrideParameters(parameters) => write!(formatter, "unknown case override parameters: {parameters}"),
    Self::UnknownFileParameters(parameters) => write!(formatter, "unknown file input parameters: {parameters}"),
    Self::EmptyParameter(parameter) => write!(formatter, "no values left for parameter {parameter:?} after filters/overrides"),
    Self::MissingRequiredFile(parameter) => write!(formatter, "required file input {parameter:?} was not provided"),
    Self::InvalidChoice { parameter, value } => write!(formatter, "parameter {parameter:?} value {value:?} is not allowed"),
    Self::InvalidType { parameter, value, value_type } => write!(formatter, "parameter {parameter:?} value {value:?} cannot be coerced to {value_type:?}"),
    Self::CaseLimitExceeded { template_id, count, limit } => write!(formatter, "materialized {count} cases for {template_id:?}; limit is {limit}"),
    Self::InvalidIdentifier(message) | Self::InvalidExpansion(message) => formatter.write_str(message),
});

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CaseMaterializationSelection {
    #[serde(default)]
    pub case_filter: BTreeMap<String, Vec<Value>>,

    #[serde(default)]
    pub case_overrides: BTreeMap<String, Vec<Value>>,

    #[serde(default)]
    pub file_overrides: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseMaterializationOptions {
    pub max_cases: usize,
    pub force: bool,
    pub registration_id: CommandId,
    pub first_label_number: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseMaterializationOutput {
    pub case_count: usize,
    pub test_cases: Vec<TestCase>,
    pub exceeds_default_cap: bool,
    pub default_cap: usize,
}

struct Materialization<'a> {
    suite_id: &'a str,
    template: &'a TestCaseTemplate,
    registration_id: &'a CommandId,
    first_label_number: usize,
    dimensions: &'a [Vec<BTreeMap<String, Value>>],
    file_parameters: &'a BTreeMap<String, Value>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq)]
pub enum CaseMaterializationError {
    UnknownFilterParameters(String),
    UnknownOverrideParameters(String),
    UnknownFileParameters(String),
    EmptyParameter(String),
    MissingRequiredFile(String),
    InvalidChoice { parameter: String, value: Value },
    InvalidType { parameter: String, value: Value, value_type: ValueType },
    CaseLimitExceeded { template_id: String, count: usize, limit: usize },
    InvalidIdentifier(String),
    InvalidExpansion(String),
}

/* === Traits === */

/* === Implementations === */

impl Materialization<'_> {
    fn push_cases(
        &self, index: usize, current: &mut BTreeMap<String, Value>, cases: &mut Vec<TestCase>,
    ) -> Result<(), CaseMaterializationError> {
        let template = self.template;
        let registration_id = self.registration_id;
        let dimensions = self.dimensions;
        let file_parameters = self.file_parameters;
        if index >= dimensions.len() {
            let mut parameters = current.clone();
            parameters.extend(file_parameters.clone());

            let generation_index = cases.len() as u64;
            let case_id = CaseId::from_seed(format!(
                "materialize:{}:{registration_id}:{}:{generation_index}",
                self.suite_id, template.test_case_template_id
            ));
            let label_number = self.first_label_number + cases.len();
            let metadata = Metadata::new(
                template.metadata.identifier.clone(),
                case_label(&template.metadata.label, label_number),
                template.metadata.description.clone(),
            );

            cases.push(TestCase {
                suite_id: self.suite_id.to_owned(),
                test_case_id: case_id,
                test_case_template_id: template.test_case_template_id.clone(),
                metadata,
                suite: None,
                parameters,
                generation_index,
                label_number: label_number as u64,
            });

            return Ok(());
        }

        for row in &dimensions[index] {
            current.extend(row.clone());
            self.push_cases(index + 1, current, cases)?;
            for name in row.keys() {
                current.remove(name);
            }
        }

        Ok(())
    }
}

/* === Public Functions === */

pub fn materialize_template_cases(
    suite_id: &str, template: &TestCaseTemplate, selection: CaseMaterializationSelection, options: CaseMaterializationOptions,
) -> crate::Result<CaseMaterializationOutput> {
    materialize_template_cases_detail(suite_id, template, selection, options).map_err(Into::into)
}

/* === Private Functions === */

fn materialize_template_cases_detail(
    suite_id: &str, template: &TestCaseTemplate, selection: CaseMaterializationSelection, options: CaseMaterializationOptions,
) -> Result<CaseMaterializationOutput, CaseMaterializationError> {
    if suite_id.trim().is_empty() {
        return Err(CaseMaterializationError::InvalidIdentifier("materialized case requires a suite registration ID".to_owned()));
    }

    let dimensions = filtered_dimensions(&template.parameters, &selection)?;
    let file_parameters = resolved_file_parameters(&template.files, &selection.file_overrides)?;
    let case_count = dimension_count(&dimensions);
    let exceeds_default_cap = case_count > options.max_cases;

    if exceeds_default_cap && !options.force {
        return Err(CaseMaterializationError::CaseLimitExceeded {
            template_id: template.metadata.identifier.as_str().to_owned(),
            count: case_count,
            limit: options.max_cases,
        });
    }

    let mut cases = Vec::with_capacity(case_count);
    let mut current = BTreeMap::new();
    Materialization {
        suite_id,
        template,
        registration_id: &options.registration_id,
        first_label_number: options.first_label_number,
        dimensions: &dimensions,
        file_parameters: &file_parameters,
    }
    .push_cases(0, &mut current, &mut cases)?;

    Ok(CaseMaterializationOutput {
        case_count: cases.len(),
        test_cases: cases,
        exceeds_default_cap,
        default_cap: options.max_cases,
    })
}

fn filtered_dimensions(
    definitions: &[ParameterDefinition], selection: &CaseMaterializationSelection,
) -> Result<Vec<Vec<BTreeMap<String, Value>>>, CaseMaterializationError> {
    let known = definitions.iter().map(parameter_name).collect::<BTreeSet<_>>();
    let filters = selection.case_filter.keys().cloned().collect::<BTreeSet<_>>();
    let overrides = selection.case_overrides.keys().cloned().collect::<BTreeSet<_>>();
    let unknown_filters = filters.difference(&known).cloned().collect::<Vec<_>>();
    let unknown_overrides = overrides.difference(&known).cloned().collect::<Vec<_>>();

    if !unknown_filters.is_empty() {
        return Err(CaseMaterializationError::UnknownFilterParameters(unknown_filters.join(", ")));
    }
    if !unknown_overrides.is_empty() {
        return Err(CaseMaterializationError::UnknownOverrideParameters(unknown_overrides.join(", ")));
    }

    // Overrides participate in expression resolution, so required inputs need
    // no default and dependent expressions see the values actually selected.
    let mut effective = definitions.to_vec();
    for definition in &mut effective {
        let name = parameter_name(definition);
        if let Some(values) = selection.case_overrides.get(&name) {
            let values = parameter_dimension_values(definition, &[], values, true)?;
            definition.expression = crate::model::definition::ParameterExpr::Array {
                values: values.into_iter().map(|value| crate::model::definition::ParameterExpr::Literal { value }).collect(),
            };
        }
    }
    let resolved_spaces = resolve_parameter_spaces_detail(&effective, &|_| None).map_err(map_resolver_error)?;

    let mut dimensions: Vec<Vec<BTreeMap<String, Value>>> = Vec::new();
    let mut groups = BTreeMap::<String, usize>::new();
    for definition in definitions {
        let name = parameter_name(definition);
        let values = resolved_spaces.get(&name).map(|space| space.values.clone()).unwrap_or_default();
        if values.is_empty() {
            return Err(CaseMaterializationError::EmptyParameter(name));
        }

        if let crate::model::definition::ParameterExpansion::Zip { group } = &definition.expansion {
            if let Some(index) = groups.get(group.as_str()) {
                let rows = &mut dimensions[*index];
                if rows.len() != values.len() {
                    return Err(CaseMaterializationError::InvalidExpansion(format!(
                        "zip group {group:?} requires equal candidate counts; parameter {name:?} has {}, expected {}",
                        values.len(),
                        rows.len()
                    )));
                }
                for (row, value) in rows.iter_mut().zip(values) {
                    row.insert(name.clone(), value);
                }
                continue;
            }
            groups.insert(group.to_string(), dimensions.len());
        }
        dimensions.push(values.into_iter().map(|value| BTreeMap::from([(name.clone(), value)])).collect());
    }

    // Filter complete rows so a selection never changes the pairing of zipped values.
    for rows in &mut dimensions {
        for definition in definitions {
            let name = parameter_name(definition);
            if !rows.first().is_some_and(|row| row.contains_key(&name)) {
                continue;
            }
            if !selection.case_overrides.contains_key(&name)
                && let Some(filter) = selection.case_filter.get(&name)
            {
                let defaults = rows.iter().map(|row| row[&name].clone()).collect::<Vec<_>>();
                let allowed = parameter_dimension_values(definition, &defaults, filter, false)?;
                rows.retain(|row| allowed.contains(&row[&name]));
            }
            if rows.is_empty() {
                return Err(CaseMaterializationError::EmptyParameter(name));
            }
            if matches!(definition.expansion, crate::model::definition::ParameterExpansion::Single) && rows.len() != 1 {
                return Err(CaseMaterializationError::InvalidExpansion(format!(
                    "single parameter {name:?} requires exactly one candidate; received {}",
                    rows.len()
                )));
            }
        }
    }

    Ok(dimensions)
}

fn parameter_dimension_values(
    definition: &ParameterDefinition, default_values: &[Value], values: &[Value], from_override: bool,
) -> Result<Vec<Value>, CaseMaterializationError> {
    let name = parameter_name(definition);

    if values == [Value::Null] {
        validate_choices(definition, &Value::Null)?;
        return Ok(if from_override {
            vec![Value::Null]
        } else {
            default_values.iter().filter(|value| matches!(value, Value::Null)).cloned().collect()
        });
    }

    let values = values
        .iter()
        .cloned()
        .map(|value| {
            if let Some(list) = &definition.list {
                coerce_list_value_detail(&name, list, value).map_err(map_resolver_error)
            } else {
                coerce_parameter_value_detail(&name, &definition.value_type, value).map_err(map_resolver_error)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    if from_override {
        for value in &values {
            validate_choices(definition, value)?;
        }

        return Ok(values);
    }

    let allowed = values.into_iter().collect::<Vec<_>>();
    let filtered = default_values.iter().filter(|value| allowed.iter().any(|selected| selected == *value)).cloned().collect::<Vec<_>>();

    Ok(filtered)
}

fn resolved_file_parameters(
    definitions: &[FileDefinition], overrides: &BTreeMap<String, Option<String>>,
) -> Result<BTreeMap<String, Value>, CaseMaterializationError> {
    let known = definitions.iter().map(file_name).collect::<BTreeSet<_>>();
    let unknown = overrides.keys().filter(|name| !known.contains(*name)).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(CaseMaterializationError::UnknownFileParameters(unknown.join(", ")));
    }

    let mut parameters = BTreeMap::new();
    for definition in definitions {
        let name = file_name(definition);
        let raw_value = overrides.get(&name).cloned().unwrap_or_else(|| definition.default.clone());
        let Some(value) = raw_value else {
            if definition.required {
                return Err(CaseMaterializationError::MissingRequiredFile(name));
            }
            continue;
        };

        parameters.insert(name, Value::Text(value));
    }

    Ok(parameters)
}

fn dimension_count(dimensions: &[Vec<BTreeMap<String, Value>>]) -> usize {
    if dimensions.is_empty() {
        return 1;
    }

    dimensions.iter().map(Vec::len).try_fold(1usize, usize::checked_mul).unwrap_or(usize::MAX)
}

fn validate_choices(definition: &ParameterDefinition, value: &Value) -> Result<(), CaseMaterializationError> {
    crate::model::definition::parameters::validate_parameter_value(definition, value).map_err(map_resolver_error)
}

fn parameter_name(definition: &ParameterDefinition) -> String {
    definition.metadata.identifier.as_str().to_owned()
}

fn file_name(definition: &FileDefinition) -> String {
    definition.metadata.identifier.as_str().to_owned()
}

fn map_resolver_error(error: ParameterResolverError) -> CaseMaterializationError {
    match error {
        ParameterResolverError::InvalidType { parameter, value, value_type } => {
            CaseMaterializationError::InvalidType { parameter, value, value_type }
        }
        ParameterResolverError::InvalidChoice { parameter, value } => CaseMaterializationError::InvalidChoice { parameter, value },
        other => CaseMaterializationError::InvalidIdentifier(other.to_string()),
    }
}

fn case_label(template_label: &str, number: usize) -> String {
    format!("{template_label} #{number}")
}

/* === Tests === */
