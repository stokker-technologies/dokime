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

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

// Local Imports
use super::templates;
use crate::core::{HealthItemId, NoteId, RecordId, RecordScope, Timestamp};
use crate::execution::{HealthItem, HealthSeverity, OperatorNote};
use crate::records::TestRecordSet;
use crate::storage::{Error, Result, records::ScopeRow};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Set {
    health: Vec<Health>,
    notes: Vec<Note>,
    subscribed_records: Vec<Captured>,
    record_subscriptions_closed: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Health {
    health_item_id: String,
    scope: ScopeRow,
    severity: String,
    message: String,
    detail: Option<String>,
    observed_at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    note_id: String,
    scope: ScopeRow,
    author: Option<String>,
    message: String,
    created_at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Captured {
    record_id: String,
    scope: ScopeRow,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&TestRecordSet> for Set {
    type Error = Error;

    fn try_from(source: &TestRecordSet) -> Result<Self> {
        if Self::encoding_memory(source)? == usize::MAX {
            return Err(Error::Invalid("annotation encoding memory overflow"));
        }
        let mut health = Vec::new();
        health
            .try_reserve_exact(source.health().len())
            .map_err(|_| Error::Invalid("health encoding allocation failed"))?;
        let mut notes = Vec::new();
        notes
            .try_reserve_exact(source.notes().len())
            .map_err(|_| Error::Invalid("note encoding allocation failed"))?;
        let mut subscribed_records = Vec::new();
        subscribed_records
            .try_reserve_exact(source.subscribed_records().len())
            .map_err(|_| Error::Invalid("record ownership encoding allocation failed"))?;

        if health.capacity() > source.health().len()
            || notes.capacity() > source.notes().len()
            || subscribed_records.capacity() > source.subscribed_records().len()
        {
            return Err(Error::Invalid("annotation slots exceed admission"));
        }
        for value in source.health() {
            health.push(Health::try_from(value)?);
        }
        for value in source.notes() {
            notes.push(Note::try_from(value)?);
        }
        for (id, scope) in source.subscribed_records() {
            subscribed_records.push(Captured {
                record_id: id.as_str().to_owned(),
                scope: ScopeRow::from(scope),
            });
        }
        Ok(Self {
            health,
            notes,
            subscribed_records,
            record_subscriptions_closed: source.record_subscriptions_closed(),
        })
    }
}

impl Set {
    pub fn encoding_memory(source: &TestRecordSet) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        for value in source.health() {
            bytes = bytes.saturating_add(Health::encoding_memory(value)?);
        }
        for value in source.notes() {
            bytes = bytes.saturating_add(Note::encoding_memory(value)?);
        }
        for (id, scope) in source.subscribed_records() {
            let scope = ScopeRow::encoding_memory(scope);
            if scope == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(std::mem::size_of::<Captured>())
                .saturating_add(id.as_str().len())
                .saturating_add(scope.saturating_sub(std::mem::size_of::<ScopeRow>()));
        }
        Ok(bytes)
    }

    pub fn projection_memory(&self) -> usize {
        let map_memory =
            harmos::btree_map_memory::<RecordId, RecordScope>(self.subscribed_records.len());
        if map_memory == usize::MAX {
            return usize::MAX;
        }
        let mut bytes = std::mem::size_of::<TestRecordSet>()
            .saturating_add(
                self.health
                    .len()
                    .saturating_mul(std::mem::size_of::<HealthItem>()),
            )
            .saturating_add(
                self.notes
                    .len()
                    .saturating_mul(std::mem::size_of::<OperatorNote>()),
            )
            .saturating_add(map_memory.saturating_sub(std::mem::size_of::<
                std::collections::BTreeMap<RecordId, RecordScope>,
            >()));
        for health in &self.health {
            let owned = health.projection_memory();
            if owned == usize::MAX {
                return usize::MAX;
            }
            bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<HealthItem>()));
        }
        for note in &self.notes {
            let owned = note.projection_memory();
            if owned == usize::MAX {
                return usize::MAX;
            }
            bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<OperatorNote>()));
        }
        for captured in &self.subscribed_records {
            let scope = captured.scope.projection_memory();
            if scope == usize::MAX {
                return usize::MAX;
            }
            bytes = bytes
                .saturating_add(captured.record_id.capacity())
                .saturating_add(scope.saturating_sub(std::mem::size_of::<RecordScope>()));
        }
        bytes
    }

    pub fn restore_owned(self) -> Result<TestRecordSet> {
        if self
            .subscribed_records
            .windows(2)
            .any(|pair| pair[0].record_id >= pair[1].record_id)
        {
            return Err(Error::Invalid(
                "noncanonical captured record ownership order",
            ));
        }
        let mut records = TestRecordSet::with_capacity(self.health.len(), self.notes.len());
        for note in self.notes {
            records.push_note(note.try_into()?);
        }
        for health in self.health {
            records.push_health(health.try_into()?);
        }
        for captured in self.subscribed_records {
            records.capture_record(
                RecordId::from_uuid(captured.record_id)?,
                captured.scope.try_into()?,
            );
        }
        if self.record_subscriptions_closed {
            records.close_record_subscriptions();
        }
        Ok(records)
    }

    pub(super) fn restore(&self) -> Result<TestRecordSet> {
        let mut records = TestRecordSet::default();
        for note in &self.notes {
            let note = templates::convert(note)?;
            records.push_note(note);
        }
        for health in &self.health {
            let health = templates::convert(health)?;
            records.push_health(health);
        }
        for captured in &self.subscribed_records {
            let id = RecordId::from_uuid(&captured.record_id)?;
            if records.subscribed_records().contains_key(&id) {
                return Err(Error::Invalid("duplicate captured record ownership"));
            }
            let scope: RecordScope = templates::convert(&captured.scope)?;
            records.capture_record(id, scope);
        }
        if self.record_subscriptions_closed {
            records.close_record_subscriptions();
        }
        let canonical = Self::try_from(&records)?;
        if canonical != *self {
            return Err(Error::Invalid(
                "noncanonical annotation or record ownership fields",
            ));
        }
        Ok(records)
    }
}

impl serde::Serialize for Set {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let records = self.restore().map_err(serde::ser::Error::custom)?;
        records.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for Set {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let records = <TestRecordSet as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(&records).map_err(serde::de::Error::custom)
    }
}

impl Health {
    pub fn encoding_memory(value: &HealthItem) -> Result<usize> {
        value.observed_at.to_datetime()?;
        let scope = ScopeRow::encoding_memory(&value.scope);
        if scope == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok([
            value.health_item_id.as_str().len(),
            value.observed_at.as_str().len(),
            scope.saturating_sub(std::mem::size_of::<ScopeRow>()),
            value.message.len(),
            value.detail.as_deref().map_or(0, str::len),
            severity(&value.severity).len(),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    pub fn projection_memory(&self) -> usize {
        let scope = self.scope.projection_memory();
        if scope == usize::MAX {
            return usize::MAX;
        }
        [
            self.health_item_id.capacity(),
            self.observed_at.capacity(),
            scope.saturating_sub(std::mem::size_of::<RecordScope>()),
            self.message.capacity(),
            self.detail.as_ref().map_or(0, String::capacity),
        ]
        .into_iter()
        .fold(std::mem::size_of::<HealthItem>(), usize::saturating_add)
    }
}
impl TryFrom<&HealthItem> for Health {
    type Error = Error;
    fn try_from(value: &HealthItem) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("annotation encoding memory overflow"));
        }
        Ok(Self {
            health_item_id: value.health_item_id.as_str().to_owned(),
            observed_at: value.observed_at.as_str().to_owned(),
            scope: ScopeRow::from(&value.scope),
            message: value.message.clone(),
            detail: value.detail.clone(),
            severity: severity(&value.severity).to_owned(),
        })
    }
}
impl TryFrom<Health> for HealthItem {
    type Error = Error;
    fn try_from(value: Health) -> Result<Self> {
        if value.projection_memory() == usize::MAX {
            return Err(Error::Invalid("annotation projection memory overflow"));
        }
        OffsetDateTime::parse(&value.observed_at, &Rfc3339)?;
        Ok(Self {
            health_item_id: HealthItemId::from_uuid(value.health_item_id)?,
            observed_at: Timestamp::new(value.observed_at),
            scope: value.scope.try_into()?,
            message: value.message,
            detail: value.detail,
            severity: match value.severity.as_str() {
                "info" => HealthSeverity::Info,
                "warning" => HealthSeverity::Warning,
                "error" => HealthSeverity::Error,
                _ => return Err(Error::Invalid("invalid persisted health severity")),
            },
        })
    }
}
impl Note {
    pub fn encoding_memory(value: &OperatorNote) -> Result<usize> {
        value.created_at.to_datetime()?;
        let scope = ScopeRow::encoding_memory(&value.scope);
        if scope == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok([
            value.note_id.as_str().len(),
            value.created_at.as_str().len(),
            scope.saturating_sub(std::mem::size_of::<ScopeRow>()),
            value.message.len(),
            value.author.as_deref().map_or(0, str::len),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    pub fn projection_memory(&self) -> usize {
        let scope = self.scope.projection_memory();
        if scope == usize::MAX {
            return usize::MAX;
        }
        [
            self.note_id.capacity(),
            self.created_at.capacity(),
            scope.saturating_sub(std::mem::size_of::<RecordScope>()),
            self.message.capacity(),
            self.author.as_ref().map_or(0, String::capacity),
        ]
        .into_iter()
        .fold(std::mem::size_of::<OperatorNote>(), usize::saturating_add)
    }
}
impl TryFrom<&OperatorNote> for Note {
    type Error = Error;
    fn try_from(value: &OperatorNote) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("annotation encoding memory overflow"));
        }
        Ok(Self {
            note_id: value.note_id.as_str().to_owned(),
            created_at: value.created_at.as_str().to_owned(),
            scope: ScopeRow::from(&value.scope),
            message: value.message.clone(),
            author: value.author.clone(),
        })
    }
}
impl TryFrom<Note> for OperatorNote {
    type Error = Error;
    fn try_from(value: Note) -> Result<Self> {
        if value.projection_memory() == usize::MAX {
            return Err(Error::Invalid("annotation projection memory overflow"));
        }
        OffsetDateTime::parse(&value.created_at, &Rfc3339)?;
        Ok(Self {
            note_id: NoteId::from_uuid(value.note_id)?,
            created_at: Timestamp::new(value.created_at),
            scope: value.scope.try_into()?,
            message: value.message,
            author: value.author,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn severity(value: &HealthSeverity) -> &'static str {
    match value {
        HealthSeverity::Info => "info",
        HealthSeverity::Warning => "warning",
        HealthSeverity::Error => "error",
    }
}

/* === Tests === */
