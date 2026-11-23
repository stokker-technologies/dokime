/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Live Plot Pages Retain Views Only When a Later Read Needs Them

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error, num::NonZeroUsize, ops::RangeInclusive, path::Path, sync::Arc, time::Duration};

// Third Party Imports
use harmos_protocol::model::{Declarations, PageRequest, StreamDeclaration};
use harmos_protocol::policy::UNBOUNDED;
use prost::Message;
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use tokio_stream::StreamExt;

// Local Imports
use crate::{CaptureSettings, ControlledClock, Host, HostSettings, PlacementSettings};
use dokime_operations::Error;
use dokime_operations::host::ClientSettings;
use dokime_operations::host::documents::plot;
use dokime_operations::host::plots::snapshots::{list, watch};
use dokime_protocol::builders::Plot;
use dokime_protocol::host::plots::{PlotRequest, SnapshotsListRequest, SnapshotsWatchRequest};
use dokime_protocol::model::{EvidenceScope, FieldReference, Import, Origin, origin};
use dokime_storage::{Execution, Sample, storage_defaults};
use dokime_test_support::credentials;

/* === Constants === */

/// Plots per Page, So Three Plots Span Two Pages.
const PAGE_ITEMS: usize = 2;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Reading {
    #[prost(uint64, tag = "1")]
    value: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/// An Open Execution Declaring One Signal Stream of Unsigned Readings.
fn execution() -> Execution {
    let field = FieldDescriptorProto {
        name: Some("value".into()),
        number: Some(1),
        label: Some(Label::Optional as i32),
        r#type: Some(Type::Uint64 as i32),
        ..Default::default()
    };
    let types = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("views.proto".into()),
            package: Some("views".into()),
            syntax: Some("proto3".into()),
            message_type: vec![DescriptorProto {
                name: Some("Reading".into()),
                field: vec![field],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let catalog = Declarations {
        types: Some(types),
        streams: vec![StreamDeclaration {
            id: "signal".into(),
            row: "views.Reading".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    Execution {
        identifier: "execution".into(),
        cycle: "cycle".into(),
        origin: Some(Origin {
            kind: Some(origin::Kind::Import(Import::default())),
        }),
        catalog: catalog.encode_to_vec(),
        ..Default::default()
    }
}

/// One Signal Reading per Second at Each Sequence.
fn samples(sequences: RangeInclusive<u64>) -> Vec<Sample> {
    sequences
        .map(|sequence| Sample {
            sequence,
            source_ns: sequence as i64 * 1_000_000_000,
            receipt_ns: sequence as i64 * 1_000_000_000,
            stream: "signal".into(),
            row: Reading { value: sequence }.encode_to_vec(),
            ..Default::default()
        })
        .collect()
}

/// Live Time Series of the Signal, Identified by a Prefix and Their Order.
fn plots(prefix: &str, count: usize) -> Vec<PlotRequest> {
    let scope = EvidenceScope {
        execution_id: Some("execution".into()),
        ..Default::default()
    };
    let request = plot::request(&Plot::time_series(FieldReference::new("signal").field("value")).message, scope).unwrap();
    (0..count)
        .map(|index| PlotRequest {
            plot_id: Some(format!("{prefix}.{index}")),
            window_s: Some(300.0),
            ..request.clone()
        })
        .collect()
}

/// Retained View Files in the Storage Root.
fn views(root: &Path) -> Result<i64, Box<dyn error::Error>> {
    let database = rusqlite::Connection::open(root.join("storage.sqlite3"))?;
    Ok(database.query_row("SELECT COUNT(*) FROM storage_files WHERE id LIKE 'view/%'", [], |row| row.get(0))?)
}

/* === Tests === */

#[test]
fn live_plot_pages_retain_views_only_when_a_later_read_needs_them() -> Result<(), Box<dyn error::Error>> {
    credentials::run(Box::pin(async {
        // Start a Host Serving Two Plots per Page
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
            page_items: PAGE_ITEMS,
            message_bytes: settings.placement.message_bytes.get(),
            stream_buffer: settings.placement.stream_window.get(),
        };
        let host = Host::open(root.path(), settings, operations).await?;
        let context = host.assembly.as_ref().unwrap().context.clone();

        // Capture One Live Signal
        host.store.create(&execution())?;
        host.store.open_streams("execution", &["signal".into()])?;
        host.store.sample_batch("execution", &samples(1..=10))?;
        let base = views(root.path())?;

        // Watch Pages That Fit One Page Retain No View, Opening or Refreshing
        let request = SnapshotsWatchRequest {
            requests: plots("single", 1),
            ..Default::default()
        };
        let mut frames = watch::handle(&context, request).await?;
        let opening = frames.next().await.unwrap()?;
        host.store.sample_batch("execution", &samples(11..=20))?;
        let refreshed = frames.next().await.unwrap()?;
        drop(frames);
        assert!(opening.opening && !refreshed.opening);
        assert_eq!((opening.updates.len(), refreshed.updates.len()), (1, 1));
        assert_eq!(views(root.path())? - base, 0);

        // A Later Read at Their Cut Asks to Resynchronize
        let request = SnapshotsListRequest {
            requests: plots("single", 1),
            at: refreshed.resolved_at,
            page: None,
        };
        let stale = list::handle(&context, request).await;
        assert!(matches!(stale, Err(Error::Precondition(_))), "{stale:?}");

        // A Watch Reply Spanning Pages Retains Its View and Continues at Its Cut
        let request = SnapshotsWatchRequest {
            requests: plots("watched", 3),
            ..Default::default()
        };
        let mut frames = watch::handle(&context, request).await?;
        let first = frames.next().await.unwrap()?;
        let second = frames.next().await.unwrap()?;
        drop(frames);
        assert_eq!((first.updates.len(), second.updates.len()), (PAGE_ITEMS, 1));
        assert_eq!(first.resolved_at, second.resolved_at);
        assert_eq!(views(root.path())? - base, 1);

        // A List Retains Even One Page, So Its Cut Reads the Same Reply Again
        let request = SnapshotsListRequest {
            requests: plots("listed", 1),
            at: None,
            page: None,
        };
        let single = list::handle(&context, request).await?;
        assert_eq!(views(root.path())? - base, 2);
        let request = SnapshotsListRequest {
            requests: plots("listed", 1),
            at: single.resolved_at.clone(),
            page: None,
        };
        assert_eq!(list::handle(&context, request).await?, single);

        // A List Spanning Pages Continues at Its Cut
        let request = SnapshotsListRequest {
            requests: plots("paged", 3),
            at: None,
            page: None,
        };
        let first = list::handle(&context, request).await?;
        assert_eq!(first.envelopes.len(), PAGE_ITEMS);
        let request = SnapshotsListRequest {
            requests: plots("paged", 3),
            at: first.resolved_at.clone(),
            page: Some(PageRequest {
                page_token: first.next_page_token.clone(),
                ..Default::default()
            }),
        };
        let second = list::handle(&context, request).await?;
        assert_eq!(second.envelopes.iter().map(|envelope| envelope.plot_id.as_str()).collect::<Vec<_>>(), ["paged.2"]);
        assert!(second.next_page_token.is_empty());
        assert_eq!(second.resolved_at, first.resolved_at);
        assert_eq!(views(root.path())? - base, 3);

        // Stop the Host
        host.shutdown().await?;
        Ok(())
    }))
}
