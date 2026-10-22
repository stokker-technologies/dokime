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
use std::time::Duration;

// Third Party Imports
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// Local Imports
use super::{RuntimeConfig, Settings};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoragePolicy {
    pub compaction_interval_ms: u64,

    #[serde(default)]
    pub local_retention_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePolicyView {
    pub revision: u64,
    pub configured: StoragePolicy,
    pub effective: StoragePolicy,
    pub restart_required: bool,
}

pub(crate) struct PolicyStore {
    settings: Settings,
    fallback: StoragePolicy,
    effective: StoragePolicy,
    revision: Mutex<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl StoragePolicy {
    pub fn validate(&self) -> Result<()> {
        for (name, value, maximum) in [
            ("compaction interval", Some(self.compaction_interval_ms), 744 * 3_600_000),
            ("local retention", self.local_retention_ms, 3650 * 86_400_000),
        ] {
            if value.is_some_and(|value| value == 0 || value > maximum) {
                return Err(Error::Admission(AdmissionFailure::Validation {
                    code: "storage_policy_invalid".into(),
                    message: format!("{name} must be between 1 and {maximum} milliseconds"),
                    issues: Vec::new(),
                }));
            }
        }
        Ok(())
    }

    fn from_config(config: &RuntimeConfig) -> Result<Self> {
        let millis = |value: Duration| {
            u64::try_from(value.as_millis()).map_err(|_| Error::Operation("storage policy duration exceeds milliseconds range".into()))
        };
        Ok(Self {
            compaction_interval_ms: millis(config.compaction_interval)?,
            local_retention_ms: config.local_retention.map(millis).transpose()?,
        })
    }

    fn apply(&self, config: &mut RuntimeConfig) {
        config.compaction_interval = Duration::from_millis(self.compaction_interval_ms);
        config.local_retention = self.local_retention_ms.map(Duration::from_millis);
    }
}

impl PolicyStore {
    pub(crate) fn open(config: &mut RuntimeConfig, settings: Settings) -> Result<Self> {
        let fallback = StoragePolicy::from_config(config)?;
        let effective = settings.read(|document| document.storage.clone()).unwrap_or_else(|| fallback.clone());
        effective.validate()?;
        effective.apply(config);
        Ok(Self { settings, fallback, effective, revision: Mutex::new(0) })
    }

    pub(crate) async fn read(&self) -> StoragePolicyView {
        let revision = self.revision.lock().await;
        self.view(*revision)
    }

    pub(crate) async fn update(&self, expected: u64, policy: StoragePolicy) -> Result<StoragePolicyView> {
        policy.validate()?;
        let mut revision = self.revision.lock().await;
        if *revision != expected {
            return Err(Error::Admission(AdmissionFailure::Conflict {
                code: "storage_policy_revision_conflict".into(),
                message: "storage policy changed; read it again before saving".into(),
            }));
        }
        if self.configured() == policy {
            return Ok(self.view(*revision));
        }
        let next = expected.checked_add(1).ok_or_else(|| Error::Operation("storage policy revision exhausted".into()))?;
        let settings = self.settings.clone();
        tokio::task::spawn_blocking(move || {
            settings.edit(|document| {
                document.storage = Some(policy);
                Ok::<_, std::io::Error>(())
            })
        })
        .await?
        .map_err(|error| Error::Operation(format!("backend storage policy: {error}")))?;
        *revision = next;
        Ok(self.view(next))
    }

    fn configured(&self) -> StoragePolicy {
        self.settings.read(|document| document.storage.clone()).unwrap_or_else(|| self.fallback.clone())
    }

    fn view(&self, revision: u64) -> StoragePolicyView {
        let configured = self.configured();
        StoragePolicyView { revision, restart_required: configured != self.effective, configured, effective: self.effective.clone() }
    }
}

impl crate::RuntimeHandle {
    pub async fn storage_policy(&self) -> Result<StoragePolicyView> {
        Ok(self.storage_policy.read().await)
    }

    pub async fn update_storage_policy(&self, expected: u64, policy: StoragePolicy) -> Result<StoragePolicyView> {
        policy.validate()?;
        let permit = self.admit_owned_work().ok_or(Error::ShuttingDown)?;
        if *self.shutdown_requested.borrow() {
            return Err(Error::ShuttingDown);
        }
        let store = self.storage_policy.clone();
        let owner = self.local_storage.clone();
        // Cancellation of an RPC cannot detach the save from shutdown or let
        // another backend acquire the directory while publication is in flight.
        tokio::spawn(async move {
            let (_permit, _owner) = (permit, owner);
            store.update(expected, policy).await
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
