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
use std::collections::BTreeSet;

// Third Party Imports
use harmos::operations::{NativeError, OperationManager};
use harmos::sidecar::operations::Definition;

// Local Imports
use crate::InstalledPlugin;
use crate::domains::{
    ActionDispatcher, ActionDispatcherError, ActionDispatcherFuture, OptionProviderOption, OptionProviderSource, OptionsResponse,
    StatusResponse,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub(crate) struct Bindings {
    options: BTreeSet<String>,
    analyses: BTreeSet<String>,
    status: Option<String>,
    discoveries: Vec<(String, String)>,
}

pub(crate) struct Dispatcher {
    bindings: Bindings,
    calls: OperationManager,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bindings {
    pub(crate) fn inspect(plugin: &InstalledPlugin, operations: &[Definition]) -> Result<Self, String> {
        let inventory: BTreeSet<_> = operations.iter().map(|operation| operation.id.as_str()).collect();
        let mut claimed = BTreeSet::new();
        let mut claim = |method: &str| -> Result<(), String> {
            if method.starts_with("harmos.") || method.starts_with("dokime.") {
                return Err(format!("owner `{}` binds reserved operation `{method}`", plugin.source));
            }
            inventory.get(method).ok_or_else(|| format!("owner `{}` binds undeclared operation `{method}`", plugin.source))?;
            if !claimed.insert(method.to_owned()) {
                return Err(format!("owner `{}` binds operation `{method}` more than once", plugin.source));
            }
            Ok(())
        };
        let mut bindings = Self::default();
        for provider in &plugin.option_providers {
            if let OptionProviderSource::Runtime { entrypoint } = &provider.source {
                claim(&entrypoint.name)?;
                bindings.options.insert(entrypoint.name.clone());
            }
        }
        for analysis in &plugin.analyses {
            analysis.check()?;
            claim(&analysis.id)?;
            bindings.analyses.insert(analysis.id.clone());
        }
        if let Some(entrypoint) = &plugin.settings_status {
            claim(&entrypoint.name)?;
            bindings.status = Some(entrypoint.name.clone());
        }
        for discovery in &plugin.settings_discoveries {
            claim(&discovery.operation)?;
            bindings.discoveries.push((discovery.field.clone(), discovery.operation.clone()));
        }
        Ok(bindings)
    }
    pub(crate) fn attach(self, calls: OperationManager) -> Dispatcher {
        Dispatcher { bindings: self, calls }
    }
}

impl Dispatcher {
    pub(crate) async fn discover(&self, saved: &BTreeSet<String>) -> crate::settings::SettingValues {
        let mut found = crate::settings::SettingValues::new();
        for (field, operation) in &self.bindings.discoveries {
            if saved.contains(field) {
                continue;
            }
            let answered = self
                .call::<(), crate::plugin::settings::Discovered>(operation, ())
                .await
                .map_err(|error| error.to_string())
                .and_then(|reply| reply.value().map_err(|error| error.to_string()));
            match answered {
                Ok(value) => {
                    found.insert(field.clone(), value);
                }
                Err(error) => tracing::warn!(
                    setting = %field,
                    %operation,
                    %error,
                    "a discovery operation did not answer, so the setting stays empty"
                ),
            }
        }
        found
    }

    async fn call<Q: harmos::Message, T: harmos::Message>(&self, method: &str, request: Q) -> Result<T, ActionDispatcherError> {
        let bytes = self.calls.call_encoded(method, request.encode_to_vec()).await.map_err(dispatch_error)?;

        T::decode(bytes.as_slice()).map_err(|error| ActionDispatcherError::Malformed(error.to_string()))
    }
}

impl ActionDispatcher for Dispatcher {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, StatusResponse> {
        Box::pin(async move {
            let method = self.bindings.status.as_ref().ok_or_else(|| ActionDispatcherError::UnknownEntrypoint("status".into()))?;

            self.call(method, ()).await
        })
    }
    fn analyze<'a>(
        &'a self, entrypoint: &'a str, request: crate::analysis::Request,
    ) -> ActionDispatcherFuture<'a, crate::analysis::Response> {
        use harmos::Message;
        let admitted = if self.bindings.analyses.contains(entrypoint) {
            self.calls.submit_encoded(entrypoint, request.encode_to_vec()).map_err(dispatch_error)
        } else {
            Err(ActionDispatcherError::UnknownEntrypoint(entrypoint.into()))
        };
        Box::pin(async move {
            let bytes = admitted?.await.map_err(dispatch_error)?;
            crate::analysis::Response::decode(bytes.as_slice()).map_err(|error| ActionDispatcherError::Malformed(error.to_string()))
        })
    }

    fn query_options<'a>(&'a self, entrypoint: &'a str) -> ActionDispatcherFuture<'a, Vec<OptionProviderOption>> {
        Box::pin(async move {
            self.bindings.options.get(entrypoint).ok_or_else(|| ActionDispatcherError::UnknownEntrypoint(entrypoint.into()))?;
            let offered: OptionsResponse = self.call(entrypoint, ()).await?;

            Ok(offered.into_options())
        })
    }
}

/* === Public Functions === */

pub(crate) fn validate_analyses(
    artifact: &InstalledPlugin, contracts: &[harmos::operations::Contract], settings: &harmos::config::SchemaSettings,
) -> Result<(), String> {
    use harmos::schema::Described;
    for analysis in &artifact.analyses {
        let contract = contracts
            .iter()
            .find(|contract| contract.operation().id.as_str() == analysis.id)
            .ok_or_else(|| format!("analysis {} has no served wire contract", analysis.id))?;
        let schema = harmos::schema::graph::Schema::from_declarations(
            &[("request", crate::analysis::Request::DEFINITION), ("response", crate::analysis::Response::DEFINITION)],
            settings,
        )
        .map_err(|error| error.to_string())?;
        let expected = harmos::operations::Contract::new_with_settings(
            contract.operation().clone(),
            analysis.version,
            harmos::operations::Codec::Postcard,
            schema,
            settings,
        )
        .map_err(|error| error.to_string())?;
        if contract != &expected {
            return Err(format!("analysis {} has an incompatible callback contract", analysis.id));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn dispatch_error(error: harmos::Error) -> ActionDispatcherError {
    match error {
        harmos::Error::OperationRefused(harmos::Refusal::TimedOut) => ActionDispatcherError::Deadline,
        harmos::Error::OperationRefused(_) | harmos::Error::Closing { .. } | harmos::Error::NotRegistered { domain: "operation", .. } => {
            ActionDispatcherError::Unavailable
        }
        harmos::Error::OperationCaller { source } => caller_error(source),
        harmos::Error::OperationFailed { ref source, .. } => {
            if matches!(source.downcast_ref::<NativeError>(), Some(NativeError::Unavailable)) {
                return ActionDispatcherError::Unavailable;
            }
            let refusal = source.downcast_ref::<harmos::sidecar::Refusal>().cloned();
            let refusal = refusal.unwrap_or_else(|| harmos::sidecar::Refusal::of(error));
            ActionDispatcherError::Refused {
                code: refusal.code(),
                message: refusal.message().to_owned(),
            }
        }
        harmos::Error::OperationPayload { source, .. } => ActionDispatcherError::Malformed(source.to_string()),
        harmos::Error::OperationUnauthorized { .. } | harmos::Error::OperationLaneCycle { .. } => {
            let refusal = harmos::sidecar::Refusal::new(-32_604, error);
            ActionDispatcherError::Refused {
                code: refusal.code(),
                message: refusal.message().to_owned(),
            }
        }
        error => {
            let refusal = harmos::sidecar::Refusal::of(error);
            ActionDispatcherError::Refused {
                code: refusal.code(),
                message: refusal.message().to_owned(),
            }
        }
    }
}

fn caller_error(error: harmos::plugin::DispatchError) -> ActionDispatcherError {
    match error {
        harmos::plugin::DispatchError::Unavailable => ActionDispatcherError::Unavailable,
        harmos::plugin::DispatchError::Deadline => ActionDispatcherError::Deadline,
        error => {
            let refusal = error.into_refusal();
            ActionDispatcherError::Refused {
                code: refusal.code(),
                message: refusal.message().to_owned(),
            }
        }
    }
}

/* === Tests === */
