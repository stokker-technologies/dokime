/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use super::appearance::Appearance;
use crate::execution::context::{ExecutionContext, ExecutionCycleContext};
use crate::storage::objects::{case::Case, cycle::Cycle, presentation as native};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TraceabilityId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::traceability::TraceabilityEntry;
use dokime_domain::model::traceability::TraceabilitySnapshot;

#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Context {
    appearance_logo_bytes: u64,
    appearance_css_depth: u64,
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

impl TryFrom<&ExecutionContext> for Context {
    type Error = Error;
    fn try_from(value: &ExecutionContext) -> Result<Self> {
        let mut execution_documents = slots(value.execution_documents.len())?;
        for (key, definition) in &value.execution_documents {
            execution_documents.push(Document {
                key: key.as_str().to_owned(),
                definition: Some(definition.try_into()?),
            });
        }
        Ok(Self {
            appearance_logo_bytes: value.appearance_policy.logo_bytes,
            appearance_css_depth: value.appearance_policy.css_depth,
            scope: Some((&value.scope).into()),
            session_id: value.session_id.as_str().to_owned(),
            cycle: Some(Cycle::from_view(&value.cycle.cycle, &value.cycle.execution_order)?),
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
        let cycle: TestCycle = value.cycle.ok_or_else(|| invalid("context cycle is missing"))?.try_into()?;
        let cycle_view = CycleView {
            saved: cycle.saved,
            test_cycle_id: cycle.test_cycle_id,
            session_id: cycle.session_id,
            metadata: cycle.metadata,
            phase: cycle.phase,
            lifecycle: cycle.lifecycle,
            created_at: cycle.created_at,
            started_at: cycle.started_at,
            updated_at: cycle.updated_at,
            test_executions: Vec::new(),
            records: cycle.records,
            continue_automatically: cycle.continue_automatically,
            result: cycle.result,
        };
        let mut documents = BTreeMap::new();
        for item in value.execution_documents {
            let key = DocumentId::from_wire(item.key).map_err(crate::storage::Error::from)?;
            let definition = item.definition.ok_or_else(|| invalid("context document is missing"))?.try_into()?;
            documents.insert(key, definition);
        }
        let appearance_policy = crate::execution::context::captured::AppearancePolicy {
            logo_bytes: value.appearance_logo_bytes,
            css_depth: value.appearance_css_depth,
        };
        let appearance = value.appearance.ok_or_else(|| invalid("context appearance is missing"))?.into_frame(&appearance_policy)?;
        let result = Self {
            appearance_policy,
            scope: value.scope.ok_or_else(|| invalid("context scope is missing"))?.try_into()?,
            session_id: SessionId::from_uuid(value.session_id).map_err(crate::storage::Error::from)?,
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
            appearance,
        };
        Ok(result)
    }
}

impl TryFrom<&TraceabilitySnapshot> for Traceability {
    type Error = Error;
    fn try_from(value: &TraceabilitySnapshot) -> Result<Self> {
        Ok(Self {
            traceability_id: value.traceability_id.as_str().to_owned(),
            scope: Some((&value.scope).into()),
            created_at: value.created_at.as_str().to_owned(),
            metadata: value.metadata.as_ref().map(native::Metadata::try_from).transpose()?,
            entries: convert(&value.entries, Entry::try_from)?,
        })
    }
}

impl TryFrom<Traceability> for TraceabilitySnapshot {
    type Error = Error;
    fn try_from(value: Traceability) -> Result<Self> {
        Ok(Self {
            traceability_id: TraceabilityId::from_uuid(value.traceability_id).map_err(crate::storage::Error::from)?,
            scope: value.scope.ok_or_else(|| invalid("traceability scope is missing"))?.try_into()?,
            created_at: Timestamp::new(value.created_at),
            metadata: value.metadata.map(Metadata::try_from).transpose()?,
            entries: convert(value.entries, TraceabilityEntry::try_from)?,
        })
    }
}

impl TryFrom<&TraceabilityEntry> for Entry {
    type Error = Error;
    fn try_from(value: &TraceabilityEntry) -> Result<Self> {
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
        Ok(Self {
            metadata: value.metadata.ok_or_else(|| invalid("traceability entry metadata is missing"))?.try_into()?,
            value: value.value.ok_or_else(|| invalid("traceability entry value is missing"))?.try_into()?,
            source: value.source,
            unit: value.unit,
        })
    }
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| invalid("context collection allocation failed"))?;
    if values.capacity() != count {
        return Err(invalid("context collection exceeds reserved slots"));
    }
    Ok(values)
}

fn convert<T, U, I: IntoIterator<Item = T>>(items: I, mut map: impl FnMut(T) -> Result<U>) -> Result<Vec<U>>
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
