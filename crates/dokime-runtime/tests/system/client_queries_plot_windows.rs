/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Queries Plot Windows

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error as StdError;

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{encoding::Message, v1};
use dokime_storage::{Execution, Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_queries_plot_windows() -> Result<(), Box<dyn StdError>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Retain More Than Two Million Samples as Ordinary Durable Evidence
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::v1::StreamDeclaration {
                id: "sensor".into(),
                row: "dokime.client.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "plot-execution".into(),
            cycle: "plot-cycle".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            ..Default::default()
        })?;
        store.open_stream("plot-execution", "sensor")?;
        let retained = 2_000_003_i64;
        for start in (0..retained).step_by(4096) {
            let samples = (start..(start + 4096).min(retained))
                .map(|index| Sample {
                    stream: "sensor".into(),
                    source_ns: index * 1_000_000_000,
                    sequence: index as u64 + 1,
                    row: dokime_protocol::client::plots::PlotDomain {
                        min: (index % 101) as f64,
                        max: index as f64,
                    }
                    .encode_to_vec(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            store.sample_batch("plot-execution", &samples)?;
        }
        store.close_streams("plot-execution")?;
        store.close("plot-execution", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("plot-execution")?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let scope = Some(v1::EvidenceScope {
            execution_id: Some("plot-execution".into()),
            ..Default::default()
        });
        let request = v1::PlotRequest {
            plot_type: v1::PlotKind::TimeSeries as i32,
            stream: "sensor".into(),
            y_keys: vec!["min".into()],
            scope,
            viewport: Some(v1::PlotWindow { from: 1_000_000.0, to: 1_000_999.0 }),
            panel_width: Some(32),
            max_points: Some(128),
            ..Default::default()
        };

        // Read Forty Plots Without a Count Cap or a Whole-Stream Read
        let requests = (0..40)
            .map(|index| v1::PlotRequest {
                plot_id: Some(format!("visible-{index}")),
                ..request.clone()
            })
            .collect::<Vec<_>>();
        let reply = dokime_protocol::client::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::plots::SnapshotsListRequest {
                requests: requests.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(reply.envelopes.len(), 40);
        let mut examined_rows = 0;
        for envelope in &reply.envelopes {
            let diagnostics = envelope.diagnostics.as_ref().unwrap();
            assert_eq!(diagnostics.examined_rows, 1000);
            assert!(diagnostics.point_count <= 128);
            assert!(diagnostics.resolution_nanos > 0);
            examined_rows += diagnostics.examined_rows;
            let payload = envelope.payload.as_ref().unwrap();
            assert_eq!(payload.summaries[0].count, 1000);
            assert_eq!(payload.summaries[0].min.as_ref().and_then(|value| value.kind.as_ref()), Some(&v1::value::Kind::Number(0.0)));
            assert_eq!(payload.summaries[0].max.as_ref().and_then(|value| value.kind.as_ref()), Some(&v1::value::Kind::Number(100.0)));
        }
        assert_eq!(examined_rows, 40_000);
        assert!(examined_rows < retained as u64);

        // Zoom In and Observe the Actual Finer Resolution
        let zoom = dokime_protocol::client::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::plots::SnapshotsListRequest {
                requests: vec![v1::PlotRequest {
                    viewport: Some(v1::PlotWindow { from: 1_000_000.0, to: 1_000_009.0 }),
                    ..request.clone()
                }],
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(zoom.envelopes[0].diagnostics.as_ref().unwrap().examined_rows, 10);
        assert!(
            zoom.envelopes[0].diagnostics.as_ref().unwrap().resolution_nanos
                < reply.envelopes[0].diagnostics.as_ref().unwrap().resolution_nanos
        );

        // Deliver the Visible Plot First and Complete the Sealed Watch
        let mut watch = dokime_protocol::client::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .watch(dokime_protocol::client::plots::SnapshotsWatchRequest { requests, ..Default::default() })
            .await?
            .into_inner();
        let mut delivered = Vec::new();
        while let Some(frame) = watch.message().await? {
            assert!(frame.opening);
            assert!(frame.resync_required.is_none());
            delivered.extend(frame.updates.into_iter().map(|update| update.envelope.unwrap().plot_id));
        }
        assert_eq!(delivered, (0..40).map(|index| format!("visible-{index}")).collect::<Vec<_>>());
        process.close().await?;
        Ok::<(), Box<dyn StdError>>(())
    })
    .await?
}
