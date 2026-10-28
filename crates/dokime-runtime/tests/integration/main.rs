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

mod domains;
mod engine;
mod execution;
mod plugin;
mod plugins;
mod report;
mod runtime;
mod state;

#[path = "../support/engine/mod.rs"]
pub mod engine_support;

#[path = "../support/services/mod.rs"]
pub mod service_support;
