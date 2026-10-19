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

pub(crate) use crate::definition::SuiteRegistry;
pub use crate::definition::SuiteSnapshot;
pub(crate) use crate::transactions::{RemoveSuiteSet, SetSuiteSet};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::definition::SuiteSet;
use crate::{RuntimeHandle, runtime::Error, unavailable_rejection};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn set_suites(&self, source: impl Into<String>, suites: SuiteSet) -> std::result::Result<(), Error> {
        let source = source.into();
        let permit = self.admit_owned_work().ok_or_else(|| unavailable_rejection("runtime is shutting down"))?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _admission = runtime.plugin_admission.lock().await;
            for registration in &suites.suites {
                crate::storage::objects::suite::Suite::new(&registration.suite_id, &registration.suite)
                    .map_err(|error| unavailable_rejection(error.to_string()))?;
            }
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
        let source = source.to_owned();
        let ids = self
            .state_manager
            .state()
            .read(move |state| {
                state.suites.suites().filter(|suite| suite.source == source).map(|suite| suite.suite_id.clone()).collect::<Vec<_>>()
            })
            .await
            .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
            .value;

        for id in ids {
            let registered = self
                .state_manager
                .state()
                .read(move |state| state.suites.get(&id).cloned())
                .await
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
                .value
                .ok_or_else(|| crate::runtime::Error::Operation("admitted suite is absent".into()))?;
            let typed = registered.telemetry_contracts.into_iter().filter(|contract| contract.storage_schema.is_some()).collect::<Vec<_>>();
            crate::runtime::storage::telemetry::register_schemas(self.storage(), &typed).await?;
            let stored = crate::storage::objects::suite::Suite::new(&registered.suite_id, &registered.suite)
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            self.storage().write_object(&stored).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
