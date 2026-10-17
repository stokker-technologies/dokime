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
use std::{fmt::Write, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use dokime_runtime::{
    RuntimeHandle,
    config::{Density, ThemeMode},
};
use dokime_domain::operations::settings as wire;

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
        _: &mut wire::ReadStoragePolicy,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::StoragePolicyView>, Status> {
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
        request: &mut wire::UpdateStoragePolicy,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::StoragePolicyView>, Status> {
        let policy = request
            .policy
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("policy is required"))?;
        let view = self
            .resources
            .runtime
            .update_storage_policy(
                request.expected_revision,
                dokime_runtime::config::StoragePolicy {
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
        _: &mut wire::ReadApplicationSettings,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::ApplicationSettings>, Status> {
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
        request: &mut wire::UpdateAppearance,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::ApplicationSettings>, Status> {
        let appearance = request
            .appearance
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("appearance is required"))?;
        if let Some(family) = &appearance.theme_family {
            super::super::value::text(family)?;
        }
        let appearance = dokime_runtime::config::Appearance {
            theme_mode: match appearance.theme_mode {
                dokime_domain::model::settings::wire::ThemeMode::Unspecified | dokime_domain::model::settings::wire::ThemeMode::System => ThemeMode::System,
                dokime_domain::model::settings::wire::ThemeMode::Light => ThemeMode::Light,
                dokime_domain::model::settings::wire::ThemeMode::Dark => ThemeMode::Dark,
            },
            density: match appearance.density {
                dokime_domain::model::settings::wire::Density::Unspecified | dokime_domain::model::settings::wire::Density::Comfortable => Density::Comfortable,
                dokime_domain::model::settings::wire::Density::Compact => Density::Compact,
                dokime_domain::model::settings::wire::Density::Touch => Density::Touch,
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
        request: &mut wire::UpdateDataRoot,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::ApplicationSettings>, Status> {
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
        request: &mut wire::UpdateReportDefaults,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::settings::wire::ApplicationSettings>, Status> {
        let report = request
            .report
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("report defaults are required"))?;
        let view = self
            .resources
            .runtime
            .update_report_defaults(dokime_runtime::config::ReportDefaults {
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
        build: impl FnOnce() -> T,
    ) -> Result<Response<T>, Status> {
        self.produce_result(|| Ok(build()))
    }
    pub(super) fn produce_result<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "settings response exceeds configured message budget",
            ));
        }
        Ok(output.into())
    }
    fn policy(
        &self,
        view: dokime_runtime::config::StoragePolicyView,
    ) -> Result<Response<dokime_domain::model::settings::wire::StoragePolicyView>, Status> {
        self.produce(|| {
            let encode = |p: dokime_runtime::config::StoragePolicy| dokime_domain::model::settings::wire::StoragePolicy {
                compaction_interval_ms: p.compaction_interval_ms,
                local_retention_ms: p.local_retention_ms,
            };
            dokime_domain::model::settings::wire::StoragePolicyView {
                revision: view.revision,
                configured: Some(encode(view.configured)),
                effective: Some(encode(view.effective)),
                restart_required: view.restart_required,
            }
        })
    }
    fn application(
        &self,
        view: dokime_runtime::config::ApplicationSettingsView,
    ) -> Result<Response<dokime_domain::model::settings::wire::ApplicationSettings>, Status> {
        let mut required = std::mem::size_of::<dokime_domain::model::settings::wire::ApplicationSettings>().saturating_add(
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
        self.produce(|| {
            let path = |p: std::path::PathBuf| {
                let mut output = String::with_capacity(path_bytes(&p));
                let _ = write!(&mut output, "{}", p.display());
                output
            };
            dokime_domain::model::settings::wire::ApplicationSettings {
                appearance: Some(dokime_domain::model::settings::wire::AppearancePreference {
                    theme_mode: match view.appearance.theme_mode {
                        ThemeMode::System => dokime_domain::model::settings::wire::ThemeMode::System,
                        ThemeMode::Light => dokime_domain::model::settings::wire::ThemeMode::Light,
                        ThemeMode::Dark => dokime_domain::model::settings::wire::ThemeMode::Dark,
                    },
                    density: match view.appearance.density {
                        Density::Comfortable => dokime_domain::model::settings::wire::Density::Comfortable,
                        Density::Compact => dokime_domain::model::settings::wire::Density::Compact,
                        Density::Touch => dokime_domain::model::settings::wire::Density::Touch,
                    },
                    theme_family: view.appearance.theme_family,
                }),
                report: Some(dokime_domain::model::settings::wire::ReportPreference {
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
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateStoragePolicy>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<ReadApplicationSettings>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateAppearance>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateDataRoot>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<UpdateReportDefaults>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
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
/* === Tests === */
