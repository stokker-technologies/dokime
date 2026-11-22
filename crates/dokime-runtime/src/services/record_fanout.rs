// SPDX-License-Identifier: Apache-2.0

// Observer Records Reach Only Capturing Executions.

/* === Imports === */

use std::{
    error::Error,
    num::NonZeroUsize,
    sync::{Arc, atomic::AtomicU32},
    time::Duration,
};

use harmos_protocol::host::PluginEntry;
use harmos_protocol::policy::UNBOUNDED;
use prost::Message;
use prost_types::Timestamp;

use crate::{CaptureSettings, ControlledClock, Host, HostSettings, PlacementSettings};
use dokime_operations::host::{ClientSettings, evidence::records::append};
use dokime_operations::{Clock, Resources};
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::model::{Origin, PluginRun, TestCase, origin};
use dokime_storage::{Execution, storage_defaults};
use dokime_test_support::credentials;

/* === Structs === */

struct FixedClock;

/* === Implementations === */

impl Clock for FixedClock {
    fn now_ns(&self) -> i64 {
        1_000_000_000
    }
}

/* === Tests === */

#[test]
fn observer_records_fan_out_to_two_capturing_executions_only() -> Result<(), Box<dyn Error>> {
    credentials::run(Box::pin(async {
        // Start the Host With a Controlled Clock.
        let root = credentials::tempdir()?;
        let settings = HostSettings {
            installation: None,
            placement: PlacementSettings {
                reconnect_delay: Duration::from_secs(1),
                stream_window: NonZeroUsize::new(8).unwrap(),
                interval_window: crate::INTERVAL_WINDOW,
                interval_lag: crate::INTERVAL_LAG,
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            },
            storage: storage_defaults(),
            clock: Arc::new(ControlledClock::manual(1_000_000_000)),
            capture: CaptureSettings::default(),
            credentials: credentials::namespace(),
        };
        let operations = ClientSettings {
            read_timeout: crate::PLUGINS_TIMEOUT,
            mutation_timeout: crate::PLUGINS_TIMEOUT,
            watch_timeout: UNBOUNDED,
            max_cases: 1000,
            page_items: 50,
            message_bytes: settings.placement.message_bytes.get(),
            stream_buffer: settings.placement.stream_window.get(),
        };
        let host = Host::open(root.path(), settings, operations).await?;

        // Open Two Capturing Executions and One Unrelated Execution.
        for identifier in ["first", "second", "unrelated"] {
            host.store.create(&Execution {
                identifier: identifier.into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::PluginRun(PluginRun { plugin: "example".into(), template: "example.capture".into(), test_case: Some(TestCase::default()), digest: vec![1], started_at: Some(Timestamp::default()) })),
                }),
                ..Default::default()
            })?;
        }
        let resources = Resources {
            plugin: Arc::new(PluginEntry::default()),
            context: host.assembly.as_ref().unwrap().context.clone(),
            store: host.store.clone(),
            execution: "first".into(),
            capturing_executions: vec!["first".into(), "second".into()],
            step: String::new(),
            operation: "example.reading".into(),
            attempt: Arc::new(AtomicU32::new(1)),
            clock: Arc::new(FixedClock),
            prompts: host.prompts.clone(),
        };

        // Append and Replay One Observer Record Through the Host Handler.
        let request = RecordsAppendRequest {
            kind: "example.event".into(),
            payload: vec![8, 42],
            request_id: "event".into(),
            ..Default::default()
        };
        let first = append::handle(&resources, request.clone()).await?;
        let replay = append::handle(&resources, request).await?;
        assert_eq!(first, replay);

        // Inspect Every Execution's Persisted Evidence.
        for (execution, expected) in [("first", 1), ("second", 1), ("unrelated", 0)] {
            let cut = host.store.cut(execution)?;
            let rows = host.store.evidence_page(execution, 0, cut.frontier, NonZeroUsize::new(50).unwrap())?;
            assert_eq!(rows.len(), expected, "{execution}");
            if let Some(row) = rows.first() {
                let record = RecordsAppendRequest::decode(row.payload.as_slice())?;
                assert_eq!(record.execution, execution);
                assert_eq!(record.payload, vec![8, 42]);
            }
        }
        host.shutdown().await?;
        Ok(())
    }))
}
