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
use harmos_protocol::model::Callers;
use harmos_runtime::operations::{Contract, ExecutionContext, HostContext, OperationFactory, ProtobufOperation, ServingError};
use harmos_runtime::plugin::grpc::CallingPlugin;

// Local Imports
use crate::host::{ClientSettings, Context};
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
            use crate::native::{Binding, context, failure};
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
                    let context = context(&execution, Self::ID).map_err(failure)?;
                    crate::native::unary!(@call $caller, context, request)
                }
            }
        }
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
    crate::authorize(action, if plugin { Callers::Plugins } else { Callers::Clients })
        .map_err(|source| Error::Permission(format!("{action}: {source}")))?;
    if plugin {
        let resources =
            host.context().get::<Resources>().ok_or_else(|| Error::Permission(format!("{action}: Enter an Active Execution")))?;
        resources.check_action_scope(&resources.execution, action)?;
        context.caller = Some(resources.clone());
    }
    Ok(context)
}

pub fn failure(source: Error) -> ServingError {
    tracing::warn!(error = %source, "Host Operation Failed");
    ServingError::Native(Arc::new(crate::host::status(source)))
}

pub fn factory<O: ProtobufOperation>(timeout: Duration) -> Result<OperationFactory> {
    let factory = OperationFactory::protobuf::<O>().map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    let contract = factory.contract().map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    let mut operation = contract.operation().clone();
    operation.policy.timeout = timeout;
    operation.policy.attempt_timeout = timeout;
    let contract = Contract::from_protobuf(
        operation,
        contract.protobuf().expect("protobuf operation").clone(),
        contract.grpc().cloned(),
        &Default::default(),
    )
    .map_err(|source| Error::Invalid(format!("Register {}: {source}", O::ID)))?;
    Ok(factory.with_contract(contract))
}

pub fn registration(settings: &ClientSettings) -> Result<harmos_runtime::operations::Registration> {
    let mut registration = harmos_runtime::operations::Registration::new(harmos_runtime::operations::Lane::default());
    registration = registration.entry(factory::<crate::host::backend::appearance::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::backend::environment::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::backend::overview::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::delete::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::parameters::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::content::artifacts::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::content::artifacts::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::content::execution_files::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::content::execution_files::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::seal::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::documents::append_parts::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::documents::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::archive::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::health_events::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::logs::append::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::notes::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::notes::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::notes::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::notes::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::records::append::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::records::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::samples::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::statistics::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::traceability::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::traceability::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::cancel::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::commands::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::commands::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::delete::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::parameters::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::progress::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::progress::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::schemas::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::start::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::steps::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::stop::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::delete::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::files::read::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::append::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::delete::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::publish::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::uploads::resume::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::plots::settings::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::plots::settings::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::plots::snapshots::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::plugins::secrets::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::prompt_definitions::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::prompt_definitions::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::prompts::answers::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::prompts::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::prompts::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::prompts::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::record_schemas::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::record_schemas::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::reports::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::reports::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::reports::publish::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::delete::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::export::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::import::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::settings::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::settings::schema::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::settings::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::storage::sources::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::storage::sources::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::stream_schemas::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::stream_schemas::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::suites::get::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::suites::list::Operation>(settings.read_timeout)?);
    registration = registration.entry(factory::<crate::host::backend::overview::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::cases::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::content::artifacts::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::content::execution_files::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::cycles::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::documents::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::health_events::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::notes::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::records::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::samples::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::evidence::traceability::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::progress::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::executions::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::files::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::plots::snapshots::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::prompts::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::secrets::watch::Operation>(settings.watch_timeout)?);
    registration = registration.entry(factory::<crate::host::content::artifacts::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::capture::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::captures::start::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::files::captures::stop::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::aggregates::get::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::effects::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::effects::get::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::effects::set::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::media::captures::create::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::media::captures::get::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::media::captures::stop::Operation>(settings.mutation_timeout)?);
    registration = registration.entry(factory::<crate::host::media::captures::watch::Operation>(settings.watch_timeout)?);
    Ok(registration)
}

/* === Private Functions === */

/* === Tests === */
