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
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{
        PyAny, PyAnyMethods, PyBool, PyDict, PyDictMethods, PyFloat, PyInt, PyList, PyListMethods,
        PyString, PyTuple, PyTupleMethods,
    },
};
use serde_json::{Map, Number, Value as JsonValue};
// Local Imports
use crate::{pyserde, to_value_error};
use dokime_app::core::{Identifier, MeasurementUnit, Metadata, Value, ValueType};
use dokime_app::definition::{
    DynamicDefaultKey, ParameterDefinition, ParameterExpr, ParameterGroup, ParameterList,
    ParameterSelection,
};
use dokime_app::document::{AxisScale, PlotStyle, PlotType};
use dokime_app::records::{RecordDisplay, RecordIconKey, RecordSeverity, RecordTone};
use dokime_app::telemetry::TelemetryRole;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn parameter_expression(
    metadata: &Metadata,
    default: Option<Value>,
    values: Vec<Value>,
    attributes: &BTreeMap<String, Value>,
) -> PyResult<ParameterExpr> {
    if let Some(expression) = attributes.get("expression") {
        let expression = authored_expression_contract(expression)?;
        return serde_json::from_value::<ParameterExpr>(expression).map_err(to_value_error);
    }

    let fallback = default
        .clone()
        .map(|value| Box::new(ParameterExpr::Literal { value }));

    if matches!(attributes.get("dynamic_default"), Some(Value::Bool(true)))
        && let Some(key) = dynamic_default_key(metadata.identifier.as_str())
    {
        return Ok(ParameterExpr::DynamicDefault { key, fallback });
    }

    if !values.is_empty() {
        return Ok(ParameterExpr::Array {
            values: values
                .into_iter()
                .map(|value| ParameterExpr::Literal { value })
                .collect(),
        });
    }

    Ok(ParameterExpr::Literal {
        value: default.unwrap_or(Value::Null),
    })
}

pub(super) fn parameter_selection(
    attributes: &BTreeMap<String, Value>,
) -> Option<ParameterSelection> {
    let choices = match attributes.get("choices") {
        Some(Value::Array(values)) => values.clone(),
        _ => Vec::new(),
    };

    if choices.is_empty() {
        None
    } else {
        Some(ParameterSelection { choices })
    }
}

pub(super) fn parameter_list(
    attributes: &BTreeMap<String, Value>,
) -> PyResult<Option<ParameterList>> {
    let Some(element_type) = attributes.get("element_type") else {
        if attributes.contains_key("min_items") || attributes.contains_key("max_items") {
            return Err(PyValueError::new_err(
                "list length bounds require an element_type",
            ));
        }
        return Ok(None);
    };
    let Value::Text(element_type) = element_type else {
        return Err(PyValueError::new_err(format!(
            "parameter element_type must be a type name, got {element_type:?}"
        )));
    };

    let element_type = value_type_from_str(element_type)?;
    if !matches!(
        element_type,
        ValueType::Boolean | ValueType::Integer | ValueType::Number | ValueType::Text
    ) {
        return Err(PyValueError::new_err(
            "list elements must be boolean, integer, number, or text",
        ));
    }
    let min_items = item_bound(attributes, "min_items")?;
    let max_items = item_bound(attributes, "max_items")?;
    if matches!((min_items, max_items), (Some(min), Some(max)) if min > max) {
        return Err(PyValueError::new_err("min_items cannot exceed max_items"));
    }
    Ok(Some(ParameterList {
        element_type,
        min_items,
        max_items,
    }))
}

pub(super) fn numeric_bound(
    attributes: &BTreeMap<String, Value>,
    name: &str,
) -> PyResult<Option<f64>> {
    match attributes.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Integer(value)) => Ok(Some(*value as f64)),
        Some(Value::Unsigned(value)) => Ok(Some(*value as f64)),
        Some(Value::Number(value)) if value.is_finite() => Ok(Some(*value)),
        _ => Err(PyValueError::new_err(format!(
            "parameter {name} must be a finite number"
        ))),
    }
}

pub(super) fn parameter_extensions(
    mut attributes: BTreeMap<String, Value>,
    group: Option<String>,
) -> BTreeMap<String, Value> {
    attributes.remove("choices");
    attributes.remove("expansion");
    attributes.remove("element_type");
    attributes.remove("min_items");
    attributes.remove("min");
    attributes.remove("max");
    attributes.remove("max_items");
    attributes.remove("dynamic_default");
    attributes.remove("expression");
    if let Some(group) = group {
        attributes.insert("group_label".to_owned(), Value::from(group));
    }
    attributes
}

pub(super) fn parameter_groups_from_definitions(
    parameters: &[ParameterDefinition],
) -> Vec<ParameterGroup> {
    let mut groups = BTreeMap::<String, ParameterGroup>::new();

    for definition in parameters {
        let Some(group_id) = &definition.group_id else {
            continue;
        };
        let label = match definition.extensions.get("group_label") {
            Some(Value::Text(label)) => label.clone(),
            _ => title_from_identifier(group_id.as_str()),
        };
        groups
            .entry(group_id.to_string())
            .or_insert_with(|| ParameterGroup {
                group_id: group_id.clone(),
                metadata: Metadata::new(group_id.clone(), label, None),
            });
    }

    groups.into_values().collect()
}

pub(super) fn dynamic_default_key(identifier: &str) -> Option<DynamicDefaultKey> {
    match identifier {
        "operator.name" => Some(DynamicDefaultKey::OperatorName),
        "operator.email" => Some(DynamicDefaultKey::OperatorEmail),
        _ => None,
    }
}

pub(super) fn group_identifier(label: &str) -> Result<Identifier, dokime_app::ApiError> {
    Ok(Identifier::new(title_to_identifier(label))?)
}

pub(super) fn title_to_identifier(label: &str) -> String {
    let mut output = String::new();
    let mut previous_separator = false;

    for character in label.trim().chars() {
        let mapped = if character.is_ascii_alphanumeric() {
            character.to_ascii_lowercase()
        } else {
            '.'
        };

        if mapped == '.' {
            if previous_separator {
                continue;
            }
            previous_separator = true;
        } else {
            previous_separator = false;
        }
        output.push(mapped);
    }

    output.trim_matches('.').to_owned()
}

pub(super) fn optional_contract<T>(value: &Bound<'_, PyAny>) -> PyResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    if value.is_none() {
        Ok(None)
    } else {
        pyserde::from_py(value).map(Some)
    }
}

pub(super) fn optional_py_contract<T>(value: &Bound<'_, PyAny>) -> PyResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    if value.is_none() {
        Ok(None)
    } else {
        pyserde::from_py(value).map(Some)
    }
}

pub(super) fn declaration_value_type_from_py(value_type: &Bound<'_, PyAny>) -> PyResult<ValueType> {
    if let Ok(value) = value_type.extract::<String>() {
        return Ok(declaration_value_type_from_str(&value));
    }

    let py = value_type.py();
    let typing = py.import("typing")?;
    let origin = typing.getattr("get_origin")?.call1((value_type,))?;
    if !origin.is_none() {
        let origin_name = py_type_name(&origin).unwrap_or_default();
        if matches!(origin_name.as_str(), "list" | "tuple" | "set") {
            return Ok(ValueType::Array);
        }
        if origin_name == "dict" {
            return Ok(ValueType::Object);
        }

        let args = typing.getattr("get_args")?.call1((value_type,))?;
        let args = args.cast::<PyTuple>()?;
        let non_none_args = args
            .iter()
            .filter(|arg| !is_none_type(arg).unwrap_or(false))
            .collect::<Vec<_>>();
        if non_none_args.len() == 1 {
            return declaration_value_type_from_py(&non_none_args[0]);
        }
    }

    Ok(
        match py_type_name(value_type).unwrap_or_default().as_str() {
            "bool" => ValueType::Boolean,
            "int" => ValueType::Integer,
            "float" => ValueType::Number,
            "datetime" => ValueType::Datetime,
            "list" | "tuple" | "set" => ValueType::Array,
            "dict" => ValueType::Object,
            _ => ValueType::Text,
        },
    )
}

pub(super) fn declaration_value_type_from_str(value: &str) -> ValueType {
    match value.trim().to_ascii_lowercase().as_str() {
        "bool" | "boolean" => ValueType::Boolean,
        "int" | "integer" => ValueType::Integer,
        "float" | "number" => ValueType::Number,
        "datetime" => ValueType::Datetime,
        "path" | "file" => ValueType::File,
        "dict" | "object" => ValueType::Object,
        "list" | "array" => ValueType::Array,
        _ => ValueType::Text,
    }
}

pub(super) fn is_nullable_value_type(value_type: &Bound<'_, PyAny>) -> PyResult<bool> {
    let py = value_type.py();
    let typing = py.import("typing")?;
    let args = typing.getattr("get_args")?.call1((value_type,))?;
    let args = args.cast::<PyTuple>()?;

    Ok(args.iter().any(|arg| is_none_type(&arg).unwrap_or(false)))
}

pub(super) fn is_none_type(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(py_type_name(value)? == "NoneType")
}

pub(super) fn py_type_name(value: &Bound<'_, PyAny>) -> PyResult<String> {
    value.getattr("__name__")?.extract()
}

pub(super) fn value_type_wire_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Boolean => "boolean",
        ValueType::Integer => "integer",
        ValueType::Number => "number",
        ValueType::Text => "text",
        ValueType::Datetime => "datetime",
        ValueType::File => "file",
        ValueType::Object => "object",
        ValueType::Array => "array",
    }
}

pub(super) fn authoring_json_value(value: &Bound<'_, PyAny>) -> PyResult<JsonValue> {
    if value.is_none() {
        return Ok(JsonValue::Null);
    }

    if value.is_exact_instance_of::<PyBool>() {
        return Ok(JsonValue::Bool(value.extract::<bool>()?));
    }

    if value.is_instance_of::<PyInt>() {
        let integer = value.extract::<i64>()?;

        return Ok(JsonValue::Number(Number::from(integer)));
    }

    if value.is_instance_of::<PyFloat>() {
        let number = Number::from_f64(value.extract::<f64>()?)
            .ok_or_else(|| PyValueError::new_err("authoring JSON numbers must be finite"))?;

        return Ok(JsonValue::Number(number));
    }

    if value.is_instance_of::<PyString>() {
        return Ok(JsonValue::String(value.extract::<String>()?));
    }

    if let Ok(dict) = value.cast::<PyDict>() {
        let mut object = Map::new();
        for (key, item) in dict.iter() {
            object.insert(authoring_json_key(&key)?, authoring_json_value(&item)?);
        }

        return Ok(JsonValue::Object(object));
    }

    if let Ok(list) = value.cast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(authoring_json_value(&item)?);
        }

        return Ok(JsonValue::Array(items));
    }

    if let Ok(tuple) = value.cast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(authoring_json_value(&item)?);
        }

        return Ok(JsonValue::Array(items));
    }

    Ok(JsonValue::String(value.str()?.extract()?))
}

pub(super) fn authoring_json_key(key: &Bound<'_, PyAny>) -> PyResult<String> {
    if key.is_instance_of::<PyString>() {
        return key.extract::<String>();
    }

    key.str()?.extract()
}

pub(super) fn canonical_json_with_spacing(value: &JsonValue) -> PyResult<String> {
    canonical_json(value, ", ", ": ")
}

pub(super) fn canonical_json_compact(value: &JsonValue) -> PyResult<String> {
    canonical_json(value, ",", ":")
}

pub(super) fn canonical_json(
    value: &JsonValue,
    item_separator: &str,
    key_separator: &str,
) -> PyResult<String> {
    match value {
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {
            serde_json::to_string(value).map_err(to_value_error)
        }
        JsonValue::Array(items) => {
            let items = items
                .iter()
                .map(|item| canonical_json(item, item_separator, key_separator))
                .collect::<PyResult<Vec<_>>>()?
                .join(item_separator);

            Ok(format!("[{items}]"))
        }
        JsonValue::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);

            let entries = entries
                .into_iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key).map_err(to_value_error)?;
                    let value = canonical_json(value, item_separator, key_separator)?;

                    Ok(format!("{key}{key_separator}{value}"))
                })
                .collect::<PyResult<Vec<_>>>()?
                .join(item_separator);

            Ok(format!("{{{entries}}}"))
        }
    }
}

pub(super) fn hex_lower(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

pub(super) fn value_type_from_str(value: &str) -> PyResult<ValueType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bool" | "boolean" => Ok(ValueType::Boolean),
        "int" | "integer" => Ok(ValueType::Integer),
        "float" | "number" => Ok(ValueType::Number),
        "str" | "string" | "text" => Ok(ValueType::Text),
        "datetime" => Ok(ValueType::Datetime),
        "path" | "file" => Ok(ValueType::File),
        "dict" | "object" => Ok(ValueType::Object),
        "list" | "array" => Ok(ValueType::Array),
        _ => Err(PyValueError::new_err(format!(
            "unsupported contract value type {value:?}"
        ))),
    }
}

pub(super) fn record_severity_from_str(value: &str) -> PyResult<RecordSeverity> {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" => Ok(RecordSeverity::Debug),
        "info" => Ok(RecordSeverity::Info),
        "warning" | "warn" => Ok(RecordSeverity::Warning),
        "error" => Ok(RecordSeverity::Error),
        _ => Err(PyValueError::new_err(format!(
            "unsupported event severity {value:?}"
        ))),
    }
}

pub(super) fn record_display(icon: Option<&str>, tone: Option<&str>) -> PyResult<RecordDisplay> {
    Ok(RecordDisplay {
        icon: icon
            .map(record_icon_key_from_str)
            .transpose()?
            .unwrap_or_default(),
        tone: tone
            .map(record_tone_from_str)
            .transpose()?
            .unwrap_or_default(),
    })
}

pub(super) fn record_icon_key_from_str(value: &str) -> PyResult<RecordIconKey> {
    match value.trim().to_ascii_lowercase().as_str() {
        "activity" => Ok(RecordIconKey::Activity),
        "alert-triangle" | "alert_triangle" => Ok(RecordIconKey::AlertTriangle),
        "archive" => Ok(RecordIconKey::Archive),
        "bell" => Ok(RecordIconKey::Bell),
        "check-circle" | "check_circle" => Ok(RecordIconKey::CheckCircle),
        "circle-dashed" | "circle_dashed" => Ok(RecordIconKey::CircleDashed),
        "clipboard-check" | "clipboard_check" => Ok(RecordIconKey::ClipboardCheck),
        "clock" => Ok(RecordIconKey::Clock),
        "cpu" => Ok(RecordIconKey::Cpu),
        "file-text" | "file_text" => Ok(RecordIconKey::FileText),
        "flag" => Ok(RecordIconKey::Flag),
        "gauge" => Ok(RecordIconKey::Gauge),
        "heart-pulse" | "heart_pulse" => Ok(RecordIconKey::HeartPulse),
        "help-circle" | "help_circle" => Ok(RecordIconKey::HelpCircle),
        "message-square-text" | "message_square_text" => Ok(RecordIconKey::MessageSquareText),
        "package" => Ok(RecordIconKey::Package),
        "plug" => Ok(RecordIconKey::Plug),
        "radio-tower" | "radio_tower" => Ok(RecordIconKey::RadioTower),
        "rotate-ccw" | "rotate_ccw" => Ok(RecordIconKey::RotateCcw),
        "shield-alert" | "shield_alert" => Ok(RecordIconKey::ShieldAlert),
        "target" => Ok(RecordIconKey::Target),
        "timer-reset" | "timer_reset" => Ok(RecordIconKey::TimerReset),
        "wrench" => Ok(RecordIconKey::Wrench),
        "zap" => Ok(RecordIconKey::Zap),
        _ => Err(PyValueError::new_err(format!(
            "unsupported event display icon {value:?}"
        ))),
    }
}

pub(super) fn record_tone_from_str(value: &str) -> PyResult<RecordTone> {
    match value.trim().to_ascii_lowercase().as_str() {
        "accent" => Ok(RecordTone::Accent),
        "danger" | "error" => Ok(RecordTone::Danger),
        "info" => Ok(RecordTone::Info),
        "muted" => Ok(RecordTone::Muted),
        "neutral" => Ok(RecordTone::Neutral),
        "success" => Ok(RecordTone::Success),
        "warning" | "warn" => Ok(RecordTone::Warning),
        _ => Err(PyValueError::new_err(format!(
            "unsupported event display tone {value:?}"
        ))),
    }
}

pub(super) fn telemetry_role_from_str(value: &str) -> PyResult<TelemetryRole> {
    match value.trim().to_ascii_lowercase().as_str() {
        "time" => Err(PyValueError::new_err(
            "unknown telemetry role: time — every row carries one automatic `timestamp` column now",
        )),
        "value" => Ok(TelemetryRole::Value),
        "counter" => Ok(TelemetryRole::Counter),
        "state" => Ok(TelemetryRole::State),
        "distance" => Ok(TelemetryRole::Distance),
        "latency" => Ok(TelemetryRole::Latency),
        "throughput" => Ok(TelemetryRole::Throughput),
        _ => Err(PyValueError::new_err(format!(
            "unsupported telemetry role {value:?}"
        ))),
    }
}

pub(super) fn plot_type_from_str(value: &str) -> PyResult<PlotType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "time_series" => Ok(PlotType::TimeSeries),
        "bar" => Ok(PlotType::Bar),
        "grouped_bar" => Ok(PlotType::GroupedBar),
        "stacked_bar" => Ok(PlotType::StackedBar),
        "pie" => Ok(PlotType::Pie),
        "donut" => Ok(PlotType::Donut),
        "pareto" => Ok(PlotType::Pareto),
        "xy" | "scatter" | "xy_scatter" => Ok(PlotType::XyScatter),
        "xy_line" => Ok(PlotType::XyLine),
        "xy_interval" => Ok(PlotType::XyInterval),
        "trajectory" => Ok(PlotType::Trajectory),
        "heatmap" | "xy_value_heatmap" => Ok(PlotType::XyValueHeatmap),
        "wifi_channel_occupancy" => Ok(PlotType::WifiChannelOccupancy),
        "aggregate_table" => Ok(PlotType::AggregateTable),
        "frequency" | "frequency_series" => Ok(PlotType::FrequencySeries),
        "histogram" => Ok(PlotType::Histogram),
        "box_summary" => Ok(PlotType::BoxSummary),
        _ => Err(PyValueError::new_err(format!(
            "unsupported document plot type {value:?}"
        ))),
    }
}

pub(super) fn plot_style_from_str(value: &str) -> PyResult<PlotStyle> {
    match value.trim().to_ascii_lowercase().as_str() {
        "line" => Ok(PlotStyle::Line),
        "points" => Ok(PlotStyle::Points),
        "line_points" => Ok(PlotStyle::LinePoints),
        "step" => Ok(PlotStyle::Step),
        _ => Err(PyValueError::new_err(format!(
            "unsupported document plot style {value:?}"
        ))),
    }
}

pub(super) fn axis_scale_from_str(value: &str) -> PyResult<AxisScale> {
    match value.trim().to_ascii_lowercase().as_str() {
        "linear" => Ok(AxisScale::Linear),
        "log" => Ok(AxisScale::Log),
        _ => Err(PyValueError::new_err(format!(
            "unsupported document axis scale {value:?}"
        ))),
    }
}

pub(super) fn measurement_unit_from_symbol(symbol: String) -> PyResult<MeasurementUnit> {
    Ok(MeasurementUnit::custom(
        Identifier::new(unit_identifier(&symbol)).map_err(to_value_error)?,
        symbol.clone(),
        symbol,
    ))
}

pub(super) fn unit_identifier(symbol: &str) -> String {
    let suffix = symbol
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();

    if suffix.is_empty() {
        "unit.custom".to_owned()
    } else {
        format!("unit.{suffix}")
    }
}

pub(super) fn identifier_from_label(prefix: &str, label: &str) -> String {
    let suffix = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '.'
            }
        })
        .collect::<String>()
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(".");
    if suffix.is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix}.{suffix}")
    }
}

pub(super) fn title_from_identifier(identifier: &str) -> String {
    identifier
        .chars()
        .map(|character| {
            if matches!(character, '.' | '_' | '-') {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_whitespace()
        .map(title_word)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn title_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_uppercase().collect::<String>() + chars.as_str()
}

pub(super) fn default_role(_name: &str) -> &str {
    "value"
}

/* === Private Functions === */
fn authored_expression_contract(value: &Value) -> PyResult<JsonValue> {
    let Value::Object(fields) = value else {
        return Err(PyValueError::new_err("expected an expression object"));
    };
    let literal = matches!(fields.get("kind"), Some(Value::Text(kind)) if kind == "literal");
    fields
        .iter()
        .map(|(key, value)| {
            let encoded = if literal && key == "value" {
                serde_json::to_value(value).map_err(to_value_error)?
            } else if matches!(key.as_str(), "left" | "right" | "value" | "fallback")
                && matches!(value, Value::Object(_))
            {
                authored_expression_contract(value)?
            } else if key == "values" {
                let Value::Array(values) = value else {
                    return Err(PyValueError::new_err("expected expression array"));
                };
                JsonValue::Array(
                    values
                        .iter()
                        .map(authored_expression_contract)
                        .collect::<PyResult<_>>()?,
                )
            } else {
                value.to_plain_json().map_err(to_value_error)?
            };
            Ok((key.clone(), encoded))
        })
        .collect::<PyResult<Map<String, JsonValue>>>()
        .map(JsonValue::Object)
}

fn item_bound(attributes: &BTreeMap<String, Value>, key: &str) -> PyResult<Option<usize>> {
    match attributes.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Integer(items)) if *items >= 0 => Ok(Some(*items as usize)),
        Some(Value::Unsigned(items)) => usize::try_from(*items).map(Some).map_err(to_value_error),
        Some(value) => Err(PyValueError::new_err(format!(
            "parameter {key} must be a non-negative integer, got {value:?}"
        ))),
    }
}

/* === Tests === */
