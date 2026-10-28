/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{intern, resolve};
use crate::definition::TestSuite;
use crate::definition::{SuiteRegistry, SuiteSet, SuiteSnapshot, TestCase};
use crate::domains::TestSuiteDefinition;
use crate::records::RecordContract;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::plugins::InstalledPlugin;

pub(super) type Plugins = BTreeMap<String, InstalledPlugin>;
pub(super) type Records = BTreeMap<Identifier, RecordContract>;
pub(super) type History = BTreeMap<Identifier, BTreeMap<u32, RecordContract>>;
pub(super) type Telemetry = BTreeMap<Identifier, TelemetryContract>;

// Plugin catalogs reach state more than once: an installed plugin lists its
// suites and contracts, the suite registry copies each suite with its
// contracts, and record history repeats every current record definition. The
// encoded state stores each distinct suite and contract once and every owner
// refers to it by position, so the managed snapshot carries one catalog.
#[derive(Default, Serialize)]
pub(super) struct Pool<'a> {
    suites: Vec<&'a TestSuite>,
    records: Vec<&'a RecordContract>,
    telemetry: Vec<&'a TelemetryContract>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DecodedPool {
    suites: Vec<TestSuite>,
    records: Vec<RecordContract>,
    telemetry: Vec<TelemetryContract>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Contracts {
    records: Vec<u32>,
    telemetry: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Definition {
    suite_id: String,
    tags: Vec<String>,
    workflow: Option<String>,
    suite: u32,
    contracts: Contracts,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plugin {
    // The installation without its suites and contracts, which follow by reference.
    plugin: InstalledPlugin,
    suites: Vec<Definition>,
    contracts: Contracts,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    source: String,
    definition: Definition,
}

#[derive(Serialize)]
pub(super) struct Registry<'a> {
    suites: BTreeMap<&'a String, Snapshot>,
    cases: &'a BTreeMap<CaseId, TestCase>,
    registrations: &'a BTreeMap<CommandId, Vec<CaseId>>,
    case_revisions: &'a BTreeMap<CaseId, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DecodedRegistry {
    suites: BTreeMap<String, Snapshot>,
    cases: BTreeMap<CaseId, TestCase>,
    registrations: BTreeMap<CommandId, Vec<CaseId>>,
    case_revisions: BTreeMap<CaseId, String>,
}

impl<'a> Pool<'a> {
    pub(super) fn plugins(&mut self, plugins: &'a Plugins) -> BTreeMap<&'a String, Plugin> {
        plugins
            .iter()
            .map(|(source, plugin)| {
                let suites = plugin
                    .suites
                    .suites
                    .iter()
                    .map(|definition| Definition {
                        suite_id: definition.suite_id.clone(),
                        tags: definition.tags.clone(),
                        workflow: definition.workflow.clone(),
                        suite: intern(&mut self.suites, &definition.suite),
                        contracts: self.contracts(&definition.record_contracts, &definition.telemetry_contracts),
                    })
                    .collect();
                let contracts = self.contracts(&plugin.record_contracts, &plugin.telemetry_contracts);
                let lean = InstalledPlugin {
                    suites: SuiteSet::default(),
                    record_contracts: Vec::new(),
                    telemetry_contracts: Vec::new(),
                    ..plugin.clone()
                };
                let encoded = Plugin { plugin: lean, suites, contracts };
                (source, encoded)
            })
            .collect()
    }

    pub(super) fn registry(&mut self, registry: &'a SuiteRegistry) -> Registry<'a> {
        let suites = registry
            .snapshots()
            .iter()
            .map(|(id, snapshot)| {
                let definition = Definition {
                    suite_id: snapshot.suite_id.clone(),
                    tags: snapshot.tags.clone(),
                    workflow: snapshot.workflow.clone(),
                    suite: intern(&mut self.suites, &snapshot.suite),
                    contracts: self.contracts(&snapshot.record_contracts, &snapshot.telemetry_contracts),
                };
                let encoded = Snapshot {
                    source: snapshot.source.clone(),
                    definition,
                };
                (id, encoded)
            })
            .collect();
        Registry {
            suites,
            cases: registry.cases(),
            registrations: registry.registrations(),
            case_revisions: registry.case_revisions(),
        }
    }

    pub(super) fn records(&mut self, records: &'a Records) -> BTreeMap<&'a Identifier, u32> {
        records.iter().map(|(id, contract)| (id, intern(&mut self.records, contract))).collect()
    }

    pub(super) fn history(&mut self, history: &'a History) -> BTreeMap<&'a Identifier, BTreeMap<u32, u32>> {
        history
            .iter()
            .map(|(id, versions)| {
                let versions = versions.iter().map(|(version, contract)| (*version, intern(&mut self.records, contract))).collect();
                (id, versions)
            })
            .collect()
    }

    pub(super) fn telemetry(&mut self, telemetry: &'a Telemetry) -> BTreeMap<&'a Identifier, u32> {
        telemetry.iter().map(|(id, contract)| (id, intern(&mut self.telemetry, contract))).collect()
    }

    fn contracts(&mut self, records: &'a [RecordContract], telemetry: &'a [TelemetryContract]) -> Contracts {
        Contracts {
            records: records.iter().map(|contract| intern(&mut self.records, contract)).collect(),
            telemetry: telemetry.iter().map(|contract| intern(&mut self.telemetry, contract)).collect(),
        }
    }
}

impl DecodedPool {
    pub(super) fn plugins(&self, plugins: BTreeMap<String, Plugin>) -> Result<Plugins, String> {
        plugins
            .into_iter()
            .map(|(source, encoded)| {
                let Plugin { mut plugin, suites, contracts } = encoded;
                let suites = suites.into_iter().map(|definition| self.definition(definition)).collect::<Result<Vec<_>, String>>()?;
                let (records, telemetry) = self.contracts(contracts)?;

                plugin.suites = SuiteSet::new(suites);
                plugin.record_contracts = records;
                plugin.telemetry_contracts = telemetry;
                Ok((source, plugin))
            })
            .collect()
    }

    pub(super) fn registry(&self, registry: DecodedRegistry) -> Result<SuiteRegistry, String> {
        let DecodedRegistry {
            suites,
            cases,
            registrations,
            case_revisions,
        } = registry;
        let suites = suites
            .into_iter()
            .map(|(id, encoded)| {
                let definition = self.definition(encoded.definition)?;
                let snapshot = SuiteSnapshot {
                    suite_id: definition.suite_id,
                    source: encoded.source,
                    tags: definition.tags,
                    workflow: definition.workflow,
                    suite: definition.suite,
                    record_contracts: definition.record_contracts,
                    telemetry_contracts: definition.telemetry_contracts,
                };
                Ok((id, snapshot))
            })
            .collect::<Result<_, String>>()?;
        Ok(SuiteRegistry::from_parts(suites, cases, registrations, case_revisions))
    }

    pub(super) fn records(&self, records: BTreeMap<Identifier, u32>) -> Result<Records, String> {
        records.into_iter().map(|(id, index)| Ok((id, resolve(&self.records, index, "record contract")?))).collect()
    }

    pub(super) fn history(&self, history: BTreeMap<Identifier, BTreeMap<u32, u32>>) -> Result<History, String> {
        history
            .into_iter()
            .map(|(id, versions)| {
                let versions = versions
                    .into_iter()
                    .map(|(version, index)| Ok((version, resolve(&self.records, index, "record contract")?)))
                    .collect::<Result<_, String>>()?;
                Ok((id, versions))
            })
            .collect()
    }

    pub(super) fn telemetry(&self, telemetry: BTreeMap<Identifier, u32>) -> Result<Telemetry, String> {
        telemetry.into_iter().map(|(id, index)| Ok((id, resolve(&self.telemetry, index, "telemetry contract")?))).collect()
    }

    fn contracts(&self, contracts: Contracts) -> Result<(Vec<RecordContract>, Vec<TelemetryContract>), String> {
        let records =
            contracts.records.into_iter().map(|index| resolve(&self.records, index, "record contract")).collect::<Result<_, _>>()?;
        let telemetry =
            contracts.telemetry.into_iter().map(|index| resolve(&self.telemetry, index, "telemetry contract")).collect::<Result<_, _>>()?;
        Ok((records, telemetry))
    }

    fn definition(&self, definition: Definition) -> Result<TestSuiteDefinition, String> {
        let Definition {
            suite_id,
            tags,
            workflow,
            suite,
            contracts,
        } = definition;
        let suite = resolve(&self.suites, suite, "suite")?;
        let (records, telemetry) = self.contracts(contracts)?;
        Ok(TestSuiteDefinition {
            suite_id,
            tags,
            workflow,
            suite,
            record_contracts: records,
            telemetry_contracts: telemetry,
        })
    }
}
