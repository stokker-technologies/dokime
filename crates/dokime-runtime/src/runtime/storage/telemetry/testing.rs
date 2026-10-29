/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved-Window And Follow-Frontier Volume Fixtures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
#[cfg(feature = "testing")]
use super::*;
use harmos::Application;
#[cfg(feature = "testing")]
use harmos::{Runtime, storage::StorageBackends};
#[cfg(feature = "testing")]
use std::sync::Arc;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) struct TestApp;
impl Application for TestApp {
    type Lanes = crate::app::Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
}

#[derive(Default, serde::Deserialize)]
pub(super) struct StorageConfig {
    #[serde(skip)]
    pub(super) storage: Option<crate::storage::Manager>,
}

impl harmos::Lifecycle for TestApp {
    type Config = StorageConfig;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;

    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().storage(config.storage.as_ref().expect("storage fixture is configured"));
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> harmos::Result<()> {
        Ok(())
    }
}

#[harmos::message]
pub(super) struct Reading {
    pub(super) value: String,
}

#[cfg(feature = "testing")]
pub async fn window(days: u64, per_day: u64) {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
    let local = backend.local_state();
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        crate::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary: backend, state }, crate::config::storage_defaults()).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let contract = crate::telemetry::TelemetryContract::declared("fixture.longrun", "Long Run").typed::<Reading>();
    let settings = harmos::config::SchemaSettings::default();
    let codec = telemetry::codec(&contract, &settings).unwrap();
    storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
    let epoch = dokime_domain::model::identity::SessionId::from_seed("longrun.epoch");
    let selection = crate::evidence_runtime::TelemetrySelection {
        streams: vec![contract.stream.clone().into()],
        ..Default::default()
    };
    let mut work = None;

    for day in 0..days {
        for start in (0..per_day).step_by(120) {
            let mut batch = Vec::new();
            for offset in 0..120_u64 {
                let minute = day * per_day + start + offset;
                let timestamp = time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(minute as i64);
                let sample = crate::telemetry::TelemetrySample::new(
                    crate::model::identity::new_id(),
                    contract.stream.clone(),
                    minute,
                    dokime_domain::model::time::Timestamp::from_datetime(timestamp),
                    "fixture.longrun",
                )
                .typed(&Reading { value: minute.to_string() });
                batch.push(prepare_staged(&epoch, &sample, None, minute, &codec).unwrap().into_input());
            }
            storage.write_stream(batch).await.unwrap();
        }
        let at = storage.flush().await.unwrap();
        assert_eq!(storage.status().pending_bytes, 0);
        let before = local.read_counters();
        let mut rows = Vec::new();
        let options = ReadOptions {
            window: Some(("fixture.longrun", 300.0)),
            ..Default::default()
        };
        visit_saved(&storage, &at, &selection, &settings, options, &local, |sample| {
            rows.push(sample);
            Ok(())
        })
        .await
        .unwrap();
        let after = local.read_counters();
        let scanned = after.data_scan_rows - before.data_scan_rows;
        assert_eq!(rows.len(), 6);
        assert_eq!(after.telemetry_anchor_rows - before.telemetry_anchor_rows, 1);
        assert!(scanned <= 16, "scanned {scanned} rows on day {day}");
        if let Some(previous) = work {
            assert_eq!(scanned, previous);
        }
        work = Some(scanned);
    }
    runtime.shutdown().await.unwrap();
}

#[cfg(feature = "testing")]
pub async fn follow(days: u64, per_day: u64) {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
    let local = backend.local_state();
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        crate::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary: backend, state }, crate::config::storage_defaults()).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let contract = crate::telemetry::TelemetryContract::declared("fixture.follow", "Follow").typed::<Reading>();
    let settings = harmos::config::SchemaSettings::default();
    let codec = telemetry::codec(&contract, &settings).unwrap();
    storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
    let epoch = dokime_domain::model::identity::SessionId::from_seed("follow.epoch");
    let streams = vec![contract.stream.clone().into()];
    let paged = crate::evidence_runtime::TelemetrySelection {
        streams: streams.clone(),
        limit: Some(100),
        ..Default::default()
    };
    let whole = crate::evidence_runtime::TelemetrySelection {
        streams,
        limit: Some(2_000),
        ..Default::default()
    };
    let (mut settled, mut work) = (0, None);

    // Each Daily Wake Follows From The Settled Position: Its Scan Covers That Day's Rows, Not The History Before.
    for day in 0..days {
        for start in (0..per_day).step_by(120) {
            let mut batch = Vec::new();
            for offset in 0..120_u64 {
                let minute = day * per_day + start + offset;
                let timestamp = time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(minute as i64);
                let sample = crate::telemetry::TelemetrySample::new(
                    crate::model::identity::new_id(),
                    contract.stream.clone(),
                    minute,
                    dokime_domain::model::time::Timestamp::from_datetime(timestamp),
                    "fixture.follow",
                )
                .typed(&Reading { value: minute.to_string() });
                batch.push(prepare_staged(&epoch, &sample, None, minute, &codec).unwrap().into_input());
            }
            storage.write_stream(batch).await.unwrap();
        }
        let at = storage.flush().await.unwrap();

        // A Limited Page Stops At The Limit And Settles Its Last Position.
        let options = ReadOptions { after: settled, ..Default::default() };
        let page = collect_page(&storage, &at, &paged, &settings, options, &local).await.unwrap();
        assert!(page.limited);
        assert_eq!(page.samples.len(), 100);
        assert_eq!(page.samples.first().map(|sample| sample.sequence), Some(day * per_day));
        let last = *page.positions.last().unwrap();
        assert!(page.positions.windows(2).all(|pair| pair[0] < pair[1]));

        let before = local.read_counters();
        let options = ReadOptions { after: last, ..Default::default() };
        let rest = collect_page(&storage, &at, &whole, &settings, options, &local).await.unwrap();
        let scanned = local.read_counters().data_scan_rows - before.data_scan_rows;
        assert!(!rest.limited);
        assert_eq!(rest.samples.len(), per_day as usize - 100);
        assert_eq!(rest.samples.first().map(|sample| sample.sequence), Some(day * per_day + 100));
        assert!(scanned <= per_day, "scanned {scanned} rows on day {day}");
        if let Some(previous) = work {
            assert_eq!(scanned, previous);
        }
        work = Some(scanned);
        settled = at.sequence;
    }
    runtime.shutdown().await.unwrap();
}
/* === Private Functions === */

/* === Tests === */
