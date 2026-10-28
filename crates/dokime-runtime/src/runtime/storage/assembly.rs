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

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn configure(setup: &mut harmos::Setup<crate::app::Dokime>, parts: &crate::runtime::handle::BootParts) {
    let session_id = parts.config.session_id.clone();
    let record_namespace = parts.record_namespace.clone();
    let records = parts.record_source.clone();
    let storage = &parts.storage;
    let managed_storage = parts.config.managed_storage.clone();
    let projection = super::record_capture::Projection(records.clone());
    setup.settings().storage(storage).managed_storage(managed_storage).store_records(projection).reconcile(
        move |state_manager, _storage| async move {
            // Validate Replayed Ownership Without Copying The Runtime Graph
            state_manager
                .state()
                .read(|state| state.validate_execution_ownership())
                .await
                .map_err(reconciliation_error)?
                .value
                .map_err(|reason| reconciliation_error(std::io::Error::other(reason)))?;
            records.begin(state_manager.applied());
            records.initialize().await.map_err(reconciliation_error)?;

            state_manager
                .commit(crate::transactions::MarkRuntimeStarted { session_id, record_namespace })
                .await
                .map_err(reconciliation_error)?;
            state_manager
                .commit(crate::transactions::RecoverInterruptedCycles { at: crate::model::time::now_utc() })
                .await
                .map_err(reconciliation_error)?;
            Ok(())
        },
    );
}

/* === Private Functions === */

fn reconciliation_error(source: impl std::error::Error + Send + Sync + 'static) -> harmos::Error {
    harmos::Error::backend("reconcile native Dokime storage", harmos::BackendFailure::Permanent, source)
}

/* === Tests === */
