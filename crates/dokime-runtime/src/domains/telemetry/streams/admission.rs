/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Owned Telemetry Fan-Out And Its Native Transient Admission.

use super::{CapturedPublication, PublishReceipt, lease_accepts_sample, telemetry_stream_id, validation_rejection};
use crate::runtime::handle::RuntimeHandle;
use crate::runtime::{Error, Result};
use crate::telemetry::{TelemetrySample, ValidationMode, check_normalized_telemetry_sample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use harmos::storage::ReadBatch;
use std::collections::{BTreeMap, BTreeSet};

const PARTIAL_ACCEPTANCE: &str = "telemetry was partially accepted; unaccepted samples were dropped";

pub(crate) struct PreparedSample {
    pub(crate) sample: TelemetrySample,
    pub(crate) contract: usize,
    pub(crate) ordinal: u64,
    pub(crate) original: bool,
}

#[derive(Default)]
pub(crate) struct TelemetryCursors {
    pub(crate) values: BTreeMap<String, u64>,
}

pub(crate) struct PreparedPublication {
    order: tokio::sync::OwnedMutexGuard<()>,
    execution: crate::runtime::storage::telemetry::TelemetryAdmission,
    accepted: Vec<PreparedSample>,
    supplied: Vec<(Option<dokime_domain::model::identity::TestExecutionId>, dokime_domain::model::identity::TelemetrySampleId)>,
    receipt: ReadBatch<PublishReceipt>,
    delivery: ReadBatch<crate::domains::telemetry::leases::LeasedRows>,
    observes_closure: bool,
}

impl PreparedPublication {
    pub(crate) fn within_quiescence(mut self) -> Self {
        self.observes_closure = false;
        self
    }
}

impl RuntimeHandle {
    pub(crate) async fn prepare_sample_admission(&self, samples: Vec<TelemetrySample>) -> Result<PreparedPublication> {
        self.storage().retry_transient_persistence();
        let order = self.telemetry_admission.clone().lock_owned().await;
        let mut supplied = slots(samples.len())?;
        supplied.extend(
            samples
                .iter()
                .map(|sample| (sample.scope.as_ref().and_then(|scope| scope.test_execution_id.clone()), sample.sample_id.clone())),
        );
        let delivery = self.leased_rows(&samples)?;
        let leased = self.execution_storage.with_telemetry_scopes(|open| -> Result<_> {
            let mut streams = BTreeSet::new();
            for sample in &samples {
                if streams.contains(&sample.stream) {
                    continue;
                }
                streams.insert(&sample.stream);
            }
            let mut leased = BTreeMap::new();
            for stream in streams {
                let scopes = delivery.capture_scopes(stream);
                let mut owners = BTreeMap::new();
                for scope in scopes {
                    let Some(id) = scope.test_execution_id.as_ref() else {
                        continue;
                    };
                    let Some(canonical) = open.get(id) else {
                        continue;
                    };
                    if owners.contains_key(id) {
                        continue;
                    }
                    owners.insert(id, canonical);
                }
                let mut scopes = slots(owners.len())?;
                scopes.extend(owners.into_values().cloned());
                leased.insert(stream.clone(), scopes);
            }
            Ok(leased)
        })?;
        let (mut accepted, contracts, delivery) = self
            .state_manager
            .state()
            .read(|state| -> Result<_> {
                for sample in &samples {
                    if let Some(scope) = &sample.scope {
                        state.capture_scope(scope).map_err(Error::Operation)?;
                    }
                }
                let mut captured = captured_samples(samples, &leased)?;
                let delivery = crate::domains::telemetry::leases::LeasedRows::attribute(delivery, state, &captured)?;
                for (sample, _) in &mut captured {
                    if let Some(scope) = &sample.scope {
                        sample.scope = Some(state.capture_scope(scope).map_err(Error::Operation)?);
                    }
                }
                let stream_count = leased.len();
                let mut contracts = slots(stream_count)?;
                let mut indices = BTreeMap::new();
                let mut accepted = slots(captured.len())?;
                for (sample, original) in captured {
                    let index = if let Some(index) = indices.get(&sample.stream) {
                        *index
                    } else {
                        let contract = state
                            .telemetry_definitions
                            .get(&sample.stream)
                            .ok_or_else(|| validation_rejection("telemetry contract is unavailable", "stream", None))?;
                        let index = contracts.len();
                        contracts.push(contract.normalized());
                        indices.insert(sample.stream.clone(), index);
                        index
                    };
                    let contract = contracts.get(index).ok_or_else(|| invalid("telemetry contract index is missing"))?;
                    check_normalized_telemetry_sample(contract, &sample, ValidationMode::Strict)
                        .map_err(|error| validation_rejection(error.to_string(), "telemetry", None))?;
                    accepted.push(PreparedSample { sample, contract: index, ordinal: 0, original });
                }
                Ok((accepted, contracts, delivery))
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?
            .value?;
        let mut receipt = prepared_receipt()?;
        {
            let known = self.telemetry_cursors.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            for row in &mut accepted {
                let key = telemetry_stream_id(row.sample.scope.as_ref(), &row.sample.stream);
                receipt = receipt.try_map(|mut receipt| {
                    let next = receipt.stream_cursors.entry(key).or_insert_with_key(|key| known.values.get(key).copied().unwrap_or(0));
                    *next = next.checked_add(1).ok_or_else(|| invalid("telemetry ordinal exhausted"))?;
                    row.ordinal = *next;
                    Ok::<_, Error>(receipt)
                })?;
            }
        }
        let execution = self.execution_storage.prepare_telemetry(&accepted, &contracts).await?;
        Ok(PreparedPublication { order, execution, accepted, supplied, receipt, delivery, observes_closure: true })
    }

    pub(crate) async fn admit_samples(&self, admission: PreparedPublication) -> Result<ReadBatch<PublishReceipt>> {
        let runtime = self.clone();
        let publication = tokio::spawn(async move { runtime.admit_samples_owned(admission).await }).await??;
        if let Some(source) = &publication.failure {
            tracing::warn!(error = ?source, accepted = publication.receipt.accepted, dropped = publication.receipt.dropped,
                "telemetry publication was partially accepted");
        }
        Ok(publication.receipt)
    }

    pub(super) async fn admit_samples_owned(&self, admission: PreparedPublication) -> Result<CapturedPublication> {
        let PreparedPublication { order, execution, mut accepted, supplied, receipt, delivery, observes_closure } = admission;
        let sampling = self.sampling();
        let admitted = execution.commit(observes_closure.then_some(sampling.as_ref())).await?;
        if admitted.accepted.len() != accepted.len() {
            return Err(invalid("native admission receipt is incomplete"));
        }
        let mut native = admitted.accepted.iter();
        accepted.retain(|_| native.next().copied().unwrap_or(false));
        // This Full Owner Was Admitted Before Commit And Has Never Been Aliased.
        // Same-Type Zero-Growth Updates Cannot Refuse New Native Capacity.
        let receipt = receipt.try_map(|mut receipt| {
            for ordinal in receipt.stream_cursors.values_mut() {
                *ordinal = 0;
            }
            for row in &accepted {
                let key = telemetry_stream_id(row.sample.scope.as_ref(), &row.sample.stream);
                let ordinal = receipt.stream_cursors.get_mut(&key).ok_or_else(|| invalid("admitted telemetry cursor is missing"))?;
                *ordinal = (*ordinal).max(row.ordinal);
            }
            receipt.stream_cursors.retain(|_, ordinal| *ordinal != 0);
            Ok::<_, Error>(receipt)
        })?;
        let identities = identity_index(&accepted)?;
        let accepted_count = supplied.iter().filter(|(execution, id)| identities.binary_search(&(execution.as_ref(), id)).is_ok()).count();
        let accepted_count = u64::try_from(accepted_count).map_err(|_| overflow())?;
        let dropped_count = u64::try_from(supplied.len()).map_err(|_| overflow())?.checked_sub(accepted_count).ok_or_else(overflow)?;
        {
            let mut known = self.telemetry_cursors.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            for (stream, ordinal) in &receipt.stream_cursors {
                match known.values.get_mut(stream) {
                    Some(known) => *known = (*known).max(*ordinal),
                    None => {
                        known.values.insert(stream.clone(), *ordinal);
                    }
                }
            }
        }
        drop(order);
        self.forward_leased_rows(delivery, &accepted, &identities).await;
        let receipt = receipt.try_map(|mut receipt| {
            receipt.accepted = accepted_count;
            receipt.dropped = dropped_count;
            receipt.revision = self.state_manager.applied().0;
            if admitted.failure.is_none() {
                receipt.message = None;
            }
            Ok::<_, Error>(receipt)
        })?;
        Ok(CapturedPublication { receipt, failure: admitted.failure })
    }
}

fn prepared_receipt() -> Result<ReadBatch<PublishReceipt>> {
    harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut receipt = PublishReceipt::accepted(0, BTreeMap::new(), 0);
        receipt.message = Some(PARTIAL_ACCEPTANCE.to_owned());
        Ok::<_, Error>(receipt)
    })
}

pub(super) fn dropped_receipt(dropped: u64, revision: u64, message: &'static str) -> Result<ReadBatch<PublishReceipt>> {
    harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(PublishReceipt::dropped(dropped, revision, message)))
}

pub(crate) type SampleIdentity<'a> =
    (Option<&'a dokime_domain::model::identity::TestExecutionId>, &'a dokime_domain::model::identity::TelemetrySampleId);

pub(crate) fn identity_index(samples: &[PreparedSample]) -> Result<Vec<SampleIdentity<'_>>> {
    let mut identities = slots(samples.len())?;
    identities.extend(
        samples.iter().map(|row| (row.sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()), &row.sample.sample_id)),
    );
    identities.sort_unstable();
    identities.dedup();
    Ok(identities)
}

fn captured_samples(
    samples: Vec<TelemetrySample>, leased: &BTreeMap<Identifier, Vec<RecordScope>>,
) -> Result<Vec<(TelemetrySample, bool)>> {
    let mut count = samples.len();
    for sample in &samples {
        if sample.scope.as_ref().is_some_and(|scope| scope.test_execution_id.is_some()) {
            continue;
        }
        for _scope in leased.get(&sample.stream).into_iter().flatten().filter(|scope| lease_accepts_sample(sample.scope.as_ref(), scope)) {
            count = count.checked_add(1).ok_or_else(overflow)?;
        }
    }
    let mut captured = slots(count)?;
    for sample in samples {
        let original = captured.len();
        captured.push((sample, true));
        if captured[original].0.scope.as_ref().is_some_and(|scope| scope.test_execution_id.is_some()) {
            continue;
        }
        for scope in leased.get(&captured[original].0.stream).into_iter().flatten() {
            if !lease_accepts_sample(captured[original].0.scope.as_ref(), scope) {
                continue;
            }
            let mut copy = captured[original].0.clone();
            copy.scope = Some(scope.clone());
            captured.push((copy, false));
        }
    }
    Ok(captured)
}

pub(crate) fn slots<T>(length: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(length).map_err(|_| invalid("telemetry staging allocation failed"))?;
    Ok(values)
}
fn overflow() -> Error {
    invalid("telemetry staging memory overflows")
}
fn invalid(reason: &str) -> Error {
    Error::Operation(reason.to_owned())
}
