/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Selection Materialization

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{HashMap, HashSet};

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CasesCreateResponse;
use dokime_protocol::host::PlannedCase;
use dokime_protocol::host::RegisteredSelection;
use dokime_protocol::host::SelectionPlan;
use dokime_protocol::host::selected_file_kind;
use dokime_protocol::model::Parameter;
use dokime_protocol::model::ParameterBinaryOperator;
use dokime_protocol::model::ParameterExpression;
use dokime_protocol::model::ParameterUnaryOperator;
use dokime_protocol::model::TestCaseTemplate;
use dokime_protocol::model::Value;
use dokime_protocol::model::ValueList;
use dokime_protocol::model::ValueType;
use dokime_protocol::model::parameter_expansion;
use dokime_protocol::model::parameter_expression;
use dokime_protocol::model::value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn resolve(context: &Context, plan: &SelectionPlan) -> Result<CasesCreateResponse> {
    if !plan.case_ids.is_empty() && !plan.selections.is_empty() {
        return Err(Error::Invalid("Plan Selection: Choose Registered Cases or Template Selections".into()));
    }
    let cap = if plan.max_cases == 0 { context.settings.max_cases } else { plan.max_cases as usize };
    if cap > context.settings.max_cases {
        return Err(Error::Invalid("Plan Selection: Reduce the Requested Case Bound".into()));
    }
    if !plan.case_ids.is_empty() {
        if plan.case_ids.len() > cap {
            return Err(Error::Invalid("Plan Selection: Reduce the Selected Cases".into()));
        }
        let mut cases = Vec::new();
        for identifier in &plan.case_ids {
            cases.push(crate::host::cases::storage::read(context, identifier, None).await?);
        }
        return Ok(CasesCreateResponse { cases, ..Default::default() });
    }
    if plan.selections.is_empty() {
        return Err(Error::Invalid("Plan Selection: Supply at Least One Selection".into()));
    }
    if !plan.cycle_id.is_empty() && crate::host::execution_state::cycle(context, &plan.cycle_id).await?.sealed {
        return Err(Error::Precondition("Plan Selection: Select an Unsealed Cycle".into()));
    }
    let catalog = context.host.plugins().await?;
    let catalog_cursor = crate::host::paging::cursor(context, "planning.catalog".into(), catalog.iter().map(crate::host::paging::bytes));
    let suites = context.host.suites()?;
    let first_ordinal =
        if plan.cycle_id.is_empty() { 0 } else { crate::host::cases::storage::list(context, &plan.cycle_id).await?.len() as u64 };
    let mut cases = Vec::new();
    for (selection_index, selection) in plan.selections.iter().enumerate() {
        let matches: Vec<_> = suites
            .iter()
            .filter(|(_, suite)| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.suite_id))
            .collect();
        if matches.len() != 1 {
            return Err(Error::Invalid("Plan Selection: Choose One Unambiguous Connected Suite".into()));
        }
        let (plugin, suite) = matches[0];
        let template = suite
            .templates
            .iter()
            .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.template_id))
            .ok_or_else(|| Error::NotFound("Plan Selection: Choose a Template in the Selected Suite".into()))?;
        let parameters = expand(template, selection, cap)?;
        if cases.len().checked_add(parameters.len()).is_none_or(|count| count > cap) {
            return Err(Error::Invalid("Plan Selection: Reduce the Expanded Case Count".into()));
        }
        for parameters in parameters {
            let origin = context.host.materialize(plugin, &selection.template_id, parameters).await?;
            let mut metadata = template.metadata.clone();
            if let Some(metadata) = &mut metadata {
                metadata.label = format!("{} #{}", metadata.label, first_ordinal + cases.len() as u64 + 1);
            }
            cases.push(PlannedCase {
                suite_id: selection.suite_id.clone(),
                template_id: selection.template_id.clone(),
                metadata,
                ordinal: first_ordinal + cases.len() as u64,
                selection_index: selection_index as u64,
                origin: Some(origin),
                ..Default::default()
            });
        }
    }
    let latest = context.host.plugins().await?;
    if crate::host::paging::cursor(context, "planning.catalog".into(), latest.iter().map(crate::host::paging::bytes)) != catalog_cursor {
        return Err(Error::Precondition("Plan Selection: Refresh the Changed Catalog Before Registering".into()));
    }
    let reply = CasesCreateResponse { cases, ..Default::default() };
    if reply.encoded_len() > context.settings.message_bytes {
        return Err(Error::Invalid("Plan Selection: Reduce the Plan to Fit the Configured Reply Bound".into()));
    }
    Ok(reply)
}

/* === Private Functions === */

fn expand(template: &TestCaseTemplate, selection: &RegisteredSelection, cap: usize) -> Result<Vec<HashMap<String, Value>>> {
    let mut supplied = HashMap::new();
    for parameter in &selection.parameters {
        if !template.parameters.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == parameter.key)) {
            return Err(Error::Invalid(format!("Plan Selection: Remove Unknown Parameter {}", parameter.key)));
        }
        let values = parameter
            .value
            .as_ref()
            .filter(|values| !values.values.is_empty())
            .ok_or_else(|| Error::Invalid("Plan Selection: Supply Nonempty Parameter Values".into()))?
            .values
            .clone();
        if supplied.insert(parameter.key.clone(), values).is_some() {
            return Err(Error::Invalid("Plan Selection: Use Unique Parameter Names".into()));
        }
    }
    let mut resolved = supplied.clone();
    let mut unresolved: Vec<_> = template
        .parameters
        .iter()
        .filter(|parameter| parameter.metadata.as_ref().is_some_and(|metadata| !resolved.contains_key(&metadata.identifier)))
        .collect();
    while !unresolved.is_empty() {
        let before = unresolved.len();
        let mut remaining = Vec::new();
        for parameter in unresolved {
            let name =
                &parameter.metadata.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Declare Named Parameters".into()))?.identifier;
            let Some(expression) = &parameter.expression else {
                if parameter.required {
                    return Err(Error::Invalid(format!("Plan Selection: Supply Required Parameter {name}")));
                }
                continue;
            };
            match expression_values(expression, &resolved, cap)? {
                Some(values) => {
                    resolved.insert(name.clone(), values);
                }
                None => remaining.push(parameter),
            }
        }
        if remaining.len() == before {
            return Err(Error::Invalid("Plan Selection: Resolve Missing or Cyclic Parameter References".into()));
        }
        unresolved = remaining;
    }
    let mut dimensions: Vec<(String, Vec<HashMap<String, Value>>)> = Vec::new();
    for parameter in &template.parameters {
        let name = &parameter.metadata.as_ref().unwrap().identifier;
        let Some(mut values) = resolved.remove(name) else {
            continue;
        };
        if parameter.r#type == ValueType::List as i32 && !matches!(values.as_slice(), [Value { kind: Some(value::Kind::List(_)) }]) {
            values = vec![Value {
                kind: Some(value::Kind::List(ValueList { values })),
            }];
        }
        if values.is_empty() || values.len() > cap {
            return Err(Error::Invalid(format!("Plan Selection: Supply Bounded Nonempty Values for {name}")));
        }
        for value in &mut values {
            let received = value.clone();
            coerce(parameter, value)?;
            validate_parameter(parameter, value).map_err(|_| parameter_error(parameter, &received))?;
        }
        let expansion = parameter.expansion.as_ref().and_then(|expansion| expansion.kind.as_ref());
        if matches!(expansion, Some(parameter_expansion::Kind::Single(true))) && values.len() != 1 {
            return Err(Error::Invalid(format!("Plan Selection: Supply One Value for {name}")));
        }
        let group = match expansion {
            Some(parameter_expansion::Kind::Zip(group)) => format!("zip:{group}"),
            _ => format!("parameter:{name}"),
        };
        if let Some((_, rows)) = dimensions.iter_mut().find(|(key, _)| key == &group) {
            if rows.len() != values.len() {
                return Err(Error::Invalid("Plan Selection: Use Equal Lengths in Each Zip Group".into()));
            }
            for (row, value) in rows.iter_mut().zip(values) {
                row.insert(name.clone(), value);
            }
        } else {
            dimensions.push((group, values.into_iter().map(|value| HashMap::from([(name.clone(), value)])).collect()));
        }
    }
    let mut combinations = vec![HashMap::new()];
    for (_, rows) in dimensions {
        if combinations.len().checked_mul(rows.len()).is_none_or(|count| count > cap) {
            return Err(Error::Invalid("Plan Selection: Reduce the Cartesian Expansion".into()));
        }
        let mut expanded = Vec::new();
        for combination in combinations {
            for row in &rows {
                let mut combined = combination.clone();
                combined.extend(row.clone());
                expanded.push(combined);
            }
        }
        combinations = expanded;
    }
    let mut file_names = HashSet::new();
    let mut files = HashMap::new();
    for file in &selection.files {
        if !file_names.insert(&file.key)
            || !template.files.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == file.key))
        {
            return Err(Error::Invalid("Plan Selection: Use Unique Declared File Parameters".into()));
        }
        let value = match file.value.as_ref().and_then(|file| file.kind.as_ref()).and_then(|kind| kind.kind.as_ref()) {
            Some(selected_file_kind::Kind::Value(value)) => Some(value.clone()),
            Some(selected_file_kind::Kind::Clear(_)) => None,
            None => return Err(Error::Invalid("Plan Selection: Select or Explicitly Clear Each File".into())),
        };
        files.insert(file.key.clone(), value);
    }
    for field in &template.files {
        let name = &field.metadata.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Name Each File Parameter".into()))?.identifier;
        let value = files.get(name).cloned().unwrap_or_else(|| field.default_value.clone());
        if field.required && value.as_ref().is_none_or(|value| value.is_empty()) {
            return Err(Error::Invalid(format!("Plan Selection: Supply File Parameter {name}")));
        }
        if let Some(value) = value {
            for combination in &mut combinations {
                combination.insert(
                    name.clone(),
                    Value {
                        kind: Some(value::Kind::Text(value.clone())),
                    },
                );
            }
        }
    }
    Ok(combinations)
}

fn coerce(parameter: &Parameter, value: &mut Value) -> Result<()> {
    value.kind = Some(match (ValueType::try_from(parameter.r#type), value.kind.take()) {
        (Ok(ValueType::Number), Some(value::Kind::Integer(value))) if value as f64 as i128 == i128::from(value) => {
            value::Kind::Number(value as f64)
        }
        (Ok(ValueType::Number), Some(value::Kind::UnsignedInteger(value))) if value as f64 as i128 == i128::from(value) => {
            value::Kind::Number(value as f64)
        }
        (Ok(ValueType::UnsignedInteger), Some(value::Kind::Integer(value))) if value >= 0 => value::Kind::UnsignedInteger(value as u64),
        (Ok(ValueType::Integer), Some(value::Kind::UnsignedInteger(value))) if i64::try_from(value).is_ok() => {
            value::Kind::Integer(value as i64)
        }
        (Ok(ValueType::Integer), Some(value::Kind::Number(value)))
            if value.is_finite() && value.fract() == 0.0 && value >= i64::MIN as f64 && value < -(i64::MIN as f64) =>
        {
            value::Kind::Integer(value as i64)
        }
        (Ok(ValueType::UnsignedInteger), Some(value::Kind::Number(value)))
            if value.is_finite() && value.fract() == 0.0 && value >= 0.0 && value < u64::MAX as f64 =>
        {
            value::Kind::UnsignedInteger(value as u64)
        }
        (_, Some(value)) => value,
        (_, None) => return Err(parameter_error(parameter, value)),
    });
    Ok(())
}

fn validate_parameter(parameter: &Parameter, value: &Value) -> Result<()> {
    if matches!(value.kind, Some(value::Kind::Null(_))) && !parameter.required {
        return Ok(());
    }
    if !crate::host::prompts::validation::type_matches(parameter.r#type, value)
        || (!parameter.choices.is_empty() && !parameter.choices.contains(value))
    {
        return Err(parameter_error(parameter, value));
    }
    if let Some(value::Kind::List(list)) = &value.kind
        && (parameter.min_items.is_some_and(|min| list.values.len() < min as usize)
            || parameter.max_items.is_some_and(|max| list.values.len() > max as usize)
            || list.values.iter().any(|value| !crate::host::prompts::validation::type_matches(parameter.element_type, value)))
    {
        return Err(parameter_error(parameter, value));
    }
    let integer = match value.kind {
        Some(value::Kind::Integer(value)) => Some(i128::from(value)),
        Some(value::Kind::UnsignedInteger(value)) => Some(i128::from(value)),
        _ => None,
    };
    let outside_range = if let Some(integer) = integer {
        parameter.min.is_some_and(|min| integer < min.ceil() as i128) || parameter.max.is_some_and(|max| integer > max.floor() as i128)
    } else {
        numeric(value).is_some_and(|number| parameter.min.is_some_and(|min| number < min) || parameter.max.is_some_and(|max| number > max))
    };
    if outside_range {
        return Err(parameter_error(parameter, value));
    }
    Ok(())
}

fn parameter_error(parameter: &Parameter, value: &Value) -> Error {
    let metadata = parameter.metadata.as_ref().unwrap();
    let label = if metadata.label.is_empty() { &metadata.identifier } else { &metadata.label };
    let (kind, minimum, maximum) = match ValueType::try_from(parameter.r#type) {
        Ok(ValueType::UnsignedInteger) => ("unsigned integer", "0", "18446744073709551615"),
        Ok(ValueType::Integer) => ("signed integer", "-9223372036854775808", "9223372036854775807"),
        Ok(ValueType::Number) => ("number", "unbounded", "unbounded"),
        Ok(ValueType::Boolean) => ("boolean", "unbounded", "unbounded"),
        Ok(ValueType::Text) => ("text", "unbounded", "unbounded"),
        Ok(ValueType::Bytes) => ("bytes", "unbounded", "unbounded"),
        Ok(ValueType::Datetime) => ("datetime", "unbounded", "unbounded"),
        Ok(ValueType::List) => ("list with the declared element type and item bounds", "unbounded", "unbounded"),
        _ => ("declared value type", "unbounded", "unbounded"),
    };
    let minimum = parameter.min.map_or_else(|| minimum.into(), |value| value.to_string());
    let maximum = parameter.max.map_or_else(|| maximum.into(), |value| value.to_string());
    let choices = if parameter.choices.is_empty() { String::new() } else { format!(", choices {:?}", parameter.choices) };
    let received = match &value.kind {
        Some(value::Kind::Integer(value)) => format!("signed integer {value}"),
        Some(value::Kind::UnsignedInteger(value)) => format!("unsigned integer {value}"),
        Some(value::Kind::Number(value)) => format!("number {value}"),
        Some(value::Kind::Text(value)) => format!("text {value:?}"),
        Some(value::Kind::Boolean(value)) => format!("boolean {value}"),
        Some(value) => format!("{value:?}"),
        None => "untyped value".into(),
    };
    Error::Invalid(format!("Plan Selection: Enter {kind} for {label} in range [{minimum}, {maximum}]{choices}; received {received}"))
}

fn numeric(value: &Value) -> Option<f64> {
    match value.kind {
        Some(value::Kind::Integer(value)) => Some(value as f64),
        Some(value::Kind::UnsignedInteger(value)) => Some(value as f64),
        Some(value::Kind::Number(value)) => Some(value),
        _ => None,
    }
}

fn expression_values(expression: &ParameterExpression, resolved: &HashMap<String, Vec<Value>>, cap: usize) -> Result<Option<Vec<Value>>> {
    let values = match expression.kind.as_ref() {
        Some(parameter_expression::Kind::Literal(value)) => vec![value.clone()],
        Some(parameter_expression::Kind::Reference(name)) => return Ok(resolved.get(name).cloned()),
        Some(parameter_expression::Kind::Array(array)) => {
            let mut values = Vec::new();
            for item in &array.values {
                let Some(items) = expression_values(item, resolved, cap)? else {
                    return Ok(None);
                };
                values.extend(items);
                if values.len() > cap {
                    return Err(Error::Invalid("Plan Selection: Reduce the Parameter Array".into()));
                }
            }
            values
        }
        Some(parameter_expression::Kind::Range(range)) => {
            if range.step == 0 {
                return Err(Error::Invalid("Plan Selection: Use a Nonzero Integer Range Step".into()));
            }
            let mut values = Vec::new();
            let mut value = range.start;
            while if range.step > 0 { value < range.stop } else { value > range.stop } {
                if values.len() >= cap {
                    return Err(Error::Invalid("Plan Selection: Reduce the Integer Range".into()));
                }
                values.push(Value { kind: Some(value::Kind::Integer(value)) });
                value = value
                    .checked_add(range.step)
                    .ok_or_else(|| Error::Invalid("Plan Selection: Keep Integer Ranges Within Their Type".into()))?;
            }
            values
        }
        Some(parameter_expression::Kind::Arange(range)) => {
            if !range.start.is_finite() || !range.stop.is_finite() || !range.step.is_finite() || range.step == 0.0 {
                return Err(Error::Invalid("Plan Selection: Use Finite Ranges with a Nonzero Step".into()));
            }
            let mut values = Vec::new();
            for index in 0..=cap {
                let value = range.start + range.step * index as f64;
                if if range.step > 0.0 { value >= range.stop } else { value <= range.stop } {
                    break;
                }
                if index == cap {
                    return Err(Error::Invalid("Plan Selection: Reduce the Number Range".into()));
                }
                values.push(Value { kind: Some(value::Kind::Number(value)) });
            }
            values
        }
        Some(parameter_expression::Kind::Linspace(range)) => {
            if range.count == 0 || range.count > cap as u64 || !range.start.is_finite() || !range.stop.is_finite() {
                return Err(Error::Invalid("Plan Selection: Use a Bounded Finite Linear Space".into()));
            }
            (0..range.count)
                .map(|index| Value {
                    kind: Some(value::Kind::Number(if range.count == 1 {
                        range.start
                    } else {
                        range.start + (range.stop - range.start) * index as f64 / (range.count - 1) as f64
                    })),
                })
                .collect()
        }
        Some(parameter_expression::Kind::DynamicDefault(default)) => {
            let fallback = default
                .fallback
                .as_ref()
                .ok_or_else(|| Error::Invalid("Plan Selection: Supply a Fallback for an Unavailable Operator Default".into()))?;
            return expression_values(fallback, resolved, cap);
        }
        Some(parameter_expression::Kind::Unary(unary)) => {
            if unary.operator != ParameterUnaryOperator::Negate as i32 {
                return Err(Error::Invalid("Plan Selection: Declare a Valid Unary Operator".into()));
            }
            let child = unary.value.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Unary Operand".into()))?;
            let Some(values) = expression_values(child, resolved, cap)? else {
                return Ok(None);
            };
            values
                .into_iter()
                .map(|value| match value.kind {
                    Some(value::Kind::Integer(value)) => value
                        .checked_neg()
                        .map(|value| Value { kind: Some(value::Kind::Integer(value)) })
                        .ok_or_else(|| Error::Invalid("Plan Selection: Avoid Integer Overflow".into())),
                    _ => numeric(&value)
                        .map(|value| Value { kind: Some(value::Kind::Number(-value)) })
                        .ok_or_else(|| Error::Invalid("Plan Selection: Use Numeric Unary Operands".into())),
                })
                .collect::<Result<_>>()?
        }
        Some(parameter_expression::Kind::Binary(binary)) => {
            let left = binary.left.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Left Operand".into()))?;
            let right = binary.right.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Right Operand".into()))?;
            let (Some(left), Some(right)) = (expression_values(left, resolved, cap)?, expression_values(right, resolved, cap)?) else {
                return Ok(None);
            };
            if left.len() != right.len() && left.len() != 1 && right.len() != 1 {
                return Err(Error::Invalid("Plan Selection: Use Equal Arithmetic Operand Lengths or a Scalar".into()));
            }
            let count = left.len().max(right.len());
            if count > cap {
                return Err(Error::Invalid("Plan Selection: Reduce the Arithmetic Expansion".into()));
            }
            let mut values = Vec::new();
            for index in 0..count {
                let left = &left[if left.len() == 1 { 0 } else { index }];
                let right = &right[if right.len() == 1 { 0 } else { index }];
                let (Some(left), Some(right)) = (numeric(left), numeric(right)) else {
                    return Err(Error::Invalid("Plan Selection: Use Numeric Arithmetic Operands".into()));
                };
                let value = match ParameterBinaryOperator::try_from(binary.operator).unwrap_or_default() {
                    ParameterBinaryOperator::Add => left + right,
                    ParameterBinaryOperator::Subtract => left - right,
                    ParameterBinaryOperator::Multiply => left * right,
                    ParameterBinaryOperator::Divide if right != 0.0 => left / right,
                    _ => return Err(Error::Invalid("Plan Selection: Use Valid Arithmetic Without Division by Zero".into())),
                };
                if !value.is_finite() {
                    return Err(Error::Invalid("Plan Selection: Keep Arithmetic Results Finite".into()));
                }
                values.push(Value { kind: Some(value::Kind::Number(value)) });
            }
            values
        }
        None => return Err(Error::Invalid("Plan Selection: Declare a Parameter Expression".into())),
    };
    if values.len() > cap {
        return Err(Error::Invalid("Plan Selection: Reduce the Parameter Expansion".into()));
    }
    Ok(Some(values))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_protocol::host::RegisteredSelectionParametersEntry;
    use dokime_protocol::host::SelectionValues;
    use dokime_protocol::model::IntegerRange;
    use dokime_protocol::model::Metadata;
    use dokime_protocol::model::ParameterBinary;
    use dokime_protocol::model::ParameterExpansion;
    use dokime_protocol::model::ParameterExpressions;
    use dokime_protocol::model::Suites;

    #[test]
    fn numeric_parameters_accept_only_lossless_conversions() -> Result<()> {
        for (declared, received, expected) in [
            (ValueType::UnsignedInteger, value::Kind::Integer(4096), Some(value::Kind::UnsignedInteger(4096))),
            (ValueType::UnsignedInteger, value::Kind::Integer(i64::MAX), Some(value::Kind::UnsignedInteger(i64::MAX as u64))),
            (ValueType::UnsignedInteger, value::Kind::Integer(0), Some(value::Kind::UnsignedInteger(0))),
            (ValueType::UnsignedInteger, value::Kind::Integer(-1), None),
            (ValueType::Integer, value::Kind::UnsignedInteger(i64::MAX as u64), Some(value::Kind::Integer(i64::MAX))),
            (ValueType::Integer, value::Kind::UnsignedInteger(i64::MAX as u64 + 1), None),
            (ValueType::Integer, value::Kind::UnsignedInteger(u64::MAX), None),
            (ValueType::Integer, value::Kind::Number(i64::MIN as f64), Some(value::Kind::Integer(i64::MIN))),
            (ValueType::Integer, value::Kind::Number(-(i64::MIN as f64)), None),
            (ValueType::UnsignedInteger, value::Kind::Number(u64::MAX as f64), None),
            (ValueType::UnsignedInteger, value::Kind::Number(4096.0), Some(value::Kind::UnsignedInteger(4096))),
            (ValueType::UnsignedInteger, value::Kind::Number(-1.0), None),
            (ValueType::UnsignedInteger, value::Kind::Number(1.5), None),
            (ValueType::Integer, value::Kind::Number(1.5), None),
            (ValueType::Integer, value::Kind::Number(f64::NAN), None),
            (ValueType::UnsignedInteger, value::Kind::Number(f64::INFINITY), None),
            (ValueType::UnsignedInteger, value::Kind::Text("4096".into()), None),
            (ValueType::Number, value::Kind::Integer(1 << 53), Some(value::Kind::Number((1_u64 << 53) as f64))),
            (ValueType::Number, value::Kind::Integer((1 << 53) + 1), None),
            (ValueType::Number, value::Kind::UnsignedInteger(u64::MAX), None),
            (ValueType::Number, value::Kind::Integer(i64::MIN), Some(value::Kind::Number(i64::MIN as f64))),
        ] {
            let parameter = Parameter {
                metadata: Some(Metadata {
                    identifier: "value".into(),
                    label: "Sample Value".into(),
                    ..Default::default()
                }),
                r#type: declared as i32,
                required: true,
                ..Default::default()
            };
            let mut value = Value { kind: Some(received) };
            coerce(&parameter, &mut value)?;
            if let Some(expected) = expected {
                validate_parameter(&parameter, &value)?;
                assert_eq!(value.kind, Some(expected));
            } else {
                assert!(validate_parameter(&parameter, &value).is_err(), "{declared:?}: {value:?}");
            }
        }
        Ok(())
    }

    #[test]
    fn numeric_parameters_enforce_declared_ranges_after_conversion() {
        let template = TestCaseTemplate {
            parameters: vec![Parameter {
                metadata: Some(Metadata {
                    identifier: "iterations".into(),
                    label: "Iterations per Sample".into(),
                    ..Default::default()
                }),
                r#type: ValueType::UnsignedInteger as i32,
                min: Some(1.0),
                max: Some(65536.0),
                ..Default::default()
            }],
            ..Default::default()
        };
        for received in [value::Kind::Integer(0), value::Kind::Integer(65537), value::Kind::Text("4096".into())] {
            let selection = RegisteredSelection {
                parameters: vec![RegisteredSelectionParametersEntry {
                    key: "iterations".into(),
                    value: Some(SelectionValues {
                        values: vec![Value { kind: Some(received) }],
                    }),
                }],
                ..Default::default()
            };
            assert!(matches!(expand(&template, &selection, 1), Err(Error::Invalid(_))));
        }

        // Compare Wide Integer Bounds Without Rounding the Received Value
        let mut template = template;
        template.parameters[0].max = Some((1_u64 << 53) as f64);
        let selection = RegisteredSelection {
            parameters: vec![RegisteredSelectionParametersEntry {
                key: "iterations".into(),
                value: Some(SelectionValues {
                    values: vec![Value {
                        kind: Some(value::Kind::Integer((1_i64 << 53) + 1)),
                    }],
                }),
            }],
            ..Default::default()
        };
        assert!(matches!(expand(&template, &selection, 1), Err(Error::Invalid(_))));
    }

    #[test]
    fn every_obc_template_plans_its_declared_defaults() -> Result<()> {
        let declarations = plugin_obc::declarations().expect("OBC Declarations");
        let section = declarations.sections.iter().find(|section| section.r#type == "dokime.model.v1.Suites").unwrap();
        let suites = Suites::decode(section.payload.as_slice())?;
        let mut kinds = HashSet::new();
        let mut templates = 0;
        for suite in suites.suites {
            for template in suite.templates {
                templates += 1;
                let defaults = expand(&template, &RegisteredSelection::default(), 256)?;
                assert!(!defaults.is_empty(), "{:?}", template.metadata);
                for parameter in &template.parameters {
                    kinds.insert(parameter.r#type);
                }
                for parameters in defaults {
                    let selection = RegisteredSelection {
                        parameters: parameters
                            .iter()
                            .map(|(key, value)| RegisteredSelectionParametersEntry {
                                key: key.clone(),
                                value: Some(SelectionValues { values: vec![value.clone()] }),
                            })
                            .collect(),
                        ..Default::default()
                    };
                    assert_eq!(expand(&template, &selection, 256)?, vec![parameters]);
                }
            }
        }
        eprintln!("Planned {templates} OBC Templates With Parameter Kinds {kinds:?}");
        Ok(())
    }

    #[test]
    fn parameter_expressions_preserve_zip_and_list_shapes() -> Result<()> {
        // Resolve a Selected Space and Its Pairwise Derived Values
        let template = TestCaseTemplate {
            parameters: vec![
                Parameter {
                    metadata: Some(Metadata {
                        identifier: "left".into(),
                        ..Default::default()
                    }),
                    r#type: ValueType::Number as i32,
                    expansion: Some(ParameterExpansion {
                        kind: Some(parameter_expansion::Kind::Zip("pair".into())),
                    }),
                    ..Default::default()
                },
                Parameter {
                    metadata: Some(Metadata {
                        identifier: "right".into(),
                        ..Default::default()
                    }),
                    r#type: ValueType::Number as i32,
                    expansion: Some(ParameterExpansion {
                        kind: Some(parameter_expansion::Kind::Zip("pair".into())),
                    }),
                    expression: Some(ParameterExpression {
                        kind: Some(parameter_expression::Kind::Binary(Box::new(ParameterBinary {
                            operator: ParameterBinaryOperator::Add as i32,
                            left: Some(Box::new(ParameterExpression {
                                kind: Some(parameter_expression::Kind::Reference("left".into())),
                            })),
                            right: Some(Box::new(ParameterExpression {
                                kind: Some(parameter_expression::Kind::Array(ParameterExpressions {
                                    values: vec![
                                        ParameterExpression {
                                            kind: Some(parameter_expression::Kind::Literal(Value {
                                                kind: Some(value::Kind::Number(10.0)),
                                            })),
                                        },
                                        ParameterExpression {
                                            kind: Some(parameter_expression::Kind::Literal(Value {
                                                kind: Some(value::Kind::Number(20.0)),
                                            })),
                                        },
                                    ],
                                })),
                            })),
                        }))),
                    }),
                    ..Default::default()
                },
                Parameter {
                    metadata: Some(Metadata {
                        identifier: "labels".into(),
                        ..Default::default()
                    }),
                    r#type: ValueType::List as i32,
                    element_type: ValueType::Text as i32,
                    expression: Some(ParameterExpression {
                        kind: Some(parameter_expression::Kind::Array(ParameterExpressions {
                            values: vec![
                                ParameterExpression {
                                    kind: Some(parameter_expression::Kind::Literal(Value {
                                        kind: Some(value::Kind::Text("a".into())),
                                    })),
                                },
                                ParameterExpression {
                                    kind: Some(parameter_expression::Kind::Literal(Value {
                                        kind: Some(value::Kind::Text("b".into())),
                                    })),
                                },
                            ],
                        })),
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let selection = RegisteredSelection {
            parameters: vec![RegisteredSelectionParametersEntry {
                key: "left".into(),
                value: Some(SelectionValues {
                    values: vec![Value { kind: Some(value::Kind::Number(1.0)) }, Value { kind: Some(value::Kind::Number(2.0)) }],
                }),
            }],
            ..Default::default()
        };
        let rows = expand(&template, &selection, 4)?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["right"].kind, Some(value::Kind::Number(11.0)));
        assert_eq!(rows[1]["right"].kind, Some(value::Kind::Number(22.0)));
        assert!(matches!(&rows[0]["labels"].kind,Some(value::Kind::List(values)) if values.values.len() == 2));

        // Refuse an Incompatible Pair Without Producing a Partial Expansion
        let mut invalid = template.clone();
        if let Some(parameter_expression::Kind::Binary(binary)) =
            invalid.parameters[1].expression.as_mut().and_then(|expression| expression.kind.as_mut())
        {
            binary.right = Some(Box::new(ParameterExpression {
                kind: Some(parameter_expression::Kind::Range(IntegerRange { start: 1, stop: 4, step: 1 })),
            }));
        }
        assert!(expand(&invalid, &selection, 8).is_err());
        assert!(expand(&template, &selection, 1).is_err());
        Ok(())
    }
}
