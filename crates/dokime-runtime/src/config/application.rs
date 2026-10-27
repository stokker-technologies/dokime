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
use super::{ApplicationPaths, DEFAULT_APPLICATION_ID, organization};

#[cfg(feature = "host")]
use super::Config;

/* === Constants === */

const APPLICATION_ROOT_ENV: &str = "DOKIME_APPLICATION_ROOT";

#[cfg(feature = "host")]
const DATA_ROOT_ENV: &str = "DOKIME_OUTPUT_ROOT";

#[cfg(feature = "host")]
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
    #[cfg(feature = "host")]
    pub fn resolve(self) -> io::Result<Config> {
        let explicit = self.application_root.is_some();
        let ambient = |name| {
            if explicit { None } else { environment_path(name) }
        };
        let (brand, paths) = self.load()?;
        let root = paths.root;

        // Compose The Launch
        let configured_data = match self.storage_root {
            Some(root) => Some(root),
            None => ambient(DATA_ROOT_ENV),
        };
        let data = match configured_data {
            Some(data) => std::path::absolute(data)?,
            None => ApplicationPaths::new(&root).default_data(),
        };
        // Keep The Root In The Form It Was Configured: Canonicalizing Only
        // Once It Exists Gave Windows A Verbatim `\\?\` Root From The Second
        // Launch On, So The Same Root Changed Identity Across Restarts.
        let mut config = Config::local(data);
        config.title = match brand.as_ref() {
            Some(brand) => brand.identity.label.clone(),
            None => "Dokime".to_owned(),
        };
        let plugins = match self.plugins_root {
            Some(root) => Some(root),
            None => ambient(PLUGIN_LOOKUP_ENV),
        };
        config.plugin_lookup = plugins.map(std::path::absolute).transpose()?;
        config.application_root = Some(root);
        config.brand = brand;
        Ok(config)
    }

    fn load(&self) -> io::Result<(Option<organization::ResolvedBrand>, ApplicationPaths)> {
        let brand = organization::compiled_brand();
        let configured_root = match &self.application_root {
            Some(root) => Some(root.clone()),
            None => environment_path(APPLICATION_ROOT_ENV),
        };
        let application_id = brand.as_ref().map_or(DEFAULT_APPLICATION_ID, |brand| &brand.identity.identifier);
        let root = match configured_root {
            Some(root) => std::path::absolute(root)?,
            None => ApplicationPaths::resolve(application_id)?.root,
        };

        let paths = ApplicationPaths::new(root);
        Ok((brand, paths))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn environment_path(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

/* === Tests === */
