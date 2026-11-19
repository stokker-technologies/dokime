/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Product Client Exposure of Native Library Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos_operations::host::declarations;
use harmos_runtime::operations::{
    ExecutionContext, Grpc, HostContext, NativeOperation, NativeOutput, OperationPolicy, ProtobufOperation, ProtobufOutput, Resource,
    ServingError, StreamSettings, Visibility, native_stream,
};
use prost_reflect::{ReflectMessage, ServiceDescriptor};

// Local Imports

/* === Constants === */

/* === Macros === */

macro_rules! unary {
    ($name:ident, $native:path, $package:literal, $service:literal, $method:literal) => {
        pub struct $name;
        impl ProtobufOperation for $name {
            type Request = <$native as NativeOperation>::Request;
            type Response = <$native as NativeOperation>::Response;
            type Endpoints = ();
            const ID: &'static str = concat!($package, ".", $service, "/", $method);
            const GRPC: Option<Grpc> = Some(Grpc::new($package, $service, $method));
            const POLICY: OperationPolicy = <$native as NativeOperation>::POLICY;
            const VISIBILITY: Visibility = Visibility::Public;
            const RESOURCES: &'static [Resource] = &[Resource::of::<HostContext>()];
            fn service() -> Option<ServiceDescriptor> {
                dokime_protocol::descriptor_pool().get_service_by_name(concat!($package, ".", $service))
            }
            async fn run(execution: ExecutionContext, request: Self::Request, _: &()) -> Result<Self::Response, ServingError> {
                let host = execution.resource::<HostContext>().map_err(|_| ServingError::Catalog)?;
                host.for_call().call::<$native>(request).await.map_err(|source| ServingError::Native(Arc::new(source)))
            }
        }
    };
}

macro_rules! watch {
    ($name:ident, $native:path, $package:literal, $service:literal) => {
        pub struct $name;
        impl ProtobufOperation for $name {
            type Request = <$native as NativeOperation>::Request;
            type Response = <$native as NativeOperation>::Response;
            type Endpoints = ProtobufOutput<Self::Response>;
            const ID: &'static str = concat!($package, ".", $service, "/Watch");
            const GRPC: Option<Grpc> = Some(Grpc::new($package, $service, "Watch"));
            const POLICY: OperationPolicy = <$native as NativeOperation>::POLICY;
            const VISIBILITY: Visibility = Visibility::Public;
            const RESOURCES: &'static [Resource] = &[Resource::of::<HostContext>(), Resource::of::<crate::native::Binding>()];
            fn service() -> Option<ServiceDescriptor> {
                dokime_protocol::descriptor_pool().get_service_by_name(concat!($package, ".", $service))
            }
            async fn run(
                execution: ExecutionContext, request: Self::Request, output: &Self::Endpoints,
            ) -> Result<Self::Response, ServingError> {
                forward::<$native, _>(execution, request, output, |response| response).await
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

unary!(ListenerGet, declarations::access::ListenerGet, "dokime.host.v1.access", "Listener", "Get");
unary!(ListenerSet, declarations::access::ListenerSet, "dokime.host.v1.access", "Listener", "Set");
unary!(PairingsCreate, declarations::access::PairingsCreate, "dokime.host.v1.access", "Pairings", "Create");
unary!(PairingsList, declarations::access::PairingsList, "dokime.host.v1.access", "Pairings", "List");
unary!(PairingsDelete, declarations::access::PairingsDelete, "dokime.host.v1.access", "Pairings", "Delete");
unary!(CertificatesCreate, declarations::access::CertificatesCreate, "dokime.host.v1.access", "Certificates", "Create");
unary!(StatusGet, declarations::status::Get, "dokime.host.v1", "Status", "Get");
watch!(StatusWatch, declarations::status::Watch, "dokime.host.v1", "Status");
watch!(LogsWatch, declarations::status::LogsWatch, "dokime.host.v1.status", "Logs");
unary!(TraceEventsList, declarations::status::TraceEventsList, "dokime.host.v1.status", "TraceEvents", "List");
watch!(TraceEventsWatch, declarations::status::TraceEventsWatch, "dokime.host.v1.status", "TraceEvents");
unary!(DiagnosticsGet, declarations::status::DiagnosticsGet, "dokime.host.v1.status", "Diagnostics", "Get");
unary!(OperationsList, declarations::host::List, "dokime.host.v1", "Operations", "List");
unary!(OperationsGet, declarations::host::Get, "dokime.host.v1", "Operations", "Get");
unary!(InvocationsCreate, declarations::lifecycle::Create, "dokime.host.v1", "Invocations", "Create");

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn forward<O, T>(
    execution: ExecutionContext, request: O::Request, output: &ProtobufOutput<T>, project: impl Fn(O::Response) -> T + Send + Sync,
) -> Result<T, ServingError>
where
    O: NativeOperation<Endpoints = NativeOutput<<O as NativeOperation>::Response>>,
    O::Response: Clone,
    T: ReflectMessage + Default + Send + 'static,
{
    let context = crate::native::context(&execution, O::ID).map_err(crate::native::failure)?;
    let settings = StreamSettings {
        items: NonZeroUsize::new(context.settings.stream_buffer).ok_or(ServingError::Catalog)?,
        bytes: NonZeroUsize::new(context.settings.message_bytes).ok_or(ServingError::Catalog)?,
    };
    let (send, receive) = native_stream(settings).map_err(|source| ServingError::Native(Arc::new(source)))?;
    let host = execution.resource::<HostContext>().map_err(|_| ServingError::Catalog)?;
    let mut invocation = Box::pin(host.for_call().submit::<O>(request, send).map_err(|source| ServingError::Native(Arc::new(source)))?);
    let mut forward = Box::pin(async {
        while let Some(item) = receive.recv().await {
            output.send(project((*item).clone())).await.map_err(|source| ServingError::Native(Arc::new(source)))?;
        }
        Ok::<(), ServingError>(())
    });
    let mut changes = context.host.changes();
    let stopped = async {
        while !context.host.shutting_down() {
            if changes.changed().await.is_err() {
                break;
            }
        }
    };
    let response = tokio::select! {
        _ = stopped => {
            invocation.request_cancel();
            let _ = invocation.await;
            return Ok(T::default());
        }
        _ = execution.cancellation.cancelled() => {
            invocation.request_cancel();
            let _ = invocation.await;
            return Err(ServingError::Catalog);
        }
        result = &mut forward => {
            if let Err(source) = result {
                invocation.request_cancel();
                let _ = invocation.await;
                return Err(source);
            }
            invocation.await
        }
        result = &mut invocation => {
            // Drain the Native Operation's Last Frames Before Returning Its Final Status.
            forward.await?;
            result
        }
    }
    .map_err(|source| ServingError::Native(Arc::new(source)))?;
    response.try_unwrap().map(&project).map_err(|_| ServingError::Catalog)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use harmos_protocol::{model::Callers, schema};
    use harmos_runtime::operations::OperationFactory;

    fn contract<O: ProtobufOperation>() {
        let factory = OperationFactory::protobuf::<O>().expect("Register Product Wrapper");
        let contract = factory.contract().expect("Read Product Wrapper Contract");
        assert_eq!(contract.callers(), Callers::Clients);
        let service = O::service().expect("Resolve Product Service Descriptor");
        let (_, method) = O::ID.split_once('/').unwrap();
        let method = service.methods().find(|candidate| candidate.name() == method).unwrap();
        assert_eq!(schema::method_callers(&method).unwrap(), Callers::Clients);
        assert_eq!(method.input().full_name(), contract.protobuf().unwrap().request);
        assert_eq!(method.output().full_name(), contract.protobuf().unwrap().response);
        assert!(method.input().full_name().starts_with("harmos.host.v1."));
    }

    #[test]
    fn wrapper_contracts_use_product_audiences_and_library_payloads() {
        contract::<ListenerGet>();
        contract::<ListenerSet>();
        contract::<PairingsCreate>();
        contract::<PairingsList>();
        contract::<PairingsDelete>();
        contract::<CertificatesCreate>();
        contract::<StatusGet>();
        contract::<StatusWatch>();
        contract::<LogsWatch>();
        contract::<TraceEventsList>();
        contract::<TraceEventsWatch>();
        contract::<DiagnosticsGet>();
        contract::<OperationsList>();
        contract::<OperationsGet>();
        contract::<InvocationsCreate>();
    }
}
