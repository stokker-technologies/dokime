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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::app::HarmosRuntime;
use crate::runtime::state::RuntimeState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn build(
    fresh: RuntimeState, records: Arc<super::record_capture::Source>, storage: &crate::storage::Manager, resources: crate::app::Resources,
) -> std::result::Result<HarmosRuntime, harmos::runtime::Incompatible> {
    let session_id = fresh.session_id.clone();
    let record_namespace = fresh.process_epoch().clone();
    let group_limits = super::record_capture::group_limits();
    let projection = super::record_capture::Projection(records.clone());
    crate::app::builder(fresh, resources)
        .managed_state(group_limits)
        .store_records(group_limits, projection)
        // The snapshot cap bounds state; Harmos also embeds the recent journal
        // tail, which the journal itself retains anyway. A short tail keeps one
        // wide cycle's entries from crowding state out of the snapshot.
        .tail(64)
        .managed_snapshots(crate::storage::snapshot_policy())
        .reconcile(move |state_manager, _storage| async move {
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
        })
        .runtime_with_storage(storage)
        .await
}

/* === Private Functions === */
fn reconciliation_error(source: impl std::error::Error + Send + Sync + 'static) -> harmos::Error {
    harmos::Error::backend("reconcile native Dokime storage", harmos::BackendFailure::Permanent, source)
}

/* === Tests === */
