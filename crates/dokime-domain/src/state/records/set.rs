/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use crate::model::execution::RecordScope;
use crate::model::execution::health::HealthItem;
use crate::model::execution::notes::OperatorNote;
use crate::model::identity::NoteId;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TestRecordSet {
    health: Vec<HealthItem>,
    notes: Vec<OperatorNote>,

    #[serde(default)]
    capture_owner: Option<crate::model::identity::TestExecutionId>,
    capture_through: u64,

    #[serde(default)]
    record_subscriptions_closed: bool,
}

impl harmos_domain::MemorySize for TestRecordSet {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.health), std::mem::size_of_val(&self.health)),
            (harmos_domain::MemorySize::memory_size(&self.notes), std::mem::size_of_val(&self.notes)),
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
            capture_owner: None,
            capture_through: 0,
            record_subscriptions_closed: false,
        }
    }

    pub fn clear(&mut self) {
        self.health.clear();
        self.notes.clear();
        self.capture_owner = None;
        self.capture_through = 0;
        self.record_subscriptions_closed = false;
    }

    pub fn retain_by_scope(&mut self, mut keep: impl FnMut(&RecordScope) -> bool) {
        self.health.retain(|health| keep(&health.scope));
        self.notes.retain(|note| keep(&note.scope));
    }

    pub fn capture_owner(&self) -> Option<&crate::model::identity::TestExecutionId> {
        self.capture_owner.as_ref()
    }
    pub fn capture_through(&self) -> u64 {
        self.capture_through
    }

    pub fn captured_scope<'a>(&self, record: &'a crate::model::records::evidence::RecordEntry) -> Option<&'a RecordScope> {
        if record.ordinal > self.capture_through {
            return None;
        }
        record.captured_scope(self.capture_owner.as_ref()?)
    }

    pub fn capture_frontier(&mut self, owner: crate::model::identity::TestExecutionId, through: u64) {
        assert!(self.capture_owner.as_ref().is_none_or(|existing| existing == &owner), "capture frontier owner changed");
        self.capture_owner = Some(owner);
        self.capture_through = self.capture_through.max(through);
    }

    pub fn record_subscriptions_closed(&self) -> bool {
        self.record_subscriptions_closed
    }

    pub fn close_record_subscriptions(&mut self) {
        self.record_subscriptions_closed = true;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_snapshot_retains_only_owner_and_frontier() {
        let owner = crate::model::identity::TestExecutionId::from_seed("compact-capture");
        let mut records = TestRecordSet::default();
        records.capture_frontier(owner.clone(), 1);
        let initial = serde_json::to_vec(&records).unwrap().len();
        records.capture_frontier(owner.clone(), 6_048_000);
        let completed = serde_json::to_vec(&records).unwrap().len();
        assert!(completed <= initial + 7);
        let restored: TestRecordSet = serde_json::from_slice(&serde_json::to_vec(&records).unwrap()).unwrap();
        assert_eq!(restored.capture_owner(), Some(&owner));
        assert_eq!(restored.capture_through(), 6_048_000);
    }
}
