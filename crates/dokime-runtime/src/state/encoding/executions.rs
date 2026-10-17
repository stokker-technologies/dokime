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
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Local Imports
use super::{intern, position, resolve};
use crate::definition::{ParameterDefinition, TestCaseTemplate};
use crate::document::DocumentDefinition;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::snapshot::ExecutionSnapshot;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type Executions = BTreeMap<TestExecutionId, TestExecution>;
type DocumentRefs = BTreeMap<DocumentId, u32>;

/* === Structs === */
// Every member of a cycle copies its template, its planned documents and its
// parameter definitions, and each execution starts with its own copy of those
// documents. The encoded state stores each distinct value once, before the
// members that reference it by position, so one wide cycle does not multiply
// its template into the managed snapshot.
#[derive(Serialize)]
struct Encoded<'a> {
    shared: &'a Shared<'a>,
    members: Members<'a>,
}

#[derive(Default, Serialize)]
struct Shared<'a> {
    templates: Vec<&'a TestCaseTemplate>,
    documents: Vec<&'a DocumentDefinition>,
    parameter_definitions: Vec<&'a [ParameterDefinition]>,
}

struct Members<'a> {
    executions: &'a Executions,
    shared: &'a Shared<'a>,
}

#[derive(Serialize)]
struct Member<'a> {
    // The execution without its pooled parts; the parts below restore them.
    execution: &'a TestExecution,
    template: Option<u32>,
    parameter_definitions: u32,
    planned_documents: DocumentRefs,
    documents: DocumentRefs,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decoded {
    shared: DecodedShared,
    members: BTreeMap<TestExecutionId, DecodedMember>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecodedShared {
    templates: Vec<TestCaseTemplate>,
    documents: Vec<DocumentDefinition>,
    parameter_definitions: Vec<Vec<ParameterDefinition>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecodedMember {
    execution: TestExecution,
    template: Option<u32>,
    parameter_definitions: u32,
    planned_documents: DocumentRefs,
    documents: DocumentRefs,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> Shared<'a> {
    fn collect(executions: &'a Executions) -> Self {
        let mut shared = Self::default();
        for execution in executions.values() {
            let snapshot = &execution.snapshot;
            if let Some(template) = snapshot.template() {
                intern(&mut shared.templates, template);
            }
            intern(&mut shared.parameter_definitions, snapshot.parameter_definitions());
            for document in snapshot.documents().values().chain(execution.documents.values()) {
                intern(&mut shared.documents, document);
            }
        }
        shared
    }

    fn documents(&self, documents: &BTreeMap<DocumentId, DocumentDefinition>) -> DocumentRefs {
        documents.iter().map(|(id, document)| (id.clone(), position(&self.documents, document))).collect()
    }
}

impl Serialize for Members<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.executions.len()))?;
        for (id, execution) in self.executions {
            // One Lean Copy At A Time Keeps The Encoder's Peak Memory To One Member
            let snapshot = &execution.snapshot;
            let lean = lean_clone(execution);
            let member = Member {
                execution: &lean,
                template: snapshot.template().map(|template| position(&self.shared.templates, template)),
                parameter_definitions: position(&self.shared.parameter_definitions, snapshot.parameter_definitions()),
                planned_documents: self.shared.documents(snapshot.documents()),
                documents: self.shared.documents(&execution.documents),
            };
            map.serialize_entry(id, &member)?;
        }
        map.end()
    }
}

impl DecodedShared {
    fn documents(&self, references: DocumentRefs) -> Result<BTreeMap<DocumentId, DocumentDefinition>, String> {
        references.into_iter().map(|(id, index)| Ok((id, resolve(&self.documents, index, "document")?))).collect()
    }

    fn restore(&self, member: DecodedMember) -> Result<TestExecution, String> {
        let DecodedMember { mut execution, template, parameter_definitions, planned_documents, documents } = member;
        let lean = &execution.snapshot;
        let template = template.map(|index| resolve(&self.templates, index, "template")).transpose()?;
        let definitions = resolve(&self.parameter_definitions, parameter_definitions, "parameter set")?;

        execution.snapshot = ExecutionSnapshot::from_captured(
            lean.case().clone(),
            template,
            definitions,
            lean.plugin().cloned(),
            lean.host_version().map(str::to_owned),
            lean.parameters().clone(),
            self.documents(planned_documents)?,
        );
        execution.documents = self.documents(documents)?;
        Ok(execution)
    }
}

/* === Public Functions === */
pub(super) fn serialize<S: Serializer>(executions: &Executions, serializer: S) -> Result<S::Ok, S::Error> {
    let shared = Shared::collect(executions);
    Encoded { shared: &shared, members: Members { executions, shared: &shared } }.serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Executions, D::Error> {
    let Decoded { shared, members } = Decoded::deserialize(deserializer)?;

    members
        .into_iter()
        .map(|(id, member)| Ok((id, shared.restore(member)?)))
        .collect::<Result<_, String>>()
        .map_err(serde::de::Error::custom)
}

/* === Private Functions === */
// Clones Every Field The Pool Does Not Hold; Pooled Ones Stay Empty
fn lean_clone(execution: &TestExecution) -> TestExecution {
    let TestExecution {
        saved,
        mode,
        test_cycle_id,
        test_execution_id,
        test_case_id,
        suite,
        status,
        snapshot,
        invalidation,
        scope,
        output_parameters,
        documents: _,
        records,
        started_at,
        finished_at,
        steps,
        step_attempts,
        active_step_attempt,
        capture_phase,
        traceability,
        result,
    } = execution;
    TestExecution {
        saved: saved.clone(),
        mode: *mode,
        test_cycle_id: test_cycle_id.clone(),
        test_execution_id: test_execution_id.clone(),
        test_case_id: test_case_id.clone(),
        suite: suite.clone(),
        status: status.clone(),
        snapshot: ExecutionSnapshot::from_captured(
            snapshot.case().clone(),
            None,
            Vec::new(),
            snapshot.plugin().cloned(),
            snapshot.host_version().map(str::to_owned),
            snapshot.parameters().clone(),
            BTreeMap::new(),
        ),
        invalidation: invalidation.clone(),
        scope: scope.clone(),
        output_parameters: output_parameters.clone(),
        documents: BTreeMap::new(),
        records: records.clone(),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        steps: steps.clone(),
        step_attempts: step_attempts.clone(),
        active_step_attempt: active_step_attempt.clone(),
        capture_phase: *capture_phase,
        traceability: traceability.clone(),
        result: result.clone(),
    }
}

/* === Tests === */
