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
use crate::model::execution::RecordScope;
use crate::model::execution::health::HealthItem;
use crate::model::execution::notes::OperatorNote;
use crate::model::identity::NoteId;
use crate::model::identity::RecordId;

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
            (harmos::MemorySize::memory_size(&self.health), std::mem::size_of_val(&self.health)),
            (harmos::MemorySize::memory_size(&self.notes), std::mem::size_of_val(&self.notes)),
            (harmos::MemorySize::memory_size(&self.subscribed_records), std::mem::size_of_val(&self.subscribed_records)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl TestRecordSet {
    pub fn with_capacity(health: usize, notes: usize) -> Self {
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

    // One Health Subject Per Identity: A Later Observation Replaces It In Place.
    pub fn push_health(&mut self, health: HealthItem) {
        match self.health.iter_mut().find(|item| item.health_item_id == health.health_item_id) {
            Some(item) if item.observed_at <= health.observed_at => *item = health,
            Some(_) => {}
            None => self.health.push(health),
        }
    }

    pub fn notes(&self) -> &[OperatorNote] {
        &self.notes
    }

    pub fn push_note(&mut self, note: OperatorNote) {
        self.notes.push(note);
    }

    // Replace One Note's Message In Place; Its Identity, Author And Filing Time Stay
    pub fn revise_note(&mut self, id: &NoteId, message: String) -> bool {
        let Some(note) = self.notes.iter_mut().find(|note| &note.note_id == id) else {
            return false;
        };
        note.message = message;
        true
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
