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
use std::{num::NonZeroUsize, ops::Bound};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectQuery, ObjectVersion, QueryValue, ReadLimits, ReadOrder, StorageView},
};

// Local Imports
use crate::{Error, Result, app::StateManager, execution::CycleView};
use dokime::{
    core::{NodeId, TestCycleId, TestExecutionId},
    storage::{
        Manager,
        objects::{cycle, execution},
        reports,
    },
};

/* === Constants === */
const MAX_BYTES: usize = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct WorkingCycle {
    cycle: CycleView,
    owner: NodeId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for WorkingCycle {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.cycle
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<CycleView>()),
            )
            .saturating_add(
                self.owner
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<NodeId>()),
            )
    }
}

/* === Public Functions === */
pub(super) async fn reconcile(state_manager: &StateManager, storage: &Manager) -> Result<()> {
    let query = ObjectQuery::<cycle::Cycle>::new();
    let mut reader = storage.query(query).await?;
    reader.retain_view()?;
    let projection = reader.decoded_bytes().min(allowance(MAX_BYTES)?);
    let mut view = reader
        .view()
        .ok_or_else(|| invalid("recovery query has no saved view"))?
        .clone();
    let mut after: Option<QueryValue<TestCycleId>> = None;
    loop {
        let selected = state_manager
            .state()
            .read(|state| {
                let lower = after.as_deref().map_or(Bound::Unbounded, Bound::Excluded);
                let Some((_id, header)) = state
                    .cycles
                    .range::<TestCycleId, _>((lower, Bound::Unbounded))
                    .next()
                else {
                    return Ok::<_, Error>(None);
                };
                // Preclaim The Joined Working Graph Before Cloning Any Member
                let mut bytes = header
                    .memory_size()
                    .checked_add(std::mem::size_of::<WorkingCycle>())
                    .and_then(|bytes| bytes.checked_add(state.identity.node.node_id.memory_size()))
                    .ok_or_else(|| invalid("working cycle memory overflow"))?;
                for execution_id in &header.execution_ids {
                    let execution = state
                        .executions
                        .get(execution_id)
                        .ok_or_else(|| invalid("working cycle member is missing"))?;
                    bytes = bytes
                        .checked_add(execution.memory_size())
                        .ok_or_else(|| invalid("working cycle memory overflow"))?;
                }
                let current = storage
                    .produce(allowance(bytes)?, |_| {
                        let mut test_executions = Vec::new();
                        test_executions
                            .try_reserve_exact(header.execution_ids.len())
                            .map_err(|_| harmos::Error::ResourceExhausted {
                                resource: "working recovery execution slots",
                            })?;
                        if test_executions.capacity() > header.execution_ids.len() {
                            return Err(harmos::Error::ResourceExhausted {
                                resource: "working recovery execution capacity",
                            });
                        }
                        for execution_id in &header.execution_ids {
                            let execution = state.executions.get(execution_id).ok_or(
                                harmos::Error::InvalidInput {
                                    field: "saved completion recovery",
                                    reason: "working cycle member is missing",
                                },
                            )?;
                            test_executions.push(execution.clone());
                        }
                        let cycle = CycleView {
                            saved: header.saved.clone(),
                            test_cycle_id: header.test_cycle_id.clone(),
                            session_id: header.session_id.clone(),
                            metadata: header.metadata.clone(),
                            lifecycle: header.lifecycle,
                            created_at: header.created_at.clone(),
                            started_at: header.started_at.clone(),
                            updated_at: header.updated_at.clone(),
                            test_executions,
                            records: header.records.clone(),
                            continue_on_failure: header.continue_on_failure,
                            result: header.result.clone(),
                        };
                        Ok(WorkingCycle {
                            cycle,
                            owner: state.identity.node.node_id.clone(),
                        })
                    })?
                    .try_into_value()?;
                Ok(Some(current))
            })
            .await
            .map_err(Error::Harmos)?
            .value?;
        let Some(mut current) = selected else {
            break;
        };
        let id_memory = allowance(current.cycle.test_cycle_id.memory_size())?;
        if id_memory > projection {
            return Err(harmos::Error::ResourceExhausted {
                resource: "saved completion recovery identity",
            }
            .into());
        }
        let id = reader.project(&current, id_memory, |working, _| {
            Ok(working.cycle.test_cycle_id.clone())
        })?;
        let recovered_executions =
            recover_execution_publications(storage, &view, &current, projection).await?;
        if !recovered_executions.is_empty() {
            let limits = view
                .execution_limits()?
                .ok_or_else(|| invalid("recovery view has no execution budget"))?;
            view = storage.view_with_limits(limits).await?;
        }
        let mut saved =
            cycle::read_in(storage, &view, &id, ObjectVersion::Latest, projection).await?;
        let recovered = if current.cycle.saved.is_none()
            && saved.as_ref().is_none_or(|row| row.value.saved.is_none())
        {
            recover_published_cycle(state_manager, storage, &view, &current, projection).await?
        } else {
            false
        };
        if recovered {
            let limits = view
                .execution_limits()?
                .ok_or_else(|| invalid("recovery view has no execution budget"))?;
            view = storage.view_with_limits(limits).await?;
            saved = cycle::read_in(storage, &view, &id, ObjectVersion::Latest, projection).await?;
        }
        if current.cycle.saved.as_ref().is_some_and(|seal| {
            saved.as_ref().and_then(|saved| saved.value.saved.as_ref()) != Some(seal)
        }) {
            return Err(invalid(
                "working cycle seal is missing or changed in native storage",
            ));
        }
        let completed = saved
            .as_ref()
            .and_then(|saved| saved.value.saved.as_ref())
            .filter(|seal| {
                seal.executions.len() == current.cycle.test_executions.len()
                    && seal
                        .executions
                        .iter()
                        .zip(&current.cycle.test_executions)
                        .all(|(saved, working)| saved.execution_id == working.test_execution_id)
            });
        if completed.is_some_and(|seal| {
            seal.owner_node_id != current.owner
                || current
                    .cycle
                    .saved
                    .as_ref()
                    .is_some_and(|saved| saved != seal)
        }) {
            return Err(invalid(
                "saved cycle differs from its working owner or seal",
            ));
        }
        let mut changed = false;
        for index in 0..current.cycle.test_executions.len() {
            let working = &current.cycle.test_executions[index];
            let version = completed.map_or(ObjectVersion::Latest, |seal| {
                ObjectVersion::Exact(seal.executions[index].version.clone())
            });
            let restored = execution::read_in(
                storage,
                &view,
                &working.test_execution_id,
                version,
                projection,
            )
            .await?;
            let Some(restored) = restored else {
                if completed.is_some() || working.saved.is_some() {
                    return Err(invalid("saved cycle member is missing"));
                }
                continue;
            };
            let Some(seal) = &restored.value.saved else {
                if completed.is_some() || working.saved.is_some() {
                    return Err(invalid("saved cycle member has no seal"));
                }
                continue;
            };
            super::execution::check_seal_position(&restored.position, &seal.dependencies_saved)?;
            if restored.value.test_cycle_id != current.cycle.test_cycle_id
                || restored.value.test_case_id != working.test_case_id
                || restored.value.snapshot != working.snapshot
                || seal.source.owner_node_id != current.owner
                || working.saved.as_ref().is_some_and(|saved| saved != seal)
                || completed
                    .is_some_and(|cycle| cycle.executions[index].position != restored.position)
            {
                return Err(invalid("saved execution differs from its working identity"));
            }
            covered(&view, &restored.position)?;
            covered(&view, &seal.dependencies_saved)?;
            for file in seal.files() {
                covered(&view, &file.position)?;
                if !recovered
                    && recovered_executions
                        .binary_search(&working.test_execution_id)
                        .is_err()
                {
                    super::execution::verify_file(view.read_file(file).await?).await?;
                }
            }
            if restored.value == *working {
                continue;
            }
            let bytes = current
                .memory_size()
                .checked_add(restored.value.memory_size())
                .ok_or_else(|| invalid("recovery projection memory overflow"))?;
            current = reader.project(
                &current,
                allowance(bytes)?.min(projection),
                |working, allowance| {
                    if bytes > allowance.get() {
                        return Err(harmos::Error::ResourceExhausted {
                            resource: "saved completion recovery projection",
                        });
                    }
                    let mut next = WorkingCycle {
                        cycle: working.cycle.clone(),
                        owner: working.owner.clone(),
                    };
                    next.cycle.test_executions[index] = restored.value.clone();
                    Ok(next)
                },
            )?;
            changed = true;
        }
        if let Some(seal) = completed {
            super::execution::check_seal_position(
                &saved.as_ref().expect("completed cycle revision").position,
                &seal.dependencies_saved,
            )?;
            covered(
                &view,
                &saved.as_ref().expect("completed cycle revision").position,
            )?;
            covered(&view, &seal.dependencies_saved)?;
            covered(&view, &seal.report.position)?;
            if !recovered {
                super::execution::verify_file(view.read_file(&seal.report).await?).await?;
            }
            complete_saved_request(
                state_manager,
                storage,
                &current.cycle,
                &saved.as_ref().expect("completed cycle revision").value,
            )
            .await?;
        }
        if changed || completed.is_some() {
            // Retain Clone Preparation Until The Original Journal Admits It
            let bytes = current
                .cycle
                .memory_size()
                .checked_add(id.memory_size())
                .and_then(|bytes| {
                    bytes.checked_add(
                        std::mem::size_of::<crate::transactions::ReconcileStoredCycle>(),
                    )
                })
                .ok_or_else(|| invalid("recovery transaction memory overflow"))?;
            let _preparation = storage.prepare_production(allowance(bytes)?)?;
            let change = crate::transactions::ReconcileStoredCycle {
                cycle_id: (*id).clone(),
                cycle: completed.is_none().then(|| current.cycle.clone()),
            };
            state_manager.commit(change).await.map_err(|source| {
                Error::Harmos(harmos::Error::backend(
                    "commit saved completion recovery",
                    harmos::BackendFailure::Permanent,
                    source,
                ))
            })?;
        }
        after = Some(id);
    }
    Ok(())
}

/* === Private Functions === */
async fn recover_execution_publications(
    storage: &Manager,
    view: &StorageView,
    current: &WorkingCycle,
    projection: NonZeroUsize,
) -> Result<QueryValue<Vec<TestExecutionId>>> {
    let mut recovered = storage
        .produce(
            allowance(std::mem::size_of::<Vec<TestExecutionId>>())?,
            |_| Ok(Vec::<TestExecutionId>::new()),
        )?
        .try_into_value()?;
    for working in &current.cycle.test_executions {
        if working.saved.is_some() {
            continue;
        }
        let Some(original) = execution::read_in(
            storage,
            view,
            &working.test_execution_id,
            ObjectVersion::Latest,
            projection,
        )
        .await?
        else {
            continue;
        };
        if original.value.saved.is_some() {
            continue;
        }
        let mut query = reports::query(reports::Owner::Execution(&working.test_execution_id), None);
        query.options.order = ReadOrder::Descending;
        query.options.page = Some(ReadLimits {
            rows: 1.try_into().unwrap(),
            bytes: projection,
        });
        query.options.decoded_bytes = Some(projection);
        let mut history = storage.query_in(view, query)?;
        while let Some(page) = history.next_page().await? {
            let record = page
                .first()
                .ok_or_else(|| invalid("report history returned an empty page"))?;
            let publication = reports::Publication::from_record(record)?;
            if !publication.is_final() {
                continue;
            }
            let seal = page.try_project(projection, |rows, budget| {
                rows[0].payload.execution_seal(budget).map_err(|source| {
                    harmos::Error::backend(
                        "project published execution",
                        harmos::BackendFailure::Corrupt,
                        source,
                    )
                })
            })?;
            if seal.source.execution.position != original.position
                || seal.source.execution.version != original.version
            {
                continue;
            }
            if seal.source.owner_node_id != current.owner
                || original.value.test_cycle_id != current.cycle.test_cycle_id
                || original.value.test_case_id != working.test_case_id
                || original.value.snapshot != working.snapshot
            {
                return Err(invalid(
                    "published execution differs from its working owner",
                ));
            }
            seal.validate(&original.value)
                .map_err(dokime::storage::Error::from)?;
            covered(view, &record.position)?;
            covered(view, &original.position)?;
            covered(view, &seal.dependencies_saved)?;
            for file in seal.files() {
                covered(view, &file.position)?;
                super::execution::verify_file(view.read_file(file).await?).await?;
            }
            let bytes = original
                .value
                .memory_size()
                .saturating_add(seal.memory_size());
            let candidate = storage
                .produce(allowance(bytes)?, |_| {
                    let mut execution = original.value.clone();
                    execution.saved = Some((*seal).clone());
                    Ok(execution)
                })?
                .try_into_value()?;
            let bytes = execution::Execution::encoding_memory(&candidate)?;
            let native = storage
                .produce(allowance(bytes)?, |_| {
                    execution::Execution::try_from(&*candidate).map_err(|source| {
                        harmos::Error::backend(
                            "encode recovered execution",
                            harmos::BackendFailure::Corrupt,
                            source,
                        )
                    })
                })?
                .try_into_value()?;
            let receipt = storage.write_object(&*native).await?;
            if !storage
                .flush_until(&receipt.position)
                .await?
                .covers(&receipt.position)?
            {
                return Err(invalid("recovered execution seal is not saved"));
            }
            let bytes = recovered
                .memory_size()
                .saturating_add(working.test_execution_id.memory_size());
            recovered = recovered.try_map(allowance(bytes)?, |mut ids, _| {
                ids.try_reserve_exact(1)
                    .map_err(|_| harmos::Error::ResourceExhausted {
                        resource: "recovered execution identities",
                    })?;
                ids.push(working.test_execution_id.clone());
                Ok(ids)
            })?;
            break;
        }
    }
    let bytes = recovered.memory_size();
    Ok(recovered.try_map(allowance(bytes)?, |mut ids, _| {
        ids.sort_unstable();
        Ok(ids)
    })?)
}

async fn recover_published_cycle(
    state_manager: &StateManager,
    storage: &Manager,
    view: &StorageView,
    current: &WorkingCycle,
    projection: NonZeroUsize,
) -> Result<bool> {
    let intent = state_manager
        .state()
        .read(|state| {
            let mut requests = state.run_requests.values().filter(|request| {
                request.summary.cycle_id == current.cycle.test_cycle_id
                    && !request.summary.phase.terminal()
            });
            let Some(request) = requests.next() else {
                return Ok::<_, Error>(None);
            };
            if requests.next().is_some() {
                return Err(invalid("cycle publication has multiple active requests"));
            }
            let Some(at) = &request.completion_at else {
                return Ok(None);
            };
            let bytes = request
                .summary
                .request_id
                .memory_size()
                .saturating_add(at.memory_size());
            let memory = storage.prepare_production(allowance(bytes)?)?;
            Ok(Some((
                request.summary.request_id.clone(),
                at.clone(),
                memory,
            )))
        })
        .await?
        .value?;
    let Some((request_id, at, _intent)) = intent else {
        return Ok(false);
    };
    let mut query = reports::query(reports::Owner::Cycle(&current.cycle.test_cycle_id), None);
    query.options.order = ReadOrder::Descending;
    query.options.page = Some(ReadLimits {
        rows: 1.try_into().unwrap(),
        bytes: projection,
    });
    query.options.decoded_bytes = Some(projection);
    let mut history = storage.query_in(view, query)?;
    while let Some(page) = history.next_page().await? {
        let record = page
            .first()
            .ok_or_else(|| invalid("report history returned an empty page"))?;
        let publication = reports::Publication::from_record(record)?;
        if !publication.is_final() || publication.generated_at() != at.as_str() {
            continue;
        }
        covered(view, &record.position)?;
        let candidate = page.try_project(projection, |rows, budget| {
            rows[0]
                .payload
                .final_cycle(&current.owner, budget)
                .map_err(|source| {
                    harmos::Error::backend(
                        "project published cycle",
                        harmos::BackendFailure::Corrupt,
                        source,
                    )
                })
        })?;
        let seal = candidate.saved.as_ref().expect("projected final seal");
        if candidate.execution_ids.len() != current.cycle.test_executions.len() {
            return Err(invalid("published cycle differs from working membership"));
        }
        let mut seals = storage
            .produce(
                allowance(std::mem::size_of::<
                    Vec<dokime::execution::saved::ExecutionSeal>,
                >())?,
                |_| Ok(Vec::<dokime::execution::saved::ExecutionSeal>::new()),
            )?
            .try_into_value()?;
        for ((revision, working), input) in seal
            .executions
            .iter()
            .zip(&current.cycle.test_executions)
            .zip(publication.inputs())
        {
            let member = execution::read_in(
                storage,
                view,
                &revision.execution_id,
                ObjectVersion::Exact(revision.version.clone()),
                projection,
            )
            .await?
            .ok_or_else(|| invalid("published cycle member is missing"))?;
            let saved = member
                .value
                .saved
                .as_ref()
                .ok_or_else(|| invalid("published cycle member is unsealed"))?;
            if revision.execution_id != working.test_execution_id
                || member.position != revision.position
                || member.value.test_cycle_id != current.cycle.test_cycle_id
                || member.value.test_case_id != working.test_case_id
                || member.value.snapshot != working.snapshot
                || saved.source.owner_node_id != current.owner
                || saved.source.context != *input
                || working
                    .saved
                    .as_ref()
                    .is_some_and(|previous| previous != saved)
            {
                return Err(invalid(
                    "published cycle member differs from its frozen owner",
                ));
            }
            super::execution::check_seal_position(&member.position, &saved.dependencies_saved)?;
            covered(view, &member.position)?;
            for file in saved.files() {
                covered(view, &file.position)?;
                super::execution::verify_file(view.read_file(file).await?).await?;
            }
            let bytes = seals.memory_size().saturating_add(saved.memory_size());
            seals = seals.try_map(allowance(bytes)?, |mut seals, _| {
                seals
                    .try_reserve_exact(1)
                    .map_err(|_| harmos::Error::ResourceExhausted {
                        resource: "recovered cycle member seals",
                    })?;
                seals.push(saved.clone());
                Ok(seals)
            })?;
        }
        covered(view, &seal.dependencies_saved)?;
        super::execution::verify_file(view.read_file(&seal.report).await?).await?;
        let bytes = candidate
            .memory_size()
            .saturating_add(seals.memory_size())
            .saturating_add(request_id.memory_size())
            .saturating_add(at.memory_size())
            .saturating_add(std::mem::size_of::<crate::transactions::CompleteRun>());
        let _completion = storage.prepare_production(allowance(bytes)?)?;
        let complete = crate::transactions::CompleteRun {
            request_id: request_id.clone(),
            at: at.clone(),
            sealed_cycle: Some((*candidate).clone()),
            sealed_executions: (*seals).clone(),
        };
        state_manager
            .state()
            .read(|state| {
                use harmos::state::Transaction;
                complete.check(state)
            })
            .await?
            .value
            .map_err(Error::Operation)?;
        let bytes = cycle::Cycle::encoding_memory(&candidate)?;
        let native = storage
            .produce(allowance(bytes)?, |_| {
                cycle::Cycle::try_from(&*candidate).map_err(|source| {
                    harmos::Error::backend(
                        "encode recovered cycle",
                        harmos::BackendFailure::Corrupt,
                        source,
                    )
                })
            })?
            .try_into_value()?;
        let receipt = storage.write_object(&*native).await?;
        if !storage
            .flush_until(&receipt.position)
            .await?
            .covers(&receipt.position)?
        {
            return Err(invalid("recovered cycle seal is not saved"));
        }
        return Ok(true);
    }
    Ok(false)
}

async fn complete_saved_request(
    state_manager: &StateManager,
    storage: &Manager,
    current: &CycleView,
    saved: &dokime::execution::TestCycle,
) -> Result<()> {
    let pending = state_manager
        .state()
        .read(|state| {
            let mut requests = state.run_requests.values().filter(|request| {
                request.summary.cycle_id == saved.test_cycle_id && !request.summary.phase.terminal()
            });
            let Some(request) = requests.next() else {
                return Ok::<_, Error>(None);
            };
            if requests.next().is_some() {
                return Err(invalid("saved cycle has multiple active requests"));
            }
            let at = request
                .completion_at
                .as_ref()
                .ok_or_else(|| invalid("saved cycle has no staged completion intent"))?;
            let bytes = current.test_executions.iter().try_fold(
                std::mem::size_of::<crate::transactions::CompleteRun>()
                    .saturating_add(saved.memory_size())
                    .saturating_add(request.summary.request_id.memory_size())
                    .saturating_add(at.memory_size())
                    .saturating_add(current.memory_size())
                    .saturating_add(current.test_cycle_id.memory_size())
                    .saturating_add(
                        std::mem::size_of::<crate::transactions::ReconcileStoredCycle>(),
                    ),
                |bytes, execution| {
                    let seal = execution
                        .saved
                        .as_ref()
                        .ok_or_else(|| invalid("saved completion member has no seal"))?;
                    Ok::<_, Error>(bytes.saturating_add(seal.memory_size()))
                },
            )?;
            let preparation = storage.prepare_production(allowance(bytes)?)?;
            let mut seals = Vec::with_capacity(current.test_executions.len());
            for execution in &current.test_executions {
                seals.push(
                    execution
                        .saved
                        .as_ref()
                        .expect("checked saved member")
                        .clone(),
                );
            }
            let complete = crate::transactions::CompleteRun {
                request_id: request.summary.request_id.clone(),
                at: at.clone(),
                sealed_cycle: Some(saved.clone()),
                sealed_executions: seals,
            };
            Ok(Some((complete, preparation)))
        })
        .await?
        .value?;
    let Some((complete, _preparation)) = pending else {
        return Ok(());
    };
    // Restore Verified Members Before Completing The Original Staged Request.
    state_manager
        .commit(crate::transactions::ReconcileStoredCycle {
            cycle_id: current.test_cycle_id.clone(),
            cycle: Some(current.clone()),
        })
        .await
        .map_err(|source| Error::Operation(source.to_string()))?;
    state_manager
        .commit(complete)
        .await
        .map_err(|source| Error::Operation(source.to_string()))?;
    Ok(())
}

fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    if bytes > MAX_BYTES {
        return Err(invalid("recovery projection exceeds memory allowance"));
    }
    NonZeroUsize::new(bytes).ok_or_else(|| invalid("empty recovery memory allowance"))
}

fn covered(view: &StorageView, position: &harmos::storage::StorageCursor) -> Result<()> {
    if !view.saved()?.covers(position)? {
        return Err(invalid(
            "saved completion exceeds the original recovery view",
        ));
    }
    Ok(())
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */
