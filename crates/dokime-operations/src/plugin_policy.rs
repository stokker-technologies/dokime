/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Product Plugin Administration and Native Payload Mapping

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos_operations::host::declarations::{plugins, settings};
use harmos_protocol::host;
use harmos_runtime::operations::{
    ExecutionContext, Grpc, HostContext, NativeOperation, OperationPolicy, ProtobufOperation, ProtobufOutput, Resource, ServingError,
    Visibility,
};
use prost_reflect::ServiceDescriptor;

// Local Imports
use dokime_protocol::host::plugins::{SettingsListRequest, SettingsListResponse, SettingsSetRequest, SettingsSetResponse};
use dokime_protocol::host::{
    PluginChange, PluginDescription, PluginEntry, PluginGuest, PluginManaged, PluginPlacement, PluginProcess, PluginRemote, PluginSnapshot,
    PluginValue, PluginsCreateRequest, PluginsCreateResponse, PluginsDeleteRequest, PluginsDeleteResponse, PluginsGetRequest,
    PluginsGetResponse, PluginsListRequest, PluginsListResponse, PluginsReplaceRequest, PluginsReplaceResponse, PluginsRestartRequest,
    PluginsRestartResponse, PluginsSetRequest, PluginsSetResponse, PluginsWatchRequest, PluginsWatchResponse, plugin_change,
    plugin_placement, plugins_watch_response,
};

/* === Constants === */

/* === Macros === */

macro_rules! mapping {
    ($source:path => $target:path, $value:ident, $body:expr) => {
        impl Project<$target> for $source {
            fn project(self) -> $target {
                let $value = self;
                $body
            }
        }
    };
}

macro_rules! unary {
    ($name:ident, $native:path, $request:path, $response:path, $package:literal, $service:literal, $method:literal) => {
        pub struct $name;
        impl ProtobufOperation for $name {
            type Request = $request;
            type Response = $response;
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
                let response =
                    host.for_call().call::<$native>(request.project()).await.map_err(|source| ServingError::Native(Arc::new(source)))?;
                Ok(response.project())
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

pub struct Watch;

/* === Enums === */

/* === Traits === */

trait Project<T> {
    fn project(self) -> T;
}

/* === Implementations === */

mapping!(PluginsGetRequest => host::PluginsGetRequest, value, host::PluginsGetRequest { name: value.name, at: value.at });

mapping!(host::PluginsGetResponse => PluginsGetResponse, value, PluginsGetResponse { entry: value.entry.map(Project::project), resolved_at: value.resolved_at });

mapping!(PluginsListRequest => host::PluginsListRequest, value, host::PluginsListRequest { at: value.at });

mapping!(host::PluginsListResponse => PluginsListResponse, value, PluginsListResponse { entries: value.entries.into_iter().map(Project::project).collect(), resolved_at: value.resolved_at });

mapping!(PluginsWatchRequest => host::PluginsWatchRequest, value, host::PluginsWatchRequest { name: value.name, at: value.at });

mapping!(PluginsCreateRequest => host::PluginsCreateRequest, value, host::PluginsCreateRequest { request_id: value.request_id, name: value.name, placement: value.placement.map(Project::project), value: value.value.map(Project::project), origin: value.origin });

mapping!(host::PluginsCreateResponse => PluginsCreateResponse, value, PluginsCreateResponse { entry: value.entry.map(Project::project), receipt: value.receipt });

mapping!(PluginsSetRequest => host::PluginsSetRequest, value, host::PluginsSetRequest { name: value.name, value: value.value.map(Project::project), expected_revision: value.expected_revision });

mapping!(host::PluginsSetResponse => PluginsSetResponse, value, PluginsSetResponse { entry: value.entry.map(Project::project), receipt: value.receipt });

mapping!(PluginsDeleteRequest => host::PluginsDeleteRequest, value, host::PluginsDeleteRequest { name: value.name, expected_revision: value.expected_revision });

mapping!(host::PluginsDeleteResponse => PluginsDeleteResponse, value, PluginsDeleteResponse { receipt: value.receipt });

mapping!(PluginsRestartRequest => host::PluginsRestartRequest, value, host::PluginsRestartRequest { name: value.name, expected_revision: value.expected_revision });

mapping!(host::PluginsRestartResponse => PluginsRestartResponse, value, PluginsRestartResponse { entry: value.entry.map(Project::project) });

mapping!(PluginsReplaceRequest => host::PluginsReplaceRequest, value, host::PluginsReplaceRequest { name: value.name, placement: value.placement.map(Project::project), expected_revision: value.expected_revision, request_id: value.request_id });

mapping!(host::PluginsReplaceResponse => PluginsReplaceResponse, value, PluginsReplaceResponse { entry: value.entry.map(Project::project), receipt: value.receipt });

mapping!(SettingsListRequest => host::plugins::SettingsListRequest, value, host::plugins::SettingsListRequest { name: value.name, at: value.at });

mapping!(host::plugins::SettingsListResponse => SettingsListResponse, value, SettingsListResponse { declarations: value.declarations, value: value.value, revision: value.revision, resolved_at: value.resolved_at });

mapping!(SettingsSetRequest => host::plugins::SettingsSetRequest, value, host::plugins::SettingsSetRequest { name: value.name, value: value.value, expected_revision: value.expected_revision, request_id: value.request_id });

mapping!(host::plugins::SettingsSetResponse => SettingsSetResponse, value, SettingsSetResponse { value: value.value, revision: value.revision, receipt: value.receipt });

mapping!(PluginValue => host::PluginValue, value, host::PluginValue { enabled: value.enabled });

mapping!(PluginPlacement => host::PluginPlacement, value, host::PluginPlacement {
    kind: value.kind.map(|kind| match kind {
        plugin_placement::Kind::Managed(value) => host::plugin_placement::Kind::Managed(host::PluginManaged { content: value.content, sha256: value.sha256 }),
        plugin_placement::Kind::Remote(value) => host::plugin_placement::Kind::Remote(host::PluginRemote { url: value.url }),
        plugin_placement::Kind::Guest(_) => host::plugin_placement::Kind::Guest(host::PluginGuest {}),
    }),
});

mapping!(host::PluginValue => PluginValue, value, PluginValue { enabled: value.enabled });

mapping!(host::PluginPlacement => PluginPlacement, value, PluginPlacement {
    kind: value.kind.map(|kind| match kind {
        host::plugin_placement::Kind::Managed(value) => plugin_placement::Kind::Managed(PluginManaged { content: value.content, sha256: value.sha256 }),
        host::plugin_placement::Kind::Remote(value) => plugin_placement::Kind::Remote(PluginRemote { url: value.url }),
        host::plugin_placement::Kind::Guest(_) => plugin_placement::Kind::Guest(PluginGuest {}),
    }),
});

mapping!(host::PluginEntry => PluginEntry, value, PluginEntry {
    name: value.name,
    placement: value.placement.map(Project::project),
    value: value.value.map(Project::project),
    revision: value.revision,
    state: value.state,
    failure: value.failure,
    description: value.description.map(|description| PluginDescription {
        id: description.id, version: description.version, started_at: description.started_at,
        declarations: description.declarations, revision: description.revision, sdk: description.sdk, sdk_version: description.sdk_version,
    }),
    declarations_digest: value.declarations_digest,
    process: value.process.map(|process| PluginProcess { id: process.id, executable_digest: process.executable_digest, started_at: process.started_at }),
    activation: value.activation,
    origin: value.origin,
});
mapping!(host::PluginsWatchResponse => PluginsWatchResponse, value, PluginsWatchResponse {
    resolved_at: value.resolved_at,
    event: value.event.map(|event| match event {
        host::plugins_watch_response::Event::Snapshot(snapshot) => plugins_watch_response::Event::Snapshot(PluginSnapshot { entries: snapshot.entries.into_iter().map(Project::project).collect() }),
        host::plugins_watch_response::Event::Change(change) => plugins_watch_response::Event::Change(PluginChange {
            kind: change.kind.map(|kind| match kind {
                host::plugin_change::Kind::Entry(entry) => plugin_change::Kind::Entry(entry.project()),
                host::plugin_change::Kind::Deleted(name) => plugin_change::Kind::Deleted(name),
            }),
        }),
        host::plugins_watch_response::Event::ResyncRequired(reason) => plugins_watch_response::Event::ResyncRequired(reason),
    }),
});

unary!(Get, plugins::Get, PluginsGetRequest, PluginsGetResponse, "dokime.host.v1", "Plugins", "Get");

unary!(List, plugins::List, PluginsListRequest, PluginsListResponse, "dokime.host.v1", "Plugins", "List");

unary!(Create, plugins::Create, PluginsCreateRequest, PluginsCreateResponse, "dokime.host.v1", "Plugins", "Create");

unary!(Set, plugins::Set, PluginsSetRequest, PluginsSetResponse, "dokime.host.v1", "Plugins", "Set");

unary!(Delete, plugins::Delete, PluginsDeleteRequest, PluginsDeleteResponse, "dokime.host.v1", "Plugins", "Delete");

unary!(Restart, plugins::Restart, PluginsRestartRequest, PluginsRestartResponse, "dokime.host.v1", "Plugins", "Restart");

unary!(Replace, plugins::Replace, PluginsReplaceRequest, PluginsReplaceResponse, "dokime.host.v1", "Plugins", "Replace");

unary!(SettingsList, settings::List, SettingsListRequest, SettingsListResponse, "dokime.host.v1.plugins", "Settings", "List");

unary!(SettingsSet, settings::Set, SettingsSetRequest, SettingsSetResponse, "dokime.host.v1.plugins", "Settings", "Set");

impl ProtobufOperation for Watch {
    type Request = PluginsWatchRequest;
    type Response = PluginsWatchResponse;
    type Endpoints = ProtobufOutput<Self::Response>;
    const ID: &'static str = "dokime.host.v1.Plugins/Watch";
    const GRPC: Option<Grpc> = Some(Grpc::new("dokime.host.v1", "Plugins", "Watch"));
    const POLICY: OperationPolicy = <plugins::Watch as NativeOperation>::POLICY;
    const VISIBILITY: Visibility = Visibility::Public;
    const RESOURCES: &'static [Resource] = &[Resource::of::<HostContext>(), Resource::of::<crate::native::Binding>()];
    fn service() -> Option<ServiceDescriptor> {
        dokime_protocol::descriptor_pool().get_service_by_name("dokime.host.v1.Plugins")
    }
    async fn run(execution: ExecutionContext, request: Self::Request, output: &Self::Endpoints) -> Result<Self::Response, ServingError> {
        crate::wrappers::forward::<plugins::Watch, _>(execution, request.project(), output, Project::project).await
    }
}

/* === Public Functions === */

pub fn entry(value: host::PluginEntry) -> PluginEntry {
    value.project()
}

/* === Private Functions === */

/* === Tests === */
