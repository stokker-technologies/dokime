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
use std::{env, io, path::PathBuf};

// Third Party Imports

// Local Imports
use super::{ApplicationPaths, Config, DEFAULT_APPLICATION_ID, Settings, organization};

/* === Constants === */

const APPLICATION_ROOT_ENV: &str = "DOKIME_APPLICATION_ROOT";
const DATA_ROOT_ENV: &str = "DOKIME_OUTPUT_ROOT";
const PLUGIN_LOOKUP_ENV: &str = "DOKIME_PLUGINS_DIR";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default)]
pub struct ApplicationOptions {
    pub application_root: Option<PathBuf>,
    pub storage_root: Option<PathBuf>,
    pub plugins_root: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ApplicationOptions {
    pub fn resolve(self) -> io::Result<Config> {
        let explicit = self.application_root.is_some();
        let ambient = |name| (!explicit).then(|| environment_path(name)).flatten();
        let brand = organization::brand();

        // Resolve The Application Root And Its Settings
        let root = match self.application_root.or_else(|| ambient(APPLICATION_ROOT_ENV)) {
            Some(root) => std::path::absolute(root)?,
            None => ApplicationPaths::resolve(brand.as_ref().map_or(DEFAULT_APPLICATION_ID, |brand| &brand.identity.identifier))?.root,
        };
        let settings = Settings::load(&root)?;

        // Compose The Launch
        let data = match self.storage_root.or_else(|| ambient(DATA_ROOT_ENV)) {
            Some(data) => std::path::absolute(data)?,
            None => settings.data_root(),
        };
        // Keep The Root In The Form It Was Configured: Canonicalizing Only
        // Once It Exists Gave Windows A Verbatim `\\?\` Root From The Second
        // Launch On, So The Same Root Changed Identity Across Restarts.
        let mut config = Config::local(data);
        config.title = brand.as_ref().map_or_else(|| "Dokime".to_owned(), |brand| brand.identity.label.clone());
        config.plugin_lookup = self.plugins_root.or_else(|| ambient(PLUGIN_LOOKUP_ENV)).map(std::path::absolute).transpose()?;
        config.application_root = Some(root);
        config.brand = brand;
        Ok(config)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn environment_path(name: &str) -> Option<PathBuf> {
    env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from)
}

/* === Tests === */
