/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod aggregate;
mod artifact_inventory;
mod artifact_reads;
pub(crate) mod artifacts;
mod assembly;
pub(crate) mod captures;
pub(crate) mod context_records;
pub(crate) mod cycle;
pub(crate) mod cycle_finalization;
mod cycles;
pub(crate) mod execution;
pub(crate) mod executions;
mod finalization;
pub(crate) mod history;
pub(crate) mod native_outputs;
mod notes;
mod paging;
pub(crate) mod presentation;
pub(crate) mod record_capture;
pub(crate) mod record_pages;
pub(crate) mod records;
mod report_capture;
pub(crate) mod report_open;
pub(crate) mod requests;
pub(crate) mod telemetry;
pub(crate) mod telemetry_pages;

pub(crate) use assembly::{configure, initialize};
