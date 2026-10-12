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

// Local Imports
use super::appearance::Appearance;
use crate::execution::context::{ExecutionContext, ExecutionCycleContext};
use crate::traceability::{TraceabilityEntry, TraceabilitySnapshot};
use crate::{Error, Result};
use dokime::core::{DocumentId, Metadata, RecordScope, SessionId, Timestamp, TraceabilityId};
use dokime::execution::{CycleView, TestCycle};
use dokime::storage::objects::{case::Case, cycle::Cycle, presentation as native};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Context {
    scope: Option<native::Scope>,
    session_id: String,
    cycle: Option<Cycle>,
    cases: Vec<Case>,
    execution_documents: Vec<Document>,
    prompts: Vec<native::PromptInstance>,
    health: Vec<native::Health>,
    notes: Vec<native::Note>,
    traceability: Vec<Traceability>,
    appearance: Option<Appearance>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Document {
    key: String,
    definition: Option<native::DocumentDefinition>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct Traceability {
    traceability_id: String,
    scope: Option<native::Scope>,
    created_at: String,
    metadata: Option<native::Metadata>,
    entries: Vec<Entry>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Entry {
    metadata: Option<native::Metadata>,
    value: Option<native::Value>,
    source: Option<String>,
    unit: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Context {
    pub(crate) fn projection_memory(&self) -> Result<usize> {
        let scope = self
            .scope
            .as_ref()
            .ok_or_else(|| invalid("context scope is missing"))?;
        let cycle = self
            .cycle
            .as_ref()
            .ok_or_else(|| invalid("context cycle is missing"))?;
        let appearance = self
            .appearance
            .as_ref()
            .ok_or_else(|| invalid("context appearance is missing"))?;
        let mut bytes =
            std::mem::size_of::<ExecutionContext>().saturating_add(self.session_id.capacity());
        bytes = excess(
            bytes,
            scope.projection_memory(),
            std::mem::size_of::<RecordScope>(),
        );
        bytes = excess(
            bytes,
            cycle.projection_memory()?,
            std::mem::size_of::<TestCycle>(),
        );
        bytes = excess(
            bytes,
            appearance.projection_memory()?,
            std::mem::size_of::<crate::installed_plugins::RuntimeAppearanceFrame>(),
        );
        for item in &self.cases {
            bytes = bytes.saturating_add(item.projection_memory()?);
        }
        bytes = bytes.saturating_add(harmos::btree_map_memory::<
            DocumentId,
            crate::document::DocumentDefinition,
        >(self.execution_documents.len()));
        let mut previous: Option<&str> = None;
        for item in &self.execution_documents {
            if previous.is_some_and(|key| key >= item.key.as_str()) {
                return Err(invalid("context document keys are not canonical"));
            }
            previous = Some(&item.key);
            let definition = item
                .definition
                .as_ref()
                .ok_or_else(|| invalid("context document is missing"))?;
            bytes = bytes.saturating_add(item.key.capacity());
            bytes = excess(
                bytes,
                definition.projection_memory()?,
                std::mem::size_of::<crate::document::DocumentDefinition>(),
            );
        }
        for item in &self.prompts {
            bytes = bytes.saturating_add(item.projection_memory()?);
        }
        for item in &self.health {
            bytes = bytes.saturating_add(item.projection_memory());
        }
        for item in &self.notes {
            bytes = bytes.saturating_add(item.projection_memory());
        }
        for item in &self.traceability {
            bytes = bytes.saturating_add(item.projection_memory()?);
        }
        // Frozen Owner Validation Uses Borrowed Identity Sets.
        bytes = bytes
            .saturating_add(harmos::btree_map_memory::<&dokime::core::NoteId, ()>(
                self.notes.len(),
            ))
            .saturating_add(harmos::btree_map_memory::<&dokime::core::PromptId, ()>(
                self.prompts.len(),
            ))
            .saturating_add(harmos::btree_map_memory::<&dokime::core::HealthItemId, ()>(
                self.health.len(),
            ))
            .saturating_add(
                harmos::btree_map_memory::<&dokime::core::TestExecutionId, ()>(
                    cycle.membership_len(),
                ),
            )
            .saturating_add(harmos::btree_map_memory::<&dokime::core::CaseId, ()>(
                self.cases.len(),
            ));
        Ok(bytes)
    }

    pub(crate) fn encoding_memory(value: &ExecutionContext) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>().saturating_add(value.session_id.as_str().len());
        bytes = excess(
            bytes,
            native::Scope::encoding_memory(&value.scope),
            std::mem::size_of::<native::Scope>(),
        );
        bytes = excess(
            bytes,
            Cycle::encoding_view_memory(&value.cycle.cycle, &value.cycle.execution_order)?,
            std::mem::size_of::<Cycle>(),
        );
        bytes = excess(
            bytes,
            Appearance::encoding_memory(&value.appearance),
            std::mem::size_of::<Appearance>(),
        );
        for item in &value.cycle.cases {
            bytes = bytes.saturating_add(Case::encoding_memory(item)?);
        }
        for (key, definition) in &value.execution_documents {
            bytes = bytes
                .saturating_add(std::mem::size_of::<Document>())
                .saturating_add(key.as_str().len());
            bytes = excess(
                bytes,
                native::DocumentDefinition::encoding_memory(definition)?,
                std::mem::size_of::<native::DocumentDefinition>(),
            );
        }
        for item in &value.prompts {
            bytes = bytes.saturating_add(native::PromptInstance::encoding_memory(item)?);
        }
        for item in &value.health {
            bytes = bytes.saturating_add(native::Health::encoding_memory(item)?);
        }
        for item in &value.notes {
            bytes = bytes.saturating_add(native::Note::encoding_memory(item)?);
        }
        for item in &value.traceability {
            bytes = bytes.saturating_add(Traceability::encoding_memory(item)?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&ExecutionContext> for Context {
    type Error = Error;
    fn try_from(value: &ExecutionContext) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("context encoding memory overflows"));
        }
        let mut execution_documents = slots(value.execution_documents.len())?;
        for (key, definition) in &value.execution_documents {
            execution_documents.push(Document {
                key: key.as_str().to_owned(),
                definition: Some(definition.try_into()?),
            });
        }
        Ok(Self {
            scope: Some((&value.scope).into()),
            session_id: value.session_id.as_str().to_owned(),
            cycle: Some(Cycle::from_view(
                &value.cycle.cycle,
                &value.cycle.execution_order,
            )?),
            cases: convert(&value.cycle.cases, |item| Ok(item.try_into()?))?,
            execution_documents,
            prompts: convert(&value.prompts, |item| Ok(item.try_into()?))?,
            health: convert(&value.health, |item| Ok(item.try_into()?))?,
            notes: convert(&value.notes, |item| Ok(item.try_into()?))?,
            traceability: convert(&value.traceability, Traceability::try_from)?,
            appearance: Some((&value.appearance).try_into()?),
        })
    }
}

impl TryFrom<Context> for ExecutionContext {
    type Error = Error;
    fn try_from(value: Context) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("context projection memory overflows"));
        }
        let cycle: TestCycle = value
            .cycle
            .ok_or_else(|| invalid("context cycle is missing"))?
            .try_into()?;
        let cycle_view = CycleView {
            saved: cycle.saved,
            test_cycle_id: cycle.test_cycle_id,
            session_id: cycle.session_id,
            metadata: cycle.metadata,
            lifecycle: cycle.lifecycle,
            created_at: cycle.created_at,
            started_at: cycle.started_at,
            updated_at: cycle.updated_at,
            test_executions: Vec::new(),
            records: cycle.records,
            continue_on_failure: cycle.continue_on_failure,
            result: cycle.result,
        };
        let mut documents = BTreeMap::new();
        for item in value.execution_documents {
            let key = DocumentId::from_wire(item.key).map_err(dokime::storage::Error::from)?;
            let definition = item
                .definition
                .ok_or_else(|| invalid("context document is missing"))?
                .try_into()?;
            documents.insert(key, definition);
        }
        Ok(Self {
            scope: value
                .scope
                .ok_or_else(|| invalid("context scope is missing"))?
                .try_into()?,
            session_id: SessionId::from_uuid(value.session_id)
                .map_err(dokime::storage::Error::from)?,
            cycle: ExecutionCycleContext {
                cycle: cycle_view,
                cases: convert(value.cases, |item| Ok(item.try_into()?))?,
                execution_order: cycle.execution_ids,
            },
            execution_documents: documents,
            prompts: convert(value.prompts, |item| Ok(item.try_into()?))?,
            health: convert(value.health, |item| Ok(item.try_into()?))?,
            notes: convert(value.notes, |item| Ok(item.try_into()?))?,
            traceability: convert(value.traceability, TraceabilitySnapshot::try_from)?,
            appearance: value
                .appearance
                .ok_or_else(|| invalid("context appearance is missing"))?
                .try_into()?,
        })
    }
}

impl Traceability {
    pub(super) fn encoding_memory(value: &TraceabilitySnapshot) -> Result<usize> {
        value
            .created_at
            .to_datetime()
            .map_err(dokime::storage::Error::from)?;
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.traceability_id.as_str().len())
            .saturating_add(value.created_at.as_str().len());
        bytes = excess(
            bytes,
            native::Scope::encoding_memory(&value.scope),
            std::mem::size_of::<native::Scope>(),
        );
        if let Some(metadata) = &value.metadata {
            bytes = excess(
                bytes,
                native::Metadata::encoding_memory(metadata)?,
                std::mem::size_of::<native::Metadata>(),
            );
        }
        for entry in &value.entries {
            bytes = bytes.saturating_add(Entry::encoding_memory(entry)?);
        }
        Ok(bytes)
    }
    pub(super) fn projection_memory(&self) -> Result<usize> {
        time::OffsetDateTime::parse(
            &self.created_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(dokime::storage::Error::from)?;
        let scope = self
            .scope
            .as_ref()
            .ok_or_else(|| invalid("traceability scope is missing"))?;
        let mut bytes = std::mem::size_of::<TraceabilitySnapshot>()
            .saturating_add(self.traceability_id.capacity())
            .saturating_add(self.created_at.capacity());
        bytes = excess(
            bytes,
            scope.projection_memory(),
            std::mem::size_of::<RecordScope>(),
        );
        if let Some(metadata) = &self.metadata {
            bytes = excess(
                bytes,
                metadata.projection_memory()?,
                std::mem::size_of::<Metadata>(),
            );
        }
        for entry in &self.entries {
            bytes = bytes.saturating_add(entry.projection_memory()?);
        }
        Ok(bytes)
    }
}
impl Entry {
    fn encoding_memory(value: &TraceabilityEntry) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.source.as_ref().map_or(0, String::len))
            .saturating_add(value.unit.as_ref().map_or(0, String::len));
        bytes = excess(
            bytes,
            native::Metadata::encoding_memory(&value.metadata)?,
            std::mem::size_of::<native::Metadata>(),
        );
        bytes = excess(
            bytes,
            native::Value::encoding_memory(&value.value)?,
            std::mem::size_of::<native::Value>(),
        );
        Ok(bytes)
    }
    fn projection_memory(&self) -> Result<usize> {
        let metadata = self
            .metadata
            .as_ref()
            .ok_or_else(|| invalid("traceability entry metadata is missing"))?;
        let value = self
            .value
            .as_ref()
            .ok_or_else(|| invalid("traceability entry value is missing"))?;
        let bytes = std::mem::size_of::<TraceabilityEntry>()
            .saturating_add(self.source.as_ref().map_or(0, String::capacity))
            .saturating_add(self.unit.as_ref().map_or(0, String::capacity));
        let bytes = excess(
            bytes,
            metadata.projection_memory()?,
            std::mem::size_of::<Metadata>(),
        );
        // The Scalar Codec Includes Its Original Single-Member Map Scratch.
        Ok(bytes.saturating_add(value.projection_memory()?))
    }
}
impl TryFrom<&TraceabilitySnapshot> for Traceability {
    type Error = Error;
    fn try_from(value: &TraceabilitySnapshot) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("traceability encoding memory overflows"));
        }
        Ok(Self {
            traceability_id: value.traceability_id.as_str().to_owned(),
            scope: Some((&value.scope).into()),
            created_at: value.created_at.as_str().to_owned(),
            metadata: value
                .metadata
                .as_ref()
                .map(native::Metadata::try_from)
                .transpose()?,
            entries: convert(&value.entries, Entry::try_from)?,
        })
    }
}
impl TryFrom<Traceability> for TraceabilitySnapshot {
    type Error = Error;
    fn try_from(value: Traceability) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("traceability projection memory overflows"));
        }
        Ok(Self {
            traceability_id: TraceabilityId::from_uuid(value.traceability_id)
                .map_err(dokime::storage::Error::from)?,
            scope: value
                .scope
                .ok_or_else(|| invalid("traceability scope is missing"))?
                .try_into()?,
            created_at: Timestamp::new(value.created_at),
            metadata: value.metadata.map(Metadata::try_from).transpose()?,
            entries: convert(value.entries, TraceabilityEntry::try_from)?,
        })
    }
}
impl TryFrom<&TraceabilityEntry> for Entry {
    type Error = Error;
    fn try_from(value: &TraceabilityEntry) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("traceability entry encoding memory overflows"));
        }
        Ok(Self {
            metadata: Some((&value.metadata).try_into()?),
            value: Some((&value.value).try_into()?),
            source: value.source.clone(),
            unit: value.unit.clone(),
        })
    }
}
impl TryFrom<Entry> for TraceabilityEntry {
    type Error = Error;
    fn try_from(value: Entry) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("traceability entry projection memory overflows"));
        }
        Ok(Self {
            metadata: value
                .metadata
                .ok_or_else(|| invalid("traceability entry metadata is missing"))?
                .try_into()?,
            value: value
                .value
                .ok_or_else(|| invalid("traceability entry value is missing"))?
                .try_into()?,
            source: value.source,
            unit: value.unit,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}
fn excess(bytes: usize, owned: usize, inline: usize) -> usize {
    if owned == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_add(owned.saturating_sub(inline))
    }
}
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| invalid("context collection allocation failed"))?;
    if values.capacity() != count {
        return Err(invalid("context collection exceeds reserved slots"));
    }
    Ok(values)
}
fn convert<T, U, I: IntoIterator<Item = T>>(
    items: I,
    mut map: impl FnMut(T) -> Result<U>,
) -> Result<Vec<U>>
where
    I::IntoIter: ExactSizeIterator,
{
    let items = items.into_iter();
    let mut values = slots(items.len())?;
    for item in items {
        values.push(map(item)?);
    }
    Ok(values)
}

/* === Tests === */
