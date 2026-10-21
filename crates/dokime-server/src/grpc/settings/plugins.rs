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
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::{preferences::Resources, wire as projection};
use crate::grpc::value::{self, Budget, canonical};
use dokime::domain::operations::settings as wire;
use dokime::{
    RuntimeHandle,
    domains::installed_plugins::{PluginCredentialSecretUpdate, PluginSettingsUpdate, RuntimePluginSettings},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct GetSettings {
    resources: Arc<Resources>,
}

struct GetPluginSettings {
    resources: Arc<Resources>,
}

struct UpdatePluginSettings {
    resources: Arc<Resources>,
}

struct UpdatePluginCredential {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for GetSettings {
    type Declaration = wire::GetSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetSettings, _: &mut (),
    ) -> Result<Response<dokime::domain::model::settings::wire::SettingsFrame>, Status> {
        let frame = self.resources.runtime.settings_frame().await;
        self.resources.produce_result(|| projection::encode_frame(frame, self.resources.maximum))
    }
}

impl Operation for GetPluginSettings {
    type Declaration = wire::GetPluginSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetPluginSettings, _: &mut (),
    ) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
        let id = plugin_id(&request.plugin_id)?;
        let settings = self.resources.runtime.plugin_settings(&id).await.map_err(projection::plugin_error)?;
        plugin_response(&self.resources, settings)
    }
}

impl Operation for UpdatePluginSettings {
    type Declaration = wire::UpdatePluginSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::UpdatePluginSettings, _: &mut (),
    ) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
        let id = plugin_id(&request.plugin_id)?;
        let values = match request.values.take() {
            None => Default::default(),
            Some(value) => match canonical::decode(value, &mut Budget::new(), 0)? {
                dokime::domain::model::values::Value::Object(values) => values,
                _ => {
                    return Err(Status::invalid_argument("plugin settings values must be an object"));
                }
            },
        };
        let settings =
            self.resources.runtime.save_plugin_settings(&id, PluginSettingsUpdate { values }).await.map_err(projection::plugin_error)?;
        plugin_response(&self.resources, settings)
    }
}

impl Operation for UpdatePluginCredential {
    type Declaration = wire::UpdatePluginCredential;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::UpdatePluginCredential, _: &mut (),
    ) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
        let id = plugin_id(&request.plugin_id)?;
        let credential = projection::bounded_text(std::mem::take(&mut request.credential_id), "credential_id")?;
        let secret = projection::bounded_text(std::mem::take(&mut request.secret), "secret")?;
        let settings = self
            .resources
            .runtime
            .save_plugin_credential_secret(&id, &credential, PluginCredentialSecretUpdate { secret })
            .await
            .map_err(projection::plugin_error)?;
        plugin_response(&self.resources, settings)
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum });
    runtime.operations().register_wire_operation::<GetSettings>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<GetPluginSettings>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<UpdatePluginSettings>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<UpdatePluginCredential>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    Ok(())
}

/* === Private Functions === */

fn plugin_id(id: &str) -> Result<dokime::domain::model::identity::Identifier, Status> {
    value::text(id)?;
    projection::identifier(id.to_owned(), "plugin_id")
}

fn plugin_response(
    resources: &Resources, settings: RuntimePluginSettings,
) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
    resources.produce_result(|| projection::encode_plugin_settings(settings, resources.maximum))
}

/* === Tests === */
