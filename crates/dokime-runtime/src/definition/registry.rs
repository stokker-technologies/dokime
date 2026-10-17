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
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::definition::{SuiteSet, SuiteSnapshot, TestCase};
use crate::records::RecordContract;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::CommandId;
use crate::domains::{
    ExecutionProviderTarget, SidecarExecutionBinding, TestSuiteDefinition, resolve_suite_plot_axes,
};
use crate::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SuiteRegistry {
    suites: BTreeMap<String, SuiteSnapshot>,
    #[serde(default)]
    cases: BTreeMap<CaseId, TestCase>,
    #[serde(default)]
    registrations: BTreeMap<CommandId, Vec<CaseId>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SuiteRegistry {
    // The Registry's Parts, For The State Encoding That Stores Each Suite Once
    pub(crate) fn snapshots(&self) -> &BTreeMap<String, SuiteSnapshot> {
        &self.suites
    }

    pub(crate) fn cases(&self) -> &BTreeMap<CaseId, TestCase> {
        &self.cases
    }

    pub(crate) fn registrations(&self) -> &BTreeMap<CommandId, Vec<CaseId>> {
        &self.registrations
    }

    pub(crate) fn from_parts(
        suites: BTreeMap<String, SuiteSnapshot>,
        cases: BTreeMap<CaseId, TestCase>,
        registrations: BTreeMap<CommandId, Vec<CaseId>>,
    ) -> Self {
        Self {
            suites,
            cases,
            registrations,
        }
    }

    pub fn case(&self, id: &CaseId) -> Option<&TestCase> {
        self.cases.get(id)
    }

    // A Registered Case Is Immutable But For The Label An Operator Renames, So
    // Later Appends Still Find The Cycle's Copy Equal To The Registered One
    pub(crate) fn relabel_case(&mut self, id: &CaseId, label: &str) {
        if let Some(case) = self.cases.get_mut(id) {
            case.metadata.label = label.to_owned();
        }
    }

    pub fn cases_for_suite<'a>(&'a self, suite: &'a str) -> impl Iterator<Item = &'a TestCase> {
        self.cases
            .values()
            .filter(move |case| case.suite_id == suite)
    }

    pub(crate) fn validate_cases<'a>(
        &self,
        cases: impl IntoIterator<Item = &'a TestCase>,
    ) -> Result<(), String> {
        let mut incoming = BTreeMap::new();
        for case in cases {
            if case.suite_id.trim().is_empty() || case.suite.is_none() {
                return Err("materialized case requires an owning suite".into());
            }
            if let Some(existing) = self.cases.get(&case.test_case_id)
                && existing != case
            {
                return Err(format!("case {} is immutable", case.test_case_id));
            }
            if let Some(existing) = incoming.insert(&case.test_case_id, case)
                && existing != case
            {
                return Err(format!(
                    "case {} has conflicting definitions",
                    case.test_case_id
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_registration(
        &self,
        registration: &CommandId,
        cases: &[TestCase],
    ) -> Result<(), String> {
        self.validate_cases(cases)?;

        if let Some(existing) = self.registrations.get(registration)
            && !existing
                .iter()
                .eq(cases.iter().map(|case| &case.test_case_id))
        {
            return Err("registration identity has conflicting case membership".into());
        }
        Ok(())
    }

    pub(crate) fn register_selection(
        &mut self,
        registration: &CommandId,
        cases: &[TestCase],
    ) -> Result<(), String> {
        self.validate_registration(registration, cases)?;
        self.register_cases(cases)?;
        self.registrations
            .entry(registration.clone())
            .or_insert_with(|| cases.iter().map(|case| case.test_case_id.clone()).collect());
        Ok(())
    }

    pub(crate) fn register_cases<'a>(
        &mut self,
        cases: impl IntoIterator<Item = &'a TestCase>,
    ) -> Result<(), String> {
        let cases = cases.into_iter().collect::<Vec<_>>();
        self.validate_cases(cases.iter().copied())?;

        for case in cases {
            self.cases
                .entry(case.test_case_id.clone())
                .or_insert_with(|| case.clone());
        }
        Ok(())
    }

    pub fn suites(&self) -> impl Iterator<Item = &SuiteSnapshot> {
        self.suites.values()
    }

    pub fn get(&self, suite_id: &str) -> Option<&SuiteSnapshot> {
        self.suites.get(suite_id)
    }

    pub fn set(&mut self, source: &str, suites: SuiteSet) -> Result<(), String> {
        let mut projected = self.clone();
        projected.unregister_source(source);
        for definition in suites.suites {
            projected.insert(source, definition)?;
        }
        *self = projected;
        Ok(())
    }

    fn insert(&mut self, source: &str, mut definition: TestSuiteDefinition) -> Result<(), String> {
        if let Some(existing) = self.suites.get(&definition.suite_id) {
            return Err(format!(
                "suite {} is already owned by {}",
                definition.suite_id, existing.source
            ));
        }
        self.validate_templates(&definition)?;
        self.validate_contracts(&definition)?;
        // The suite declares both the streams it publishes and the documents
        // that draw them, so this is the one hand that holds a plot and the
        // unit of the column its x axis reads. Resolving here is why every
        // consumer downstream - the live document wire, a checkpoint, a
        // report - reads an axis it never has to infer.
        resolve_suite_plot_axes(&mut definition);
        let snapshot = SuiteSnapshot {
            suite_id: definition.suite_id.clone(),
            source: source.to_owned(),
            tags: definition.tags,
            workflow: definition.workflow,
            suite: definition.suite,
            record_contracts: definition.record_contracts,
            telemetry_contracts: definition.telemetry_contracts,
        };
        self.suites.insert(definition.suite_id, snapshot);
        Ok(())
    }

    fn validate_templates(&self, definition: &TestSuiteDefinition) -> Result<(), String> {
        let mut incoming = BTreeSet::new();
        for template in &definition.suite.test_case_templates {
            let template_id = template.test_case_template_id.to_string();
            if !incoming.insert(template_id.clone()) {
                return Err(format!(
                    "test case template {template_id} is declared more than once in suite {}",
                    definition.suite_id
                ));
            }
            if let Some(existing) = self.suites.values().find(|suite| {
                suite.suite.test_case_templates.iter().any(|candidate| {
                    candidate.test_case_template_id == template.test_case_template_id
                })
            }) {
                return Err(format!(
                    "test case template {template_id} is already owned by suite {}",
                    existing.suite_id
                ));
            }
        }
        Ok(())
    }

    fn validate_contracts(&self, definition: &TestSuiteDefinition) -> Result<(), String> {
        for existing in self.suites.values() {
            for contract in &definition.record_contracts {
                validate_record_contract(&existing.record_contracts, contract)?;
            }
            for contract in &definition.telemetry_contracts {
                validate_telemetry_contract(&existing.telemetry_contracts, contract)?;
            }
        }
        Ok(())
    }

    pub fn unregister_source(&mut self, source: &str) -> usize {
        let stale = self
            .suites
            .iter()
            .filter(|(_, suite)| suite.source == source)
            .map(|(suite_id, _)| suite_id.clone())
            .collect::<Vec<_>>();
        let count = stale.len();
        for suite_id in stale {
            self.suites.remove(&suite_id);
        }
        count
    }

    pub fn suite_for_case(&self, case: &TestCase) -> Option<&SuiteSnapshot> {
        let entry = self.get(&case.suite_id)?;
        let owner = case.suite.as_ref()?;
        (entry.suite.metadata == *owner
            && entry
                .suite
                .test_case_templates
                .iter()
                .any(|template| template.test_case_template_id == case.test_case_template_id))
        .then_some(entry)
    }

    pub fn provider_target_for_case(&self, case: &TestCase) -> Option<ExecutionProviderTarget> {
        let suite = self.suite_for_case(case)?;
        Some(ExecutionProviderTarget {
            source: suite.source.clone(),
            binding: SidecarExecutionBinding {
                workflow: suite.workflow.clone()?,
            },
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn validate_record_contract(
    existing: &[RecordContract],
    candidate: &RecordContract,
) -> Result<(), String> {
    if existing
        .iter()
        .any(|contract| contract.record_schema() == candidate.record_schema())
    {
        return Ok(());
    }
    if existing
        .iter()
        .any(|contract| contract.record_type == candidate.record_type)
    {
        return Err(format!(
            "conflicting record contract {}",
            candidate.record_type
        ));
    }
    Ok(())
}

fn validate_telemetry_contract(
    existing: &[TelemetryContract],
    candidate: &TelemetryContract,
) -> Result<(), String> {
    if existing
        .iter()
        .any(|contract| contract.stream_schema() == candidate.stream_schema())
    {
        return Ok(());
    }
    if existing
        .iter()
        .any(|contract| contract.stream == candidate.stream)
    {
        return Err(format!(
            "conflicting telemetry contract {}",
            candidate.stream
        ));
    }
    Ok(())
}

/* === Tests === */
