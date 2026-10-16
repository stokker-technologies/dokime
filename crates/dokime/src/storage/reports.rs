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
use harmos::schema::Described;
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    Identifier, MemorySize, Message,
    storage::{FileReference, RecordQuery, SchemaKey, StorageCursor},
};

// Local Imports
use super::{
    Error, Manager, Result,
    objects::{
        cycle::Cycle,
        execution::{Revision, Seal},
    },
};
use crate::{
    core::{TestCycleId, TestExecutionId, Timestamp},
    execution::saved::ExecutionRevision,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Publication {
    owner: String,
    generated_at: String,
    final_report: bool,
    evidence_through: Option<StorageCursor>,
    dependencies_saved: Option<StorageCursor>,
    executions: Vec<Revision>,
    inputs: Vec<FileReference>,
    materialized_inputs: Vec<FileReference>,
    file: Option<FileReference>,
    cycle: Option<Cycle>,
    execution: Option<Seal>,
}

pub struct Input<'a> {
    pub owner: Owner<'a>,
    pub generated_at: &'a Timestamp,
    pub final_report: bool,
    pub evidence_through: &'a StorageCursor,
    pub dependencies_saved: &'a StorageCursor,
    pub executions: &'a [ExecutionRevision],
    pub inputs: &'a [FileReference],
    pub materialized_inputs: &'a [FileReference],
    pub file: &'a FileReference,
    pub cycle: Option<&'a crate::execution::TestCycle>,
    pub execution: Option<&'a crate::execution::saved::ExecutionSeal>,
}

pub struct Prepared {
    manager: Manager,
    row: harmos::storage::QueryValue<Publication>,
    payload: harmos::storage::QueryValue<Vec<u8>>,
    scope: Identifier,
    schema: SchemaKey,
    schema_position: Option<StorageCursor>,
    position: Option<StorageCursor>,
    _memory: harmos::storage::StorageProductionPreparation,
}

/* === Enums === */
#[derive(Clone, Copy)]
pub enum Owner<'a> {
    Cycle(&'a TestCycleId),
    Execution(&'a TestExecutionId),
}

/* === Traits === */

/* === Implementations === */
impl Prepared {
    pub fn new(manager: &Manager, input: &Input<'_>) -> Result<Self> {
        if !manager.cursor_saved().covers(input.dependencies_saved)? {
            return Err(Error::Invalid(
                "report dependencies are not durable in this manager",
            ));
        }
        let memory = manager.prepare_production(bound(
            std::mem::size_of::<Self>()
                .saturating_add(
                    64 + std::mem::size_of::<SchemaKey>() + "dokime.report.publication".len(),
                )
                .saturating_add(input.dependencies_saved.memory_size().saturating_mul(3)),
        )?)?;
        let row = manager
            .produce(bound(Publication::encoding_memory(input)?)?, |_| {
                Publication::new(input).map_err(failure)
            })?
            .try_into_value()?;
        let payload = manager
            .produce(
                bound(std::mem::size_of::<Vec<u8>>().saturating_add(row.encoded_len()))?,
                |_| Ok(row.encode_to_vec()),
            )?
            .try_into_value()?;
        Ok(Self {
            manager: manager.clone(),
            row,
            payload,
            scope: input.owner.scope(),
            schema: schema(),
            schema_position: None,
            position: None,
            _memory: memory,
        })
    }

    pub async fn finish(&mut self) -> Result<StorageCursor> {
        if self.schema_position.is_none() {
            let graph = harmos::schema::graph::Schema::from_declaration(
                Publication::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .map_err(|error| {
                harmos::Error::backend(
                    "prepare report schema",
                    harmos::BackendFailure::Permanent,
                    error,
                )
            })?;
            self.schema_position = Some(self.manager.write_schema(&self.schema, &graph).await?);
        }
        if self.position.is_none() {
            self.position = Some(
                self.manager
                    .write_records(&[harmos::storage::RecordWrite {
                        schema: &self.schema,
                        scope: Some(&self.scope),
                        item: &self.row.file()?.metadata.id,
                        source: None,
                        source_time: None,
                        payload: &self.payload,
                    }])
                    .await?,
            );
        }
        let position = self.position.as_ref().expect("report admission receipt");
        let saved = self.manager.flush_until(position).await?;
        if !saved.covers(position)? {
            return Err(Error::Invalid("report publication is not saved"));
        }
        Ok(position.clone())
    }
}

impl Owner<'_> {
    pub fn scope(self) -> Identifier {
        let (kind, id) = match self {
            Self::Cycle(id) => ("cycle", id.as_str()),
            Self::Execution(id) => ("execution", id.as_str()),
        };
        Identifier::new(format!("{kind}.{id}")).expect("typed report owner")
    }
}

impl Publication {
    pub fn from_record(record: &harmos::storage::StoredRecord<Self>) -> Result<&Self> {
        let publication = &record.payload;
        publication.check()?;
        let dependencies = publication
            .dependencies_saved
            .as_ref()
            .expect("checked publication dependencies");
        if record.schema != schema()
            || record.scope.as_ref().map(Identifier::as_str) != Some(publication.owner())
            || record.item != publication.file()?.metadata.id
            || record.source.is_some()
            || record.source_time.is_some()
            || !record.position.covers(dependencies)?
            || record.position == *dependencies
        {
            return Err(Error::Invalid(
                "report publication envelope differs from payload",
            ));
        }
        Ok(publication)
    }

    pub fn encoding_memory(input: &Input<'_>) -> Result<usize> {
        validate(input)?;
        let sources = input.executions.iter().try_fold(0usize, |total, source| {
            Ok::<_, Error>(total.saturating_add(Revision::encoding_memory(source)?))
        })?;
        let inputs = input.inputs.iter().fold(0usize, |total, file| {
            total.saturating_add(file.memory_size())
        });
        Ok(std::mem::size_of::<Self>()
            .saturating_add(64)
            .saturating_add(input.generated_at.as_str().len())
            .saturating_add(input.evidence_through.memory_size())
            .saturating_add(input.dependencies_saved.memory_size())
            .saturating_add(sources)
            .saturating_add(inputs)
            .saturating_add(
                input
                    .materialized_inputs
                    .iter()
                    .fold(0usize, |bytes, file| {
                        bytes.saturating_add(file.memory_size())
                    }),
            )
            .saturating_add(
                input
                    .execution
                    .map(Seal::encoding_memory)
                    .transpose()?
                    .unwrap_or(0),
            )
            .saturating_add(
                input
                    .cycle
                    .map(Cycle::encoding_memory)
                    .transpose()?
                    .unwrap_or(0),
            )
            .saturating_add(input.file.memory_size()))
    }

    pub fn new(input: &Input<'_>) -> Result<Self> {
        if Self::encoding_memory(input)? == usize::MAX {
            return Err(Error::Invalid("report publication memory overflow"));
        }
        let mut executions = Vec::with_capacity(input.executions.len());
        for source in input.executions {
            executions.push(Revision::try_from(source)?);
        }
        Ok(Self {
            owner: input.owner.scope().to_string(),
            generated_at: input.generated_at.as_str().to_owned(),
            final_report: input.final_report,
            evidence_through: Some(input.evidence_through.clone()),
            dependencies_saved: Some(input.dependencies_saved.clone()),
            executions,
            inputs: input.inputs.to_vec(),
            materialized_inputs: input.materialized_inputs.to_vec(),
            file: Some(input.file.clone()),
            cycle: input.cycle.map(Cycle::try_from).transpose()?,
            execution: input.execution.map(Seal::try_from).transpose()?,
        })
    }

    pub fn file(&self) -> Result<&FileReference> {
        self.file
            .as_ref()
            .ok_or(Error::Invalid("report publication has no file"))
    }

    pub fn materialized_inputs(&self) -> &[FileReference] {
        &self.materialized_inputs
    }

    pub fn files(&self) -> impl Iterator<Item = &FileReference> {
        self.inputs
            .iter()
            .chain(self.materialized_inputs.iter())
            .chain(self.file.iter())
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn cycle(&self) -> Option<&Cycle> {
        self.cycle.as_ref()
    }

    fn check(&self) -> Result<()> {
        let (kind, owner) = self
            .owner
            .split_once('.')
            .ok_or(Error::Invalid("invalid report owner"))?;
        if !matches!(kind, "cycle" | "execution")
            || owner.len() != 36
            || owner.bytes().any(|byte| byte.is_ascii_uppercase())
            || TestCycleId::from_uuid(owner).is_err()
        {
            return Err(Error::Invalid("invalid report owner"));
        }
        time::OffsetDateTime::parse(
            &self.generated_at,
            &time::format_description::well_known::Rfc3339,
        )?;
        let evidence = self
            .evidence_through
            .as_ref()
            .ok_or(Error::Invalid("missing report evidence cut"))?;
        let saved = self
            .dependencies_saved
            .as_ref()
            .ok_or(Error::Invalid("missing report durability cut"))?;
        let file = self.file()?;
        if !saved.covers(evidence)?
            || !saved.covers(&file.position)?
            || file.metadata.media_type != "text/html"
            || self.inputs.len() != self.executions.len()
            || self.materialized_inputs.len() != self.executions.len()
            || file.metadata.scope.as_ref().map(Identifier::as_str) != Some(owner)
        {
            return Err(Error::Invalid("invalid report dependencies"));
        }
        if kind == "execution" && self.executions.len() != 1 {
            return Err(Error::Invalid("execution report requires one source"));
        }
        match (kind, &self.cycle) {
            ("cycle", Some(cycle)) => {
                cycle.check_report_source(
                    owner,
                    self.final_report.then_some(self.generated_at.as_str()),
                    self.executions
                        .iter()
                        .map(|source| source.check_cut(evidence)),
                )?;
            }
            ("execution", None) => {}
            _ => return Err(Error::Invalid("report owner differs from its cycle header")),
        }
        match (kind, self.final_report, &self.execution) {
            ("execution", true, Some(seal)) => {
                seal.check_publication(super::objects::execution::PublicationCheck {
                    source: &self.executions[0],
                    input: &self.inputs[0],
                    materialized_input: &self.materialized_inputs[0],
                    report: file,
                    evidence,
                    saved,
                    at: &self.generated_at,
                })?
            }
            ("execution", true, None) => {
                return Err(Error::Invalid("final execution report has no seal"));
            }
            (_, _, None) => {}
            _ => {
                return Err(Error::Invalid(
                    "execution seal requires a final execution report",
                ));
            }
        }
        for (index, (source, input)) in self.executions.iter().zip(&self.inputs).enumerate() {
            let execution = source.check_cut(evidence)?;
            crate::execution::saved::validate_materialized_input(
                &self.materialized_inputs[index],
                &TestExecutionId::from_uuid(execution)?,
                input,
                saved,
            )?;
            source.check_saved(saved)?;
            if (kind == "execution" && execution != owner)
                || !saved.covers(&input.position)?
                || input.metadata.scope.as_ref().map(Identifier::as_str) != Some(execution)
                || input.metadata.media_type
                    != crate::execution::saved::EXECUTION_CONTEXT_MEDIA_TYPE
            {
                return Err(Error::Invalid("report captured input differs from source"));
            }
        }
        Ok(())
    }

    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        Ok(<Self as Message>::decoding_memory(bytes)?.saturating_add(
            std::mem::size_of::<TestCycleId>().max(std::mem::size_of::<TestExecutionId>()) + 36,
        ))
    }
}

/* === Public Functions === */
pub fn schema() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: Identifier::new("dokime.report.publication").unwrap(),
        version: 1.try_into().unwrap(),
    }
}

pub fn query(owner: Owner<'_>, report: Option<Identifier>) -> RecordQuery<Publication> {
    let scope = owner.scope();
    let selected_scope = scope.clone();
    let selected_item = report.clone();
    let mut query = RecordQuery::new(schema(), move |bytes, allowance: NonZeroUsize| {
        if Publication::decoding_memory(bytes).map_err(failure)? > allowance.get() {
            return Err(harmos::Error::ResourceExhausted {
                resource: "report publication decoded allowance",
            });
        }
        let decode = || -> Result<Publication> {
            let publication = Publication::decode(bytes)?;
            publication.check()?;
            if publication.owner() != selected_scope.as_str()
                || selected_item.as_ref().is_some_and(|id| {
                    publication
                        .file
                        .as_ref()
                        .is_none_or(|file| &file.metadata.id != id)
                })
            {
                return Err(Error::Invalid("report publication differs from selection"));
            }
            Ok(publication)
        };
        decode().map_err(failure)
    });
    query.scope = Some(scope);
    query.item = report;
    query
}

/* === Private Functions === */
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    if bytes == usize::MAX {
        return Err(Error::Invalid("report publication memory overflow"));
    }
    NonZeroUsize::new(bytes).ok_or(Error::Invalid("empty report publication allowance"))
}

fn failure(error: Error) -> harmos::Error {
    match error {
        Error::Runtime(error) => error,
        other => {
            harmos::Error::backend("report publication", harmos::BackendFailure::Corrupt, other)
        }
    }
}

fn validate(input: &Input<'_>) -> Result<()> {
    input.generated_at.to_datetime()?;
    let owner = match input.owner {
        Owner::Cycle(id) => id.as_str(),
        Owner::Execution(id) => id.as_str(),
    };
    match (input.owner, input.cycle) {
        (Owner::Cycle(id), Some(cycle))
            if cycle.test_cycle_id == *id
                && cycle.saved.is_none()
                && (!input.final_report || cycle.updated_at == *input.generated_at)
                && cycle
                    .execution_ids
                    .iter()
                    .eq(input.executions.iter().map(|source| &source.execution_id)) => {}
        (Owner::Execution(_), None) => {}
        _ => return Err(Error::Invalid("report owner differs from its cycle header")),
    }
    if input.file.metadata.scope.as_ref().map(Identifier::as_str) != Some(owner) {
        return Err(Error::Invalid("report file scope differs from owner"));
    }
    if input.executions.len() != input.inputs.len()
        || input.executions.len() != input.materialized_inputs.len()
    {
        return Err(Error::Invalid(
            "report source and captured input counts differ",
        ));
    }
    if !input.dependencies_saved.covers(input.evidence_through)?
        || !input.dependencies_saved.covers(&input.file.position)?
        || input.file.metadata.media_type != "text/html"
    {
        return Err(Error::Invalid(
            "report dependencies are not saved HTML evidence",
        ));
    }
    if let Owner::Execution(owner) = input.owner
        && (input.executions.len() != 1 || &input.executions[0].execution_id != owner)
    {
        return Err(Error::Invalid("execution report sources differ from owner"));
    }
    match (input.owner, input.final_report, input.execution) {
        (Owner::Execution(_), true, Some(seal))
            if seal.source.execution == input.executions[0]
                && input.inputs.first() == Some(&seal.source.context)
                && seal.source.evidence_through == *input.evidence_through
                && input.materialized_inputs.first() == Some(&seal.materialized_input)
                && seal.report == *input.file
                && seal.dependencies_saved == *input.dependencies_saved
                && seal.sealed_at == *input.generated_at => {}
        (Owner::Execution(_), true, _) => {
            return Err(Error::Invalid(
                "final execution report differs from its seal",
            ));
        }
        (_, _, None) => {}
        _ => {
            return Err(Error::Invalid(
                "execution seal requires a final execution report",
            ));
        }
    }
    for (index, (source, captured)) in input.executions.iter().zip(input.inputs).enumerate() {
        source.version.check_position(&source.position)?;
        crate::execution::saved::validate_materialized_input(
            &input.materialized_inputs[index],
            &source.execution_id,
            captured,
            input.dependencies_saved,
        )?;
        input
            .evidence_through
            .history
            .check(&source.position.history)?;
        if !input.dependencies_saved.covers(&source.position)?
            || !input.dependencies_saved.covers(&captured.position)?
            || captured.metadata.scope.as_ref().map(Identifier::as_str)
                != Some(source.execution_id.as_str())
            || captured.metadata.media_type != crate::execution::saved::EXECUTION_CONTEXT_MEDIA_TYPE
        {
            return Err(Error::Invalid(
                "report source differs from saved evidence cut",
            ));
        }
    }
    Ok(())
}

/* === Tests === */
