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

use std::collections::{HashMap, HashSet};
use prost::Message;
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn resolve(context: &Context, plan: &v1::SelectionPlan) -> Result<v1::PreviewSelectionReply> {
    if !plan.case_ids.is_empty() && !plan.selections.is_empty() { return Err(Error::Invalid("Plan Selection: Choose Registered Cases or Template Selections".into())); }
    let cap = if plan.max_cases == 0 { context.settings.max_cases } else { plan.max_cases as usize };
    if cap > context.settings.max_cases { return Err(Error::Invalid("Plan Selection: Reduce the Requested Case Bound".into())); }
    if !plan.case_ids.is_empty() {
        if plan.case_ids.len() > cap { return Err(Error::Invalid("Plan Selection: Reduce the Selected Cases".into())); }
        let mut cases = Vec::new();
        for identifier in &plan.case_ids { cases.push(context.host.store().planned_case(identifier)?); }
        return Ok(v1::PreviewSelectionReply { cases, ..Default::default() });
    }
    if plan.selections.is_empty() { return Err(Error::Invalid("Plan Selection: Supply at Least One Selection".into())); }
    let suites = context.host.suites()?;
    let mut cases = Vec::new();
    for (selection_index, selection) in plan.selections.iter().enumerate() {
        let matches: Vec<_> = suites.iter().filter(|(_, suite)| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.suite_id)).collect();
        if matches.len() != 1 { return Err(Error::Invalid("Plan Selection: Choose One Unambiguous Connected Suite".into())); }
        let (plugin, suite) = matches[0];
        let template = suite.templates.iter().find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.template_id)).ok_or_else(|| Error::NotFound("Plan Selection: Choose a Template in the Selected Suite".into()))?;
        let parameters = expand(template, selection, cap)?;
        if cases.len().checked_add(parameters.len()).is_none_or(|count| count > cap) { return Err(Error::Invalid("Plan Selection: Reduce the Expanded Case Count".into())); }
        for parameters in parameters {
            let origin = context.host.materialize(plugin, &selection.template_id, parameters).await?;
            cases.push(v1::PlannedCase { suite_id: selection.suite_id.clone(), template_id: selection.template_id.clone(), metadata: template.metadata.clone(), ordinal: cases.len() as u64, selection_index: selection_index as u64, origin: Some(origin), ..Default::default() });
        }
    }
    let reply = v1::PreviewSelectionReply { cases, ..Default::default() };
    if reply.encoded_len() > context.settings.message_bytes { return Err(Error::Invalid("Plan Selection: Reduce the Plan to Fit the Configured Reply Bound".into())); }
    Ok(reply)
}

/* === Private Functions === */

fn expand(template: &v1::TestCaseTemplate, selection: &v1::RegisteredSelection, cap: usize) -> Result<Vec<HashMap<String, v1::Value>>> {
    let mut supplied = HashMap::new();
    for parameter in &selection.parameters {
        if !template.parameters.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == parameter.key)) { return Err(Error::Invalid(format!("Plan Selection: Remove Unknown Parameter {}", parameter.key))); }
        let values = parameter.value.as_ref().filter(|values| !values.values.is_empty()).ok_or_else(|| Error::Invalid("Plan Selection: Supply Nonempty Parameter Values".into()))?.values.clone();
        if supplied.insert(parameter.key.clone(), values).is_some() { return Err(Error::Invalid("Plan Selection: Use Unique Parameter Names".into())); }
    }
    let mut resolved = supplied.clone();
    let mut unresolved: Vec<_> = template.parameters.iter().filter(|parameter| parameter.metadata.as_ref().is_some_and(|metadata| !resolved.contains_key(&metadata.identifier))).collect();
    while !unresolved.is_empty() {
        let before = unresolved.len();
        let mut remaining = Vec::new();
        for parameter in unresolved {
            let name = &parameter.metadata.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Declare Named Parameters".into()))?.identifier;
            let Some(expression) = &parameter.expression else {
                if parameter.required { return Err(Error::Invalid(format!("Plan Selection: Supply Required Parameter {name}"))); }
                continue;
            };
            match expression_values(expression, &resolved, cap)? { Some(values) => { resolved.insert(name.clone(), values); }, None => remaining.push(parameter) }
        }
        if remaining.len() == before { return Err(Error::Invalid("Plan Selection: Resolve Missing or Cyclic Parameter References".into())); }
        unresolved = remaining;
    }
    let mut dimensions: Vec<(String, Vec<HashMap<String, v1::Value>>)> = Vec::new();
    for parameter in &template.parameters {
        let name = &parameter.metadata.as_ref().unwrap().identifier;
        let Some(mut values) = resolved.remove(name) else { continue; };
        if parameter.r#type == v1::ValueType::List as i32 && supplied.contains_key(name) && !matches!(values.as_slice(), [v1::Value { kind: Some(v1::value::Kind::List(_)) }]) { values = vec![v1::Value { kind: Some(v1::value::Kind::List(v1::ValueList { values })) }]; }
        if values.is_empty() || values.len() > cap { return Err(Error::Invalid(format!("Plan Selection: Supply Bounded Nonempty Values for {name}"))); }
        for value in &values { validate_parameter(parameter, value)?; }
        let expansion = parameter.expansion.as_ref().and_then(|expansion| expansion.kind.as_ref());
        if matches!(expansion, Some(v1::parameter_expansion::Kind::Single(true))) && values.len() != 1 { return Err(Error::Invalid(format!("Plan Selection: Supply One Value for {name}"))); }
        let group = match expansion { Some(v1::parameter_expansion::Kind::Zip(group)) => format!("zip:{group}"), _ => format!("parameter:{name}") };
        if let Some((_, rows)) = dimensions.iter_mut().find(|(key, _)| key == &group) {
            if rows.len() != values.len() { return Err(Error::Invalid("Plan Selection: Use Equal Lengths in Each Zip Group".into())); }
            for (row, value) in rows.iter_mut().zip(values) { row.insert(name.clone(), value); }
        } else { dimensions.push((group, values.into_iter().map(|value| HashMap::from([(name.clone(), value)])).collect())); }
    }
    let mut combinations = vec![HashMap::new()];
    for (_, rows) in dimensions {
        if combinations.len().checked_mul(rows.len()).is_none_or(|count| count > cap) { return Err(Error::Invalid("Plan Selection: Reduce the Cartesian Expansion".into())); }
        let mut expanded = Vec::new();
        for combination in combinations { for row in &rows { let mut combined = combination.clone(); combined.extend(row.clone()); expanded.push(combined); } }
        combinations = expanded;
    }
    let mut file_names = HashSet::new();
    let mut files = HashMap::new();
    for file in &selection.files {
        if !file_names.insert(&file.key) || !template.files.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == file.key)) { return Err(Error::Invalid("Plan Selection: Use Unique Declared File Parameters".into())); }
        let value = match file.value.as_ref().and_then(|file| file.kind.as_ref()).and_then(|kind| kind.kind.as_ref()) { Some(v1::selected_file_kind::Kind::Value(value)) => Some(value.clone()), Some(v1::selected_file_kind::Kind::Clear(_)) => None, None => return Err(Error::Invalid("Plan Selection: Select or Explicitly Clear Each File".into())) };
        files.insert(file.key.clone(), value);
    }
    for field in &template.files {
        let name = &field.metadata.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Name Each File Parameter".into()))?.identifier;
        let value = files.get(name).cloned().unwrap_or_else(|| field.default_value.clone());
        if field.required && value.as_ref().is_none_or(|value| value.is_empty()) { return Err(Error::Invalid(format!("Plan Selection: Supply File Parameter {name}"))); }
        if let Some(value) = value { for combination in &mut combinations { combination.insert(name.clone(), v1::Value { kind: Some(v1::value::Kind::Text(value.clone())) }); } }
    }
    Ok(combinations)
}

fn validate_parameter(parameter: &v1::Parameter, value: &v1::Value) -> Result<()> {
    let name = &parameter.metadata.as_ref().unwrap().identifier;
    if !super::prompt_validation::type_matches(parameter.r#type, value) || (!parameter.choices.is_empty() && !parameter.choices.contains(value)) { return Err(Error::Invalid(format!("Plan Selection: Use the Declared Type and Choices for {name}"))); }
    if let Some(v1::value::Kind::List(list)) = &value.kind {
        if parameter.min_items.is_some_and(|min| list.values.len() < min as usize) || parameter.max_items.is_some_and(|max| list.values.len() > max as usize) || list.values.iter().any(|value| !super::prompt_validation::type_matches(parameter.element_type, value)) { return Err(Error::Invalid(format!("Plan Selection: Match the Declared List Shape for {name}"))); }
    }
    let number = numeric(value);
    if number.is_some_and(|number| parameter.min.is_some_and(|min| number < min) || parameter.max.is_some_and(|max| number > max)) { return Err(Error::Invalid(format!("Plan Selection: Use the Declared Range for {name}"))); }
    Ok(())
}

fn numeric(value: &v1::Value) -> Option<f64> {
    match value.kind { Some(v1::value::Kind::Integer(value)) => Some(value as f64), Some(v1::value::Kind::UnsignedInteger(value)) => Some(value as f64), Some(v1::value::Kind::Number(value)) => Some(value), _ => None }
}

fn expression_values(expression: &v1::ParameterExpression, resolved: &HashMap<String, Vec<v1::Value>>, cap: usize) -> Result<Option<Vec<v1::Value>>> {
    use v1::parameter_expression::Kind;
    use v1::value::Kind as Value;
    let values = match expression.kind.as_ref() {
        Some(Kind::Literal(value)) => vec![value.clone()],
        Some(Kind::Reference(name)) => return Ok(resolved.get(name).cloned()),
        Some(Kind::Array(array)) => { let mut values = Vec::new(); for item in &array.values { let Some(items) = expression_values(item, resolved, cap)? else { return Ok(None); }; values.extend(items); if values.len() > cap { return Err(Error::Invalid("Plan Selection: Reduce the Parameter Array".into())); } } values },
        Some(Kind::Range(range)) => { if range.step == 0 { return Err(Error::Invalid("Plan Selection: Use a Nonzero Integer Range Step".into())); } let mut values = Vec::new(); let mut value = range.start; while if range.step > 0 { value < range.stop } else { value > range.stop } { if values.len() >= cap { return Err(Error::Invalid("Plan Selection: Reduce the Integer Range".into())); } values.push(v1::Value { kind: Some(Value::Integer(value)) }); value = value.checked_add(range.step).ok_or_else(|| Error::Invalid("Plan Selection: Keep Integer Ranges Within Their Type".into()))?; } values },
        Some(Kind::Arange(range)) => { if !range.start.is_finite() || !range.stop.is_finite() || !range.step.is_finite() || range.step == 0.0 { return Err(Error::Invalid("Plan Selection: Use Finite Ranges with a Nonzero Step".into())); } let mut values = Vec::new(); for index in 0..=cap { let value = range.start + range.step * index as f64; if if range.step > 0.0 { value >= range.stop } else { value <= range.stop } { break; } if index == cap { return Err(Error::Invalid("Plan Selection: Reduce the Number Range".into())); } values.push(v1::Value { kind: Some(Value::Number(value)) }); } values },
        Some(Kind::Linspace(range)) => { if range.count == 0 || range.count > cap as u64 || !range.start.is_finite() || !range.stop.is_finite() { return Err(Error::Invalid("Plan Selection: Use a Bounded Finite Linear Space".into())); } (0..range.count).map(|index| v1::Value { kind: Some(Value::Number(if range.count == 1 { range.start } else { range.start + (range.stop - range.start) * index as f64 / (range.count - 1) as f64 })) }).collect() },
        Some(Kind::DynamicDefault(default)) => { let fallback = default.fallback.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply a Fallback for an Unavailable Operator Default".into()))?; return expression_values(fallback, resolved, cap); },
        Some(Kind::Unary(unary)) => { if unary.operator != v1::ParameterUnaryOperator::Negate as i32 { return Err(Error::Invalid("Plan Selection: Declare a Valid Unary Operator".into())); } let child = unary.value.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Unary Operand".into()))?; let Some(values) = expression_values(child, resolved, cap)? else { return Ok(None); }; values.into_iter().map(|value| match value.kind { Some(Value::Integer(value)) => value.checked_neg().map(|value| v1::Value { kind: Some(Value::Integer(value)) }).ok_or_else(|| Error::Invalid("Plan Selection: Avoid Integer Overflow".into())), _ => numeric(&value).map(|value| v1::Value { kind: Some(Value::Number(-value)) }).ok_or_else(|| Error::Invalid("Plan Selection: Use Numeric Unary Operands".into())) }).collect::<Result<_>>()? },
        Some(Kind::Binary(binary)) => {
            let left = binary.left.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Left Operand".into()))?;
            let right = binary.right.as_ref().ok_or_else(|| Error::Invalid("Plan Selection: Supply the Right Operand".into()))?;
            let (Some(left), Some(right)) = (expression_values(left, resolved, cap)?, expression_values(right, resolved, cap)?) else { return Ok(None); };
            if left.len().checked_mul(right.len()).is_none_or(|count| count > cap) { return Err(Error::Invalid("Plan Selection: Reduce the Arithmetic Expansion".into())); }
            let mut values = Vec::new();
            for left in &left { for right in &right {
                let (Some(left), Some(right)) = (numeric(left), numeric(right)) else { return Err(Error::Invalid("Plan Selection: Use Numeric Arithmetic Operands".into())); };
                let value = match v1::ParameterBinaryOperator::try_from(binary.operator).unwrap_or_default() { v1::ParameterBinaryOperator::Add => left + right, v1::ParameterBinaryOperator::Subtract => left - right, v1::ParameterBinaryOperator::Multiply => left * right, v1::ParameterBinaryOperator::Divide if right != 0.0 => left / right, _ => return Err(Error::Invalid("Plan Selection: Use Valid Arithmetic Without Division by Zero".into())) };
                if !value.is_finite() { return Err(Error::Invalid("Plan Selection: Keep Arithmetic Results Finite".into())); }
                values.push(v1::Value { kind: Some(Value::Number(value)) });
            } }
            values
        },
        None => return Err(Error::Invalid("Plan Selection: Declare a Parameter Expression".into())),
    };
    if values.len() > cap { return Err(Error::Invalid("Plan Selection: Reduce the Parameter Expansion".into())); }
    Ok(Some(values))
}

/* === Tests === */
