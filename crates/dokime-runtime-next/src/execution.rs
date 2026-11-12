/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use std::sync::Arc;

use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{
    Error, Host, Result,
    dispatch::Dispatch,
    host::{Active, lock},
    plugin::Entry,
    streams::ExecutionTime,
};
use dokime_operations_next::Resources;
use dokime_protocol::v1;
use dokime_storage::{Execution, ExecutionSnapshot, OutcomeCounts};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Cycle {
    pub identifier: String,
    pub executions: Vec<String>,
    task: JoinHandle<Result<CycleResult>>,
}
#[derive(Debug)]
pub struct CycleResult {
    pub executions: Vec<ExecutionSnapshot>,
    pub states: Vec<RunState>,
    pub counts: OutcomeCounts,
}
struct ActiveRun {
    host: Host,
    execution: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Queued,
    Initializing,
    Stepping,
    Evaluating,
    CleaningUp,
    Finished,
    Cancelled,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */

impl Cycle {
    pub async fn wait(self) -> Result<CycleResult> {
        self.task.await?
    }
}
impl Drop for ActiveRun {
    fn drop(&mut self) {
        if let Ok(mut active) = self.host.inner.active.lock() {
            active.remove(&self.execution);
        }
        self.host.inner.prompts.clear(&self.execution);
        self.host.inner.idle.notify_waiters();
        self.host.signal_change();
    }
}

/* === Public Functions === */

pub(crate) async fn start(host: Host, origins: Vec<v1::Origin>) -> Result<Cycle> {
    if origins.is_empty() {
        return Err(Error::Invalid("Materialize at Least One Test Case Before Starting a Cycle".into()));
    }
    let identifier = uuid::Uuid::now_v7().to_string();
    let mut prepared = Vec::new();
    for mut origin in origins {
        let Some(v1::origin::Kind::PluginRun(mut run)) = origin.kind.take() else {
            return Err(Error::Invalid("Start Cycles with PluginRun Origins; Import Evidence Separately".into()));
        };
        run.validate().map_err(Error::Invalid)?;
        let entry = host.entry(&run.plugin)?;
        let digest = &entry.description.catalog.as_ref().ok_or_else(|| Error::Invalid("Reconnect the Plugin Catalog".into()))?.digest;
        if &run.digest != digest || run.started_at != entry.description.started_at {
            origin = host.materialize(&run.plugin, &run.template, run.test_case.take().unwrap().parameters).await?;
        } else {
            origin.kind = Some(v1::origin::Kind::PluginRun(run));
        }
        prepared.push((uuid::Uuid::now_v7().to_string(), origin, entry));
    }
    let mut executions = Vec::new();
    for (execution, origin, entry) in &prepared {
        host.inner.store.create(&Execution {
            identifier: execution.clone(),
            cycle: identifier.clone(),
            origin: Some(origin.clone()),
            catalog: entry.description.catalog.as_ref().unwrap().encode_to_vec(),
            ..Default::default()
        })?;
        lock(&host.inner.active)?.insert(
            execution.clone(),
            Active {
                plugin: entry.description.id.clone(),
                cancellation: CancellationToken::new(),
                control: Arc::new(crate::run_control::RunControl::default()),
            },
        );
        executions.push(execution.clone());
    }
    Ok(launch(host, identifier, executions, prepared))
}

pub(crate) async fn start_existing(host: Host, execution: &str) -> Result<Cycle> {
    start_existing_mode(host, execution, v1::ExecutionMode::Automatic).await
}

pub(crate) async fn start_existing_mode(host: Host, execution: &str, mode: v1::ExecutionMode) -> Result<Cycle> {
    let cut = host.inner.store.cut(execution)?;
    if cut.state != dokime_storage::ExecutionState::Open || cut.frontier != 0 {
        return Err(Error::Invalid("Start Execution: Select Unstarted Open PluginRun Evidence; Rerun Sealed Evidence Separately".into()));
    }
    let origin = cut.execution.origin.ok_or_else(|| Error::Invalid("Start Execution: Supply a PluginRun Origin".into()))?;
    let Some(v1::origin::Kind::PluginRun(run)) = origin.kind.as_ref() else {
        return Err(Error::Invalid("Start Execution: Imports Cannot Enter the Run Queue".into()));
    };
    run.validate().map_err(Error::Invalid)?;
    let entry = host.entry(&run.plugin)?;
    if entry.description.catalog.as_ref().is_none_or(|catalog| catalog.digest != run.digest)
        || entry.description.started_at != run.started_at
    {
        return Err(Error::Invalid("Start Execution: Rematerialize the Changed Plugin Catalog into New Execution Evidence".into()));
    }
    {
        let mut active = lock(&host.inner.active)?;
        if active.contains_key(execution) {
            return Err(Error::Invalid("Start Execution: The Execution Already Has an Active Run".into()));
        }
        active.insert(
            execution.into(),
            Active {
                plugin: run.plugin.clone(),
                cancellation: CancellationToken::new(),
                control: Arc::new(if mode == v1::ExecutionMode::Interactive { crate::run_control::RunControl::interactive() } else { crate::run_control::RunControl::default() }),
            },
        );
    }
    Ok(launch(host, cut.execution.cycle, vec![execution.into()], vec![(execution.into(), origin, entry)]))
}

/* === Private Functions === */

fn launch(host: Host, identifier: String, executions: Vec<String>, prepared: Vec<(String, v1::Origin, Arc<Entry>)>) -> Cycle {
    let task = tokio::spawn(async move {
        let mut workers = JoinSet::new();
        for (index, (execution, origin, entry)) in prepared.into_iter().enumerate() {
            let host = host.clone();
            workers.spawn(async move { (index, drive(host, execution, origin, entry).await) });
        }
        let mut finished = Vec::new();
        while let Some(result) = workers.join_next().await {
            let (index, result) = result?;
            finished.push((index, result?));
        }
        finished.sort_by_key(|(index, _)| *index);
        let mut snapshots = Vec::new();
        let mut states = Vec::new();
        for (_, (snapshot, execution_states)) in finished {
            snapshots.push(snapshot);
            states.extend(execution_states);
        }
        let counts = OutcomeCounts::from_outcomes(snapshots.iter().filter_map(|snapshot| snapshot.execution.outcome.as_ref()));
        Ok(CycleResult { executions: snapshots, states, counts })
    });
    Cycle { identifier, executions, task }
}

async fn drive(host: Host, execution: String, origin: v1::Origin, entry: Arc<Entry>) -> Result<(ExecutionSnapshot, Vec<RunState>)> {
    let mut states = Vec::new();
    let _active = ActiveRun {
        host: host.clone(),
        execution: execution.clone(),
    };
    let cancellation = lock(&host.inner.active)?.get(&execution).unwrap().cancellation.clone();
    let control = lock(&host.inner.active)?.get(&execution).unwrap().control.clone();
    tracing::info!(execution = %execution, "Queued Execution");
    states.push(RunState::Queued);
    let Some(v1::origin::Kind::PluginRun(run)) = origin.kind.as_ref() else { unreachable!() };
    let case = run.test_case.as_ref().unwrap();
    let template = entry.template(&run.template)?;
    let catalog = entry.description.catalog.as_ref().unwrap();
    let time = ExecutionTime::open(
        &entry.client,
        catalog,
        &case.streams,
        &execution,
        host.inner.store.clone(),
        host.inner.settings.clock.clone(),
        &host.inner.settings.placement,
    )
    .await?;
    let resources = Resources {
        store: host.inner.store.clone(),
        execution: execution.clone(),
        step: String::new(),
        operation: String::new(),
        attempt: 0,
        clock: time.clone(),
        prompts: host.inner.prompts.clone(),
    };
    let mut dispatch = Dispatch {
        entry: entry.clone(),
        resources: Some(resources),
        cancellation,
        settings: host.inner.settings.placement.clone(),
        clock: host.inner.settings.clock.clone(),
    };
    states.push(RunState::Initializing);
    let mut initialized = template.initialize.is_none();
    let mut evaluated = Vec::new();
    let result: Result<()> = async {
        if dispatch.cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if let Some(operation) = &template.initialize {
            dispatch
                .invoke::<_, v1::InitializeResponse>(
                    operation,
                    v1::InitializeRequest {
                        execution: execution.clone(),
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await?;
            initialized = true;
        }
        states.push(RunState::Stepping);
        while let Some((index, command)) = control.next(case, &dispatch.cancellation).await? {
            let step = &case.steps[index];
            if let Some(mut command) = command.clone() {
                command.phase = v1::ExecutionCommandPhase::Running as i32;
                host.inner.store.save_command(&command)?;
            }
            host.signal_change();
            let identifier =
                step.metadata.as_ref().ok_or_else(|| Error::Invalid("Materialize Steps with Identifiers".into()))?.identifier.clone();
            dispatch.resources.as_mut().unwrap().step = identifier.clone();
            dispatch.resources.as_ref().unwrap().evidence("step:start", Vec::new(), None)?;
            let result = dispatch
                .invoke::<_, v1::StepResponse>(
                    &step.operation,
                    v1::StepRequest {
                        execution: execution.clone(),
                        step: identifier,
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await;
            dispatch.resources.as_ref().unwrap().evidence("step:end", Vec::new(), None)?;
            if let Some(mut command) = command {
                command.phase = if result.is_ok() { v1::ExecutionCommandPhase::Completed } else { v1::ExecutionCommandPhase::Failed } as i32;
                command.detail = result.as_ref().err().map(ToString::to_string);
                host.inner.store.save_command(&command)?;
            }
            control.complete_step(index)?;
            host.signal_change();
            result?;
        }
        dispatch.resources.as_mut().unwrap().step.clear();
        states.push(RunState::Evaluating);
        for criterion in &case.criteria {
            let identifier = criterion
                .metadata
                .as_ref()
                .ok_or_else(|| Error::Invalid("Materialize Criteria with Identifiers".into()))?
                .identifier
                .clone();
            let response: v1::CriterionResponse = dispatch
                .invoke(
                    &criterion.operation,
                    v1::CriterionRequest {
                        execution: execution.clone(),
                        criterion: identifier.clone(),
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await?;
            dispatch.resources.as_ref().unwrap().evidence(&format!("criterion:{identifier}"), response.encode_to_vec(), None)?;
            evaluated.push(response.passed);
        }
        Ok(())
    }
    .await;
    if let Some(mut command) = control.take_pending()? {
        command.phase = if result.is_ok() { v1::ExecutionCommandPhase::Completed } else { v1::ExecutionCommandPhase::Interrupted } as i32;
        host.inner.store.save_command(&command)?;
    }
    let mut termination = match &result {
        Ok(()) => v1::Termination::Completed,
        Err(Error::Cancelled) => v1::Termination::Cancelled,
        Err(Error::Plugin(_)) => v1::Termination::Failed,
        Err(_) => v1::Termination::Interrupted,
    };
    states.push(RunState::CleaningUp);
    dispatch.cancellation = CancellationToken::new();
    dispatch.resources.as_mut().unwrap().step.clear();
    if initialized && let Some(operation) = &template.cleanup {
        let cleanup = dispatch
            .invoke::<_, v1::CleanupResponse>(
                operation,
                v1::CleanupRequest {
                    execution: execution.clone(),
                    termination: termination as i32,
                    test_case: Some(case.clone()),
                    origin: Some(origin.clone()),
                },
            )
            .await;
        if let Err(error) = cleanup {
            termination = v1::Termination::Interrupted;
            dispatch.resources.as_ref().unwrap().evidence("cleanup:failure", error.to_string().into_bytes(), None)?;
        }
    }
    if let Err(error) = &result {
        dispatch.resources.as_ref().unwrap().evidence("execution:failure", error.to_string().into_bytes(), None)?;
    }
    if let Err(error) = time.close().await {
        termination = v1::Termination::Interrupted;
        dispatch.resources.as_ref().unwrap().evidence("streams:failure", error.to_string().into_bytes(), None)?;
    }
    let attribution = if termination == v1::Termination::Failed {
        v1::execution_outcome::Attribution::Evaluated(v1::EvaluatedOutcome {
            status: v1::ResultStatus::Failed as i32,
            reason: result.as_ref().err().map_or_else(String::new, ToString::to_string),
        })
    } else if termination == v1::Termination::Completed && !evaluated.is_empty() {
        v1::execution_outcome::Attribution::Evaluated(v1::EvaluatedOutcome {
            status: if evaluated.iter().all(|passed| *passed) { v1::ResultStatus::Passed } else { v1::ResultStatus::Failed } as i32,
            reason: String::new(),
        })
    } else {
        v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome {
            termination: termination as i32,
            reason: result.as_ref().err().map_or_else(|| "No Evaluated Criteria".into(), ToString::to_string),
        })
    };
    host.inner.store.close(&execution, termination, v1::ExecutionOutcome { attribution: Some(attribution) })?;
    host.inner.store.seal(&execution)?;
    states.push(match termination {
        v1::Termination::Cancelled => RunState::Cancelled,
        v1::Termination::Interrupted => RunState::Interrupted,
        _ => RunState::Finished,
    });
    tracing::info!(execution = %execution, ?termination, "Sealed Execution");
    Ok((host.inner.store.read(&execution)?, states))
}

/* === Tests === */
