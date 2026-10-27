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

use super::ApplicationPaths;
use crate::{
    plugins::PluginSettings,
    storage::{Error, sqlite::State},
};
use dokime_domain::model::{
    identity::{DokimeNodeIdentity, NodeId},
    secrets::catalog::Catalog,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Settings {
    paths: ApplicationPaths,
    database: State,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationMetadata {
    pub identity: DokimeNodeIdentity,
    pub plugins: BTreeMap<Uuid, PluginSettings>,
    pub secrets: Catalog,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for Settings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Settings").field("paths", &self.paths).finish_non_exhaustive()
    }
}

impl Settings {
    pub async fn open(application_root: impl Into<PathBuf>, database: State) -> Result<Self, Error> {
        let paths = ApplicationPaths::new(application_root);
        database.initialize_settings().await?;
        database.read_application_settings().await?;
        database.read_application_metadata().await?;
        Ok(Self { paths, database })
    }

    pub fn paths(&self) -> &ApplicationPaths {
        &self.paths
    }

    pub async fn read<T>(&self, view: impl FnOnce(&ApplicationMetadata) -> T) -> Result<T, Error> {
        let metadata = self.database.read_application_metadata().await?;
        Ok(view(&metadata))
    }

    pub async fn edit<T: Send + 'static, E: From<Error> + Send + 'static>(
        &self, change: impl FnOnce(&mut ApplicationMetadata) -> Result<T, E> + Send + 'static,
    ) -> Result<T, E> {
        self.database.edit_application_metadata(change).await
    }
}

impl ApplicationMetadata {
    pub(crate) fn new() -> Self {
        let label = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")).ok();
        let label = label.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty());
        Self {
            identity: DokimeNodeIdentity::new(crate::model::identity::new_id::<NodeId>(), label, crate::model::time::now_utc()),
            plugins: BTreeMap::new(),
            secrets: Catalog::default(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        self.secrets.validate().map_err(str::to_owned)?;
        self.secrets.check_limits().map_err(str::to_owned)?;
        for (id, plugin) in &self.plugins {
            plugin.validate().map_err(|error| format!("plugin `{id}`: {error}"))?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
