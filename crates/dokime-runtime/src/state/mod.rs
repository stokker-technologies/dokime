/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod encoding;
mod foreign_streams;
pub(crate) mod notes;
mod ownership;
mod record_history;

/* === Re-Exports === */

pub use ownership::{BundleCatalog, PluginOwner, ProviderCatalog};
pub(crate) use record_history::Redefinition;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::definition::SuiteRegistry;
use crate::definition::{MaterializedParameter, SuiteSnapshot};
use crate::prompts::{PromptDefinition, PromptInstance, PromptResponse};
use crate::records::RecordContract;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::identity::DokimeIdentity;
use dokime_domain::model::identity::DokimeNodeIdentity;
use dokime_domain::model::identity::HealthItemId;
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::plugins::InstalledPlugin;
use dokime_domain::model::plugins::InstalledPluginErrorRecord;
use dokime_domain::model::records::resources::ResourceRecord;
use dokime_domain::model::time::Timestamp;
use dokime_domain::state::records::EvidenceRecords;
use dokime_domain::state::records::resources::ResourceRegistry;
use dokime_domain::state::records::set::TestRecordSet;
use foreign_streams::{admit_foreign_view, declares_foreign_view};

/* === Constants === */

/* === Macros === */

product_error!(RuntimeStateInstallError, formatter {
    Self::RecordHistory(message) => write!(formatter, "record contract history: {message}"),
    Self::DuplicateRecord(identifier) => write!(formatter, "event definition {identifier} is already registered"),
    Self::DuplicateTelemetry(identifier) => write!(formatter, "telemetry definition {identifier} is already registered"),
    Self::ForeignTelemetryColumn { stream, column, detail } => write!(
        formatter,
        "foreign view of telemetry stream {stream} reads column {column}, which {detail}"
    ),
});

/* === Type Aliases === */

/* === Structs === */

// The managed snapshot encodes this state through `encoding`.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeState {
    pub session_id: SessionId,
    pub(crate) record_namespace: SessionId,
    pub identity: DokimeIdentity,
    pub run_parameters: BTreeMap<String, MaterializedParameter>,
    pub shutting_down: bool,
    pub installed_plugins: BTreeMap<String, InstalledPlugin>,
    pub plugin_owners: BTreeMap<String, PluginOwner>,
    pub suites: SuiteRegistry,
    pub record_definitions: BTreeMap<dokime_domain::model::identity::Identifier, RecordContract>,
    record_history: record_history::RecordHistory,
    pub telemetry_definitions: BTreeMap<dokime_domain::model::identity::Identifier, TelemetryContract>,
    pub cycles: BTreeMap<TestCycleId, dokime_domain::model::execution::result::TestCycle>,
    pub executions: BTreeMap<dokime_domain::model::identity::TestExecutionId, dokime_domain::model::execution::result::TestExecution>,
    // Live Projections Can Retire; Attempt Identities Must Never Be Reused.
    pub retired_executions: std::collections::BTreeSet<dokime_domain::model::identity::TestExecutionId>,
    pub cycle_report_finalizations: BTreeMap<TestCycleId, crate::transactions::CycleReportFinalization>,
    pub report_finalizations: BTreeMap<dokime_domain::model::identity::TestExecutionId, crate::transactions::ReportFinalization>,
    pub run_requests: BTreeMap<dokime_domain::model::identity::CommandId, crate::transactions::provider::request::RunRequest>,
    pub run_queue: std::collections::VecDeque<dokime_domain::model::identity::CommandId>,
    pub note_admissions: BTreeMap<dokime_domain::model::identity::NoteId, dokime_domain::model::execution::notes::NoteAdmission>,
    pub records: EvidenceRecords,
    pub resources: ResourceRegistry,
    pub installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
    pub prompt_definitions: BTreeMap<String, PromptDefinition>,
    pub record_count: u64,
    pub prompt_requests: BTreeMap<PromptId, PromptInstance>,
    pub prompt_responses: BTreeMap<PromptId, PromptResponse>,
    pub closed_prompts: BTreeMap<PromptId, Timestamp>,
    pub archived_cycles: std::collections::BTreeSet<TestCycleId>,
    pub archived_executions: std::collections::BTreeSet<dokime_domain::model::identity::TestExecutionId>,
}

// The Fields A Plugin Catalog Change Rewrites, Restored Together When The Change Is Refused.
struct CatalogSnapshot {
    installed_plugins: BTreeMap<String, InstalledPlugin>,
    suites: SuiteRegistry,
    prompt_definitions: BTreeMap<String, PromptDefinition>,
    record_definitions: BTreeMap<dokime_domain::model::identity::Identifier, RecordContract>,
    record_history: record_history::RecordHistory,
    telemetry_definitions: BTreeMap<dokime_domain::model::identity::Identifier, TelemetryContract>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeStateInstallError {
    DuplicateRecord(dokime_domain::model::identity::Identifier),
    RecordHistory(String),
    DuplicateTelemetry(dokime_domain::model::identity::Identifier),
    ForeignTelemetryColumn { stream: dokime_domain::model::identity::Identifier, column: String, detail: String },
}

/* === Traits === */

/* === Implementations === */

impl RuntimeState {
    // Catalog Validation Needs Cross-Provider Definitions, Never Execution Evidence.
    pub(crate) fn catalog_candidate(&self) -> Self {
        Self {
            installed_plugins: self.installed_plugins.clone(),
            plugin_owners: self.plugin_owners.clone(),
            suites: self.suites.clone(),
            prompt_definitions: self.prompt_definitions.clone(),
            record_definitions: self.record_definitions.clone(),
            record_history: self.record_history.clone(),
            telemetry_definitions: self.telemetry_definitions.clone(),
            ..Self::new(self.session_id.clone())
        }
    }

    fn adopt_catalog(&mut self, candidate: Self) {
        self.installed_plugins = candidate.installed_plugins;
        self.plugin_owners = candidate.plugin_owners;
        self.suites = candidate.suites;
        self.prompt_definitions = candidate.prompt_definitions;
        self.record_definitions = candidate.record_definitions;
        self.record_history = candidate.record_history;
        self.telemetry_definitions = candidate.telemetry_definitions;
    }

    fn same_catalog(&self, candidate: &Self) -> bool {
        let owners = self.installed_plugins == candidate.installed_plugins
            && self.plugin_owners == candidate.plugin_owners
            && self.suites == candidate.suites;
        let definitions = self.prompt_definitions == candidate.prompt_definitions
            && self.record_definitions == candidate.record_definitions
            && self.record_history == candidate.record_history
            && self.telemetry_definitions == candidate.telemetry_definitions;
        owners && definitions
    }

    pub fn case_definition_revision(&self, case: &crate::definition::TestCase) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        let suite = self.suites.suite_for_case(case).ok_or("case owning suite is not installed")?;
        let identity = self.execution_identity(&suite.source);
        let captured = serde_json::to_vec(&(suite, identity)).map_err(|error| error.to_string())?;
        let digest = Sha256::digest(captured);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn process_epoch(&self) -> &SessionId {
        &self.record_namespace
    }

    pub fn validate_execution_ownership(&self) -> Result<(), String> {
        let mut membership = std::collections::BTreeSet::new();
        for (cycle_id, cycle) in &self.cycles {
            if cycle_id != &cycle.test_cycle_id {
                return Err("cycle identity differs from its canonical key".into());
            }
            for id in &cycle.execution_ids {
                if self.retired_executions.contains(id) {
                    return Err("live execution identity is marked retired".into());
                }
                if !membership.insert(id) {
                    return Err("execution has duplicate cycle membership".into());
                }
                let execution = self.executions.get(id).ok_or("cycle references a missing execution owner")?;
                if &execution.test_execution_id != id {
                    return Err("execution identity differs from its canonical key".into());
                }
                if &execution.test_cycle_id != cycle_id {
                    return Err("execution belongs to another cycle".into());
                }
            }
        }
        for (id, intent) in &self.cycle_report_finalizations {
            let owner = self.cycles.get(id).ok_or("cycle report intent has no owner")?;
            if &intent.source.test_cycle_id != id
                || intent.source.execution_ids != owner.execution_ids
                || intent.source.saved.is_some()
                || intent.source.session_id.is_none()
            {
                return Err("cycle report intent differs from its canonical owner".into());
            }
        }
        if membership.len() != self.executions.len() {
            return Err("execution owner is absent from its cycle membership".into());
        }
        Ok(())
    }

    pub fn executions_for_cycle<'a>(
        &'a self, cycle: &'a dokime_domain::model::execution::result::TestCycle,
    ) -> impl ExactSizeIterator<Item = &'a dokime_domain::model::execution::result::TestExecution> + Clone {
        cycle.execution_ids.iter().map(|id| self.executions.get(id).expect("cycle execution owner exists"))
    }

    pub fn cases_for_cycle<'a>(
        &'a self, cycle: &'a dokime_domain::model::execution::result::TestCycle,
    ) -> impl Iterator<Item = &'a crate::definition::TestCase> {
        let mut seen = std::collections::BTreeSet::new();
        self.executions_for_cycle(cycle).filter_map(move |execution| {
            let case = execution.snapshot.case();
            seen.insert(&case.test_case_id).then_some(case)
        })
    }

    pub fn cycle_views(&self) -> impl Iterator<Item = dokime_domain::model::execution::result::CycleView> + '_ {
        self.cycles.keys().map(|id| self.cycle_view(id).expect("canonical cycle membership is valid"))
    }

    pub fn cycle_view(&self, id: &TestCycleId) -> Option<dokime_domain::model::execution::result::CycleView> {
        let cycle = self.cycles.get(id)?;
        let mut test_executions = Vec::with_capacity(cycle.execution_ids.len());
        for id in &cycle.execution_ids {
            test_executions.push(self.executions.get(id)?.clone());
        }
        Some(dokime_domain::model::execution::result::CycleView {
            saved: cycle.saved.clone(),
            test_cycle_id: cycle.test_cycle_id.clone(),
            session_id: cycle.session_id.clone(),
            metadata: cycle.metadata.clone(),
            phase: cycle.phase,
            lifecycle: cycle.lifecycle,
            created_at: cycle.created_at.clone(),
            started_at: cycle.started_at.clone(),
            updated_at: cycle.updated_at.clone(),
            test_executions,
            records: cycle.records.clone(),
            continue_automatically: cycle.continue_automatically,
            result: cycle.result.clone(),
        })
    }

    // Retire Sealed Cycles From Live State; Their Saved Closure Stays In Native History
    pub fn retire_sealed_cycles(&mut self) {
        let sealed: Vec<_> = self.cycles.values().filter(|cycle| cycle.saved.is_some()).map(|cycle| cycle.test_cycle_id.clone()).collect();
        for id in sealed {
            let cycle = self.cycles.remove(&id).expect("selected sealed cycle");
            for execution_id in &cycle.execution_ids {
                self.retired_executions.insert(execution_id.clone());
                self.executions.remove(execution_id);
            }
        }
    }

    pub fn validate_cycle_view(&self, view: &dokime_domain::model::execution::result::CycleView) -> Result<(), String> {
        use dokime_domain::model::execution::result::CyclePhase;
        if (view.phase == CyclePhase::Sealed) != view.saved.is_some() {
            return Err("sealed cycle phase requires its durable seal".into());
        }
        if self.cycles.get(&view.test_cycle_id).is_some_and(|existing| {
            (existing.saved.is_some() && existing.saved != view.saved)
                || (existing.phase != CyclePhase::Active && view.phase == CyclePhase::Active)
        }) {
            return Err("saved cycle closure cannot be changed".into());
        }
        if self.cycles.get(&view.test_cycle_id).is_some_and(|existing| existing.saved.is_some())
            && self.cycle_view(&view.test_cycle_id).as_ref() != Some(view)
        {
            return Err("sealed cycle is immutable".into());
        }
        if let Some(saved) = &view.saved {
            saved.validate_view(view).map_err(|error| error.to_string())?;
        }
        let mut ids = std::collections::BTreeSet::new();
        for execution in &view.test_executions {
            if self.retired_executions.contains(&execution.test_execution_id) {
                return Err("execution identity belongs to retired history".into());
            }
            if execution.test_cycle_id != view.test_cycle_id {
                return Err("execution belongs to another cycle".into());
            }
            if !ids.insert(execution.test_execution_id.clone()) {
                return Err("cycle contains duplicate execution membership".into());
            }
            if self.executions.get(&execution.test_execution_id).is_some_and(|existing| existing.test_cycle_id != view.test_cycle_id) {
                return Err("execution identity is owned by another cycle".into());
            }
            if self.executions.get(&execution.test_execution_id).is_some_and(|existing| existing.saved.is_some() && existing != execution) {
                return Err("sealed execution is immutable".into());
            }
        }
        for request in
            self.run_requests.values().filter(|request| request.summary.cycle_id == view.test_cycle_id && !request.summary.phase.terminal())
        {
            if request.selected_execution_ids.iter().any(|id| !ids.contains(id)) {
                return Err("an admitted request still owns this execution membership".into());
            }
        }
        Ok(())
    }

    pub fn replace_cycle(&mut self, view: dokime_domain::model::execution::result::CycleView) -> Result<(), String> {
        self.validate_cycle_view(&view)?;
        let ids: std::collections::BTreeSet<_> = view.test_executions.iter().map(|execution| execution.test_execution_id.clone()).collect();
        if let Some(previous) = self.cycles.get(&view.test_cycle_id) {
            for id in &previous.execution_ids {
                if !ids.contains(id) {
                    self.retired_executions.insert(id.clone());
                    self.executions.remove(id);
                }
            }
        }
        let execution_ids = view.test_executions.iter().map(|execution| execution.test_execution_id.clone()).collect();
        for execution in view.test_executions {
            self.executions.insert(execution.test_execution_id.clone(), execution);
        }
        self.cycles.insert(
            view.test_cycle_id.clone(),
            dokime_domain::model::execution::result::TestCycle {
                saved: view.saved,
                test_cycle_id: view.test_cycle_id,
                session_id: view.session_id,
                metadata: view.metadata,
                phase: view.phase,
                lifecycle: view.lifecycle,
                created_at: view.created_at,
                started_at: view.started_at,
                updated_at: view.updated_at,
                execution_ids,
                records: view.records,
                continue_automatically: view.continue_automatically,
                result: view.result,
            },
        );
        Ok(())
    }

    pub fn new(session_id: SessionId) -> Self {
        Self {
            record_namespace: session_id.clone(),
            session_id,
            identity: DokimeIdentity::node_only(unestablished_node_identity()),
            run_parameters: BTreeMap::new(),
            shutting_down: false,
            installed_plugins: BTreeMap::new(),
            plugin_owners: BTreeMap::new(),
            suites: SuiteRegistry::default(),
            record_definitions: BTreeMap::new(),
            record_history: Default::default(),
            telemetry_definitions: BTreeMap::new(),
            cycles: BTreeMap::new(),
            executions: BTreeMap::new(),
            retired_executions: std::collections::BTreeSet::new(),
            run_requests: BTreeMap::new(),
            run_queue: std::collections::VecDeque::new(),
            cycle_report_finalizations: BTreeMap::new(),
            report_finalizations: BTreeMap::new(),
            note_admissions: BTreeMap::new(),
            records: EvidenceRecords::default(),
            resources: ResourceRegistry::default(),
            installed_plugin_errors: Vec::new(),
            prompt_definitions: BTreeMap::new(),
            record_count: 0,
            prompt_requests: BTreeMap::new(),
            prompt_responses: BTreeMap::new(),
            closed_prompts: BTreeMap::new(),
            archived_cycles: std::collections::BTreeSet::new(),
            archived_executions: std::collections::BTreeSet::new(),
        }
    }

    pub fn install_builtin_domains(&mut self) -> Result<(), RuntimeStateInstallError> {
        for contract in crate::records::builtin_record_contracts() {
            self.ensure_record_definition(contract)?;
        }
        Ok(())
    }

    pub fn establish_node_identity(&mut self, node: DokimeNodeIdentity) {
        if self.identity.node.node_id == unestablished_node_id() {
            self.identity.node = node;
        }
    }

    pub fn record_installed_plugin_error(&mut self, record: InstalledPluginErrorRecord, maximum: u64) {
        self.installed_plugin_errors.push(record);
        let maximum = usize::try_from(maximum).expect("validated plugin diagnostic count");
        let excess = self.installed_plugin_errors.len().saturating_sub(maximum);
        self.installed_plugin_errors.drain(..excess);
    }

    pub fn resource_records(&self) -> Vec<ResourceRecord> {
        self.resources.records().cloned().collect()
    }

    pub fn install_plugin(&mut self, installation: InstalledPlugin) -> Result<(), String> {
        self.install_plugin_with_redefinition(installation, Redefinition::Refuse)
    }

    pub(crate) fn install_plugin_with_redefinition(
        &mut self, installation: InstalledPlugin, redefinition: Redefinition,
    ) -> Result<(), String> {
        installation.settings.validate().map_err(|error| error.to_string())?;
        let mut analyses = std::collections::BTreeSet::new();
        for analysis in &installation.analyses {
            analysis.check()?;
            if !analysis.id.starts_with(&format!("{}.", installation.source)) || !analyses.insert(&analysis.id) {
                return Err("analysis identities must be unique and owned by their artifact".into());
            }
        }
        let source = installation.source.clone();
        self.change_catalog(|state| {
            state.suites.set(&source, installation.suites.clone())?;
            state.installed_plugins.insert(source.clone(), installation);
            state.rebuild_installed_domains_with((redefinition == Redefinition::Supersede).then_some(source.as_str()))
        })
    }

    pub fn remove_plugin(&mut self, source: &str) -> Result<(), String> {
        self.change_catalog(|state| {
            state.installed_plugins.remove(source);
            state.suites.unregister_source(source);
            state.rebuild_installed_domains()
        })
    }

    // Applies A Catalog Change In Place, Copying Only The Catalog So A Refusal Leaves The State Untouched.
    fn change_catalog(&mut self, change: impl FnOnce(&mut Self) -> Result<(), String>) -> Result<(), String> {
        let snapshot = CatalogSnapshot {
            installed_plugins: self.installed_plugins.clone(),
            suites: self.suites.clone(),
            prompt_definitions: self.prompt_definitions.clone(),
            record_definitions: self.record_definitions.clone(),
            record_history: self.record_history.clone(),
            telemetry_definitions: self.telemetry_definitions.clone(),
        };
        let changed = change(self);
        if changed.is_err() {
            self.installed_plugins = snapshot.installed_plugins;
            self.suites = snapshot.suites;
            self.prompt_definitions = snapshot.prompt_definitions;
            self.record_definitions = snapshot.record_definitions;
            self.record_history = snapshot.record_history;
            self.telemetry_definitions = snapshot.telemetry_definitions;
        }
        changed
    }

    fn rebuild_installed_domains(&mut self) -> Result<(), String> {
        self.rebuild_installed_domains_with(None)
    }

    fn rebuild_installed_domains_with(&mut self, redefine_source: Option<&str>) -> Result<(), String> {
        let suites = self.suites.suites().cloned().collect::<Vec<_>>();
        let installations = self.installed_plugins.values().cloned().collect::<Vec<_>>();

        self.prompt_definitions.clear();
        self.record_definitions.clear();
        self.telemetry_definitions.clear();
        self.install_builtin_domains().map_err(|error| error.to_string())?;

        // Every stream declaration in the desired catalog, kept with the
        // source that declared it so a refusal can name the plugin to open.
        // Held until both walks are done because which of them owns a stream
        // and which only reads one is a question about the whole set.
        let mut streams: Vec<(String, TelemetryContract)> = Vec::new();

        for suite in suites {
            let source = suite.source;
            for contract in suite.record_contracts {
                self.ensure_record_definition_with(
                    contract,
                    if redefine_source == Some(source.as_str()) { Redefinition::Supersede } else { Redefinition::Refuse },
                )
                .map_err(|error| error.to_string())?;
            }
            streams.extend(suite.telemetry_contracts.into_iter().map(|contract| (source.clone(), contract)));
        }
        for installation in installations {
            let mut declarations = BTreeMap::new();
            let template_prompts = installation
                .suites
                .suites
                .iter()
                .flat_map(|suite| &suite.suite.test_case_templates)
                .flat_map(|template| &template.prompt_definitions)
                .cloned();
            for definition in installation.prompts.into_iter().chain(template_prompts) {
                let id = definition.prompt_type.as_str().to_owned();
                if let Some(existing) = declarations.get(&id) {
                    if existing != &definition {
                        return Err(format!("conflicting definitions for prompt {id} in {}", installation.source));
                    }
                    continue;
                }
                declarations.insert(id, definition);
            }
            for definition in declarations.into_values() {
                let issues = dokime_domain::model::prompts::validate_definition(&definition);
                if !issues.is_empty() {
                    return Err(format!(
                        "invalid prompt {} from {}: {}",
                        definition.prompt_type,
                        installation.source,
                        issues.into_iter().map(|issue| issue.message).collect::<Vec<_>>().join("; ")
                    ));
                }
                let id = definition.prompt_type.as_str().to_owned();
                if self.prompt_definitions.insert(id.clone(), definition).is_some() {
                    return Err(format!("prompt {id} is declared by multiple installed sources"));
                }
            }
            for contract in installation.record_contracts {
                self.ensure_record_definition_with(
                    contract,
                    if redefine_source == Some(installation.source.as_str()) { Redefinition::Supersede } else { Redefinition::Refuse },
                )
                .map_err(|error| error.to_string())?;
            }
            let source = installation.source;
            streams.extend(installation.telemetry_contracts.into_iter().map(|contract| (source.clone(), contract)));
        }
        self.admit_declared_streams(streams)
    }

    fn admit_declared_streams(&mut self, streams: Vec<(String, TelemetryContract)>) -> Result<(), String> {
        let (views, owned): (Vec<_>, Vec<_>) = streams.into_iter().partition(|(_, contract)| declares_foreign_view(contract));

        for (source, contract) in owned {
            self.ensure_telemetry_definition(&source, contract).map_err(|error| error.to_string())?;
        }
        // Only the views name their plugin: a reader is wrong about a contract
        // it does not control, so the refusal has to say whose declaration to
        // go and fix. An owner's collision already names the stream, and the
        // stream is the plugin's own.
        for (source, contract) in views {
            self.ensure_telemetry_definition(&source, contract).map_err(|error| format!("plugin {source}: {error}"))?;
        }
        Ok(())
    }

    pub(crate) fn admit_requested_streams(&self) -> Result<(), String> {
        for installation in self.installed_plugins.values() {
            for suite in &installation.suites.suites {
                for template in &suite.suite.test_case_templates {
                    for (index, subscription) in template.subscriptions.iter().enumerate() {
                        if template.subscriptions[..index].contains(subscription) {
                            return Err(format!(
                                "plugin {} template {} declares duplicate input {subscription:?}",
                                installation.source, template.metadata.identifier,
                            ));
                        }
                    }
                    for record in template.subscriptions.iter().filter_map(crate::definition::Subscription::record) {
                        if !self.record_definitions.contains_key(record) {
                            return Err(format!(
                                "plugin {} template {} subscribes to record {record}, which no installed plugin publishes",
                                installation.source, template.metadata.identifier,
                            ));
                        }
                    }
                    for stream in template.subscriptions.iter().filter_map(crate::definition::Subscription::stream) {
                        if self.telemetry_definitions.contains_key(stream) || is_host_stream(stream) {
                            continue;
                        }
                        return Err(format!(
                            "plugin {} template {} requests telemetry stream {stream}, which no installed plugin publishes",
                            installation.source, template.metadata.identifier,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn sync_suite_definitions(&mut self) -> Result<(), RuntimeStateInstallError> {
        let record_contracts = self.suites.suites().flat_map(|snapshot| snapshot.record_contracts.clone()).collect::<Vec<_>>();
        // Owners before the consumers that only read them, for the reason
        // [`Self::admit_declared_streams`] gives: the registry is keyed by
        // suite id, which orders the declarations by a name that has nothing to
        // do with who owns which stream.
        let (foreign_views, owned_streams): (Vec<_>, Vec<_>) = self
            .suites
            .suites()
            .flat_map(|snapshot| snapshot.telemetry_contracts.iter().map(|contract| (snapshot.source.clone(), contract.clone())))
            .partition(|(_, contract)| declares_foreign_view(contract));
        for contract in record_contracts {
            self.ensure_record_definition(contract)?;
        }
        for (source, contract) in owned_streams.into_iter().chain(foreign_views) {
            self.ensure_telemetry_definition(&source, contract)?;
        }
        Ok(())
    }

    pub fn ensure_record_definition(&mut self, contract: RecordContract) -> Result<(), RuntimeStateInstallError> {
        self.ensure_record_definition_with(contract, Redefinition::Refuse)
    }

    fn ensure_record_definition_with(&mut self, contract: RecordContract, policy: Redefinition) -> Result<(), RuntimeStateInstallError> {
        if let Some(definition) = self.record_definitions.get(&contract.record_type) {
            return if definition.record_schema() == contract.record_schema() {
                Ok(())
            } else {
                Err(RuntimeStateInstallError::DuplicateRecord(contract.record_type))
            };
        }
        match policy {
            Redefinition::Refuse => self.record_history.admit(&contract),
            Redefinition::Supersede => self.record_history.supersede(&contract),
        }
        .map_err(RuntimeStateInstallError::RecordHistory)?;
        self.record_definitions.insert(contract.record_type.clone(), contract);
        Ok(())
    }

    pub fn ensure_telemetry_definition(&mut self, source: &str, contract: TelemetryContract) -> Result<(), RuntimeStateInstallError> {
        let Some(definition) = self.telemetry_definitions.get(&contract.stream) else {
            self.telemetry_definitions.insert(contract.stream.clone(), contract);
            return Ok(());
        };
        if declares_foreign_view(&contract) {
            return if definition.normalized() == contract.normalized() { Ok(()) } else { admit_foreign_view(definition, &contract) };
        }
        if definition.stream_schema() == contract.stream_schema() && !self.owned_elsewhere(source, &contract.stream) {
            return Ok(());
        }
        Err(RuntimeStateInstallError::DuplicateTelemetry(contract.stream))
    }

    fn owned_elsewhere(&self, source: &str, stream: &dokime_domain::model::identity::Identifier) -> bool {
        let suites = self.suites.suites().filter(|snapshot| snapshot.source != source).flat_map(|snapshot| &snapshot.telemetry_contracts);
        let plugins = self
            .installed_plugins
            .values()
            .filter(|installation| installation.source != source)
            .flat_map(|installation| &installation.telemetry_contracts);

        suites.chain(plugins).any(|declared| !declares_foreign_view(declared) && &declared.stream == stream)
    }

    pub(crate) fn historical_record_contracts(
        &self,
    ) -> &BTreeMap<dokime_domain::model::identity::Identifier, BTreeMap<u32, RecordContract>> {
        self.record_history.contracts()
    }

    pub fn historical_record_contract(&self, kind: &dokime_domain::model::identity::Identifier, version: u32) -> Option<&RecordContract> {
        self.record_history.get(kind, version)
    }

    pub fn record_contracts(&self) -> Vec<RecordContract> {
        self.record_definitions.values().cloned().collect()
    }

    pub fn telemetry_contracts(&self) -> Vec<TelemetryContract> {
        self.telemetry_definitions.values().cloned().collect()
    }

    pub fn test_suites(&self) -> Vec<SuiteSnapshot> {
        self.suites.suites().cloned().collect()
    }

    pub fn prompt_definitions(&self) -> Vec<PromptDefinition> {
        self.prompt_definitions.values().cloned().collect()
    }

    pub fn pending_prompt_projection(&self) -> Vec<PromptInstance> {
        self.open_prompts().cloned().collect()
    }

    pub fn open_prompts(&self) -> impl Iterator<Item = &PromptInstance> {
        self.prompt_requests.values().filter(|prompt| self.prompt_open(prompt))
    }

    pub fn prompt_open(&self, prompt: &PromptInstance) -> bool {
        if self.prompt_responses.contains_key(&prompt.prompt_id) || self.closed_prompts.contains_key(&prompt.prompt_id) {
            return false;
        }
        if let Some(id) = &prompt.scope.test_execution_id {
            return self.executions.get(id).is_some_and(|execution| {
                execution.finished_at.is_none()
                    && matches!(
                        execution.status,
                        dokime_domain::model::execution::result::ExecutionStatus::Pending
                            | dokime_domain::model::execution::result::ExecutionStatus::Running
                    )
            });
        }
        prompt.scope.test_cycle_id.as_ref().is_none_or(|id| self.cycles.contains_key(id))
    }

    pub fn projected_prompt_request(&self, prompt_id: &PromptId) -> Option<PromptInstance> {
        self.prompt_requests.get(prompt_id).cloned()
    }

    pub fn prompt_response(&self, prompt_id: &PromptId) -> Option<PromptResponse> {
        self.prompt_responses.get(prompt_id).cloned()
    }

    pub fn prompt_condition_to_check(
        &self, prompt_id: &PromptId, action: &str,
    ) -> Result<Option<crate::definition::RuntimeCondition>, String> {
        // Existence/action/duplicate errors belong to the response transaction.
        // Only a pending, valid action can require this external precheck.
        let Some(prompt) = self.prompt_requests.get(prompt_id).filter(|prompt| self.prompt_open(prompt)) else {
            return Ok(None);
        };
        let Some(action) = prompt.definition.actions.iter().find(|candidate| candidate.id == action.trim()) else {
            return Ok(None);
        };
        if !action.requires_condition {
            return Ok(None);
        }
        prompt.definition.condition.clone().map(Some).ok_or_else(|| "required prompt condition is missing".to_owned())
    }

    pub fn session_records(&self) -> &TestRecordSet {
        &self.records.session
    }

    pub fn session_records_mut(&mut self) -> &mut TestRecordSet {
        &mut self.records.session
    }

    pub fn health_items(&self) -> Vec<HealthItem> {
        self.iter_health_items().cloned().collect()
    }
    pub fn notes(&self) -> Vec<OperatorNote> {
        self.iter_notes().cloned().collect()
    }

    pub fn records_for_scope_mut(&mut self, scope: &RecordScope) -> Option<&mut TestRecordSet> {
        let Some(test_cycle_id) = scope.test_cycle_id.as_ref() else {
            return Some(self.session_records_mut());
        };
        if let Some(test_execution_id) = scope.test_execution_id.as_ref() {
            let execution = self.executions.get_mut(test_execution_id)?;
            if &execution.test_cycle_id != test_cycle_id {
                return None;
            }
            return Some(&mut execution.records);
        }
        Some(&mut self.cycles.get_mut(test_cycle_id)?.records)
    }

    pub fn record_health(
        &mut self, scope: RecordScope, severity: HealthSeverity, message: impl Into<String>, detail: Option<String>, at: Timestamp,
    ) {
        let health = HealthItem {
            health_item_id: HealthItemId::from_seed(format!("health:{at}:{}", self.health_items().len())),
            scope: scope.clone(),
            severity,
            message: message.into(),
            detail,
            observed_at: at,
        };
        self.record_health_item(health);
    }

    pub fn record_health_item(&mut self, health: HealthItem) {
        let scope = health.scope.clone();
        if let Some(records) = self.records_for_scope_mut(&scope) {
            records.push_health(health);
        }
    }

    pub fn next_record_id(&self) -> dokime_domain::model::identity::RecordId {
        dokime_domain::model::identity::RecordId::from_seed(format!("dokime-record:{}:{}", self.record_namespace, self.record_count))
    }

    pub(crate) fn capture_scope(&self, supplied: &RecordScope) -> Result<RecordScope, String> {
        let Some(id) = supplied.test_execution_id.as_ref() else {
            let mut scope = supplied.clone();
            scope.origin = dokime_domain::model::execution::EvidenceOrigin::Default;
            scope.step_id = None;
            return Ok(scope);
        };
        let execution = self.executions.get(id).ok_or("capture execution is missing")?;
        if supplied.test_cycle_id.as_ref().is_some_and(|id| id != &execution.test_cycle_id)
            || supplied.test_case_id.as_ref().is_some_and(|id| id != &execution.test_case_id)
        {
            return Err("capture scope differs from its execution owner".into());
        }
        Ok(execution.capture_scope())
    }

    pub fn commit_record(&mut self, captured: dokime_domain::state::records::CapturedRecord) {
        for scope in &captured.scopes {
            if let Some(owner) = &scope.test_execution_id
                && let Some(execution) = self.executions.get_mut(owner)
            {
                execution.records.capture_frontier(owner.clone(), captured.ordinal);
            }
        }
        self.records.through = captured.ordinal;
        self.resources.record_authored_record(&captured.record);
        self.record_count = self.record_count.checked_add(1).expect("admitted record count fits");
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn unestablished_node_id() -> NodeId {
    NodeId::from_seed("dokime.unestablished-node")
}

fn unestablished_node_identity() -> DokimeNodeIdentity {
    DokimeNodeIdentity::new(unestablished_node_id(), None, Timestamp::new("1970-01-01T00:00:00Z"))
}

fn is_host_stream(stream: &dokime_domain::model::identity::Identifier) -> bool {
    dokime_domain::model::telemetry::HOST_STREAMS.iter().any(|host| host.as_str() == stream.as_str())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_catalog_change_keeps_the_state() {
        let mut state = RuntimeState::new(SessionId::from_seed("catalog.fixture"));
        state.install_builtin_domains().unwrap();
        let plugin = |source: &str, label: &str| InstalledPlugin {
            source: source.to_owned(),
            record_contracts: vec![RecordContract::declared("fixture.reading", label)],
            ..Default::default()
        };
        state.install_plugin(plugin("fixture.first", "First")).unwrap();
        let installed = state.clone();

        // A Conflicting Record Refuses Inside The Rebuild And Leaves Every Catalog Field
        assert!(state.install_plugin(plugin("fixture.second", "Second")).is_err());
        assert_eq!(state, installed);

        state.remove_plugin("fixture.first").unwrap();
        assert!(!state.record_definitions.contains_key(&dokime_domain::model::identity::Identifier::new("fixture.reading").unwrap()));
    }
}
