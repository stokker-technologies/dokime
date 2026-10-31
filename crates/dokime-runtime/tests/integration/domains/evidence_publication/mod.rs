/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Publication Story: records, notes, health and telemetry published
// through the runtime and read back through the evidence snapshot, journal
// subscriptions and traceability projections.

/* === Modules === */

mod model;
mod notes;
mod publication;
mod records;
mod traceability;

/* === Re-Exports === */
pub(crate) use crate::service_support::prelude::*;

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

/* === Private Functions === */

/* === Tests === */

// One runtime with the host wifi contracts carries the publication stages. The refusals run
// first because they assert an empty evidence set; later stages scope their counts to what
// they publish.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn evidence_publication() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = started(wifi_config(root.path())).await;
    publication::publish_record_warning(&runtime).await;
    publication::telemetry_refuses_partial(&runtime).await;
    notes::empty_cycle_datasets(&owner).await;
    publication::evidence_write_revision(&runtime).await;
    publication::subscriptions_deliver_selection(&runtime).await;
    records::native_publications_order(&mut owner).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// Execution-published evidence: definitions, contract conflicts and traceability share one
// runtime; the tracking transitions need an executor of their own.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn evidence_model_traceability() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    model::definitions_survive_completion(&runtime).await;
    model::conflicting_contracts_rejected(&runtime).await;
    traceability::resource_records_absent(&runtime).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
    model::tracking_derives_kpis().await;
}

// Each batch size publishes into a runtime of its own so the stream cursors start at one.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn evidence_telemetry_batches() {
    publication::telemetry_batches_sample().await;
}
