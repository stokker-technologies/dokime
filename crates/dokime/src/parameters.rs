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
use std::fmt::Display;
// Third Party Imports

// Local Imports
use crate::core::Value;
use crate::definition::Parameter;
use crate::expressions::DataExpression;
use crate::values::FieldValue;

/* === Constants === */
const CHOICE_LABELS: &str = "choice_labels";

/* === Macros === */
macro_rules! carried {
    ($($type:ty),+ $(,)?) => {
        $(impl ParameterValue for $type {
            type Base = Self;

            fn refine(base: Self) -> Result<Self, String> {
                Ok(base)
            }
        })+
    };
}

carried!(bool, f32, f64, i32, i64, u16, u32, u64, String);

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterChoice {
    pub value: Value,
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterHandle {
    name: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
}

/* === Enums === */

/* === Traits === */
pub trait FromParameters: Send + Sync + Sized + 'static {
    fn declaration() -> Vec<Parameter>;

    fn from_parameters(values: &BTreeMap<String, Value>) -> crate::Result<Self>;
}

pub trait ParameterValue: Send + Sync + Sized + 'static {
    type Base: FieldValue;

    fn choices() -> Option<Vec<ParameterChoice>> {
        None
    }

    fn refine(base: Self::Base) -> Result<Self, String>;
}

/* === Implementations === */
impl ParameterChoice {
    pub fn new(value: impl Into<Value>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

impl ParameterHandle {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn expression(&self) -> DataExpression {
        crate::expressions::parameter(self.name)
    }
}

impl From<ParameterHandle> for DataExpression {
    fn from(handle: ParameterHandle) -> Self {
        handle.expression()
    }
}

impl Display for ParameterHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name)
    }
}

impl FromParameters for () {
    fn declaration() -> Vec<Parameter> {
        Vec::new()
    }

    fn from_parameters(_values: &BTreeMap<String, Value>) -> crate::Result<Self> {
        Ok(())
    }
}

/* === Public Functions === */
pub fn refined<B, T>(base: B) -> Result<T, String>
where
    T: TryFrom<B>,
    T::Error: Display,
{
    T::try_from(base).map_err(|error| error.to_string())
}

pub fn declare<T: ParameterValue>(identifier: &str, label: &str, required: bool) -> Parameter {
    let declared = Parameter::new(identifier, label, <T::Base as FieldValue>::VALUE_TYPE);

    with_choices(optionality(declared, required), T::choices())
}

pub fn declare_list<T: ParameterValue>(
    identifier: &str,
    label: &str,
    required: bool,
    bounds: Bounds,
) -> Parameter {
    let declared = Parameter::list(identifier, label, <T::Base as FieldValue>::VALUE_TYPE);
    let declared = match bounds.min_items {
        Some(fewest) => declared.min_items(fewest),
        None => declared,
    };
    let declared = match bounds.max_items {
        Some(most) => declared.max_items(most),
        None => declared,
    };

    with_choices(optionality(declared, required), T::choices())
}

pub fn scalar<T: ParameterValue>(
    values: &BTreeMap<String, Value>,
    name: &str,
    bounds: Bounds,
) -> crate::Result<T> {
    let value = present(values, name)?;
    bounded(name, value, bounds)?;
    refine::<T>(name, Some(value))
}

pub fn optional<T: ParameterValue>(
    values: &BTreeMap<String, Value>,
    name: &str,
    bounds: Bounds,
) -> crate::Result<Option<T>> {
    match present(values, name)? {
        Value::Null => Ok(None),
        value => {
            bounded(name, value, bounds)?;
            refine::<T>(name, Some(value)).map(Some)
        }
    }
}

pub fn list<T: ParameterValue>(
    values: &BTreeMap<String, Value>,
    name: &str,
    bounds: Bounds,
) -> crate::Result<Vec<T>> {
    items::<T>(name, present(values, name)?, bounds)
}

pub fn optional_list<T: ParameterValue>(
    values: &BTreeMap<String, Value>,
    name: &str,
    bounds: Bounds,
) -> crate::Result<Option<Vec<T>>> {
    match present(values, name)? {
        Value::Null => Ok(None),
        value => items::<T>(name, value, bounds).map(Some),
    }
}

/* === Private Functions === */
fn refused(parameter: impl Into<String>, reason: impl Into<String>) -> crate::Error {
    crate::Error::Parameter {
        name: parameter.into(),
        reason: reason.into(),
    }
}

fn optionality(declared: Parameter, required: bool) -> Parameter {
    if required {
        declared
    } else {
        declared.optional()
    }
}

fn with_choices(declared: Parameter, choices: Option<Vec<ParameterChoice>>) -> Parameter {
    let Some(choices) = choices else {
        return declared;
    };
    let labels = Value::Object(
        choices
            .iter()
            .map(|choice| (label_key(&choice.value), Value::from(choice.label.clone())))
            .collect(),
    );
    let mut declared = declared.choices(choices.into_iter().map(|choice| choice.value));
    declared.extensions.insert(CHOICE_LABELS.to_owned(), labels);

    declared
}

fn label_key(value: &Value) -> String {
    match value {
        Value::Text(text) => text.clone(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Integer(integer) => integer.to_string(),
        Value::Unsigned(integer) => integer.to_string(),
        Value::Number(number) => number.to_string(),
        other => format!("{other:?}"),
    }
}

fn present<'a>(values: &'a BTreeMap<String, Value>, name: &str) -> crate::Result<&'a Value> {
    values
        .get(name)
        .ok_or_else(|| refused(name, "the case resolved no value for it"))
}

fn refine<T: ParameterValue>(name: &str, value: Option<&Value>) -> crate::Result<T> {
    let base = <T::Base as FieldValue>::from_value(value)
        .map_err(|error| refused(name, error.named(name).to_string()))?;

    T::refine(base).map_err(|message| refused(name, message))
}

fn items<T: ParameterValue>(name: &str, value: &Value, bounds: Bounds) -> crate::Result<Vec<T>> {
    let Value::Array(items) = value else {
        return Err(refused(name, "is not a list"));
    };

    items
        .iter()
        .map(|item| {
            bounded(name, item, bounds)?;
            refine::<T>(name, Some(item))
        })
        .collect()
}

fn bounded(name: &str, value: &Value, bounds: Bounds) -> crate::Result<()> {
    let number = match value {
        Value::Number(number) => *number,
        Value::Integer(integer) => *integer as f64,
        Value::Unsigned(integer) => *integer as f64,
        _ => return Ok(()),
    };
    if (bounds.min.is_some() || bounds.max.is_some()) && !number.is_finite() {
        return Err(refused(name, "must be a finite number"));
    }
    if let Some(min) = bounds.min
        && number < min
    {
        return Err(refused(
            name,
            format!("is {number} and the lowest it may be is {min}"),
        ));
    }
    if let Some(max) = bounds.max
        && number > max
    {
        return Err(refused(
            name,
            format!("is {number} and the highest it may be is {max}"),
        ));
    }

    Ok(())
}

/* === Tests === */
