/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt, Response};
use tonic::Status;

use super::{preferences::Resources, wire as projection};
use crate::grpc::value::{Budget, canonical};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::settings as wire;
use dokime::{
    RuntimeHandle,
    domains::installed_plugins::{PluginSecretUpdate, PluginSettingsUpdate, RuntimePluginSettings},
};

impl Serve<wire::GetSettings> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetSettings, _: Context<'a, wire::GetSettings>,
    ) -> Served<'a, dokime::domain::model::settings::wire::SettingsFrame> {
        Box::pin(async move {
            let frame = self.runtime.settings_frame().await;
            self.produce_result(|| projection::encode_frame(frame, self.maximum))
        })
    }
}

impl Serve<wire::GetPluginSettings> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetPluginSettings, _: Context<'a, wire::GetPluginSettings>,
    ) -> Served<'a, dokime::domain::model::settings::wire::PluginRuntimeSettings> {
        Box::pin(async move {
            let id = plugin_id(&request.plugin_id)?;
            let settings = self.runtime.plugin_settings(&id).await.map_err(projection::plugin_error)?;
            plugin_response(self, settings)
        })
    }
}

impl Serve<wire::UpdatePluginSettings> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::UpdatePluginSettings, _: Context<'a, wire::UpdatePluginSettings>,
    ) -> Served<'a, dokime::domain::model::settings::wire::PluginRuntimeSettings> {
        Box::pin(async move {
            let id = plugin_id(&request.plugin_id)?;
            let mut budget = Budget::new()?;
            let values = match request.values.take() {
                None => Default::default(),
                Some(value) => match canonical::decode(value, &mut budget, 0)? {
                    dokime::domain::model::values::Value::Object(values) => values,
                    _ => {
                        return Err(Status::invalid_argument("plugin settings values must be an object"));
                    }
                },
            };
            let settings =
                self.runtime.save_plugin_settings(&id, PluginSettingsUpdate { values }).await.map_err(projection::plugin_error)?;
            plugin_response(self, settings)
        })
    }
}

impl Serve<wire::UpdatePluginSecret> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::UpdatePluginSecret, _: Context<'a, wire::UpdatePluginSecret>,
    ) -> Served<'a, dokime::domain::model::settings::wire::PluginRuntimeSettings> {
        Box::pin(async move {
            let id = plugin_id(&request.plugin_id)?;
            let secret_id = std::mem::take(&mut request.secret_id);
            let secret = std::mem::take(&mut request.secret);
            let settings =
                self.runtime.save_plugin_secret(&id, &secret_id, PluginSecretUpdate { secret }).await.map_err(projection::plugin_error)?;
            plugin_response(self, settings)
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::GetSettings>::new(served.clone()))?;
    dependencies.insert(Host::<wire::GetPluginSettings>::new(served.clone()))?;
    dependencies.insert(Host::<wire::UpdatePluginSettings>::new(served.clone()))?;
    dependencies.insert(Host::<wire::UpdatePluginSecret>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::GetSettings>()
        .operation::<wire::GetPluginSettings>()
        .operation::<wire::UpdatePluginSettings>()
        .operation::<wire::UpdatePluginSecret>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

fn plugin_id(id: &str) -> Result<dokime::domain::model::identity::Identifier, Status> {
    projection::identifier(id.to_owned(), "plugin_id")
}

fn plugin_response(
    resources: &Resources, settings: RuntimePluginSettings,
) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
    resources.produce_result(|| projection::encode_plugin_settings(settings, resources.maximum))
}
