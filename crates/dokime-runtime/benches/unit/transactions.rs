/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Transaction Benchmarks: Folding Record Publications Into Runtime State

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use harmos::state::{Records, Transaction};

use dokime_domain::model;
use {
    dokime_runtime::state::RuntimeState, dokime_runtime::transactions::PublishRecord, model::records::product::ProductCalibrationDetails,
    model::records::product::ProductCalibrationKind, model::records::product::ProductCalibrationPhase,
};

pub fn benchmark(criterion: &mut Criterion) {
    let folded = fold(prepared(500));
    assert_eq!(folded.record_count, 500);

    let mut group = criterion.benchmark_group("transactions_fold_records");
    for count in [500_u64, 1_000, 2_000] {
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter_batched(|| prepared(count), |input| black_box(fold(input)), BatchSize::LargeInput);
        });
    }
    group.finish();
}

fn prepared(count: u64) -> (RuntimeState, Vec<PublishRecord>) {
    let mut state = RuntimeState::new(dokime_runtime::model::identity::new_id());
    state.install_builtin_domains().unwrap();
    let transactions = (0..count)
        .map(|_| {
            PublishRecord::host(
                ProductCalibrationDetails::new("benchmark-charger", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                    .into_draft("benchmark.charger")
                    .into(),
                dokime_runtime::model::time::now_utc(),
            )
        })
        .collect();
    (state, transactions)
}

fn fold((mut state, transactions): (RuntimeState, Vec<PublishRecord>)) -> RuntimeState {
    for transaction in transactions {
        transaction.check(&state).unwrap();
        let mut records = Records::default();
        transaction.records(&state, &mut records);
        black_box(&records);
        transaction.apply(&mut state);
    }
    state
}
