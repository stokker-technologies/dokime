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

use crate::grpc::transcode::Transcode;

use dokime::RuntimeHandle;
use dokime::domain::model::settings::wire::SettingsSnapshot;
use dokime::harmos::domain::Portable;
use dokime::harmos::{Context, RegistrationExt, Response};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::settings as wire;
use std::sync::Arc;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ReadApplicationSettings> for Resources {
    fn serve<'a>(
        &'a self, _: wire::ReadApplicationSettings, _: Context<'a, wire::ReadApplicationSettings>,
    ) -> Served<'a, SettingsSnapshot> {
        Box::pin(async move {
            let snapshot = self.runtime.read_application_settings().await.map_err(super::super::execution::status)?;
            self.produce(|| snapshot)
        })
    }
}

impl Serve<wire::UpdateApplicationSettings> for Resources {
    fn serve<'a>(
        &'a self, request: wire::UpdateApplicationSettings, _: Context<'a, wire::UpdateApplicationSettings>,
    ) -> Served<'a, SettingsSnapshot> {
        Box::pin(async move {
            let snapshot =
                self.runtime.update_application_settings(request.update.clone()).await.map_err(super::super::execution::status)?;
            self.produce(|| snapshot)
        })
    }
}

impl Resources {
    pub(super) fn produce<T: Portable + Send + Sync + 'static>(&self, build: impl FnOnce() -> T) -> Result<Response<T>, Status> {
        self.produce_result(|| Ok(build()))
    }

    pub(super) fn produce_result<T: Portable + Send + Sync + 'static>(
        &self, build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.wire_len()? > self.maximum {
            return Err(Status::resource_exhausted("settings response exceeds transport limit"));
        }
        Ok(output.into())
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::ReadApplicationSettings>::new(served.clone()))?;
    dependencies.insert(Host::<wire::UpdateApplicationSettings>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::ReadApplicationSettings>()
        .operation::<wire::UpdateApplicationSettings>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
