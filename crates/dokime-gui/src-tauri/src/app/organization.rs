/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Installed Organization Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env;

// Third Party Imports
use serde::Serialize;
use tauri::Manager;

// Local Imports
use dokime_packaging::organization::{self, Appearance, ResolvedPack, policy};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct OrganizationState {
    pack: Option<LoadedPack>,
    error: Option<String>,
}

// The pack as the frontend applies it before it renders.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadedPack {
    #[serde(flatten)]
    appearance: Appearance,

    pub(crate) updates: Option<policy::UpdatePolicy>,

    #[serde(skip)]
    application_icon: Option<Vec<u8>>,

    #[serde(skip)]
    tray_icon: Option<Vec<u8>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootScript<'a> {
    pack: Option<&'a LoadedPack>,
    error: Option<&'a str>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl OrganizationState {
    pub(crate) fn label(&self) -> &str {
        self.pack.as_ref().map(|pack| pack.appearance.label.as_str()).unwrap_or(crate::PRODUCT)
    }

    pub(crate) fn window_icon(&self) -> Option<tauri::image::Image<'static>> {
        let bytes = self.pack.as_ref()?.application_icon.as_deref()?;
        tauri::image::Image::from_bytes(bytes).ok()
    }

    pub(crate) fn title(&self) -> Option<String> {
        let appearance = &self.pack.as_ref()?.appearance;
        Some(appearance.title_template.replace("{app}", &appearance.label))
    }

    pub(crate) fn update_policy(&self) -> Result<policy::UpdatePolicy, String> {
        let updates = self.pack.as_ref().and_then(|pack| pack.updates.clone()).unwrap_or_default();
        updates.validate()?;
        Ok(updates)
    }

    // The script every main window runs before the frontend loads.
    pub(crate) fn initialization_script(&self) -> String {
        let boot = BootScript {
            pack: self.pack.as_ref(),
            error: self.error.as_deref(),
        };
        let json = serde_json::to_string(&boot).unwrap_or_else(|_| "{}".to_owned());
        format!("window.__DOKIME_ORGANIZATION_PACK__ = {json};")
    }
}

/* === Public Functions === */

pub(crate) fn resolve() -> OrganizationState {
    match installed_or_bundled().and_then(loaded) {
        Ok(pack) => OrganizationState { pack: Some(pack), error: None },
        Err(error) => OrganizationState { pack: None, error: Some(error) },
    }
}

pub(crate) fn setup(app: &tauri::App) -> tauri::Result<()> {
    let state = app.state::<OrganizationState>();
    if let Some(error) = &state.error {
        tracing::error!(%error, "the installed organization could not be loaded");
    }
    if let Some(pack) = &state.pack
        && let Some(bytes) = &pack.tray_icon
    {
        let icon = tauri::image::Image::from_bytes(bytes)?;
        let menu = tauri::menu::Menu::new(app)?;
        let tray =
            tauri::tray::TrayIconBuilder::with_id("organization").icon(icon).tooltip(&pack.appearance.label).menu(&menu).build(app)?;
        app.manage(tray);
    }
    Ok(())
}

/* === Private Functions === */

// The Installed Pack, Falling Back To the Embedded Default Only When Absent.
fn installed_or_bundled() -> Result<ResolvedPack, String> {
    let executable = env::current_exe().ok();
    let installation = executable.as_deref().and_then(|executable| dokime_packaging::installation::directory(executable).ok());
    match installation.and_then(organization::installed) {
        Some(directory) => organization::read(&directory),
        None => bundled(),
    }
}

fn loaded(pack: ResolvedPack) -> Result<LoadedPack, String> {
    let appearance = pack.appearance()?;
    let application_icon = pack.asset("application").map(|(_, bytes)| bytes.to_vec());
    let tray_icon = pack.asset("tray").map(|(_, bytes)| bytes.to_vec());
    for bytes in [&application_icon, &tray_icon].into_iter().flatten() {
        tauri::image::Image::from_bytes(bytes).map_err(|_| "organization application and tray artwork must be valid PNG images")?;
    }
    // Reading the Pack Validated Its Update Policy.
    let updates = pack.manifest.get("updates").and_then(|updates| updates.clone().try_into().ok());
    Ok(LoadedPack {
        appearance,
        updates,
        application_icon,
        tray_icon,
    })
}

// The Default Pack Embedded in the Product.
fn bundled() -> Result<ResolvedPack, String> {
    let files: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/organization.rs"));
    organization::from_files(files.iter().map(|(name, bytes)| ((*name).to_owned(), bytes.to_vec())).collect())
}

#[cfg(feature = "testing")]
pub(crate) mod testing {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports

    use super::*;

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    pub fn inspect_bundled() -> Result<serde_json::Value, String> {
        serde_json::to_value(loaded(bundled()?)?).map_err(|error| error.to_string())
    }

    /* === Private Functions === */

    /* === Tests === */
}

/* === Tests === */
