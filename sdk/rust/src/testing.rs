/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-Process Template Harness

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    any::Any,
    collections::HashMap,
    fmt, mem, process,
    sync::{Arc, Mutex, Weak},
};

// Third Party Imports
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage};
use prost_types::{self, Timestamp};

// Local Imports
use crate::context::{Execution, Local, LocalHost};
use crate::operation::FutureResult;
use crate::plugin::Window;
use crate::stream::Sampler;
use crate::template::Parts;
use crate::wire::host::evidence::{
    CapturedRecord, LogsAppendRequest, LogsAppendResponse, RecordsAppendRequest, RecordsAppendResponse, RecordsListRequest,
    RecordsListResponse,
};
use crate::wire::host::executions::{ProgressGetRequest, ProgressGetResponse, ProgressSetRequest, ProgressSetResponse};
use crate::wire::host::{
    BackendEnvironmentRequest, BackendEnvironmentResponse, EvidenceStatisticsRequest, EvidenceStatisticsResponse, PromptsCreateRequest,
    PromptsCreateResponse, Statistics,
};
use crate::{Classification, Context, Error, ExecutionProgressValue, Origin, PluginRun, Result, Severity, Termination};
use crate::{Value, origin, plugin::Plugin, value};
use harmos_protocol::host::{TimeGetRequest, TimeGetResponse, TimeSleepRequest, TimeSleepResponse};

/* === Constants === */

const EXECUTION: &str = "testing";
const RESPONDER: &str = "dokime.testing";
const CANCEL: &str = "cancel";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Run One Template In-Process Against an In-Memory Host With a Controlled Clock.
pub struct Harness {
    selected: Result<Selected>,
    parameters: HashMap<String, Value>,
    answers: HashMap<String, Answer>,
    origin_ns: i64,
}

/// What One Harness Run Captured; Nothing Outlives the Report.
pub struct Report {
    termination: Termination,
    failure: Option<Error>,
    criteria: Vec<Evaluated>,
    records: Vec<Recorded>,
    samples: Vec<Sampled>,
    logs: Vec<Logged>,
    prompts: Vec<Prompted>,
    progress: Option<ExecutionProgressValue>,
    now_ns: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Evaluated {
    pub criterion: String,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Logged {
    pub step: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prompted {
    pub step: String,
    pub prompt_type: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample<T> {
    pub step: String,
    pub at: Timestamp,
    pub value: T,
}

pub(crate) struct Selected {
    pub plugin: String,
    pub state: Arc<dyn Any + Send + Sync>,
    pub template: Parts,
    pub prompts: Vec<crate::wire::model::Prompt>,
    pub requirements: Vec<&'static str>,
    pub samplers: Vec<Sampler>,
}

#[derive(Clone)]
struct Answer {
    action: String,
    values: HashMap<String, Value>,
}

#[derive(Clone)]
struct Recorded {
    step: String,
    kind: String,
    payload: Vec<u8>,
    at_ns: i64,
}

#[derive(Clone)]
struct Sampled {
    stream: String,
    step: String,
    payload: Vec<u8>,
    at_ns: i64,
}

struct Memory {
    plugin_state: Arc<dyn Any + Send + Sync>,
    me: Weak<Memory>,
    execution: Arc<Execution>,
    requirements: Vec<&'static str>,
    prompts: Vec<crate::wire::model::Prompt>,
    answers: HashMap<String, Answer>,
    observers: Vec<(Sampler, i64)>,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    now_ns: i64,
    step: String,
    due_ns: Vec<i64>,
    sequences: Vec<u64>,
    records: Vec<Recorded>,
    samples: Vec<Sampled>,
    logs: Vec<Logged>,
    prompts: Vec<Prompted>,
    progress: Option<ExecutionProgressValue>,
    revision: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Harness {
    pub fn new(plugin: Plugin, template: &str) -> Self {
        Self {
            selected: plugin.into_testing(template),
            parameters: HashMap::new(),
            answers: HashMap::new(),
            origin_ns: 0,
        }
    }
    /// Override Declared Defaults With Every Field of a Typed Parameters Value.
    pub fn parameters<T: crate::Message>(mut self, values: T) -> Self {
        match crate::data::parameters(values) {
            Ok(parameters) => {
                for parameter in parameters {
                    let name = parameter.metadata.map(|metadata| metadata.identifier).unwrap_or_default();
                    if let Some(value) = literal(parameter.expression) {
                        self.parameters.insert(name, value);
                    }
                }
            }
            Err(error) => self.selected = Err(error),
        }
        self
    }
    pub fn parameter(mut self, name: impl Into<String>, value: Value) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }
    /// Answer a Prompt Type; Unanswered Declared Prompts Take Their First Non-Cancel Action.
    pub fn answer(self, prompt_type: impl Into<String>, action: impl Into<String>) -> Self {
        self.answer_with(prompt_type, action, HashMap::new())
    }
    pub fn answer_with(mut self, prompt_type: impl Into<String>, action: impl Into<String>, values: HashMap<String, Value>) -> Self {
        self.answers.insert(prompt_type.into(), Answer { action: action.into(), values });
        self
    }
    pub fn starting_at(mut self, at: Timestamp) -> Self {
        self.origin_ns = at.seconds.saturating_mul(1_000_000_000).saturating_add(i64::from(at.nanos));
        self
    }

    /// Materialize, Initialize, Run Each Step and Criterion, Then Clean Up, Like the Host.
    pub async fn run(self) -> Result<Report> {
        let Selected {
            plugin,
            state: plugin_state,
            template,
            prompts,
            requirements,
            samplers,
        } = self.selected?;
        let Parts {
            definition,
            case,
            initialize,
            cleanup,
            steps,
            criteria,
            materialize,
            registrations: _,
            samplers: owned,
        } = template;
        let identifier = definition.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();

        // Resolve Parameters and Materialize the Frozen Case.
        let mut resolved = HashMap::new();
        for parameter in &definition.parameters {
            let name = parameter.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
            let value = match self.parameters.get(&name) {
                Some(value) => value.clone(),
                None => literal(parameter.expression.clone())
                    .ok_or_else(|| Error::Invalid(format!("Test Template: Supply Parameter {name} With Harness::parameter")))?,
            };
            resolved.insert(name, value);
        }
        if let Some(unknown) = self.parameters.keys().find(|name| !resolved.contains_key(*name)) {
            return Err(Error::Invalid(format!("Test Template: Remove Undeclared Parameter {unknown}")));
        }
        let mut case = case;
        case.parameters = resolved.clone();
        if let Some(materialize) = &materialize {
            case = materialize(case, &resolved)?;
        }

        // Observers of the Case's Streams Sample From Their First Due Interval.
        let mut observers = Vec::new();
        for sampler in owned.into_iter().chain(samplers) {
            if !case.streams.contains(&sampler.stream) || observers.iter().any(|(known, _): &(Sampler, i64)| known.stream == sampler.stream)
            {
                continue;
            }
            let Some(interval) = sampler.interval else {
                return Err(Error::Invalid(format!("Test Template: Give Observer {} a Fixed Interval With every", sampler.stream)));
            };
            let interval_ns = i64::try_from(interval.as_nanos()).unwrap_or(i64::MAX).max(1);
            observers.push((sampler, interval_ns));
        }
        let origin = Origin {
            kind: Some(origin::Kind::PluginRun(PluginRun {
                test_case: Some(case.clone()),
                plugin,
                template: identifier,
                ..Default::default()
            })),
        };
        let execution = Execution::new(case.clone(), origin);
        let due_ns = observers.iter().map(|(_, interval_ns)| self.origin_ns.saturating_add(*interval_ns)).collect();
        let sequences = vec![0; observers.len()];
        let host = Arc::new_cyclic(|me| Memory {
            plugin_state,
            me: me.clone(),
            execution: execution.clone(),
            requirements,
            prompts,
            answers: self.answers,
            observers,
            state: Mutex::new(State {
                now_ns: self.origin_ns,
                sequences,
                due_ns,
                ..State::default()
            }),
        });

        // Initialize, Then Each Frozen Step and Criterion in Order.
        let mut failure = None;
        let mut evaluated = Vec::new();
        let initialized = match &initialize {
            Some(initialize) => initialize.execute(&host.context("", 1)).await,
            None => Ok(()),
        };
        let initialized = match initialized {
            Ok(()) => true,
            Err(error) => {
                failure = Some(error);
                false
            }
        };
        if initialized {
            for frozen in &case.steps {
                let Some((_, step)) = steps.iter().find(|(operation, _)| *operation == frozen.operation) else {
                    failure = Some(Error::Invalid(format!("Test Template: Materialize Only Declared Steps, Not {}", frozen.operation)));
                    break;
                };
                let name = frozen.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
                let attempts =
                    step.policy.retries.as_ref().map_or(1, |retries| u32::try_from(retries.total_attempts.max(1)).unwrap_or(u32::MAX));
                let delay = step.policy.retries.as_ref().and_then(|retries| retries.initial).map_or(0, duration_ns);
                let mut attempt = 1;
                let result = loop {
                    match step.handler.execute(&host.context(&name, attempt)).await {
                        Err(_) if step.replay_safe && attempt < attempts => {
                            host.advance(delay).await?;
                            attempt += 1;
                        }
                        result => break result,
                    }
                };
                if let Err(error) = result {
                    failure = Some(error);
                    break;
                }
            }
        }
        if failure.is_none() {
            for frozen in &case.criteria {
                let Some((_, criterion)) = criteria.iter().find(|(operation, _)| *operation == frozen.operation) else {
                    failure = Some(Error::Invalid(format!("Test Template: Materialize Only Declared Criteria, Not {}", frozen.operation)));
                    break;
                };
                let name = frozen.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
                match criterion.handler.execute(&host.context("", 1)).await {
                    Ok(passed) => evaluated.push(Evaluated { criterion: name, passed }),
                    Err(error) => {
                        failure = Some(error);
                        break;
                    }
                }
            }
        }
        let mut termination = if failure.is_none() { Termination::Completed } else { Termination::Failed };
        // Cleanup Runs Once Initialization Succeeded, With the Execution's Termination.
        if initialized && let Some(cleanup) = &cleanup {
            let context = host.context("", 1).with_termination(termination as i32)?;
            if let Err(error) = cleanup.execute(&context).await {
                termination = Termination::Interrupted;
                failure.get_or_insert(error);
            }
        }
        execution.clear()?;
        let state = mem::take(&mut *host.state.lock().map_err(|_| poisoned())?);
        Ok(Report {
            termination,
            failure,
            criteria: evaluated,
            records: state.records,
            samples: state.samples,
            logs: state.logs,
            prompts: state.prompts,
            progress: state.progress,
            now_ns: state.now_ns,
        })
    }
}

impl Report {
    /// Completed With Every Evaluated Criterion Passing; No Criteria Means Not Evaluated.
    pub fn passed(&self) -> bool {
        self.termination == Termination::Completed && !self.criteria.is_empty() && self.criteria.iter().all(|criterion| criterion.passed)
    }
    pub fn termination(&self) -> Termination {
        self.termination
    }
    pub fn failure(&self) -> Option<&Error> {
        self.failure.as_ref()
    }
    pub fn criteria(&self) -> &[Evaluated] {
        &self.criteria
    }
    pub fn records<T: crate::Payload>(&self, record_type: &str) -> Result<Vec<T>> {
        self.records.iter().filter(|record| record.kind == record_type).map(|record| crate::payload::decode(&record.payload)).collect()
    }
    pub fn samples<T: crate::Message>(&self, stream: &str) -> Result<Vec<Sample<T>>> {
        self.samples
            .iter()
            .filter(|sample| sample.stream == stream)
            .map(|sample| {
                Ok(Sample {
                    step: sample.step.clone(),
                    at: timestamp(sample.at_ns),
                    value: T::decode(sample.payload.as_slice())
                        .map_err(|error| Error::Invalid(format!("Decode Sample {stream}: {error}")))?,
                })
            })
            .collect()
    }
    pub fn logs(&self) -> &[Logged] {
        &self.logs
    }
    pub fn prompts(&self) -> &[Prompted] {
        &self.prompts
    }
    pub fn progress(&self) -> Option<&ExecutionProgressValue> {
        self.progress.as_ref()
    }
    pub fn now(&self) -> Timestamp {
        timestamp(self.now_ns)
    }
}

impl fmt::Debug for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Report")
            .field("termination", &self.termination)
            .field("failure", &self.failure)
            .field("criteria", &self.criteria)
            .field("records", &self.records.len())
            .field("samples", &self.samples.len())
            .field("logs", &self.logs)
            .field("prompts", &self.prompts)
            .field("progress", &self.progress)
            .finish()
    }
}

impl Memory {
    fn context(&self, step: &str, attempt: u32) -> Context {
        if let Ok(mut state) = self.state.lock() {
            state.step = step.into();
        }
        let host: Arc<dyn LocalHost> = self.me.upgrade().expect("Testing Host Outlives Its Contexts");
        Context::local(Local { host, scope: EXECUTION.into(), attempt }, self.execution.clone(), self.plugin_state.clone())
    }

    // Advance Virtual Time; Before the Wait Settles, Each Observer Answers One Window of Every Sample Due by Its End.
    async fn advance(&self, duration_ns: i64) -> Result<()> {
        let (step, grants) = {
            let mut state = self.state.lock().map_err(|_| poisoned())?;
            let target = state.now_ns.saturating_add(duration_ns.max(0));
            state.now_ns = target;
            let mut grants = Vec::new();
            for (index, (_, period)) in self.observers.iter().enumerate() {
                let first = state.due_ns[index];
                if first > target {
                    continue;
                }
                let count = (target - first) / period + 1;
                state.due_ns[index] = first.saturating_add(count.saturating_mul(*period));
                grants.push((index, first, count as u64, state.sequences[index] + 1));
                state.sequences[index] += count as u64;
            }
            (state.step.clone(), grants)
        };
        for (index, first, count, sequence) in grants {
            let (sampler, period) = &self.observers[index];
            let host: Arc<dyn LocalHost> = self.me.upgrade().ok_or_else(poisoned)?;
            let context = Context::local(
                Local {
                    host,
                    scope: EXECUTION.into(),
                    attempt: 1,
                },
                self.execution.clone(),
                self.plugin_state.clone(),
            );
            let window = Window {
                start: timestamp(first),
                period: prost_types::Duration {
                    seconds: period / 1_000_000_000,
                    nanos: (period % 1_000_000_000) as i32,
                },
                count,
                sequence,
            };
            let rows = sampler.sample.rows(&context, &window).await?;
            let mut state = self.state.lock().map_err(|_| poisoned())?;
            for (offset, payload) in rows.into_iter().enumerate() {
                state.samples.push(Sampled {
                    stream: sampler.stream.clone(),
                    step: step.clone(),
                    payload,
                    at_ns: first.saturating_add((offset as i64).saturating_mul(*period)),
                });
            }
        }
        Ok(())
    }

    async fn dispatch(&self, operation: &'static str, request: Vec<u8>) -> Result<Vec<u8>> {
        if !self.requirements.contains(&operation) {
            return Err(Error::Invalid(format!("Undeclared Requirement: {operation}")));
        }
        match operation {
            "harmos.host.v1.Time/Get" => {
                TimeGetRequest::decode(request.as_slice()).map_err(decode)?;
                let now = self.state.lock().map_err(|_| poisoned())?.now_ns;
                Ok(TimeGetResponse { now: Some(timestamp(now)) }.encode_to_vec())
            }
            "harmos.host.v1.Time/Sleep" => {
                let request = TimeSleepRequest::decode(request.as_slice()).map_err(decode)?;
                self.advance(request.duration.map_or(0, duration_ns)).await?;
                Ok(TimeSleepResponse {}.encode_to_vec())
            }
            "dokime.host.v1.Backend/Environment" => {
                BackendEnvironmentRequest::decode(request.as_slice()).map_err(decode)?;
                let in_step = !self.state.lock().map_err(|_| poisoned())?.step.is_empty();
                Ok(BackendEnvironmentResponse {
                    backend_pid: process::id(),
                    in_step,
                    resolved_at: None,
                }
                .encode_to_vec())
            }
            "dokime.host.v1.evidence.Records/Append" => {
                let request = RecordsAppendRequest::decode(request.as_slice()).map_err(decode)?;
                scoped(&request.execution)?;
                let mut state = self.state.lock().map_err(|_| poisoned())?;
                let at_ns = request.source_at.as_ref().map_or(state.now_ns, timestamp_ns);
                let step = state.step.clone();
                state.records.push(Recorded {
                    step,
                    kind: request.kind,
                    payload: request.payload,
                    at_ns,
                });
                Ok(RecordsAppendResponse {
                    ordinal: state.records.len() as u64,
                    receipt: None,
                }
                .encode_to_vec())
            }
            "dokime.host.v1.evidence.Records/List" => {
                let request = RecordsListRequest::decode(request.as_slice()).map_err(decode)?;
                let state = self.state.lock().map_err(|_| poisoned())?;
                let matching = state
                    .records
                    .iter()
                    .enumerate()
                    .filter(|(_, record)| record.kind == request.record_type && (request.step.is_empty() || record.step == request.step))
                    .map(|(index, record)| CapturedRecord {
                        payload: record.payload.clone(),
                        record_id: (index + 1).to_string(),
                        sequence: index as u64 + 1,
                        timestamp: Some(timestamp(record.at_ns)),
                        record_type: record.kind.clone(),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();
                let limit = request.page.as_ref().map_or(0, |page| page.page_size as usize);
                let limit = if limit == 0 { matching.len() } else { limit };
                Ok(RecordsListResponse {
                    count: matching.len() as u64,
                    last: matching.last().cloned(),
                    records: matching.into_iter().take(limit).collect(),
                    ..Default::default()
                }
                .encode_to_vec())
            }
            "dokime.host.v1.evidence.Logs/Append" => {
                let request = LogsAppendRequest::decode(request.as_slice()).map_err(decode)?;
                scoped(&request.execution)?;
                let mut state = self.state.lock().map_err(|_| poisoned())?;
                let step = state.step.clone();
                state.logs.push(Logged {
                    step,
                    severity: Severity::try_from(request.severity).unwrap_or_default(),
                    message: request.message,
                });
                Ok(LogsAppendResponse { receipt: None }.encode_to_vec())
            }
            "dokime.host.v1.Prompts/Create" => {
                let request = PromptsCreateRequest::decode(request.as_slice()).map_err(decode)?;
                let answer = if let Some(definition) = &request.definition {
                    let identity = definition.metadata.as_ref().map(|metadata| metadata.identifier.as_str()).unwrap_or_default();
                    self.answers
                        .get(identity)
                        .cloned()
                        .or_else(|| {
                            definition.actions.first().map(|action| Answer {
                                action: action.identifier.clone(),
                                values: HashMap::new(),
                            })
                        })
                        .ok_or_else(|| Error::Invalid("Answer Prompt: Declare a Choice or Supply an Answer".into()))?
                } else {
                    self.respond(&request.prompt_type)?
                };
                let mut state = self.state.lock().map_err(|_| poisoned())?;
                let step = state.step.clone();
                state.prompts.push(Prompted {
                    step,
                    prompt_type: request.prompt_type.clone(),
                    action: answer.action.clone(),
                });
                let mut values = request.values;
                values.extend(answer.values);
                Ok(PromptsCreateResponse {
                    prompt: request.prompt,
                    action: answer.action,
                    values,
                    responder: RESPONDER.into(),
                    submitted_at: Some(timestamp(state.now_ns)),
                    accepted: true,
                }
                .encode_to_vec())
            }
            "dokime.host.v1.executions.Progress/Get" => {
                ProgressGetRequest::decode(request.as_slice()).map_err(decode)?;
                let state = self.state.lock().map_err(|_| poisoned())?;
                Ok(ProgressGetResponse {
                    value: state.progress.clone(),
                    revision: state.revision.to_be_bytes().to_vec(),
                    resolved_at: None,
                }
                .encode_to_vec())
            }
            "dokime.host.v1.executions.Progress/Set" => {
                let request = ProgressSetRequest::decode(request.as_slice()).map_err(decode)?;
                let mut state = self.state.lock().map_err(|_| poisoned())?;
                if request.expected_revision != state.revision.to_be_bytes() {
                    return Err(Error::failure(
                        Classification::Refused,
                        "revision_conflict",
                        "Set Progress: The Revision Changed",
                        "Read the Current Progress and Retry",
                    ));
                }
                state.revision += 1;
                state.progress = request.value;
                Ok(ProgressSetResponse {
                    value: state.progress.clone(),
                    revision: state.revision.to_be_bytes().to_vec(),
                    receipt: None,
                }
                .encode_to_vec())
            }
            "dokime.host.v1.Evidence/Statistics" => {
                let request = EvidenceStatisticsRequest::decode(request.as_slice()).map_err(decode)?;
                let statistics = self.statistics(&request.stream, &request.field, &request.step)?;
                Ok(EvidenceStatisticsResponse {
                    statistics: Some(statistics),
                    resolved_at: None,
                }
                .encode_to_vec())
            }
            _ => Err(Error::failure(
                Classification::Refused,
                "testing_host_unsupported",
                format!("Testing Host: {operation} Is Not Simulated"),
                "Exercise This Operation Against a Real Host",
            )),
        }
    }

    fn respond(&self, prompt_type: &str) -> Result<Answer> {
        if let Some(answer) = self.answers.get(prompt_type) {
            return Ok(answer.clone());
        }
        self.prompts
            .iter()
            .filter(|prompt| prompt.metadata.as_ref().is_some_and(|metadata| metadata.identifier == prompt_type))
            .flat_map(|prompt| prompt.actions.iter())
            .find(|action| action.identifier != CANCEL)
            .map(|action| Answer {
                action: action.identifier.clone(),
                values: HashMap::new(),
            })
            .ok_or_else(|| Error::Invalid(format!("Testing Host: Answer Prompt {prompt_type} With Harness::answer")))
    }

    // Count, Extremes, Mean, Sum, First and Last of One Numeric Field; Exact Ranks Need a Real Host.
    fn statistics(&self, stream: &str, field: &str, step: &str) -> Result<Statistics> {
        let state = self.state.lock().map_err(|_| poisoned())?;
        let samples =
            state.samples.iter().filter(|sample| sample.stream == stream && (step.is_empty() || sample.step == step)).collect::<Vec<_>>();
        let mut statistics = Statistics {
            count: samples.len() as u64,
            ..Default::default()
        };
        if field.is_empty() || samples.is_empty() {
            return Ok(statistics);
        }
        let sampler = &self
            .observers
            .iter()
            .find(|(sampler, _)| sampler.stream == stream)
            .ok_or_else(|| Error::Invalid(format!("Testing Host: Observe Stream {stream} Before Reading Its Statistics")))?
            .0;
        let pool = DescriptorPool::from_file_descriptor_set(sampler.types.clone()).map_err(|error| Error::Invalid(error.to_string()))?;
        let descriptor =
            pool.get_message_by_name(&sampler.name).ok_or_else(|| Error::Invalid(format!("Missing Descriptor {}", sampler.name)))?;
        let member = descriptor.get_field_by_name(field).ok_or_else(|| Error::Invalid(format!("Statistics: Unknown Field {field}")))?;
        let mut values = Vec::new();
        for sample in samples {
            let message = DynamicMessage::decode(descriptor.clone(), sample.payload.as_slice()).map_err(decode)?;
            let value = message.get_field(&member);
            let number = numeric(value.as_ref()).ok_or_else(|| Error::Invalid(format!("Statistics: Field {field} Is Not Numeric")))?;
            values.push((number, crate::data::scalar(value.as_ref())?));
        }
        let sum = values.iter().map(|(number, _)| number).sum::<f64>();
        let number = |number: f64| Value { kind: Some(value::Kind::Number(number)) };
        statistics.min = values.iter().min_by(|left, right| left.0.total_cmp(&right.0)).map(|(_, value)| value.clone());
        statistics.max = values.iter().max_by(|left, right| left.0.total_cmp(&right.0)).map(|(_, value)| value.clone());
        statistics.first = values.first().map(|(_, value)| value.clone());
        statistics.last = values.last().map(|(_, value)| value.clone());
        statistics.sum = Some(number(sum));
        statistics.mean = Some(number(sum / values.len() as f64));
        Ok(statistics)
    }
}

impl LocalHost for Memory {
    fn call<'a>(&'a self, operation: &'static str, request: Vec<u8>) -> FutureResult<'a, Vec<u8>> {
        Box::pin(self.dispatch(operation, request))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn literal(expression: Option<crate::ParameterExpression>) -> Option<Value> {
    match expression?.kind? {
        crate::parameter_expression::Kind::Literal(value) => Some(value),
        _ => None,
    }
}

fn scoped(execution: &str) -> Result<()> {
    if execution != EXECUTION {
        return Err(Error::Invalid(format!("Testing Host: Select the Calling Execution, Not {execution}")));
    }
    Ok(())
}

fn numeric(value: &prost_reflect::Value) -> Option<f64> {
    match value {
        prost_reflect::Value::F64(value) => Some(*value),
        prost_reflect::Value::F32(value) => Some(f64::from(*value)),
        prost_reflect::Value::I32(value) => Some(f64::from(*value)),
        prost_reflect::Value::I64(value) => Some(*value as f64),
        prost_reflect::Value::U32(value) => Some(f64::from(*value)),
        prost_reflect::Value::U64(value) => Some(*value as f64),
        _ => None,
    }
}

fn duration_ns(duration: prost_types::Duration) -> i64 {
    duration.seconds.saturating_mul(1_000_000_000).saturating_add(i64::from(duration.nanos))
}

fn timestamp(ns: i64) -> Timestamp {
    Timestamp {
        seconds: ns.div_euclid(1_000_000_000),
        nanos: ns.rem_euclid(1_000_000_000) as i32,
    }
}

fn timestamp_ns(at: &Timestamp) -> i64 {
    at.seconds.saturating_mul(1_000_000_000).saturating_add(i64::from(at.nanos))
}

fn decode(error: impl fmt::Display) -> Error {
    Error::Invalid(format!("Testing Host: {error}"))
}

fn poisoned() -> Error {
    Error::Invalid("Testing Host State Poisoned".into())
}

/* === Tests === */
