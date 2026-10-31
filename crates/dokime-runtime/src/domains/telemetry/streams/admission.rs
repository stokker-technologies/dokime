/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Owned Telemetry Fan-Out And Its Native Transient Admission.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

// Third Party Imports
use harmos::storage::ReadBatch;

// Local Imports
use super::{CapturedPublication, PublishReceipt, lease_accepts_sample, telemetry_stream_id, validation_rejection};
use crate::runtime::handle::RuntimeHandle;
use crate::runtime::{Error, Result};
use crate::telemetry::{TelemetrySample, ValidationMode, check_normalized_telemetry_sample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

const PARTIAL_ACCEPTANCE: &str = "telemetry was partially accepted; unaccepted samples were dropped";

/* === Macros === */

/* === Type Aliases === */

pub(crate) type SampleIdentity<'a> =
    (Option<&'a dokime_domain::model::identity::TestExecutionId>, &'a dokime_domain::model::identity::TelemetrySampleId);

/* === Structs === */

// One Admitted Row In One Capture Scope. Every Lease Scope Shares The Original's Sample, So A
// Row Costs One Payload And One Value Map However Many Executions Capture It.
pub(crate) struct PreparedSample {
    pub(crate) sample: Arc<TelemetrySample>,
    pub(crate) lease: Option<RecordScope>,
    pub(crate) contract: usize,
    pub(crate) ordinal: u64,
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

/* === Enums === */

// How The Payloads Of One Admission Were Already Read.
pub(crate) enum Payloads {
    // Values Arrived Beside The Bytes; Admission Checks Each Original Payload Once.
    Unchecked,
    // Values Were Projected From The Bytes With This Registered Codec; No Further Pass Is Needed.
    Decoded(Arc<crate::storage::telemetry::TypedCodec>),
}

/* === Traits === */

/* === Implementations === */

impl PreparedSample {
    pub(crate) fn scope(&self) -> Option<&RecordScope> {
        self.lease.as_ref().or(self.sample.scope.as_ref())
    }

    pub(crate) fn original(&self) -> bool {
        self.lease.is_none()
    }
}

impl PreparedPublication {
    pub(crate) fn within_quiescence(mut self) -> Self {
        self.observes_closure = false;
        self
    }
}

impl RuntimeHandle {
    pub(crate) async fn prepare_sample_admission(&self, samples: Vec<TelemetrySample>, payloads: Payloads) -> Result<PreparedPublication> {
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
                let delivery = crate::domains::telemetry::leases::LeasedRows::attribute(delivery, state, &samples)?;
                let mut accepted = share_scopes(samples, &leased, |scope| state.capture_scope(scope).map_err(Error::Operation))?;
                let mut contracts = slots(leased.len())?;
                let mut indices = BTreeMap::new();
                for row in &mut accepted {
                    let index = if let Some(index) = indices.get(&row.sample.stream) {
                        *index
                    } else {
                        let contract = state
                            .telemetry_definitions
                            .get(&row.sample.stream)
                            .ok_or_else(|| validation_rejection("telemetry contract is unavailable", "stream", None))?;
                        let index = contracts.len();
                        contracts.push(contract.normalized());
                        indices.insert(row.sample.stream.clone(), index);
                        index
                    };
                    row.contract = index;
                    if !row.original() {
                        continue;
                    }
                    let contract = contracts.get(index).ok_or_else(|| invalid("telemetry contract index is missing"))?;
                    check_normalized_telemetry_sample(contract, &row.sample, ValidationMode::Strict)
                        .map_err(|error| validation_rejection(error.to_string(), "telemetry", None))?;
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
                let key = telemetry_stream_id(row.scope(), &row.sample.stream);
                receipt = receipt.try_map(|mut receipt| {
                    let next = receipt.stream_cursors.entry(key).or_insert_with_key(|key| known.values.get(key).copied().unwrap_or(0));
                    *next = next.checked_add(1).ok_or_else(|| invalid("telemetry ordinal exhausted"))?;
                    row.ordinal = *next;
                    Ok::<_, Error>(receipt)
                })?;
            }
        }
        let execution = self.execution_storage.prepare_telemetry(&accepted, &contracts, &payloads).await?;
        Ok(PreparedPublication {
            order,
            execution,
            accepted,
            supplied,
            receipt,
            delivery,
            observes_closure: true,
        })
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
        let PreparedPublication {
            order,
            execution,
            mut accepted,
            supplied,
            receipt,
            delivery,
            observes_closure,
        } = admission;
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
                let key = telemetry_stream_id(row.scope(), &row.sample.stream);
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
        // Delivery Is Queued Per Leaseholder Before Admission Opens, So Each Run Sees Rows In
        // Ordinal Order; Only This Publisher Then Waits For Its Own Rows To Be Taken.
        let delivered = self.queue_leased_rows(delivery, &accepted, &identities).await;
        drop(order);
        for delivered in delivered {
            let _ = delivered.await;
        }
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

/* === Public Functions === */

pub(super) fn dropped_receipt(dropped: u64, revision: u64, message: &'static str) -> Result<ReadBatch<PublishReceipt>> {
    harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(PublishReceipt::dropped(dropped, revision, message)))
}

pub(crate) fn identity_index(samples: &[PreparedSample]) -> Result<Vec<SampleIdentity<'_>>> {
    let mut identities = slots(samples.len())?;
    identities.extend(samples.iter().map(|row| (row.scope().and_then(|scope| scope.test_execution_id.as_ref()), &row.sample.sample_id)));
    identities.sort_unstable();
    identities.dedup();
    Ok(identities)
}

// Captures Each Original Once And Attributes It To Every Accepting Lease Scope By Reference.
// Lease Matching Reads The Supplied Scope; Stored Rows Carry The Captured Ones.
pub(crate) fn share_scopes(
    samples: Vec<TelemetrySample>, leased: &BTreeMap<Identifier, Vec<RecordScope>>,
    mut capture: impl FnMut(&RecordScope) -> Result<RecordScope>,
) -> Result<Vec<PreparedSample>> {
    let mut count = samples.len();
    for sample in &samples {
        count = count.checked_add(accepting(leased, sample).count()).ok_or_else(overflow)?;
    }
    let mut rows = slots(count)?;
    for mut sample in samples {
        sample.captured_scopes.clear();
        let mut leases = slots(accepting(leased, &sample).count())?;
        for scope in accepting(leased, &sample) {
            leases.push(capture(scope)?);
        }
        if let Some(scope) = &sample.scope {
            sample.scope = Some(capture(scope)?);
        }
        let sample = Arc::new(sample);
        rows.push(PreparedSample {
            sample: Arc::clone(&sample),
            lease: None,
            contract: 0,
            ordinal: 0,
        });
        for lease in leases {
            rows.push(PreparedSample {
                sample: Arc::clone(&sample),
                lease: Some(lease),
                contract: 0,
                ordinal: 0,
            });
        }
    }
    Ok(rows)
}

pub(crate) fn slots<T>(length: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(length).map_err(|_| invalid("telemetry staging allocation failed"))?;
    Ok(values)
}

/* === Private Functions === */

fn prepared_receipt() -> Result<ReadBatch<PublishReceipt>> {
    harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut receipt = PublishReceipt::accepted(0, BTreeMap::new(), 0);
        receipt.message = Some(PARTIAL_ACCEPTANCE.to_owned());
        Ok::<_, Error>(receipt)
    })
}

// The Lease Scopes That Capture An Unowned Sample; A Sample Owned By An Execution Captures Only There.
fn accepting<'a>(leased: &'a BTreeMap<Identifier, Vec<RecordScope>>, sample: &'a TelemetrySample) -> impl Iterator<Item = &'a RecordScope> {
    let unowned = sample.scope.as_ref().is_none_or(|scope| scope.test_execution_id.is_none());
    let scopes = leased.get(&sample.stream).into_iter().flatten();
    scopes.filter(move |scope| unowned && lease_accepts_sample(sample.scope.as_ref(), scope))
}

fn overflow() -> Error {
    invalid("telemetry staging memory overflows")
}

fn invalid(reason: &str) -> Error {
    Error::Operation(reason.to_owned())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Payloads, share_scopes};
    use crate::storage::telemetry::{PAYLOAD_PASSES, codec};
    use crate::telemetry::{TelemetryContract, TelemetrySample};
    use dokime_domain::model::execution::RecordScope;
    use dokime_domain::model::identity::{SessionId, TestExecutionId};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[harmos::message]
    struct Reading {
        value: String,
    }

    fn passes() -> usize {
        PAYLOAD_PASSES.with(std::cell::Cell::take)
    }

    // One Unowned Sample Captured By Several Executions, Each Under Its Own Lease Scope.
    fn leased(
        scopes: usize,
    ) -> (TelemetryContract, TelemetrySample, BTreeMap<dokime_domain::model::identity::Identifier, Vec<RecordScope>>) {
        let contract = TelemetryContract::declared("fixture.shared", "Shared").typed::<Reading>();
        let timestamp = dokime_domain::model::time::Timestamp::new("2026-01-01T00:00:00Z");
        let sample = TelemetrySample::new(crate::model::identity::new_id(), contract.stream.clone(), 1, timestamp, "fixture.shared")
            .typed(&Reading { value: "7".into() });
        let holders = (0..scopes)
            .map(|_| RecordScope {
                test_execution_id: Some(crate::model::identity::new_id::<TestExecutionId>()),
                ..Default::default()
            })
            .collect();
        let leased = BTreeMap::from([(contract.stream.clone(), holders)]);
        (contract, sample, leased)
    }

    #[test]
    fn lease_scopes_share_one_sample() {
        for scopes in [1, 8, 64] {
            let (_, sample, leased) = leased(scopes);
            let rows = share_scopes(vec![sample], &leased, |scope| Ok(scope.clone())).unwrap();
            assert_eq!(rows.len(), scopes + 1);
            assert!(rows[0].original() && rows[1..].iter().all(|row| !row.original()));
            assert!(rows.iter().all(|row| Arc::ptr_eq(&row.sample, &rows[0].sample)));
            assert_eq!(Arc::strong_count(&rows[0].sample), scopes + 1);
            assert!(rows[1..].iter().zip(&leased.values().next().unwrap()[..]).all(|(row, scope)| row.scope() == Some(scope)));
        }
    }

    #[test]
    fn admission_reads_each_payload_once() {
        let settings = harmos::config::SchemaSettings::default();
        let epoch = SessionId::from_seed("once.epoch");
        let (contract, sample, leased) = leased(8);
        let registered = Arc::new(codec(&contract, &settings).unwrap());
        let codecs = [Arc::clone(&registered)];

        // Values Beside The Bytes: Only The Original's Payload Is Checked, Never A Lease Row's.
        let rows = share_scopes(vec![sample.clone()], &leased, |scope| Ok(scope.clone())).unwrap();
        passes();
        let staged = crate::runtime::storage::telemetry::stage_rows(&epoch, &rows, &codecs, &Payloads::Unchecked).unwrap();
        assert_eq!((staged.len(), passes()), (9, 1));

        // Values Projected From The Bytes: That Projection Is The Row's Only Pass.
        let mut projected = sample;
        projected.values = registered.decode_payload(projected.storage_payload.as_deref().unwrap()).unwrap();
        let rows = share_scopes(vec![projected], &leased, |scope| Ok(scope.clone())).unwrap();
        let staged = crate::runtime::storage::telemetry::stage_rows(&epoch, &rows, &codecs, &Payloads::Decoded(registered)).unwrap();
        assert_eq!((staged.len(), passes()), (9, 1));
    }
}
