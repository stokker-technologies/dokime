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
use std::{fmt::Write, num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use dokime_app::{
    RuntimeHandle,
    config::{Density, ThemeMode},
};
use dokime_contract::operations::settings as wire;

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
pub(super) struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) maximum: usize,
}
struct ReadStoragePolicy {
    resources: Arc<Resources>,
}
struct UpdateStoragePolicy {
    resources: Arc<Resources>,
}
struct ReadApplicationSettings {
    resources: Arc<Resources>,
}
struct UpdateAppearance {
    resources: Arc<Resources>,
}
struct UpdateDataRoot {
    resources: Arc<Resources>,
}
struct UpdateReportDefaults {
    resources: Arc<Resources>,
}

/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl Operation for ReadStoragePolicy {
    type Declaration = wire::ReadStoragePolicy;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut wire::ReadStoragePolicyRequest,
        _: &mut (),
    ) -> Result<Response<wire::StoragePolicyView>, Status> {
        let view = self
            .resources
            .runtime
            .storage_policy()
            .await
            .map_err(super::super::execution::status)?;
        self.resources.policy(view)
    }
}
impl Operation for UpdateStoragePolicy {
    type Declaration = wire::UpdateStoragePolicy;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::UpdateStoragePolicyRequest,
        _: &mut (),
    ) -> Result<Response<wire::StoragePolicyView>, Status> {
        let policy = request
            .policy
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("policy is required"))?;
        let view = self
            .resources
            .runtime
            .update_storage_policy(
                request.expected_revision,
                dokime_app::config::StoragePolicy {
                    save_interval_ms: policy.save_interval_ms,
                    compaction_interval_ms: policy.compaction_interval_ms,
                    local_retention_ms: policy.local_retention_ms,
                },
            )
            .await
            .map_err(super::super::execution::status)?;
        self.resources.policy(view)
    }
}
impl Operation for ReadApplicationSettings {
    type Declaration = wire::ReadApplicationSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut wire::ReadApplicationSettingsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ApplicationSettings>, Status> {
        self.resources
            .application(self.resources.runtime.application_settings())
    }
}
impl Operation for UpdateAppearance {
    type Declaration = wire::UpdateAppearance;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::UpdateAppearanceRequest,
        _: &mut (),
    ) -> Result<Response<wire::ApplicationSettings>, Status> {
        let appearance = request
            .appearance
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("appearance is required"))?;
        if let Some(family) = &appearance.theme_family {
            super::super::value::text(family)?;
        }
        let appearance = dokime_app::config::Appearance {
            theme_mode: match appearance.theme_mode {
                wire::ThemeMode::Unspecified | wire::ThemeMode::System => ThemeMode::System,
                wire::ThemeMode::Light => ThemeMode::Light,
                wire::ThemeMode::Dark => ThemeMode::Dark,
            },
            density: match appearance.density {
                wire::Density::Unspecified | wire::Density::Comfortable => Density::Comfortable,
                wire::Density::Compact => Density::Compact,
                wire::Density::Touch => Density::Touch,
            },
            theme_family: appearance.theme_family.clone(),
        };
        let view = self
            .resources
            .runtime
            .update_appearance(appearance)
            .await
            .map_err(super::super::execution::status)?;
        self.resources.application(view)
    }
}
impl Operation for UpdateDataRoot {
    type Declaration = wire::UpdateDataRoot;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::UpdateDataRootRequest,
        _: &mut (),
    ) -> Result<Response<wire::ApplicationSettings>, Status> {
        if let Some(path) = &request.data_root {
            super::super::value::text(path)?;
        }
        let root = request.data_root.as_ref().map(std::path::PathBuf::from);
        let view = self
            .resources
            .runtime
            .update_data_root(root)
            .await
            .map_err(super::super::execution::status)?;
        self.resources.application(view)
    }
}
impl Operation for UpdateReportDefaults {
    type Declaration = wire::UpdateReportDefaults;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::UpdateReportDefaultsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ApplicationSettings>, Status> {
        let report = request
            .report
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("report defaults are required"))?;
        let view = self
            .resources
            .runtime
            .update_report_defaults(dokime_app::config::ReportDefaults {
                plot_points: report.plot_points,
                plot_downsample: report.plot_downsample,
            })
            .await
            .map_err(super::super::execution::status)?;
        self.resources.application(view)
    }
}
impl Resources {
    pub(super) fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        required: usize,
        build: impl FnOnce() -> T,
    ) -> Result<Response<T>, Status> {
        self.produce_result(required, || Ok(build()))
    }
    pub(super) fn produce_result<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        required: usize,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        if required > self.maximum {
            return Err(Status::resource_exhausted(
                "settings response exceeds configured memory budget",
            ));
        }
        let bytes = NonZeroUsize::new(required)
            .ok_or_else(|| Status::resource_exhausted("invalid settings response allowance"))?;
        let preparation = self
            .runtime
            .storage()
            .prepare_production(bytes)
            .map_err(memory_error)?;
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "settings response exceeds configured message budget",
            ));
        }
        preparation
            .finish(output)
            .map_err(memory_error)?
            .try_into_value()
            .map_err(memory_error)?
            .try_into_response()
            .map_err(memory_error)
    }
    fn policy(
        &self,
        view: dokime_app::config::StoragePolicyView,
    ) -> Result<Response<wire::StoragePolicyView>, Status> {
        self.produce(std::mem::size_of::<wire::StoragePolicyView>(), || {
            let encode = |p: dokime_app::config::StoragePolicy| wire::StoragePolicy {
                save_interval_ms: p.save_interval_ms,
                compaction_interval_ms: p.compaction_interval_ms,
                local_retention_ms: p.local_retention_ms,
            };
            wire::StoragePolicyView {
                revision: view.revision,
                configured: Some(encode(view.configured)),
                effective: Some(encode(view.effective)),
                restart_required: view.restart_required,
            }
        })
    }
    fn application(
        &self,
        view: dokime_app::config::ApplicationSettingsView,
    ) -> Result<Response<wire::ApplicationSettings>, Status> {
        let mut required = std::mem::size_of::<wire::ApplicationSettings>().saturating_add(
            view.appearance
                .theme_family
                .as_ref()
                .map_or(0, String::capacity),
        );
        for path in [
            Some(&view.application_root),
            view.configured_data_root.as_ref(),
            Some(&view.default_data_root),
            Some(&view.effective_data_root),
        ]
        .into_iter()
        .flatten()
        {
            required = required.saturating_add(path_bytes(path));
        }
        self.produce(required, || {
            let path = |p: std::path::PathBuf| {
                let mut output = String::with_capacity(path_bytes(&p));
                let _ = write!(&mut output, "{}", p.display());
                output
            };
            wire::ApplicationSettings {
                appearance: Some(wire::AppearancePreference {
                    theme_mode: match view.appearance.theme_mode {
                        ThemeMode::System => wire::ThemeMode::System,
                        ThemeMode::Light => wire::ThemeMode::Light,
                        ThemeMode::Dark => wire::ThemeMode::Dark,
                    },
                    density: match view.appearance.density {
                        Density::Comfortable => wire::Density::Comfortable,
                        Density::Compact => wire::Density::Compact,
                        Density::Touch => wire::Density::Touch,
                    },
                    theme_family: view.appearance.theme_family,
                }),
                report: Some(wire::ReportPreference {
                    plot_points: view.report.plot_points,
                    plot_downsample: view.report.plot_downsample,
                }),
                application_root: path(view.application_root),
                configured_data_root: view.configured_data_root.map(path),
                default_data_root: path(view.default_data_root),
                effective_data_root: path(view.effective_data_root),
                restart_required: view.restart_required,
            }
        })
    }
}
/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
    });
    runtime
        .operations()
        .register_wire_operation::<ReadStoragePolicy>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateStoragePolicy>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<ReadApplicationSettings>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateAppearance>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateDataRoot>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateReportDefaults>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    Ok(())
}
/* === Private Functions === */
fn path_bytes(path: &std::path::Path) -> usize {
    struct Counter(usize);
    impl std::fmt::Write for Counter {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.saturating_add(text.len());
            Ok(())
        }
    }
    let mut counter = Counter(0);
    let _ = write!(&mut counter, "{}", path.display());
    counter.0
}
fn memory_error(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("settings response admission failed")
    } else {
        Status::internal("settings response admission failed")
    };
    crate::grpc::error::detailed(status, &source.into())
}
/* === Tests === */
