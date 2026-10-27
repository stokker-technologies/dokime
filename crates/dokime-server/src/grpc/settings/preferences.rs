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
use dokime::domain::operations::settings as wire;
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::{Context, Operation, Response};
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

struct ReadApplicationSettings {
    resources: Arc<Resources>,
}

struct UpdateApplicationSettings {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReadApplicationSettings {
    type Declaration = wire::ReadApplicationSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::ReadApplicationSettings, _: &mut (),
    ) -> Result<Response<SettingsSnapshot>, Status> {
        let snapshot = self.resources.runtime.read_application_settings().await.map_err(super::super::execution::status)?;
        self.resources.produce(|| snapshot)
    }
}

impl Operation for UpdateApplicationSettings {
    type Declaration = wire::UpdateApplicationSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::UpdateApplicationSettings, _: &mut (),
    ) -> Result<Response<SettingsSnapshot>, Status> {
        let snapshot =
            self.resources.runtime.update_application_settings(request.update.clone()).await.map_err(super::super::execution::status)?;
        self.resources.produce(|| snapshot)
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

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum });
    runtime.operations().register_wire_operation::<ReadApplicationSettings>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<UpdateApplicationSettings>("dokime", dokime::app::Lanes::Open, resources)?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
