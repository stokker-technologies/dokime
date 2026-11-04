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
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::{preferences::Resources, wire as projection};
use crate::grpc::value::{Budget, canonical};
use dokime::app::LiveRuntime;
use dokime::domains::installed_plugins::{PluginSettingsUpdate, RuntimePluginSettings};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::settings as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::GetSettings),
    Operation1(wire::GetPluginSettings),
    Operation2(wire::UpdatePluginSettings),
}

/* === Traits === */

/* === Implementations === */

impl Serve<wire::GetSettings> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetSettings, _: Context<'a, wire::GetSettings>,
    ) -> Served<'a, dokime::domain::model::settings::wire::SettingsFrame> {
        Box::pin(async move {
            let frame = dokime::app::live_runtime(&self.runtime).await?.settings_frame().await;
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
            let settings = dokime::app::live_runtime(&self.runtime).await?.plugin_settings(&id).await.map_err(projection::plugin_error)?;
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
            let settings = dokime::app::live_runtime(&self.runtime)
                .await?
                .save_plugin_settings(&id, PluginSettingsUpdate { values })
                .await
                .map_err(projection::plugin_error)?;
            plugin_response(self, settings)
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<wire::GetSettings>>(Host::<wire::GetSettings>::new(served.clone()));
    setup.resource::<Host<wire::GetPluginSettings>>(Host::<wire::GetPluginSettings>::new(served.clone()));
    setup.resource::<Host<wire::UpdatePluginSettings>>(Host::<wire::UpdatePluginSettings>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

/* === Private Functions === */

fn plugin_id(id: &str) -> Result<dokime::domain::model::identity::Identifier, Status> {
    projection::identifier(id.to_owned(), "plugin_id")
}

fn plugin_response(
    resources: &Resources, settings: RuntimePluginSettings,
) -> Result<Response<dokime::domain::model::settings::wire::PluginRuntimeSettings>, Status> {
    resources.produce_result(|| projection::encode_plugin_settings(settings, resources.maximum))
}

/* === Tests === */
