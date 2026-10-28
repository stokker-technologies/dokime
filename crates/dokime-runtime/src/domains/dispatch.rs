/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;
use std::{future::Future, pin::Pin};

use serde::{Deserialize, Serialize};

use super::{ActionDispatcherError, OptionProviderOption};
use dokime_domain::model::values::Value;

json_message!(StatusResponse);

json_message!(OptionsResponse);

pub type ActionDispatcherFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ActionDispatcherError>> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DispatchContext {
    pub secrets_configured: bool,
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

pub trait ActionDispatcher: Send + Sync + 'static {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, StatusResponse>;

    fn analyze<'a>(
        &'a self, entrypoint: &'a str, _request: crate::analysis::Request,
    ) -> ActionDispatcherFuture<'a, crate::analysis::Response> {
        Box::pin(async move { Err(ActionDispatcherError::UnknownEntrypoint(entrypoint.into())) })
    }

    fn query_options<'a>(&'a self, entrypoint: &'a str) -> ActionDispatcherFuture<'a, Vec<OptionProviderOption>>;
}

impl StatusResponse {
    pub fn ready() -> Self {
        Self::new("ready")
    }

    pub fn new(state: impl Into<String>) -> Self {
        Self {
            state: state.into(),
            fields: BTreeMap::new(),
        }
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
        Self::Refused {
            code: -32000,
            message: crate::diagnostics::redacted(&error.to_string()),
        }
    }
}
