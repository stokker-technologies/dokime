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
use std::{future::Future, pin::Pin};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{ActionDispatcherError, OptionProviderOption};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

json_message!(StatusResponse);

json_message!(OptionsResponse);

/* === Type Aliases === */

pub type ActionDispatcherFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ActionDispatcherError>> + Send + 'a>>;

/* === Structs === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DispatchContext {
    pub credentials_configured: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub state: String,
    pub fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OptionsResponse {
    options: Vec<OptionProviderOption>,
}

/* === Enums === */

/* === Traits === */

pub trait ActionDispatcher: Send + Sync + 'static {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, StatusResponse>;

    fn analyze<'a>(
        &'a self, entrypoint: &'a str, _request: crate::analysis::Request,
    ) -> ActionDispatcherFuture<'a, crate::analysis::Response> {
        Box::pin(async move { Err(ActionDispatcherError::UnknownEntrypoint(entrypoint.into())) })
    }

    fn query_options<'a>(&'a self, entrypoint: &'a str) -> ActionDispatcherFuture<'a, Vec<OptionProviderOption>>;
}

/* === Implementations === */

impl StatusResponse {
    pub fn ready() -> Self {
        Self::new("ready")
    }

    pub fn new(state: impl Into<String>) -> Self {
        Self { state: state.into(), fields: BTreeMap::new() }
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(name.into(), value.into());
        self
    }
}

impl OptionsResponse {
    pub fn of(options: impl IntoIterator<Item = OptionProviderOption>) -> Self {
        Self { options: options.into_iter().collect() }
    }

    pub fn options(&self) -> &[OptionProviderOption] {
        &self.options
    }

    pub fn into_options(self) -> Vec<OptionProviderOption> {
        self.options
    }
}

impl FromIterator<OptionProviderOption> for OptionsResponse {
    fn from_iter<I: IntoIterator<Item = OptionProviderOption>>(options: I) -> Self {
        Self::of(options)
    }
}

impl From<crate::Error> for ActionDispatcherError {
    fn from(error: crate::Error) -> Self {
        Self::Refused { code: -32000, message: crate::diagnostics::redacted(&error.to_string()) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
