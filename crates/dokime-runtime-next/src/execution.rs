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
use std::sync::Arc;

// Third Party Imports
use prost::Message;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{
    Error, Result,
    dispatch::Dispatch,
    execution_manager::{Active, ExecutionManager},
    host::lock,
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
    result: tokio::sync::oneshot::Receiver<Result<CycleResult>>,
}
#[derive(Debug)]
pub struct CycleResult {
    pub executions: Vec<ExecutionSnapshot>,
    pub states: Vec<RunState>,
    pub counts: OutcomeCounts,
}
struct ActiveRun {
    host: ExecutionManager,
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
        self.result.await.map_err(|_| Error::Invalid("Wait for Cycle: Inspect the Failed Execution Task".into()))?
    }
}
impl Drop for ActiveRun {
    fn drop(&mut self) {
        if let Ok(mut active) = self.host.inner.active.lock() {
            active.remove(&self.execution);
        }
        self.host.inner.prompts.clear(&self.execution);
        self.host.signal_change();
    }
}

/* === Public Functions === */

pub(crate) async fn start(host: ExecutionManager, origins: Vec<v1::Origin>) -> Result<Cycle> {
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
    let admitted: Vec<_> = prepared
        .iter()
        .map(|(execution, origin, entry)| Execution {
            identifier: execution.clone(),
            cycle: identifier.clone(),
            origin: Some(origin.clone()),
            catalog: entry.description.catalog.as_ref().unwrap().encode_to_vec(),
            ..Default::default()
        })
        .collect();
    host.register_cycle(&identifier)?;
    host.inner.store.create_executions(&admitted)?;
    let executions = admitted.iter().map(|execution| execution.identifier.clone()).collect();
    {
        let mut active = lock(&host.inner.active)?;
        for (execution, _, entry) in &prepared {
            active.insert(
                execution.clone(),
                Active {
                    plugin: entry.description.id.clone(),
                    cancellation: CancellationToken::new(),
                    control: Arc::new(crate::run_control::RunControl::default()),
                },
            );
        }
    }
    launch(host, identifier, executions, prepared)
}

pub(crate) async fn start_existing_mode(
    host: ExecutionManager, execution: &str, mode: dokime_protocol::client::ExecutionMode,
) -> Result<Cycle> {
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
    host.register_cycle(&cut.execution.cycle)?;
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
                control: Arc::new(if mode == dokime_protocol::client::ExecutionMode::Interactive {
                    crate::run_control::RunControl::interactive()
                } else {
                    crate::run_control::RunControl::default()
                }),
            },
        );
    }
    launch(host, cut.execution.cycle, vec![execution.into()], vec![(execution.into(), origin, entry)])
}

/* === Private Functions === */

fn launch(
    host: ExecutionManager, identifier: String, executions: Vec<String>, prepared: Vec<(String, v1::Origin, Arc<Entry>)>,
) -> Result<Cycle> {
    host.track(&executions)?;
    let (send, result) = tokio::sync::oneshot::channel();
    let owner = host.clone();
    let observed = executions.clone();
    owner.spawn(async move {
        let result = async {
            let mut workers = JoinSet::new();
            for (index, (execution, origin, entry)) in prepared.into_iter().enumerate() {
                let host = host.clone();
                workers.spawn(async move {
                    let result = drive(host.clone(), execution.clone(), origin, entry).await;
                    if result.is_err() {
                        let cut = host.inner.store.cut(&execution)?;
                        if cut.state == dokime_storage::ExecutionState::Open {
                            host.inner.store.close_streams(&execution)?;
                            host.inner.store.close(
                                &execution,
                                v1::Termination::Interrupted,
                                v1::ExecutionOutcome {
                                    attribution: Some(v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome {
                                        termination: v1::Termination::Interrupted as i32,
                                        reason: "Execution Infrastructure Failed; Inspect the Host Diagnostic".into(),
                                    })),
                                },
                            )?;
                            host.inner.store.seal(&execution)?;
                        }
                    }
                    host.finish_progress(&execution, result.as_ref().err().map(ToString::to_string))?;
                    Ok::<_, Error>((index, result))
                });
            }
            let mut finished = Vec::new();
            let mut failure = None;
            while let Some(result) = workers.join_next().await {
                match result {
                    Ok(Ok((index, result))) => match result {
                        Ok(result) => finished.push((index, result)),
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    },
                    Ok(Err(error)) => {
                        failure.get_or_insert(error);
                    }
                    Err(error) => {
                        failure.get_or_insert(Error::Join(error));
                    }
                }
            }
            if let Some(error) = failure {
                return Err(error);
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
        }
        .await;
        // Settle Every Admitted Execution Even If a Worker Failed to Join
        let mut result = result;
        for execution in observed {
            let settled = (|| -> Result<()> {
                let cut = host.inner.store.cut(&execution)?;
                if cut.state == dokime_storage::ExecutionState::Open {
                    host.inner.store.close_streams(&execution)?;
                    host.inner.store.close(
                        &execution,
                        v1::Termination::Interrupted,
                        v1::ExecutionOutcome {
                            attribution: Some(v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome {
                                termination: v1::Termination::Interrupted as i32,
                                reason: "Execution Worker Failed; Inspect the Host Diagnostic".into(),
                            })),
                        },
                    )?;
                }
                if host.inner.store.cut(&execution)?.state == dokime_storage::ExecutionState::Closed {
                    host.inner.store.seal(&execution)?;
                }
                Ok(())
            })();
            if let Err(error) = settled {
                result = Err(error);
            }
            if let Err(error) = host.finish_progress(&execution, result.as_ref().err().map(ToString::to_string)) {
                result = Err(error);
            }
        }
        let _ = send.send(result);
    })?;
    Ok(Cycle { identifier, executions, result })
}

async fn drive(
    host: ExecutionManager, execution: String, origin: v1::Origin, entry: Arc<Entry>,
) -> Result<(ExecutionSnapshot, Vec<RunState>)> {
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
    host.set_run_phase(&execution, "initializing")?;
    let mut initialized = template.initialize.is_none();
    let mut evaluated = Vec::new();
    let result: Result<()> = async {
        if dispatch.cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if let Some(operation) = &template.initialize {
            dispatch
                .invoke::<_, dokime_protocol::plugin::ExecutionsInitializeResponse>(
                    operation,
                    dokime_protocol::plugin::ExecutionsInitializeRequest {
                        execution: execution.clone(),
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await?;
            initialized = true;
        }
        states.push(RunState::Stepping);
        host.set_run_phase(&execution, "stepping")?;
        while let Some((index, mut command)) = control.next(case, &dispatch.cancellation).await? {
            let step = &case.steps[index];
            host.signal_change();
            let identifier =
                step.metadata.as_ref().ok_or_else(|| Error::Invalid("Materialize Steps with Identifiers".into()))?.identifier.clone();
            dispatch.resources.as_mut().unwrap().step = identifier.clone();
            let position = dispatch.resources.as_ref().unwrap().evidence("step:start", Vec::new(), None)?;
            if let Some(command) = &mut command {
                command.phase = dokime_protocol::client::executions::ExecutionCommandPhase::Running as i32;
                command.attempt_id = Some(format!("{execution}/{position}"));
                let now = host.now_ns();
                command.updated_at = Some(prost_types::Timestamp {
                    seconds: now.div_euclid(1_000_000_000),
                    nanos: now.rem_euclid(1_000_000_000) as i32,
                });
                host.inner.store.save_command(command)?;
            }
            host.signal_change();
            let result = dispatch
                .invoke::<_, dokime_protocol::plugin::StepsExecuteResponse>(
                    &step.operation,
                    dokime_protocol::plugin::StepsExecuteRequest {
                        execution: execution.clone(),
                        step: identifier,
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await;
            dispatch.resources.as_ref().unwrap().evidence("step:end", Vec::new(), None)?;
            if result.is_err() {
                dispatch.resources.as_ref().unwrap().evidence("step:failure", Vec::new(), None)?;
            }
            if let Some(mut command) = command {
                command.phase = if result.is_ok() {
                    dokime_protocol::client::executions::ExecutionCommandPhase::Completed
                } else {
                    dokime_protocol::client::executions::ExecutionCommandPhase::Failed
                } as i32;
                command.detail = result.as_ref().err().map(ToString::to_string);
                let now = host.now_ns();
                command.updated_at = Some(prost_types::Timestamp {
                    seconds: now.div_euclid(1_000_000_000),
                    nanos: now.rem_euclid(1_000_000_000) as i32,
                });
                host.inner.store.save_command(&command)?;
            }
            control.complete_step(index)?;
            host.signal_change();
            result?;
        }
        dispatch.resources.as_mut().unwrap().step.clear();
        states.push(RunState::Evaluating);
        host.set_run_phase(&execution, "evaluating")?;
        for criterion in &case.criteria {
            let identifier = criterion
                .metadata
                .as_ref()
                .ok_or_else(|| Error::Invalid("Materialize Criteria with Identifiers".into()))?
                .identifier
                .clone();
            let response: dokime_protocol::plugin::CriteriaEvaluateResponse = dispatch
                .invoke(
                    &criterion.operation,
                    dokime_protocol::plugin::CriteriaEvaluateRequest {
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
        command.phase = if result.is_ok() {
            dokime_protocol::client::executions::ExecutionCommandPhase::Completed
        } else {
            dokime_protocol::client::executions::ExecutionCommandPhase::Interrupted
        } as i32;
        host.inner.store.save_command(&command)?;
    }
    let mut termination = match &result {
        Ok(()) => v1::Termination::Completed,
        Err(Error::Cancelled) => v1::Termination::Cancelled,
        Err(Error::Plugin(_)) => v1::Termination::Failed,
        Err(_) => v1::Termination::Interrupted,
    };
    states.push(RunState::CleaningUp);
    host.set_run_phase(&execution, "cleaning_up")?;
    dispatch.cancellation = CancellationToken::new();
    dispatch.resources.as_mut().unwrap().step.clear();
    if initialized && let Some(operation) = &template.cleanup {
        let cleanup = dispatch
            .invoke::<_, dokime_protocol::plugin::ExecutionsCleanupResponse>(
                operation,
                dokime_protocol::plugin::ExecutionsCleanupRequest {
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
