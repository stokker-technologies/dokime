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
use std::collections::BTreeMap;
// Third Party Imports
use serde::Serialize;
// Local Imports
use super::ReportValues;
use dokime::core::{Metadata, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Serialize)]
pub(super) struct Cycle<'a> {
    pub(super) test_cycle_id: &'a str,
    pub(super) title: &'a str,
    pub(super) suite_id: &'a str,
    pub(super) suite_name: &'a str,
    pub(super) status: &'static str,
    pub(super) case_count: usize,
    pub(super) queue_count: usize,
    pub(super) current_test_execution_id: Option<&'a str>,
    pub(super) current_queue_item_id: Option<&'a str>,
    pub(super) continue_on_failure: bool,
    pub(super) started_at: &'a Timestamp,
    pub(super) finished_at: Option<&'a Timestamp>,
    pub(super) duration_s: Option<()>,
    pub(super) duration_delta_s: Option<()>,
    pub(super) metadata: &'a Metadata,
    pub(super) generated_at: &'a Timestamp,
}

#[derive(Serialize)]
pub(super) struct Case<'a> {
    pub(super) case_id: &'a str,
    pub(super) template_id: &'a str,
    pub(super) parameters: ReportValues<'a>,
    pub(super) generation_index: usize,
    pub(super) label: &'a str,
    pub(super) metadata: &'a Metadata,
    pub(super) latest_test_execution_id: Option<&'a str>,
    pub(super) status: &'static str,
}

#[derive(Serialize)]
pub(super) struct QueueItem<'a> {
    pub(super) queue_item_id: &'a str,
    pub(super) test_cycle_id: &'a str,
    pub(super) ordinal: usize,
    pub(super) template_id: &'a str,
    pub(super) case_id: &'a str,
    pub(super) repeat_index: u8,
    pub(super) status: &'static str,
    pub(super) current_test_execution_id: Option<&'a str>,
    pub(super) attempt_count: usize,
    pub(super) blocked_reason: Option<()>,
}

#[derive(Serialize)]
pub(super) struct Template<'a> {
    pub(super) template_id: &'a str,
    pub(super) name: &'a str,
    pub(super) description: Option<()>,
    pub(super) source_file: Option<()>,
    pub(super) parameters: &'static [()],
    pub(super) files: &'static [()],
    pub(super) steps: &'static [()],
    pub(super) measurements: &'static [()],
    pub(super) criteria: &'static [()],
    pub(super) initialize_hooks: &'static [()],
    pub(super) finalize_hooks: &'static [()],
    pub(super) interrupt_hooks: &'static [()],
    pub(super) cycle_summary: Option<()>,
    pub(super) metadata: BTreeMap<(), ()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
