/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-process SDK stories: one behaviour of the runtime per story, with test
// hosts and in-process sidecars, grouped by SDK domain.

#![recursion_limit = "256"]

/* === Modules === */

#[path = "../support/engine/evidence_durability.rs"]
pub mod evidence_support;

#[path = "../support/engine/managed_durability.rs"]
pub mod managed_support;

#[cfg(feature = "testing")]
mod retained_evidence;

mod evidence_durability;
mod managed_durability;

mod domains;
mod engine_artifacts;
mod execution_seal_report;
mod execution_workflow;
mod plugin;
mod plugin_store_leases;
mod query_buckets;
mod query_samples;
mod query_statistics;
#[path = "../support/query_telemetry.rs"]
mod query_telemetry;

#[cfg(feature = "testing")]
mod record_queries;

mod runtime;
mod runtime_queries;
mod settings_backends;
mod storage_recovery;

#[path = "../support/engine/mod.rs"]
pub mod engine_support;

#[path = "../support/services/mod.rs"]
pub mod service_support;

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

/* === Private Functions === */

/* === Tests === */
