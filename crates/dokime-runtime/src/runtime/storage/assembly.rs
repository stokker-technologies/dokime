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
use crate::runtime_state::RuntimeState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn configure(setup: &mut harmos::Setup<crate::app::Dokime>, parts: &crate::runtime::handle::BootParts) {
    let storage = &parts.storage;
    let managed_storage = parts.config.managed_storage.clone();
    let projection = super::record_capture::Projection(parts.record_source.clone());
    setup.settings().storage(storage).managed_storage(managed_storage).store_records(projection);
}

pub(crate) async fn initialize(
    runtime: &harmos::runtime::Handle<crate::app::Dokime>, parts: &crate::runtime::handle::BootParts,
) -> crate::runtime::Result<()> {
    let state_manager = &runtime.state;
    let state = state_manager.state();
    let ownership = state.read(RuntimeState::validate_execution_ownership).await;
    let ownership = ownership.map_err(reconciliation_error)?;
    ownership.value.map_err(|reason| {
        let source = std::io::Error::other(reason);
        reconciliation_error(source)
    })?;
    parts.record_source.begin(state_manager.applied());
    parts.record_source.initialize().await?;

    let started = crate::transactions::MarkRuntimeStarted {
        session_id: parts.config.session_id.clone(),
        record_namespace: parts.record_namespace.clone(),
    };
    state_manager.commit(started).await.map_err(reconciliation_error)?;
    let recovered = crate::transactions::RecoverInterruptedCycles {
        at: crate::model::time::now_utc(),
        record_interruptions: true,
    };
    state_manager.commit(recovered).await.map_err(reconciliation_error)?;
    parts.storage.flush().await?;
    parts.storage.flush_recovery().await?;
    let archives = state
        .read(|state| {
            (
                state.archived_cycles.iter().map(ToString::to_string).collect(),
                state.archived_executions.iter().map(ToString::to_string).collect(),
            )
        })
        .await?
        .value;
    parts.local_state.history_archives(archives.0, archives.1).await?;
    Ok(())
}

/* === Private Functions === */

fn reconciliation_error(source: impl std::error::Error + Send + Sync + 'static) -> harmos::Error {
    harmos::Error::backend("reconcile native Dokime storage", harmos::BackendFailure::Permanent, source)
}

/* === Tests === */
