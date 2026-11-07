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

mod artifacts;
mod domains;
mod execution;
mod numeric_query_parity;
mod plugin;
#[cfg(feature = "testing")]
mod plugin_completion;
mod plugins;
mod query_buckets;
mod query_running;
mod query_samples;
mod query_statistics;
#[path = "../support/query_telemetry.rs"]
mod query_telemetry;

#[cfg(feature = "testing")]
mod record_queries;

mod config;
mod runtime;
mod storage;
mod storage_format;

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
