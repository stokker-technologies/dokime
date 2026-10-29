/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{env, io, path::PathBuf};

use super::{ApplicationPaths, DEFAULT_APPLICATION_ID, organization};

#[cfg(feature = "native")]
use super::Config;

#[cfg(feature = "native")]
const DATA_ROOT_ENV: &str = "DOKIME_OUTPUT_ROOT";

#[cfg(feature = "native")]
const PLUGIN_LOOKUP_ENV: &str = "DOKIME_PLUGINS_DIR";

#[derive(Clone, Debug, Default)]
pub struct ApplicationOptions {
    pub application_root: Option<PathBuf>,
    pub storage_root: Option<PathBuf>,
    pub plugins_root: Option<PathBuf>,
}

impl ApplicationOptions {
    #[cfg(feature = "native")]
    pub fn resolve(self) -> io::Result<Config> {
        let explicit = self.application_root.is_some();
        let ambient = |name| {
            if explicit { None } else { environment_path(name) }
        };
        let (brand, paths) = self.load()?;

        let configured_data = match self.storage_root {
            Some(root) => Some(root),
            None => ambient(DATA_ROOT_ENV),
        };
        let data = match configured_data {
            Some(data) => std::path::absolute(data)?,
            None => paths.default_data(),
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
        config.application = Some(paths);
        config.brand = brand;
        Ok(config)
    }

    fn load(&self) -> io::Result<(Option<organization::ResolvedBrand>, ApplicationPaths)> {
        let executable = env::current_exe()?;
        let paths = ApplicationPaths::resolve_for(&executable, DEFAULT_APPLICATION_ID, self.application_root.clone())?;
        let active = paths.root.join("organization/active");
        let pack = match organization::resolve(active.is_dir().then_some(active.as_path())) {
            Ok(pack) => pack,
            Err(error) if active.is_dir() => {
                tracing::error!(%error, "the active organization is invalid; using bundled defaults");
                organization::resolve(None).map_err(io::Error::other)?
            }
            Err(error) => return Err(io::Error::other(error)),
        };
        let brand = Some(pack.brand().map_err(io::Error::other)?);
        Ok((brand, paths))
    }
}

fn environment_path(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}
