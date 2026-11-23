// SPDX-License-Identifier: Apache-2.0

// A Client Follows a Long Stream and Receives Each Sample Once.

/* === Imports === */

use std::{error::Error, num::NonZeroUsize, ops::Range, sync::Arc, time::Duration};

use harmos_protocol::model::{Declarations, PageRequest, StreamDeclaration};
use harmos_protocol::policy::UNBOUNDED;
use prost::Message;
use tokio_stream::StreamExt;

use crate::{CaptureSettings, ControlledClock, Host, HostSettings, PlacementSettings};
use dokime_operations::host::ClientSettings;
use dokime_operations::host::evidence::samples::{list, watch};
use dokime_protocol::host::ExecutionOutcome;
use dokime_protocol::host::evidence::{SampleRow, SamplesListRequest, SamplesWatchRequest, TelemetrySample, sample_row};
use dokime_protocol::model::{EvidenceScope, Import, Origin, Termination, origin};
use dokime_storage::{Execution, Sample, storage_defaults};
use dokime_test_support::credentials;

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "fixture.follow")]
struct Level {
    value: u64,
    limit: u64,
}

/* === Private Functions === */

/// The Raw Samples of a Page, Each Checked to Hold Only the Selected `value` Field.
fn selected(rows: Vec<SampleRow>) -> Vec<TelemetrySample> {
    let samples = rows
        .into_iter()
        .filter_map(|row| match row.kind {
            Some(sample_row::Kind::Raw(sample)) => Some(sample),
            _ => None,
        })
        .collect::<Vec<_>>();
    for sample in &samples {
        assert_eq!(sample.values.as_ref().unwrap().values.keys().collect::<Vec<_>>(), ["value"]);
        assert_eq!(Level::decode(sample.payload.as_slice()).unwrap().limit, 0, "Read Only the Selected Field");
    }
    samples
}

/* === Tests === */

#[test]
fn client_follows_a_long_stream_without_receiving_it_again() -> Result<(), Box<dyn Error>> {
    credentials::run(Box::pin(async {
        // Start the Host With Fifty-Sample Pages.
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
        let context = &host.assembly.as_ref().unwrap().context;

        // Capture a History Longer Than One Page.
        let catalog = Declarations {
            types: Some(<Level as dokime::encoding::contract::Message>::types()),
            streams: vec![StreamDeclaration {
                id: "levels".into(),
                row: <Level as dokime::encoding::contract::Message>::name(),
                ..Default::default()
            }],
            ..Default::default()
        };
        host.store.create(&Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            catalog: catalog.encode_to_vec(),
            ..Default::default()
        })?;
        host.store.open_stream("execution", "levels")?;
        let capture = |sequences: Range<u64>| {
            let samples = sequences
                .map(|sequence| Sample {
                    stream: "levels".into(),
                    sequence,
                    source_ns: sequence as i64 * 1_000_000,
                    row: Level { value: sequence, limit: 100 }.encode_to_vec(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            host.store.sample_batch("execution", &samples)?;
            host.store.flush_samples()
        };
        capture(0..120)?;

        // List Only the Selected Field; an Undeclared Field Is Refused.
        let scope = Some(EvidenceScope {
            execution_id: Some("execution".into()),
            ..Default::default()
        });
        let request = SamplesListRequest {
            scope: scope.clone(),
            fields: vec!["value".into()],
            ..Default::default()
        };
        let listed = selected(list::handle(context, request.clone()).await?.samples);
        assert_eq!(listed.iter().map(|sample| sample.sequence).collect::<Vec<_>>(), (0..50).collect::<Vec<_>>());
        let undeclared = SamplesListRequest {
            fields: vec!["missing".into()],
            ..request
        };
        assert!(list::handle(context, undeclared).await.is_err());

        // Follow Every Sample, and Separately Only the Newest Two.
        let request = SamplesWatchRequest {
            scope,
            fields: vec!["value".into()],
            ..Default::default()
        };
        let mut every = watch::handle(context, request.clone()).await?;
        let latest = SamplesWatchRequest {
            latest: true,
            page: Some(PageRequest { page_size: 2, ..Default::default() }),
            ..request
        };
        let mut newest = watch::handle(context, latest).await?;

        // The Opening Pages Carry the History Once, the Latest Opening Only Its Newest Page.
        let mut opening = Vec::new();
        while opening.len() < 120 {
            let frame = every.next().await.unwrap()?;
            assert!(frame.opening);
            opening.extend(selected(frame.samples).iter().map(|sample| sample.sequence));
        }
        assert_eq!(opening, (0..120).collect::<Vec<_>>());
        let frame = newest.next().await.unwrap()?;
        assert!(frame.opening);
        assert_eq!(selected(frame.samples).iter().map(|sample| sample.sequence).collect::<Vec<_>>(), [118, 119]);

        // Later Frames Carry Only the Samples Captured After the Opening, in Order, Even Across Pages.
        capture(120..180)?;
        let mut followed = Vec::new();
        while followed.len() < 60 {
            let frame = every.next().await.unwrap()?;
            assert!(!frame.opening && frame.resync_required.is_none());
            followed.extend(selected(frame.samples).iter().map(|sample| sample.sequence));
        }
        assert_eq!(followed, (120..180).collect::<Vec<_>>());
        let mut tail = Vec::new();
        while tail.last() != Some(&179) {
            let frame = newest.next().await.unwrap()?;
            assert!(!frame.opening && frame.resync_required.is_none());
            tail.extend(selected(frame.samples).iter().map(|sample| sample.sequence));
        }
        assert!(tail.is_sorted() && tail.iter().all(|sequence| (120..180).contains(sequence)), "{tail:?}");

        // Closing the Execution Ends Both Watches Without Sending Any Sample Again.
        host.store.close_streams("execution")?;
        host.store.close("execution", Termination::Completed, ExecutionOutcome::default())?;
        for frames in [&mut every, &mut newest] {
            while let Some(frame) = frames.next().await {
                assert!(frame?.samples.is_empty());
            }
        }
        drop((every, newest));
        host.shutdown().await?;
        Ok(())
    }))
}
