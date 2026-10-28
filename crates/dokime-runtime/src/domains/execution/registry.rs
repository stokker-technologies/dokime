/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::definition::SuiteRegistry;
pub use crate::definition::SuiteSnapshot;
pub(crate) use crate::transactions::{RemoveSuiteSet, SetSuiteSet};

use crate::definition::SuiteSet;
use crate::{RuntimeHandle, runtime::Error, unavailable_rejection};

impl RuntimeHandle {
    pub async fn set_suites(&self, source: impl Into<String>, suites: SuiteSet) -> std::result::Result<(), Error> {
        let source = source.into();
        let permit = self.admit_owned_work().ok_or_else(|| unavailable_rejection("runtime is shutting down"))?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _admission = runtime.plugin_admission.lock().await;
            let session = runtime.storage().clone();
            for registration in &suites.suites {
                let validation = crate::storage::objects::suite::Suite::new(&registration.suite_id, &registration.suite)
                    .map_err(|error| unavailable_rejection(error.to_string()))?;
                // Validation Settles Before Independent Admission And State Commit.
                drop(validation);
            }
            drop(session);
            crate::prompts::admission::validate_suites(&source, &suites).map_err(unavailable_rejection)?;
            let transaction = SetSuiteSet { source: source.clone(), catalog: suites };
            runtime.state_manager.commit(transaction).await.map_err(|error| unavailable_rejection(error.to_string()))?;
            runtime.refresh_state().await.map_err(|error| unavailable_rejection(error.to_string()))?;

            runtime.capture_suite_objects(&source).await.map_err(|error| unavailable_rejection(error.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|error| unavailable_rejection(error.to_string()))?
    }

    pub(crate) async fn capture_suite_objects(&self, source: &str) -> crate::runtime::Result<()> {
        use crate::storage::{Error as StorageError, objects::suite::Suite};
        use harmos::MemorySize;

        // One Borrowed Catalog Cut Owns Both Selection And Per-Suite Preparation.
        let catalog = self.watch_state().latest();
        for registered in catalog.state().suites.suites().filter(|suite| suite.source == source) {
            let (stored, typed) = {
                let stored = Suite::new(&registered.suite_id, &registered.suite)?;
                let contracts = || registered.telemetry_contracts.iter().filter(|contract| contract.storage_schema.is_some());
                let mut peak = 0usize;
                let mut count = 0usize;
                for contract in contracts() {
                    let memory = contract.memory_size();
                    if memory == usize::MAX {
                        return Err(StorageError::Invalid("suite schema memory overflows").into());
                    }
                    peak = peak.checked_add(memory).ok_or(StorageError::Invalid("suite schema memory overflows"))?;
                    count += 1;
                }
                let seed = harmos::storage::ReadBatch::new(());
                let typed = seed.try_map(|_| -> crate::storage::Result<_> {
                    let mut typed = Vec::new();
                    typed.try_reserve_exact(count).map_err(|_| StorageError::Invalid("suite schemas allocation failed"))?;
                    for contract in contracts() {
                        typed.push(contract.clone());
                    }
                    Ok(typed)
                })?;
                (stored, typed)
            };
            self.execution_storage.register_schemas(&typed).await?;
            self.storage().write_object(&*stored).await?;
        }
        Ok(())
    }

    pub async fn remove_suites(&self, source: impl Into<String>) -> std::result::Result<(), Error> {
        self.state_manager
            .commit(RemoveSuiteSet { source: source.into() })
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        self.refresh_state().await.map_err(|error| unavailable_rejection(error.to_string()))?;
        Ok(())
    }
}
