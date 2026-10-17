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
use std::time::Duration;
// Third Party Imports
// Third-Party Imports
use harmos::sidecar::Refusal;
// Local Imports
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::values::Value;
use crate::definition::{
    CaseMaterializationOptions, CaseMaterializationSelection, TestCaseTemplate,
    materialize_template_cases, parameter_name,
};
use crate::document::{DocumentAddition, DocumentDefinition};
use crate::execution::log;
use crate::execution::{ExecutionHost, Runnable};
use crate::plugin::Diagnostic;
use crate::prompts::{PromptDefinition, PromptRequest, PromptResponse};
use crate::records::DeclaredRecord;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::records::publication::RecordPublishRequest;
use crate::telemetry::{TelemetryRow, TelemetrySample, TelemetryStream};
use dokime_domain::model::traceability::TraceabilityRow;
use dokime_domain::model::traceability::TraceabilitySection;
use crate::transport::{
    ArtifactChunk, DeliverRecordRequest, DeliverSamplesRequest, EvidenceBatch, EvidenceReceipt,
    ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion,
    RunExecution, SidecarCriterionOutcome, SidecarExecutionOutcome, StepAdmission,
    TransportFuture,
};

/* === Constants === */
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

const HOST_SOURCE: &str = "dokime.harness";

const RESPONDER: &str = "dokime.harness";

const CANCEL: &str = "cancel";

/* === Macros === */

/* === Type Aliases === */
type ParameterValues = BTreeMap<String, Value>;

/* === Structs === */
pub struct Harness {
    template: String,
    parameters: BTreeMap<String, Value>,
    settings: BTreeMap<String, Value>,
    secrets: BTreeMap<String, String>,
    published: Vec<Record>,
    answers: BTreeMap<String, Answer>,
    measured: BTreeMap<String, Vec<BTreeMap<String, Value>>>,
    operations: BTreeMap<String, crate::execution::Answering>,
    timeout: Duration,
}

struct Host {
    pages: std::sync::Mutex<Pages>,
    answers: BTreeMap<String, Answer>,
    prompts: Vec<PromptDefinition>,
    measured: std::collections::BTreeSet<String>,
}

struct Answer {
    action: String,
    values: BTreeMap<String, Value>,
}

#[derive(Debug)]
pub struct Report {
    outcome: SidecarExecutionOutcome,
    samples: BTreeMap<String, Vec<BTreeMap<String, Value>>>,
    records: Vec<RecordPublishRequest>,
    subscribed_records: Vec<dokime_domain::model::records::evidence::Record>,
    traceability: Vec<TraceabilitySection>,
    artifacts: Vec<Artifact>,
    log: Vec<log::Entry>,
    document: DocumentDefinition,
}

#[derive(Clone)]
pub struct Artifact {
    pub name: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
    pub abandoned: bool,
}

pub struct Section<'a> {
    identifier: String,
    section: Option<&'a TraceabilitySection>,
}

pub struct Row<'a> {
    section: String,
    identifier: String,
    row: Option<&'a TraceabilityRow>,
}

#[derive(Default)]
struct Pages {
    samples: BTreeMap<String, Vec<BTreeMap<String, Value>>>,
    records: Vec<RecordPublishRequest>,
    subscribed_records: Vec<dokime_domain::model::records::evidence::Record>,
    traceability: Vec<TraceabilitySection>,
    documents: Vec<DocumentAddition>,
    transfers: Vec<Transfer>,
    log: Vec<log::Entry>,
    outcome: Option<SidecarExecutionOutcome>,
}

struct Transfer {
    transfer_id: String,
    next: u64,
    artifact: Artifact,
}

/* === Enums === */
#[derive(Debug)]
pub enum HarnessError {
    UnknownTemplate(String),
    AmbiguousTemplate {
        template: String,
        workflows: Vec<String>,
    },
    Parameters(String),
    Start(Refusal),
    Deliver(Refusal),
    Timeout(Duration),
    Task(tokio::task::JoinError),
}

/* === Traits === */

/* === Implementations === */
impl Harness {
    pub fn template(identifier: impl Into<String>) -> Self {
        Self {
            template: identifier.into(),
            parameters: BTreeMap::new(),
            settings: BTreeMap::new(),
            secrets: BTreeMap::new(),
            published: Vec::new(),
            answers: BTreeMap::new(),
            measured: BTreeMap::new(),
            operations: BTreeMap::new(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    #[must_use]
    pub fn answering<O, F>(mut self, answer: F) -> Self
    where
        O: harmos::operations::Declares,
        <O::Declaration as harmos::operations::OperationDeclaration>::Request: harmos::Message,
        <O::Declaration as harmos::operations::OperationDeclaration>::Response: harmos::Message,
        F: Fn(
                <O::Declaration as harmos::operations::OperationDeclaration>::Request,
            )
                -> <O::Declaration as harmos::operations::OperationDeclaration>::Response
            + Send
            + Sync
            + 'static,
    {
        let answer = std::sync::Arc::new(answer);
        self.operations.insert(
            <O::Declaration as harmos::operations::OperationDeclaration>::ID.to_owned(),
            std::sync::Arc::new(move |payload: Vec<u8>| {
                let answer = std::sync::Arc::clone(&answer);
                Box::pin(async move {
                    let request = <<O::Declaration as harmos::operations::OperationDeclaration>::Request as harmos::Message>::decode(payload.as_slice())
                        .map_err(|error| error.to_string())?;
                    Ok(harmos::Message::encode_to_vec(&answer(request)))
                })
                    as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
            }),
        );
        self
    }

    #[must_use]
    pub fn parameter(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.parameters.insert(name.into(), value.into());
        self
    }

    #[must_use]
    pub fn setting(mut self, identifier: impl Into<String>, value: impl Into<Value>) -> Self {
        self.settings.insert(identifier.into(), value.into());
        self
    }

    #[must_use]
    pub fn secret(mut self, identifier: impl Into<String>, value: impl Into<String>) -> Self {
        self.secrets.insert(identifier.into(), value.into());
        self
    }

    #[must_use]
    pub fn publish<R: DeclaredRecord>(mut self, record: R) -> Self {
        let mut filed = Record::new(
            crate::model::identity::new_id::<dokime_domain::model::identity::RecordId>(),
            crate::model::time::now_utc(),
            R::record_type(),
            R::SCHEMA_VERSION,
            HOST_SOURCE,
            R::SEVERITY,
        );
        filed.message = Some(record.message());
        filed.details = record.into_details();
        // An open scope reaches every run, which is the only run there is here.
        filed.scope = None::<RecordScope>;
        self.published.push(filed);
        self
    }

    #[must_use]
    pub fn answer(
        mut self,
        prompt_type: impl Into<String>,
        action: impl Into<String>,
        values: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
    ) -> Self {
        self.answers.insert(
            prompt_type.into(),
            Answer {
                action: action.into(),
                values: values
                    .into_iter()
                    .map(|(name, value)| (name.into(), value.into()))
                    .collect(),
            },
        );
        self
    }

    #[must_use]
    pub fn measuring<C: crate::telemetry::TelemetryStream>(
        mut self,
        rows: impl IntoIterator<Item = C::Row>,
    ) -> Self {
        self.measured.entry(C::ID.to_owned()).or_default().extend(
            rows.into_iter()
                .map(crate::telemetry::TelemetryRow::into_row),
        );
        self
    }

    #[must_use]
    pub fn timeout(mut self, within: Duration) -> Self {
        self.timeout = within;
        self
    }

    pub async fn run<S>(self) -> Result<Report, HarnessError>
    where
        S: crate::catalog::Catalog<crate::catalog::Suites>
            + crate::catalog::Catalog<crate::catalog::SuiteRunners>,
    {
        let declarations = crate::catalog::entries::<S, crate::catalog::Suites>()
            .into_iter()
            .map(|definition| definition())
            .collect::<Vec<_>>();
        let runners = crate::catalog::entries::<S, crate::catalog::SuiteRunners>()
            .into_iter()
            .flat_map(|runners| runners())
            .collect::<Vec<_>>();
        tokio::spawn(self.execute(declarations, runners))
            .await
            .map_err(HarnessError::Task)?
    }

    async fn execute(
        self,
        suites: Vec<crate::domains::TestSuiteDefinition>,
        runners: Vec<std::sync::Arc<dyn Runnable>>,
    ) -> Result<Report, HarnessError> {
        let (suite_id, workflow, template) = self.declaration(&suites)?;
        let runner = runners
            .into_iter()
            .find(|runner| runner.template_identifier() == self.template)
            .ok_or_else(|| HarnessError::UnknownTemplate(self.template.clone()))?;
        // Taken before the declaration is consumed field by field: what the
        // report rebuilds is the skeleton this template declared plus what the
        // run added to it.
        let declared_document = template.document.clone();
        let mut parameters = self.resolved(&suite_id, &template)?;
        for (key, value) in self.settings.clone() {
            if parameters.insert(key.clone(), value).is_some() {
                return Err(HarnessError::Parameters(format!(
                    "plugin setting {key:?} collides with a case parameter"
                )));
            }
        }
        let record_inputs = template
            .subscriptions
            .iter()
            .filter_map(crate::definition::Subscription::record)
            .cloned()
            .collect::<Vec<_>>();
        let execution_id = crate::model::identity::new_id::<TestExecutionId>().to_string();
        let start = RunExecution {
            mode: "automatic".into(),
            execution_token: execution_id.clone(),
            test_cycle_id: crate::model::identity::new_id::<TestCycleId>().to_string(),
            test_execution_id: execution_id.clone(),
            workflow,
            template_identifier: self.template.clone(),
            parameters_json: serde_json::to_string(&parameters)
                .expect("resolved parameter values serialize"),
            secrets_json: serde_json::to_string(&self.secrets).expect("secret bindings serialize"),
        };
        let capture = dokime_domain::model::execution::RecordScope {
            test_cycle_id: dokime_domain::model::identity::TestCycleId::from_uuid(start.test_cycle_id.clone()).ok(),
            test_execution_id: dokime_domain::model::identity::TestExecutionId::from_uuid(
                start.test_execution_id.clone(),
            )
            .ok(),
            origin: dokime_domain::model::execution::EvidenceOrigin::Initialization,
            ..Default::default()
        };
        let host = std::sync::Arc::new(Host {
            pages: std::sync::Mutex::new(Pages {
                subscribed_records: self
                    .published
                    .iter()
                    .filter(|record| record_inputs.contains(&record.record_type))
                    .cloned()
                    .collect(),
                ..Pages::default()
            }),
            answers: self.answers,
            prompts: template.prompt_definitions,
            measured: self.measured.keys().cloned().collect(),
        });
        let caller = crate::execution::Caller::local(self.operations);
        let run = runner.run(
            start,
            std::sync::Arc::clone(&host) as std::sync::Arc<dyn ExecutionHost>,
            None,
            caller,
        );
        let deliveries = async {
            // The rows a global stream would have measured, and the records
            // another party filed, handed over once the run exists and before
            // a step can have reached anything.
            tokio::task::yield_now().await;
            for signal in deliveries(&self.measured, &self.published, &execution_id, &capture) {
                runner.signal(signal).await.map_err(HarnessError::Deliver)?;
            }
            Ok::<(), HarnessError>(())
        };
        let ran = tokio::time::timeout(self.timeout, async {
            let (outcome, delivered) = tokio::join!(run, deliveries);
            delivered?;
            outcome.map_err(HarnessError::Start)
        })
        .await
        .unwrap_or(Err(HarnessError::Timeout(self.timeout)));
        if ran.is_err() {
            runner.interrupt();
        }
        runner.shutdown().await;
        let samples = runner.take_local_samples(&execution_id).unwrap_or_default();
        let mut pages = std::mem::take(
            &mut *host
                .pages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for sample in samples {
            pages
                .samples
                .entry(sample.stream.to_string())
                .or_default()
                .push(sample.values);
        }

        ran.map(|outcome| pages.report(outcome, declared_document))
    }
}

impl Host {
    fn respond(&self, request: &PromptRequest) -> PromptResponse {
        let prompt_type = request.prompt_type.as_str();
        let declared = self
            .prompts
            .iter()
            .find(|prompt| prompt.prompt_type.as_str() == prompt_type);
        let answer = self.answers.get(prompt_type);
        let action = answer
            .map(|answer| answer.action.clone())
            .unwrap_or_else(|| {
                declared
                    .and_then(|prompt| {
                        prompt
                            .actions
                            .iter()
                            .find(|action| action.id != CANCEL)
                            .map(|action| action.id.clone())
                    })
                    .unwrap_or_else(|| CANCEL.to_owned())
            });
        let mut values = request.values.clone();
        if let Some(answer) = answer {
            values.extend(answer.values.clone());
        }

        PromptResponse {
            prompt_id: request.prompt_id.clone(),
            action,
            responder: RESPONDER.to_owned(),
            submitted_at: crate::model::time::now_utc(),
            values,
            accepted: true,
        }
    }
}

impl Harness {
    fn declaration(
        &self,
        suites: &[crate::domains::TestSuiteDefinition],
    ) -> Result<(String, String, TestCaseTemplate), HarnessError> {
        let mut found: Vec<(String, String, TestCaseTemplate)> = Vec::new();
        for suite in suites.iter().cloned() {
            let workflow = suite
                .workflow
                .clone()
                .unwrap_or_else(|| suite.suite_id.clone());
            for template in suite.suite.test_case_templates {
                if template.metadata.identifier.as_str() == self.template {
                    found.push((suite.suite_id.clone(), workflow.clone(), template));
                }
            }
        }
        match found.len() {
            0 => Err(HarnessError::UnknownTemplate(self.template.clone())),
            1 => Ok(found.remove(0)),
            _ => Err(HarnessError::AmbiguousTemplate {
                template: self.template.clone(),
                workflows: found.into_iter().map(|(_, workflow, _)| workflow).collect(),
            }),
        }
    }

    fn resolved(
        &self,
        suite_id: &str,
        template: &TestCaseTemplate,
    ) -> Result<ParameterValues, HarnessError> {
        let declared = template
            .parameters
            .iter()
            .map(|parameter| parameter_name(parameter).to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        let selection = CaseMaterializationSelection {
            case_overrides: self
                .parameters
                .iter()
                .filter(|(name, _)| declared.contains(name.as_str()))
                .map(|(name, value)| (name.clone(), vec![value.clone()]))
                .collect(),
            ..CaseMaterializationSelection::default()
        };
        let generated = materialize_template_cases(
            suite_id,
            template,
            selection,
            CaseMaterializationOptions {
                max_cases: 1,
                force: true,
                registration_id: crate::model::identity::new_id(),
                first_label_number: 1,
            },
        )
        .map_err(|error| HarnessError::Parameters(error.to_string()))?;
        let mut parameters = generated
            .test_cases
            .into_iter()
            .next()
            .map(|case| case.parameters)
            .unwrap_or_default();
        parameters.extend(self.parameters.clone());

        Ok(parameters)
    }
}

impl Host {
    fn lease(&self, ask: &LeaseAsk) -> LeaseResolution {
        let Ok(request) = serde_json::from_str::<crate::telemetry::LeaseRequest>(&ask.lease_json)
        else {
            return LeaseResolution {
                granted: false,
                message: "the lease request could not be read".into(),
            };
        };
        let granted = self.measured.contains(request.stream.as_str());
        LeaseResolution {
            granted,
            message: match granted {
                true => String::new(),
                false => format!(
                    "the harness has no host to start an instance of {}; stage what it \
                     measures with `Harness::measuring`",
                    request.stream
                ),
            },
        }
    }
}

impl ExecutionHost for Host {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>> {
        let admitted = batch
            .step_begins
            .iter()
            .filter_map(|encoded| {
                serde_json::from_str::<dokime_domain::model::execution::result::StepAttemptStart>(encoded).ok()
            })
            .map(|start| StepAdmission {
                attempt_id: start.attempt_id.to_string(),
                accepted: true,
                message: String::new(),
            })
            .collect();
        let settled = batch.record_subscriptions_settling;
        self.pages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take(batch);
        Box::pin(std::future::ready(Ok(EvidenceReceipt {
            admitted,
            settled,
        })))
    }

    fn ask(
        &self,
        question: OperatorQuestion,
    ) -> TransportFuture<'_, Result<OperatorAnswer, String>> {
        let answered = serde_json::from_str::<PromptRequest>(&question.prompt_json)
            .map_err(|error| error.to_string())
            .map(|request| OperatorAnswer {
                response_json: serde_json::to_string(&self.respond(&request))
                    .expect("an operator answer serializes"),
            });
        Box::pin(std::future::ready(answered))
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>> {
        Box::pin(std::future::ready(Ok(Host::lease(self, &ask))))
    }
}

impl Pages {
    fn take(&mut self, reply: EvidenceBatch) {
        for encoded in reply.records {
            if let Ok(record) = serde_json::from_str::<RecordPublishRequest>(&encoded) {
                self.records.push(record);
            }
        }
        for encoded in reply.traceability {
            if let Ok(section) = serde_json::from_str::<TraceabilitySection>(&encoded) {
                self.traceability.push(section);
            }
        }
        for encoded in reply.documents {
            if let Ok(addition) = serde_json::from_str::<DocumentAddition>(&encoded) {
                self.documents.push(addition);
            }
        }
        for chunk in reply.artifact_chunks {
            self.piece(chunk);
        }
        self.log.extend(reply.diagnostics);
        if let Some(outcome) = reply.outcome {
            self.outcome = Some(outcome);
        }
    }

    fn piece(&mut self, chunk: ArtifactChunk) {
        let transfer = match self
            .transfers
            .iter_mut()
            .find(|transfer| transfer.transfer_id == chunk.transfer_id)
        {
            Some(transfer) => transfer,
            None => {
                self.transfers.push(Transfer {
                    transfer_id: chunk.transfer_id.clone(),
                    next: 0,
                    artifact: Artifact {
                        name: chunk.name.clone(),
                        media_type: chunk.mime.clone(),
                        bytes: Vec::new(),
                        abandoned: false,
                    },
                });
                self.transfers.last_mut().expect("the transfer was pushed")
            }
        };
        if chunk.sequence != transfer.next {
            return;
        }
        transfer.next += 1;
        if chunk.replace {
            // The watched file was rewritten, so the artifact is what it says
            // now. A test reads the same content the host would have stored.
            transfer.artifact.bytes.clear();
        }
        transfer.artifact.bytes.extend(chunk.bytes);
        transfer.artifact.abandoned |= chunk.aborted;
    }

    fn report(
        self,
        outcome: SidecarExecutionOutcome,
        declared: Option<DocumentDefinition>,
    ) -> Report {
        let mut document =
            declared.unwrap_or_else(|| DocumentDefinition::new("default", "Document"));
        for addition in self.documents {
            let _ = document.add(addition);
        }

        Report {
            outcome,
            document,
            samples: self.samples,
            records: self.records,
            subscribed_records: self.subscribed_records,
            traceability: self.traceability,
            artifacts: self
                .transfers
                .into_iter()
                .map(|transfer| transfer.artifact)
                .collect(),
            log: self.log,
        }
    }
}

impl Report {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.outcome.passed
    }

    #[must_use]
    pub fn criteria(&self) -> &[SidecarCriterionOutcome] {
        &self.outcome.criteria
    }

    #[must_use]
    pub fn samples<C: TelemetryStream>(&self) -> Vec<C::Row> {
        self.samples
            .get(C::ID)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|values| {
                <C::Row as TelemetryRow>::from_row(values).unwrap_or_else(|error| {
                    panic!(
                        "the run published a row on {:?} the declaration cannot read: {error}",
                        C::ID
                    )
                })
            })
            .collect()
    }

    pub fn streams(&self) -> impl Iterator<Item = &str> {
        self.samples.keys().map(String::as_str)
    }

    #[must_use]
    pub fn records<R: DeclaredRecord>(&self) -> Vec<R> {
        self.records
            .iter()
            .filter(|record| record.record_type.as_str() == R::RECORD_TYPE)
            .map(|record| &record.details)
            .chain(
                self.subscribed_records
                    .iter()
                    .filter(|record| record.record_type.as_str() == R::RECORD_TYPE)
                    .map(|record| &record.details),
            )
            .map(|details| {
                R::from_details(details).unwrap_or_else(|error| {
                    panic!(
                        "the run published a {:?} the declaration cannot read: {error}",
                        R::RECORD_TYPE
                    )
                })
            })
            .collect()
    }

    pub fn record_types(&self) -> impl Iterator<Item = &str> {
        self.records
            .iter()
            .map(|record| record.record_type.as_str())
    }

    #[must_use]
    pub fn traceability(&self, section_id: &str) -> Section<'_> {
        Section {
            identifier: section_id.to_owned(),
            section: self
                .traceability
                .iter()
                .find(|section| section.identifier.as_str() == section_id),
        }
    }

    pub fn document(&self) -> &DocumentDefinition {
        &self.document
    }

    #[must_use]
    pub fn sections(&self) -> &[TraceabilitySection] {
        &self.traceability
    }

    #[must_use]
    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }

    #[must_use]
    pub fn artifact(&self, name: &str) -> &Artifact {
        self.artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "the run handed over no artifact named {name:?}; it produced {:?}",
                    self.artifacts
                        .iter()
                        .map(|artifact| artifact.name.as_str())
                        .collect::<Vec<_>>()
                )
            })
    }

    #[must_use]
    pub fn log(&self) -> &[log::Entry] {
        &self.log
    }

    #[must_use]
    pub fn diagnostics(&self) -> String {
        let mut rendered = String::new();
        if let Some(message) = &self.outcome.message {
            rendered.push_str(message);
            rendered.push('\n');
        }
        if let Some(detail) = self
            .outcome
            .detail_json
            .as_deref()
            .and_then(|encoded| serde_json::from_str::<Diagnostic>(encoded).ok())
        {
            rendered.push_str(&detail.to_string());
        }
        for criterion in &self.outcome.criteria {
            rendered.push_str(&format!(
                "criterion {}: {:?}{}\n",
                criterion.identifier,
                crate::transport::SidecarCriterionStatus::try_from(criterion.status)
                    .unwrap_or(crate::transport::SidecarCriterionStatus::Unspecified),
                criterion
                    .message
                    .as_deref()
                    .map(|message| format!(" — {message}"))
                    .unwrap_or_default()
            ));
        }
        for entry in &self.log {
            rendered.push_str(&format!("  {}\n", entry.message));
        }

        rendered
    }
}

impl<'a> Section<'a> {
    #[must_use]
    pub fn label(&self) -> &'a str {
        &self.found().label
    }

    #[must_use]
    pub fn rows(&self) -> &'a [TraceabilityRow] {
        &self.found().rows
    }

    #[must_use]
    pub fn row(&self, row_id: &str) -> Row<'a> {
        Row {
            section: self.identifier.clone(),
            identifier: row_id.to_owned(),
            row: self
                .found()
                .rows
                .iter()
                .find(|row| row.identifier.as_str() == row_id),
        }
    }

    fn found(&self) -> &'a TraceabilitySection {
        self.section.unwrap_or_else(|| {
            panic!(
                "the template stated no traceability section {:?}",
                self.identifier
            )
        })
    }
}

impl<'a> Row<'a> {
    #[must_use]
    pub fn value(&self) -> &'a Value {
        &self.found().value
    }

    #[must_use]
    pub fn unit(&self) -> Option<&'a str> {
        self.found().unit.as_deref()
    }

    #[must_use]
    pub fn text(&self) -> &'a str {
        match self.value() {
            Value::Text(text) => text,
            other => self.mistyped("text", other),
        }
    }

    #[must_use]
    pub fn number(&self) -> f64 {
        match self.value() {
            Value::Number(number) => *number,
            Value::Integer(integer) => *integer as f64,
            other => self.mistyped("a number", other),
        }
    }

    #[must_use]
    pub fn integer(&self) -> i64 {
        match self.value() {
            Value::Integer(integer) => *integer,
            other => self.mistyped("an integer", other),
        }
    }

    #[must_use]
    pub fn boolean(&self) -> bool {
        match self.value() {
            Value::Bool(boolean) => *boolean,
            other => self.mistyped("a boolean", other),
        }
    }

    fn found(&self) -> &'a TraceabilityRow {
        self.row.unwrap_or_else(|| {
            panic!(
                "traceability section {:?} states no row {:?}",
                self.section, self.identifier
            )
        })
    }

    fn mistyped(&self, wanted: &str, found: &Value) -> ! {
        panic!(
            "traceability row {:?}.{:?} is not {wanted}: {found:?}",
            self.section, self.identifier
        )
    }
}

impl std::fmt::Debug for Artifact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Artifact")
            .field("name", &self.name)
            .field("media_type", &self.media_type)
            .field("bytes", &self.bytes.len())
            .field("abandoned", &self.abandoned)
            .finish()
    }
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTemplate(template) => write!(
                formatter,
                "no suite in this catalog declares the template {template:?}"
            ),
            Self::AmbiguousTemplate {
                template,
                workflows,
            } => write!(
                formatter,
                "the template {template:?} is declared by more than one workflow: {workflows:?}"
            ),
            Self::Parameters(reason) => {
                write!(formatter, "the case could not be resolved: {reason}")
            }
            Self::Start(refusal) => write!(formatter, "the execution was refused: {refusal}"),
            Self::Deliver(refusal) => {
                write!(formatter, "the execution refused a delivery: {refusal}")
            }
            Self::Timeout(within) => write!(formatter, "the run did not finish within {within:?}"),
            Self::Task(error) => write!(formatter, "the run's own task did not settle: {error}"),
        }
    }
}

impl std::error::Error for HarnessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Start(source) | Self::Deliver(source) => Some(source),
            Self::Task(source) => Some(source),
            _ => None,
        }
    }
}

/* === Public Functions === */
fn deliveries(
    measured: &BTreeMap<String, Vec<BTreeMap<String, Value>>>,
    published: &[Record],
    execution_id: &str,
    scope: &dokime_domain::model::execution::RecordScope,
) -> Vec<ExecutionSignal> {
    let mut signals = Vec::new();
    for (stream, rows) in measured {
        let Ok(identifier) = dokime_domain::model::identity::Identifier::new(stream) else {
            continue;
        };
        let samples = rows
            .iter()
            .enumerate()
            .filter_map(|(sequence, values)| {
                let mut sample = TelemetrySample::new(
                    crate::model::identity::new_id(),
                    identifier.clone(),
                    sequence as u64,
                    crate::model::time::now_utc(),
                    HOST_SOURCE,
                );
                sample.values = values.clone();
                sample.scope = Some(scope.clone());
                serde_json::to_string(&sample).ok()
            })
            .collect();
        signals.push(ExecutionSignal {
            execution_token: execution_id.to_owned(),
            samples: Some(DeliverSamplesRequest {
                samples,
                gaps: Vec::new(),
                scope_json: serde_json::to_string(scope).expect("fixture scope serializes"),
            }),
            ..ExecutionSignal::default()
        });
    }
    for delivered in published {
        signals.push(ExecutionSignal {
            execution_token: execution_id.to_owned(),
            record: Some(DeliverRecordRequest {
                record_json: serde_json::to_string(delivered)
                    .expect("a host-filed record serializes"),
                captured_scopes: vec![
                    serde_json::to_string(scope).expect("capture scope serializes"),
                ],
            }),
            ..ExecutionSignal::default()
        });
    }
    signals
}

/* === Private Functions === */

/* === Tests === */
