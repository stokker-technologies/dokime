/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Operation Admission

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;

// Third Party Imports
use harmos_runtime::operations::{ExecutionContext, HostContext, Lane, OperationFactory, ProtobufOperation, Registration, ServingError};
use harmos_runtime::plugin::grpc::CallingPlugin;

// Local Imports
use crate::host::{self, ClientSettings, Context};
use crate::{Error, Resources, Result};

/* === Constants === */

/* === Macros === */

macro_rules! unary {
    ($request:ident, $response:ident, $caller:ident, $package:literal, $service:literal, $method:literal) => {
        pub use native::Operation;
        mod native {
            use super::{$request, $response};
            use harmos_runtime::operations::{
                ExecutionContext, Grpc, HostContext, OperationPolicy, ProtobufOperation, Resource,
                RetryPolicy, ServingError, Visibility,
            };
            use harmos_protocol::policy::UNBOUNDED;
            use std::time::Duration;
            use std::result;
            use crate::native::{Binding, failure};
            pub struct Operation;
            impl ProtobufOperation for Operation {
                type Request = $request;
                type Response = $response;
                type Endpoints = ();
                const ID: &'static str = super::ID;
                const GRPC: Option<Grpc> = Some(Grpc::new($package, $service, $method));
                const POLICY: OperationPolicy = OperationPolicy {
                    timeout: UNBOUNDED,
                    attempt_timeout: UNBOUNDED,
                    cancellation_timeout: Duration::ZERO,
                    retries: RetryPolicy::Never,
                };
                const VISIBILITY: Visibility = Visibility::Public;
                const RESOURCES: &'static [Resource] = &[Resource::of::<Binding>(), Resource::of::<HostContext>()];
                async fn run(
                    execution: ExecutionContext, request: Self::Request, _: &(),
                ) -> result::Result<Self::Response, ServingError> {
                    let context = crate::native::unary!(@context $caller, execution, Self::ID)?;
                    crate::native::unary!(@call $caller, context, request)
                }
            }
        }
    };
    (@context plugin, $execution:ident, $id:expr) => {
        crate::native::plugin_context(&$execution, $id).map_err(failure)
    };
    (@context $caller:ident, $execution:ident, $id:expr) => {
        crate::native::context(&$execution, $id).map_err(failure)
    };
    (@call plugin, $context:ident, $request:ident) => {
        Box::pin(super::handle(&$context, $request)).await.map_err(failure)
    };
    (@call context, $context:ident, $request:ident) => {
        Box::pin(super::handle(&$context, $request)).await.map_err(failure)
    };
    (@call resources, $context:ident, $request:ident) => {
        Box::pin(super::handle($context.caller.as_ref().ok_or(ServingError::Catalog)?, $request)).await.map_err(failure)
    };
}

pub(crate) use unary;

macro_rules! download {
    ($request:ident, $response:ident, $package:literal, $service:literal, $method:literal) => {
        pub use native::Operation;
        mod native {
            use super::{$request, $response};
            use crate::native::{Binding, context, failure};
            use harmos_protocol::policy::UNBOUNDED;
            use harmos_runtime::operations::{
                ExecutionContext, Grpc, HostContext, OperationPolicy, ProtobufOperation, ProtobufOutput, Resource, RetryPolicy,
                ServingError, Visibility,
            };
            use std::result;
            use std::{sync::Arc, time::Duration};
            use tokio_stream::StreamExt;
            pub struct Operation;
            impl ProtobufOperation for Operation {
                type Request = $request;
                type Response = $response;
                type Endpoints = ProtobufOutput<$response>;
                const ID: &'static str = super::ID;
                const GRPC: Option<Grpc> = Some(Grpc::new($package, $service, $method));
                const POLICY: OperationPolicy = OperationPolicy {
                    timeout: UNBOUNDED,
                    attempt_timeout: UNBOUNDED,
                    cancellation_timeout: Duration::ZERO,
                    retries: RetryPolicy::Never,
                };
                const VISIBILITY: Visibility = Visibility::Public;
                const RESOURCES: &'static [Resource] = &[Resource::of::<Binding>(), Resource::of::<HostContext>()];
                async fn run(
                    execution: ExecutionContext, request: Self::Request, output: &Self::Endpoints,
                ) -> result::Result<Self::Response, ServingError> {
                    let context = context(&execution, Self::ID).map_err(failure)?;
                    let mut stream = super::handle(&context, request).await.map_err(failure)?;
                    while let Some(response) = stream.next().await {
                        let response = response.map_err(|source| ServingError::Native(Arc::new(source)))?;
                        output.send(response).await.map_err(|source| ServingError::Native(Arc::new(source)))?;
                    }
                    Ok(Self::Response::default())
                }
            }
        }
    };
}

pub(crate) use download;

/* === Type Aliases === */

type Factory = fn(Duration) -> Result<OperationFactory>;

/* === Structs === */

#[derive(Default)]
pub struct Binding(OnceLock<Weak<Context>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Binding {
    pub fn bind(&self, context: &Arc<Context>) -> Result<()> {
        self.0.set(Arc::downgrade(context)).map_err(|_| Error::Invalid("Bind Native Operations: Assemble the Host Only Once".into()))
    }
}

/* === Public Functions === */

pub fn context(execution: &ExecutionContext, action: &str) -> Result<Context> {
    admitted(execution, action, false)
}

/// The Host Context for an Operation a Plugin May Also Call Outside an Execution, Scoped to It When Inside One.
pub fn plugin_context(execution: &ExecutionContext, action: &str) -> Result<Context> {
    admitted(execution, action, true)
}

pub fn failure(source: Error) -> ServingError {
    tracing::warn!(error = %source, "Host Operation Failed");
    let carried = match &source {
        Error::Caller(failure) => Some(failure),
        Error::Native { source, .. } => source.failure(),
        _ => None,
    };
    if let Some(failure) = carried {
        return ServingError::Handler(failure.clone());
    }
    ServingError::Native(Arc::new(crate::host::status(source)))
}

pub fn factory<O: ProtobufOperation>(timeout: Duration) -> Result<OperationFactory> {
    let factory = OperationFactory::protobuf::<O>().map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    let contract = factory.contract().map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    let mut policy = contract.operation().policy;
    policy.timeout = timeout;
    policy.attempt_timeout = timeout;
    let contract = contract.with_policy(policy).map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    Ok(factory.with_contract(contract))
}

pub fn registration(settings: &ClientSettings) -> Result<Registration> {
    let factories: &[(Factory, Duration)] = &[
        (factory::<crate::plugin_policy::Get>, settings.read_timeout),
        (factory::<crate::plugin_policy::List>, settings.read_timeout),
        (factory::<crate::plugin_policy::Watch>, settings.watch_timeout),
        (factory::<crate::plugin_policy::Create>, settings.mutation_timeout),
        (factory::<crate::plugin_policy::Set>, settings.mutation_timeout),
        (factory::<crate::plugin_policy::Delete>, settings.mutation_timeout),
        (factory::<crate::plugin_policy::Restart>, settings.mutation_timeout),
        (factory::<crate::plugin_policy::Replace>, settings.mutation_timeout),
        (factory::<crate::plugin_policy::SettingsList>, settings.read_timeout),
        (factory::<crate::plugin_policy::SettingsSet>, settings.mutation_timeout),
        (factory::<crate::wrappers::ListenerGet>, settings.read_timeout),
        (factory::<crate::wrappers::ListenerSet>, settings.mutation_timeout),
        (factory::<crate::wrappers::PairingsCreate>, settings.mutation_timeout),
        (factory::<crate::wrappers::PairingsList>, settings.read_timeout),
        (factory::<crate::wrappers::PairingsDelete>, settings.mutation_timeout),
        (factory::<crate::wrappers::CertificatesCreate>, settings.mutation_timeout),
        (factory::<crate::wrappers::StatusGet>, settings.read_timeout),
        (factory::<crate::wrappers::StatusWatch>, settings.watch_timeout),
        (factory::<crate::wrappers::LogsWatch>, settings.watch_timeout),
        (factory::<crate::wrappers::TraceEventsList>, settings.read_timeout),
        (factory::<crate::wrappers::TraceEventsWatch>, settings.watch_timeout),
        (factory::<crate::wrappers::DiagnosticsGet>, settings.read_timeout),
        (factory::<crate::wrappers::OperationsList>, settings.read_timeout),
        (factory::<crate::wrappers::OperationsGet>, settings.read_timeout),
        (factory::<crate::wrappers::InvocationsCreate>, settings.mutation_timeout),
        (factory::<host::backend::appearance::Operation>, settings.read_timeout),
        (factory::<host::backend::environment::Operation>, settings.mutation_timeout),
        (factory::<host::backend::overview::get::Operation>, settings.read_timeout),
        (factory::<host::cases::create::Operation>, settings.mutation_timeout),
        (factory::<host::cases::delete::Operation>, settings.mutation_timeout),
        (factory::<host::cases::get::Operation>, settings.read_timeout),
        (factory::<host::cases::list::Operation>, settings.read_timeout),
        (factory::<host::cases::parameters::Operation>, settings.read_timeout),
        (factory::<host::content::artifacts::get::Operation>, settings.read_timeout),
        (factory::<host::content::artifacts::list::Operation>, settings.read_timeout),
        (factory::<host::content::execution_files::get::Operation>, settings.read_timeout),
        (factory::<host::content::execution_files::list::Operation>, settings.read_timeout),
        (factory::<host::cycles::create::Operation>, settings.mutation_timeout),
        (factory::<host::cycles::get::Operation>, settings.read_timeout),
        (factory::<host::cycles::list::Operation>, settings.read_timeout),
        (factory::<host::cycles::seal::Operation>, settings.watch_timeout),
        (factory::<host::cycles::set::Operation>, settings.mutation_timeout),
        (factory::<host::documents::append_parts::Operation>, settings.mutation_timeout),
        (factory::<host::documents::get::Operation>, settings.read_timeout),
        (factory::<host::evidence::archive::Operation>, settings.read_timeout),
        (factory::<host::evidence::health_events::list::Operation>, settings.read_timeout),
        (factory::<host::evidence::logs::append::Operation>, settings.mutation_timeout),
        (factory::<host::evidence::notes::create::Operation>, settings.mutation_timeout),
        (factory::<host::evidence::notes::get::Operation>, settings.read_timeout),
        (factory::<host::evidence::notes::list::Operation>, settings.read_timeout),
        (factory::<host::evidence::notes::set::Operation>, settings.mutation_timeout),
        (factory::<host::evidence::records::append::Operation>, settings.mutation_timeout),
        (factory::<host::evidence::records::list::Operation>, settings.read_timeout),
        (factory::<host::evidence::samples::list::Operation>, settings.read_timeout),
        (factory::<host::evidence::statistics::Operation>, settings.read_timeout),
        (factory::<host::evidence::traceability::get::Operation>, settings.read_timeout),
        (factory::<host::evidence::traceability::set::Operation>, settings.mutation_timeout),
        (factory::<host::executions::cancel::Operation>, settings.mutation_timeout),
        (factory::<host::executions::commands::create::Operation>, settings.mutation_timeout),
        (factory::<host::executions::commands::get::Operation>, settings.read_timeout),
        (factory::<host::executions::create::Operation>, settings.mutation_timeout),
        (factory::<host::executions::delete::Operation>, settings.mutation_timeout),
        (factory::<host::executions::get::Operation>, settings.read_timeout),
        (factory::<host::executions::list::Operation>, settings.read_timeout),
        (factory::<host::executions::parameters::Operation>, settings.read_timeout),
        (factory::<host::executions::progress::get::Operation>, settings.read_timeout),
        (factory::<host::executions::progress::set::Operation>, settings.mutation_timeout),
        (factory::<host::executions::schemas::get::Operation>, settings.read_timeout),
        (factory::<host::executions::seal::Operation>, settings.mutation_timeout),
        (factory::<host::executions::set::Operation>, settings.mutation_timeout),
        (factory::<host::executions::start::Operation>, settings.mutation_timeout),
        (factory::<host::executions::steps::Operation>, settings.read_timeout),
        (factory::<host::executions::stop::Operation>, settings.mutation_timeout),
        (factory::<host::files::delete::Operation>, settings.mutation_timeout),
        (factory::<host::files::get::Operation>, settings.read_timeout),
        (factory::<host::files::read::Operation>, settings.read_timeout),
        (factory::<host::files::uploads::append::Operation>, settings.mutation_timeout),
        (factory::<host::files::uploads::create::Operation>, settings.mutation_timeout),
        (factory::<host::files::uploads::delete::Operation>, settings.mutation_timeout),
        (factory::<host::files::uploads::get::Operation>, settings.read_timeout),
        (factory::<host::files::uploads::publish::Operation>, settings.mutation_timeout),
        (factory::<host::files::uploads::resume::Operation>, settings.mutation_timeout),
        (factory::<host::plots::settings::get::Operation>, settings.read_timeout),
        (factory::<host::plots::settings::set::Operation>, settings.mutation_timeout),
        (factory::<host::plots::snapshots::list::Operation>, settings.read_timeout),
        (factory::<host::plugins::secrets::list::Operation>, settings.read_timeout),
        (factory::<host::prompt_definitions::get::Operation>, settings.read_timeout),
        (factory::<host::prompt_definitions::list::Operation>, settings.read_timeout),
        (factory::<host::prompts::answers::create::Operation>, settings.mutation_timeout),
        (factory::<host::prompts::create::Operation>, host::prompts::create::Operation::POLICY.timeout),
        (factory::<host::prompts::get::Operation>, settings.read_timeout),
        (factory::<host::prompts::list::Operation>, settings.read_timeout),
        (factory::<host::record_schemas::get::Operation>, settings.read_timeout),
        (factory::<host::record_schemas::list::Operation>, settings.read_timeout),
        (factory::<host::reports::create::Operation>, settings.mutation_timeout),
        (factory::<host::reports::get::Operation>, settings.read_timeout),
        (factory::<host::reports::publish::Operation>, settings.mutation_timeout),
        (factory::<host::secrets::delete::Operation>, settings.mutation_timeout),
        (factory::<host::secrets::export::Operation>, settings.read_timeout),
        (factory::<host::secrets::get::Operation>, settings.read_timeout),
        (factory::<host::secrets::import::Operation>, settings.mutation_timeout),
        (factory::<host::secrets::list::Operation>, settings.read_timeout),
        (factory::<host::secrets::set::Operation>, settings.mutation_timeout),
        (factory::<host::settings::get::Operation>, settings.read_timeout),
        (factory::<host::settings::schema::Operation>, settings.read_timeout),
        (factory::<host::settings::set::Operation>, settings.mutation_timeout),
        (factory::<host::storage::sources::get::Operation>, settings.read_timeout),
        (factory::<host::storage::sources::set::Operation>, settings.mutation_timeout),
        (factory::<host::stream_schemas::get::Operation>, settings.read_timeout),
        (factory::<host::stream_schemas::list::Operation>, settings.read_timeout),
        (factory::<host::suites::get::Operation>, settings.read_timeout),
        (factory::<host::suites::list::Operation>, settings.read_timeout),
        (factory::<host::backend::overview::watch::Operation>, settings.watch_timeout),
        (factory::<host::cases::watch::Operation>, settings.watch_timeout),
        (factory::<host::content::artifacts::watch::Operation>, settings.watch_timeout),
        (factory::<host::content::execution_files::watch::Operation>, settings.watch_timeout),
        (factory::<host::cycles::watch::Operation>, settings.watch_timeout),
        (factory::<host::documents::watch::Operation>, settings.watch_timeout),
        (factory::<host::evidence::health_events::watch::Operation>, settings.watch_timeout),
        (factory::<host::evidence::notes::watch::Operation>, settings.watch_timeout),
        (factory::<host::evidence::records::watch::Operation>, settings.watch_timeout),
        (factory::<host::evidence::samples::watch::Operation>, settings.watch_timeout),
        (factory::<host::evidence::traceability::watch::Operation>, settings.watch_timeout),
        (factory::<host::executions::progress::watch::Operation>, settings.watch_timeout),
        (factory::<host::executions::watch::Operation>, settings.watch_timeout),
        (factory::<host::files::watch::Operation>, settings.watch_timeout),
        (factory::<host::plots::snapshots::watch::Operation>, settings.watch_timeout),
        (factory::<host::prompts::watch::Operation>, settings.watch_timeout),
        (factory::<host::secrets::watch::Operation>, settings.watch_timeout),
        (factory::<host::content::artifacts::create::Operation>, settings.mutation_timeout),
        (factory::<host::files::capture::Operation>, settings.mutation_timeout),
        (factory::<host::files::captures::start::Operation>, settings.mutation_timeout),
        (factory::<host::files::captures::stop::Operation>, settings.mutation_timeout),
        (factory::<host::aggregates::get::Operation>, settings.mutation_timeout),
        (factory::<host::media::captures::create::Operation>, settings.mutation_timeout),
        (factory::<host::media::captures::get::Operation>, settings.mutation_timeout),
        (factory::<host::media::captures::stop::Operation>, settings.mutation_timeout),
        (factory::<host::media::captures::watch::Operation>, settings.watch_timeout),
    ];
    let mut registration = Registration::new(Lane::default());
    for (factory, timeout) in factories {
        registration = registration.entry(factory(*timeout)?);
    }
    Ok(registration)
}

/* === Private Functions === */

fn admitted(execution: &ExecutionContext, action: &str, plugin_wide: bool) -> Result<Context> {
    let host = execution.resource::<HostContext>().map_err(|source| Error::Unavailable(format!("{action}: {source}")))?;
    let binding = execution.resource::<Binding>().map_err(|source| Error::Unavailable(format!("{action}: {source}")))?;
    let mut context = binding
        .0
        .get()
        .and_then(Weak::upgrade)
        .ok_or_else(|| Error::Unavailable(format!("{action}: Assemble the Host Context")))?
        .as_ref()
        .clone();
    let host = host.borrow();
    let plugin = execution.resource::<CallingPlugin>().is_ok();
    if plugin {
        let Some(resources) = host.context().get::<Resources>() else {
            if plugin_wide {
                return Ok(context);
            }
            return Err(Error::Permission(format!("{action}: Enter an Active Execution")));
        };
        resources.check_action_scope(&resources.execution, action)?;
        context.caller = Some(resources.clone());
    }
    Ok(context)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::failure;
    use crate::Error;
    use harmos_protocol::model;
    use harmos_runtime::Failure;
    use harmos_runtime::operations::ServingError;
    use std::collections::HashMap;

    #[test]
    fn caller_refusal_preserves_its_structured_details() {
        let mut original = model::Failure::from(&Failure::wrong_caller("dokime.host.v1.evidence.Logs/Append"));
        original.details = HashMap::from([("caller".into(), "client".into()), ("required".into(), "plugin".into())]);
        let source = Error::Caller(Failure::from(original.clone()));
        let ServingError::Handler(carried) = failure(source) else {
            panic!("Return Caller Refusal: Preserve Its Structured Failure");
        };
        assert_eq!(model::Failure::from(&carried), original);
    }
}
