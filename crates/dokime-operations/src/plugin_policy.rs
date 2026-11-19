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

impl Project<host::PluginsGetRequest> for PluginsGetRequest {
    fn project(self) -> host::PluginsGetRequest {
        let value = self;
        host::PluginsGetRequest { name: value.name, at: value.at }
    }
}

impl Project<PluginsGetResponse> for host::PluginsGetResponse {
    fn project(self) -> PluginsGetResponse {
        let value = self;
        PluginsGetResponse {
            entry: value.entry.map(Project::project),
            resolved_at: value.resolved_at,
        }
    }
}

impl Project<host::PluginsListRequest> for PluginsListRequest {
    fn project(self) -> host::PluginsListRequest {
        let value = self;
        host::PluginsListRequest { at: value.at }
    }
}

impl Project<PluginsListResponse> for host::PluginsListResponse {
    fn project(self) -> PluginsListResponse {
        let value = self;
        PluginsListResponse {
            entries: value.entries.into_iter().map(Project::project).collect(),
            resolved_at: value.resolved_at,
        }
    }
}

impl Project<host::PluginsWatchRequest> for PluginsWatchRequest {
    fn project(self) -> host::PluginsWatchRequest {
        let value = self;
        host::PluginsWatchRequest { name: value.name, at: value.at }
    }
}

impl Project<host::PluginsCreateRequest> for PluginsCreateRequest {
    fn project(self) -> host::PluginsCreateRequest {
        let value = self;
        host::PluginsCreateRequest {
            request_id: value.request_id,
            name: value.name,
            placement: value.placement.map(Project::project),
            value: value.value.map(Project::project),
            origin: value.origin,
        }
    }
}

impl Project<PluginsCreateResponse> for host::PluginsCreateResponse {
    fn project(self) -> PluginsCreateResponse {
        let value = self;
        PluginsCreateResponse {
            entry: value.entry.map(Project::project),
            receipt: value.receipt,
        }
    }
}

impl Project<host::PluginsSetRequest> for PluginsSetRequest {
    fn project(self) -> host::PluginsSetRequest {
        let value = self;
        host::PluginsSetRequest {
            name: value.name,
            value: value.value.map(Project::project),
            expected_revision: value.expected_revision,
        }
    }
}

impl Project<PluginsSetResponse> for host::PluginsSetResponse {
    fn project(self) -> PluginsSetResponse {
        let value = self;
        PluginsSetResponse {
            entry: value.entry.map(Project::project),
            receipt: value.receipt,
        }
    }
}

impl Project<host::PluginsDeleteRequest> for PluginsDeleteRequest {
    fn project(self) -> host::PluginsDeleteRequest {
        let value = self;
        host::PluginsDeleteRequest {
            name: value.name,
            expected_revision: value.expected_revision,
        }
    }
}

impl Project<PluginsDeleteResponse> for host::PluginsDeleteResponse {
    fn project(self) -> PluginsDeleteResponse {
        let value = self;
        PluginsDeleteResponse { receipt: value.receipt }
    }
}

impl Project<host::PluginsRestartRequest> for PluginsRestartRequest {
    fn project(self) -> host::PluginsRestartRequest {
        let value = self;
        host::PluginsRestartRequest {
            name: value.name,
            expected_revision: value.expected_revision,
        }
    }
}

impl Project<PluginsRestartResponse> for host::PluginsRestartResponse {
    fn project(self) -> PluginsRestartResponse {
        let value = self;
        PluginsRestartResponse { entry: value.entry.map(Project::project) }
    }
}

impl Project<host::PluginsReplaceRequest> for PluginsReplaceRequest {
    fn project(self) -> host::PluginsReplaceRequest {
        let value = self;
        host::PluginsReplaceRequest {
            name: value.name,
            placement: value.placement.map(Project::project),
            expected_revision: value.expected_revision,
            request_id: value.request_id,
        }
    }
}

impl Project<PluginsReplaceResponse> for host::PluginsReplaceResponse {
    fn project(self) -> PluginsReplaceResponse {
        let value = self;
        PluginsReplaceResponse {
            entry: value.entry.map(Project::project),
            receipt: value.receipt,
        }
    }
}

impl Project<host::plugins::SettingsListRequest> for SettingsListRequest {
    fn project(self) -> host::plugins::SettingsListRequest {
        let value = self;
        host::plugins::SettingsListRequest { name: value.name, at: value.at }
    }
}

impl Project<SettingsListResponse> for host::plugins::SettingsListResponse {
    fn project(self) -> SettingsListResponse {
        let value = self;
        SettingsListResponse {
            declarations: value.declarations,
            value: value.value,
            revision: value.revision,
            resolved_at: value.resolved_at,
        }
    }
}

impl Project<host::plugins::SettingsSetRequest> for SettingsSetRequest {
    fn project(self) -> host::plugins::SettingsSetRequest {
        let value = self;
        host::plugins::SettingsSetRequest {
            name: value.name,
            value: value.value,
            expected_revision: value.expected_revision,
            request_id: value.request_id,
        }
    }
}

impl Project<SettingsSetResponse> for host::plugins::SettingsSetResponse {
    fn project(self) -> SettingsSetResponse {
        let value = self;
        SettingsSetResponse {
            value: value.value,
            revision: value.revision,
            receipt: value.receipt,
        }
    }
}

impl Project<host::PluginValue> for PluginValue {
    fn project(self) -> host::PluginValue {
        let value = self;
        host::PluginValue { enabled: value.enabled }
    }
}

impl Project<host::PluginPlacement> for PluginPlacement {
    fn project(self) -> host::PluginPlacement {
        let value = self;
        host::PluginPlacement {
            kind: value.kind.map(|kind| match kind {
                plugin_placement::Kind::Managed(value) => host::plugin_placement::Kind::Managed(host::PluginManaged {
                    content: value.content,
                    sha256: value.sha256,
                }),
                plugin_placement::Kind::Remote(value) => host::plugin_placement::Kind::Remote(host::PluginRemote { url: value.url }),
                plugin_placement::Kind::Guest(_) => host::plugin_placement::Kind::Guest(host::PluginGuest {}),
            }),
        }
    }
}

impl Project<PluginValue> for host::PluginValue {
    fn project(self) -> PluginValue {
        let value = self;
        PluginValue { enabled: value.enabled }
    }
}

impl Project<PluginPlacement> for host::PluginPlacement {
    fn project(self) -> PluginPlacement {
        let value = self;
        PluginPlacement {
            kind: value.kind.map(|kind| match kind {
                host::plugin_placement::Kind::Managed(value) => plugin_placement::Kind::Managed(PluginManaged {
                    content: value.content,
                    sha256: value.sha256,
                }),
                host::plugin_placement::Kind::Remote(value) => plugin_placement::Kind::Remote(PluginRemote { url: value.url }),
                host::plugin_placement::Kind::Guest(_) => plugin_placement::Kind::Guest(PluginGuest {}),
            }),
        }
    }
}

impl Project<PluginEntry> for host::PluginEntry {
    fn project(self) -> PluginEntry {
        let value = self;
        PluginEntry {
            name: value.name,
            placement: value.placement.map(Project::project),
            value: value.value.map(Project::project),
            revision: value.revision,
            state: value.state,
            failure: value.failure,
            description: value.description.map(|description| PluginDescription {
                id: description.id,
                version: description.version,
                started_at: description.started_at,
                declarations: description.declarations,
                revision: description.revision,
                sdk: description.sdk,
                sdk_version: description.sdk_version,
            }),
            declarations_digest: value.declarations_digest,
            process: value.process.map(|process| PluginProcess {
                id: process.id,
                executable_digest: process.executable_digest,
                started_at: process.started_at,
            }),
            activation: value.activation,
            origin: value.origin,
        }
    }
}
impl Project<PluginsWatchResponse> for host::PluginsWatchResponse {
    fn project(self) -> PluginsWatchResponse {
        let value = self;
        PluginsWatchResponse {
            resolved_at: value.resolved_at,
            event: value.event.map(|event| match event {
                host::plugins_watch_response::Event::Snapshot(snapshot) => plugins_watch_response::Event::Snapshot(PluginSnapshot {
                    entries: snapshot.entries.into_iter().map(Project::project).collect(),
                }),
                host::plugins_watch_response::Event::Change(change) => plugins_watch_response::Event::Change(PluginChange {
                    kind: change.kind.map(|kind| match kind {
                        host::plugin_change::Kind::Entry(entry) => plugin_change::Kind::Entry(entry.project()),
                        host::plugin_change::Kind::Deleted(name) => plugin_change::Kind::Deleted(name),
                    }),
                }),
                host::plugins_watch_response::Event::ResyncRequired(reason) => plugins_watch_response::Event::ResyncRequired(reason),
            }),
        }
    }
}

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
