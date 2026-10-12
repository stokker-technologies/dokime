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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::{RecordId, RecordScope};
use crate::execution::{HealthItem, OperatorNote};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TestRecordSet {
    health: Vec<HealthItem>,
    notes: Vec<OperatorNote>,
    #[serde(default)]
    subscribed_records: BTreeMap<RecordId, RecordScope>,
    #[serde(default)]
    record_subscriptions_closed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for TestRecordSet {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.health),
                std::mem::size_of_val(&self.health),
            ),
            (
                harmos::MemorySize::memory_size(&self.notes),
                std::mem::size_of_val(&self.notes),
            ),
            (
                harmos::MemorySize::memory_size(&self.subscribed_records),
                std::mem::size_of_val(&self.subscribed_records),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl TestRecordSet {
    #[cfg(feature = "storage")]
    pub(crate) fn with_capacity(health: usize, notes: usize) -> Self {
        Self {
            health: Vec::with_capacity(health),
            notes: Vec::with_capacity(notes),
            subscribed_records: BTreeMap::new(),
            record_subscriptions_closed: false,
        }
    }

    pub fn clear(&mut self) {
        self.health.clear();
        self.notes.clear();
        self.subscribed_records.clear();
        self.record_subscriptions_closed = false;
    }

    pub fn retain_by_scope(&mut self, mut keep: impl FnMut(&RecordScope) -> bool) {
        self.health.retain(|health| keep(&health.scope));
        self.notes.retain(|note| keep(&note.scope));
    }

    pub fn subscribed_records(&self) -> &BTreeMap<RecordId, RecordScope> {
        &self.subscribed_records
    }

    pub fn record_subscriptions_closed(&self) -> bool {
        self.record_subscriptions_closed
    }

    pub fn close_record_subscriptions(&mut self) {
        self.record_subscriptions_closed = true;
    }

    pub fn capture_record(&mut self, record_id: RecordId, scope: RecordScope) {
        self.subscribed_records.entry(record_id).or_insert(scope);
    }

    pub fn health(&self) -> &[HealthItem] {
        &self.health
    }

    pub fn push_health(&mut self, health: HealthItem) {
        self.health.push(health);
    }

    pub fn notes(&self) -> &[OperatorNote] {
        &self.notes
    }

    pub fn push_note(&mut self, note: OperatorNote) {
        self.notes.push(note);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
