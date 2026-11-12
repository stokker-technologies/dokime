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
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{
    Error, Host, Result,
    dispatch::Dispatch,
    host::{Active, lock},
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
            },
        );
        executions.push(execution.clone());
    }
    let task = tokio::spawn(async move {
        let mut snapshots = Vec::new();
        let mut states = Vec::new();
        for (execution, origin, entry) in prepared {
            let _active = ActiveRun {
                host: host.clone(),
                execution: execution.clone(),
            };
            let cancellation = lock(&host.inner.active)?.get(&execution).unwrap().cancellation.clone();
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
                for step in &case.steps {
                    let identifier = step
                        .metadata
                        .as_ref()
                        .ok_or_else(|| Error::Invalid("Materialize Steps with Identifiers".into()))?
                        .identifier
                        .clone();
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
            let attribution = if termination == v1::Termination::Completed && !evaluated.is_empty() {
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
            snapshots.push(host.inner.store.read(&execution)?);
        }
        let counts = OutcomeCounts::from_outcomes(snapshots.iter().filter_map(|snapshot| snapshot.execution.outcome.as_ref()));
        Ok(CycleResult { executions: snapshots, states, counts })
    });
    Ok(Cycle { identifier, executions, task })
}

/* === Private Functions === */

/* === Tests === */
