/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::storage::{Error, Result, records::ScopeRow};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::identity::HealthItemId;
use dokime_domain::model::identity::NoteId;

use dokime_domain::model::time::Timestamp;
use dokime_domain::state::records::set::TestRecordSet;

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Set {
    health: Vec<Health>,
    notes: Vec<Note>,
    capture_owner: Option<String>,
    capture_through: u64,
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

impl TryFrom<&TestRecordSet> for Set {
    type Error = Error;
    fn try_from(source: &TestRecordSet) -> Result<Self> {
        Ok(Self {
            health: source.health().iter().map(Health::try_from).collect::<Result<_>>()?,
            notes: source.notes().iter().map(Note::try_from).collect::<Result<_>>()?,
            capture_owner: source.capture_owner().map(|id| id.as_str().to_owned()),
            capture_through: source.capture_through(),
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
        Ok(bytes.saturating_add(source.capture_owner().map_or(0, |owner| owner.as_str().len())))
    }

    pub fn projection_memory(&self) -> usize {
        self.health
            .iter()
            .fold(std::mem::size_of::<TestRecordSet>(), |bytes, health| bytes.saturating_add(health.projection_memory()))
            .saturating_add(self.notes.iter().map(Note::projection_memory).sum::<usize>())
    }

    pub fn restore_owned(self) -> Result<TestRecordSet> {
        let mut records = TestRecordSet::with_capacity(self.health.len(), self.notes.len());
        for note in self.notes {
            records.push_note(note.try_into()?);
        }
        for health in self.health {
            records.push_health(health.try_into()?);
        }
        if let Some(owner) = self.capture_owner {
            records.capture_frontier(dokime_domain::model::identity::TestExecutionId::from_uuid(owner)?, self.capture_through);
        } else if self.capture_through != 0 {
            return Err(Error::Invalid("capture frontier has no owner"));
        }
        if self.record_subscriptions_closed {
            records.close_record_subscriptions();
        }
        Ok(records)
    }

    pub(super) fn restore(&self) -> Result<TestRecordSet> {
        let records = self.clone().restore_owned()?;
        if Self::try_from(&records)? != *self {
            return Err(Error::Invalid("noncanonical annotation fields"));
        }
        Ok(records)
    }
}

impl serde::Serialize for Set {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let records = self.restore().map_err(serde::ser::Error::custom)?;
        records.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Set {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
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

fn severity(value: &HealthSeverity) -> &'static str {
    match value {
        HealthSeverity::Info => "info",
        HealthSeverity::Warning => "warning",
        HealthSeverity::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annotations() -> Set {
        let scope = ScopeRow::from(&RecordScope::default());
        let note = Note {
            note_id: "00000000-0000-0000-0000-000000000001".into(),
            scope: scope.clone(),
            author: Some("operator".into()),
            message: "note".into(),
            created_at: "1999-12-31T23:59:59Z".into(),
        };
        let health = Health {
            health_item_id: "00000000-0000-0000-0000-000000000002".into(),
            scope: scope.clone(),
            severity: "warning".into(),
            message: "health".into(),
            detail: Some("detail".into()),
            observed_at: "1999-12-31T23:59:59Z".into(),
        };
        Set {
            health: vec![health],
            notes: vec![note],
            capture_owner: Some("00000000-0000-0000-0000-000000000003".into()),
            capture_through: 42,
            record_subscriptions_closed: true,
        }
    }

    #[test]
    fn typed_annotations_preserve_domain_serialization() {
        let source = annotations();
        let restored = source.restore().unwrap();
        let owned = source.clone().restore_owned().unwrap();

        assert_eq!(restored, owned);
        assert!(Set::try_from(&restored).unwrap() == source);
        let serialized = serde_json::to_value(&source).unwrap();
        assert_eq!(serialized, serde_json::to_value(&restored).unwrap());
        let decoded: Set = serde_json::from_value(serialized).unwrap();
        assert!(decoded == source);
    }

    #[test]
    fn typed_annotations_reject_invalid_leaf_fields() {
        let mut source = annotations();
        source.notes[0].created_at = "invalid".into();
        assert!(source.restore().is_err());

        let mut source = annotations();
        source.health[0].severity = "invalid".into();
        assert!(source.restore().is_err());

        let mut source = annotations();
        source.notes[0].scope.test_execution_id = Some("invalid".into());
        assert!(source.restore().is_err());

        let mut source = annotations();
        source.notes[0].note_id = "invalid".into();
        assert!(source.restore().is_err());
    }

    #[test]
    fn typed_annotations_preserve_duplicate_refusals() {
        let mut source = annotations();
        source.capture_owner = None;
        assert!(source.restore().is_err());

        let mut source = annotations();
        source.health.push(source.health[0].clone());
        assert!(source.restore().is_err());
    }
}
