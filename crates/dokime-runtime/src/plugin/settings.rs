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

pub use crate::domains::SettingDiscovery as Discovery;

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use crate::domains::{OptionProviderContribution, OptionProviderSource, PluginEntrypoint};
use crate::parameters::{Bounds, ParameterValue};
use crate::plugin::{Configuration, Secret};
use crate::settings::{Choice, SettingValues, Settings, SettingsEntry};
use crate::values::FieldValue;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoSettings;

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Discovered {
    #[row(description = "The value, JSON-encoded as one [`Value`](dokime_domain::model::values::Value).")]
    value_json: String,
}

/* === Enums === */

/* === Traits === */

pub trait SettingsSection: Sized {
    const LABEL: &'static str;

    const DESCRIPTION: &'static str = "";

    fn entries() -> Vec<SettingsEntry>;

    fn status_operation() -> Option<&'static str>;

    fn option_providers() -> Vec<OptionProviderContribution> {
        Vec::new()
    }

    fn decode(values: &SettingValues, configuration: &Configuration) -> crate::Result<Self>;

    fn discoveries() -> Vec<Discovery> {
        Vec::new()
    }
}

pub trait PluginSettings: Sized + Send + Sync + 'static {
    fn catalog() -> Settings;

    fn status_operation() -> Option<&'static str>;

    fn option_providers() -> Vec<OptionProviderContribution> {
        Vec::new()
    }

    fn decode(values: &SettingValues, configuration: &Configuration) -> crate::Result<Self>;

    fn discoveries() -> Vec<Discovery> {
        Vec::new()
    }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` is linked by a settings section, but its operation answers something other than `{Answered}`",
    label = "this operation answers something else",
    note = "a `status` link answers `StatusResponse`, an `options` link `OptionsResponse`, and a `discovered` link `Discovered`, each for a `()` request"
)]
pub trait Answers<Answered> {}

/* === Implementations === */

impl PluginSettings for NoSettings {
    fn catalog() -> Settings {
        Settings { contributions: Vec::new() }
    }

    fn status_operation() -> Option<&'static str> {
        None
    }

    fn decode(_values: &SettingValues, _configuration: &Configuration) -> crate::Result<Self> {
        Ok(Self)
    }
}

// Hidden from the refusal chain on purpose: without this, a link naming an
// operation that answers something else fails as a projection mismatch and
// rustc reports the projection rather than the message above.
#[diagnostic::do_not_recommend]
impl<R, Answered> Answers<Answered> for R where R: harmos::operations::OperationDeclaration<Response = Answered> {}

impl Discovered {
    pub fn of(value: impl Into<Value>) -> Self {
        Self {
            value_json: serde_json::to_string(&value.into()).expect("a settings value serializes"),
        }
    }

    pub fn value(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.value_json)
    }
}

/* === Public Functions === */

pub fn option_provider(operation: &str) -> OptionProviderContribution {
    OptionProviderContribution {
        metadata: crate::domains::authoring::metadata(operation, &titled(operation), "Offered by this plugin while it is running."),
        input_schema: None,
        permissions: Vec::new(),
        source: OptionProviderSource::Runtime {
            entrypoint: PluginEntrypoint { name: operation.to_owned() },
        },
    }
}

pub const fn answers<R: Answers<Answered>, Answered>() {}

pub fn operation_of<R: harmos::operations::OperationDeclaration>() -> &'static str {
    R::ID
}

pub fn discovery(field: &str, operation: &str) -> Discovery {
    Discovery {
        field: field.to_owned(),
        operation: operation.to_owned(),
    }
}

pub fn choices<T: ParameterValue>() -> Vec<Choice> {
    <T as ParameterValue>::choices()
        .unwrap_or_default()
        .into_iter()
        .map(|choice| Choice {
            value: text(choice.value),
            title: choice.label,
        })
        .collect()
}

pub fn selected<T: FieldValue>(default: T) -> String {
    text(<T as FieldValue>::into_value(default))
}

pub fn first_choice<T: ParameterValue>() -> String {
    choices::<T>().into_iter().next().map(|choice| choice.value).unwrap_or_default()
}

pub fn written_path(default: impl AsRef<std::path::Path>) -> String {
    default.as_ref().display().to_string()
}

pub fn numbers<T: ParameterValue>(values: &SettingValues, id: &str) -> SettingValues {
    if <<T as ParameterValue>::Base as FieldValue>::VALUE_TYPE != dokime_domain::model::values::ValueType::Integer {
        return values.clone();
    }

    let mut restored = values.clone();
    if let Some(value) = restored.get_mut(id) {
        *value = integral(value.clone());
    }

    restored
}

pub fn optional_text(values: &SettingValues, id: &str) -> crate::Result<Option<String>> {
    Ok(crate::parameters::optional::<String>(values, id, Bounds::default())?.filter(|written| !written.is_empty()))
}

pub fn path(values: &SettingValues, id: &str) -> crate::Result<PathBuf> {
    Ok(PathBuf::from(crate::parameters::scalar::<String>(values, id, Bounds::default())?))
}

pub fn optional_path(values: &SettingValues, id: &str) -> crate::Result<Option<PathBuf>> {
    Ok(crate::parameters::optional::<String>(values, id, Bounds::default())?.filter(|written| !written.is_empty()).map(PathBuf::from))
}

pub fn secret(configuration: &Configuration, id: &str) -> crate::Result<Secret> {
    configuration.secret(id)
}

pub fn bound_secret(configuration: &Configuration, id: &str) -> Option<Secret> {
    configuration.secret(id).ok()
}

/* === Private Functions === */

fn titled(id: &str) -> String {
    id.split(['.', '-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();

            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn integral(value: Value) -> Value {
    match value {
        Value::Number(number) if number.fract() == 0.0 && number >= i64::MIN as f64 && number <= i64::MAX as f64 => {
            Value::Integer(number as i64)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(integral).collect()),
        other => other,
    }
}

fn text(value: Value) -> String {
    match value {
        Value::Text(text) => text,
        _ => String::new(),
    }
}

/* === Tests === */
