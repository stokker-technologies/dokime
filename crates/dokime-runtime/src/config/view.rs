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
use std::path::PathBuf;
// Third Party Imports

// Local Imports
use super::{Appearance, ReportDefaults};
use crate::{runtime::Error, runtime::Result};

/* === Constants === */
/// The most points one plot may request, live or in a report.
pub const MAX_PLOT_POINTS: u64 = 100_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationSettingsView {
    pub appearance: Appearance,
    pub report: ReportDefaults,
    pub application_root: PathBuf,
    pub configured_data_root: Option<PathBuf>,
    pub default_data_root: PathBuf,
    pub effective_data_root: PathBuf,
    pub restart_required: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl crate::RuntimeHandle {
    pub fn application_settings(&self) -> ApplicationSettingsView {
        let settings = self.settings();
        let (appearance, report, configured) = settings.read(|document| {
            (
                document.appearance.clone(),
                document.report,
                document.data_root.clone(),
            )
        });
        let effective = self.storage_root();
        ApplicationSettingsView {
            appearance,
            report,
            application_root: settings.paths().root.clone(),
            restart_required: settings.data_root() != effective,
            configured_data_root: configured,
            default_data_root: settings.paths().default_data(),
            effective_data_root: effective,
        }
    }

    pub async fn update_appearance(
        &self,
        appearance: Appearance,
    ) -> Result<ApplicationSettingsView> {
        if appearance
            .theme_family
            .as_ref()
            .is_some_and(|family| family.is_empty() || family.len() > 256)
        {
            return Err(invalid("theme family must be between 1 and 256 bytes"));
        }
        let settings = self.settings().clone();
        tokio::task::spawn_blocking(move || {
            settings.edit(|document| {
                document.appearance = appearance;
                Ok::<_, std::io::Error>(())
            })
        })
        .await??;
        Ok(self.application_settings())
    }

    /// The points and strategy every plot starts from. Points stay within the
    /// live plot cap; the strategy is one a plot can name.
    pub async fn update_report_defaults(
        &self,
        report: ReportDefaults,
    ) -> Result<ApplicationSettingsView> {
        if report.plot_points == 0 || u64::from(report.plot_points) > MAX_PLOT_POINTS {
            return Err(invalid(&format!(
                "default plot points must be between 1 and {MAX_PLOT_POINTS}"
            )));
        }
        if report.plot_downsample == crate::document::DownsampleTechnique::Unspecified {
            return Err(invalid("default plot strategy is required"));
        }
        let settings = self.settings().clone();
        tokio::task::spawn_blocking(move || {
            settings.edit(|document| {
                document.report = report;
                Ok::<_, std::io::Error>(())
            })
        })
        .await??;
        Ok(self.application_settings())
    }

    pub async fn update_data_root(
        &self,
        data_root: Option<PathBuf>,
    ) -> Result<ApplicationSettingsView> {
        if data_root.as_ref().is_some_and(|root| !root.is_absolute()) {
            return Err(invalid("data root must be an absolute path"));
        }
        if data_root
            .as_ref()
            .is_some_and(|root| root.starts_with(&self.settings().paths().root))
            && data_root.as_ref() != Some(&self.settings().paths().default_data())
        {
            return Err(invalid("data root must not be inside the application root"));
        }
        let settings = self.settings().clone();
        tokio::task::spawn_blocking(move || {
            let selected = data_root
                .clone()
                .unwrap_or_else(|| settings.paths().default_data());
            preflight(&selected)?;
            settings.edit(|document| {
                document.data_root = data_root;
                Ok::<_, std::io::Error>(())
            })
        })
        .await??;
        Ok(self.application_settings())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn preflight(root: &std::path::Path) -> std::io::Result<()> {
    dokime_filesystem::publication::create_dir_all(root)?;
    let probe = root.join(format!(".dokime-probe-{}", std::process::id()));
    dokime_filesystem::publication::write(&probe, b"dokime storage probe")?;
    std::fs::remove_file(&probe)
}

fn invalid(message: &str) -> Error {
    Error::Validation {
        code: "application_settings_invalid".into(),
        message: message.into(),
        issues: Vec::new(),
    }
}

/* === Tests === */
