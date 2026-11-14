/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Queries Every Plot Kind

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{encoding::Message, v1};
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::plots::snapshots_client::SnapshotsClient;
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
async fn client_queries_plot_kinds() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Retain Numeric Coordinates, Categories and a Known Sampled Signal
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::model::Declarations {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: ["cells", "wave"]
                .into_iter()
                .map(|id| harmos_protocol::model::StreamDeclaration {
                    id: id.into(),
                    row: "dokime.host.v1.plots.PlotDomain".into(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "kinds".into(),
            cycle: "plot-kinds".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(dokime_protocol::model::Origin {
                kind: Some(model::origin::Kind::Import(dokime_protocol::model::Import::default())),
            }),
            ..Default::default()
        })?;
        for stream in ["cells", "wave"] {
            store.open_stream("kinds", stream)?;
            let samples = (0..32)
                .map(|index| Sample {
                    stream: stream.into(),
                    source_ns: index * 1_000_000_000,
                    sequence: index as u64 + 1,
                    row: if stream == "cells" {
                        dokime_protocol::host::plots::PlotDomain {
                            min: (index % 2) as f64,
                            max: (index % 3) as f64,
                        }
                    } else {
                        dokime_protocol::host::plots::PlotDomain {
                            min: (std::f64::consts::TAU * index as f64 / 8.0).sin(),
                            max: index as f64,
                        }
                    }
                    .encode_to_vec(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            store.sample_batch("kinds", &samples)?;
        }
        store.close_streams("kinds")?;
        store.close("kinds", dokime_protocol::model::Termination::Completed, dokime_protocol::host::ExecutionOutcome::default())?;
        store.seal("kinds")?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut plots = dokime_protocol::host::plots::snapshots_client::SnapshotsClient::new(channel);
        let scope = Some(dokime_protocol::model::EvidenceScope {
            execution_id: Some("kinds".into()),
            ..Default::default()
        });
        let request = dokime_protocol::host::plots::PlotRequest {
            scope,
            stream: "cells".into(),
            y_keys: vec!["max".into()],
            max_points: Some(512),
            ..Default::default()
        };
        let reference = |field: &str| dokime_protocol::model::DataExpression {
            kind: Some(model::data_expression::Kind::Stream(dokime_protocol::model::FieldReference {
                identifier: "cells".into(),
                field: field.into(),
            })),
            ..Default::default()
        };

        // Query the Ten Landed Kinds Through the Real Client Service
        for kind in [
            dokime_protocol::model::PlotKind::TimeSeries,
            dokime_protocol::model::PlotKind::Bar,
            dokime_protocol::model::PlotKind::Pie,
            dokime_protocol::model::PlotKind::Xy,
            dokime_protocol::model::PlotKind::Trajectory,
            dokime_protocol::model::PlotKind::FrequencySeries,
            dokime_protocol::model::PlotKind::Histogram,
            dokime_protocol::model::PlotKind::Contour,
            dokime_protocol::model::PlotKind::GridChart,
            dokime_protocol::model::PlotKind::BoxPlot,
        ] {
            let mut selected = dokime_protocol::host::plots::PlotRequest {
                plot_type: kind as i32,
                plot_id: Some(kind.as_str_name().into()),
                ..request.clone()
            };
            match kind {
                dokime_protocol::model::PlotKind::Bar
                | dokime_protocol::model::PlotKind::Xy
                | dokime_protocol::model::PlotKind::Trajectory => selected.x_key = Some("min".into()),
                dokime_protocol::model::PlotKind::Pie => {
                    selected.x_key = Some("min".into());
                    selected.y_keys.clear();
                    selected.y_expressions.push(dokime_protocol::model::DataExpression {
                        kind: Some(model::data_expression::Kind::Aggregate(Box::new(dokime_protocol::model::AggregateExpression {
                            input: Some(Box::new(reference("max"))),
                            kind: dokime_protocol::model::Aggregate::Sum as i32,
                            ..Default::default()
                        }))),
                        ..Default::default()
                    });
                }
                dokime_protocol::model::PlotKind::Contour => {
                    selected.x_key = Some("min".into());
                    selected.z_field = Some("max".into());
                    selected.y_keys = vec!["max".into()];
                }
                dokime_protocol::model::PlotKind::GridChart => {
                    selected.grid = Some(dokime_protocol::model::GridDefinition {
                        row: Some(reference("min")),
                        column: Some(reference("max")),
                        ..Default::default()
                    });
                    selected.reduction = Some(dokime_protocol::model::Reduction::Mean as i32);
                }
                dokime_protocol::model::PlotKind::FrequencySeries => {
                    selected.stream = "wave".into();
                    selected.y_keys = vec!["min".into()];
                    selected.fft = Some(dokime_protocol::model::PlotFftOptions {
                        sample_rate_hz: Some(1.0),
                        window: dokime_protocol::model::FftWindow::Rectangular as i32,
                        detrend: dokime_protocol::model::FftDetrend::None as i32,
                        ..Default::default()
                    });
                }
                _ => (),
            }
            let envelope = dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
                .await?
                .list(dokime_protocol::host::plots::SnapshotsListRequest {
                    requests: vec![selected],
                    ..Default::default()
                })
                .await?
                .into_inner()
                .envelopes
                .remove(0);
            assert_eq!(envelope.diagnostics.as_ref().unwrap().examined_rows, 32, "{kind:?}");
            let payload = envelope.payload.unwrap();
            if kind == dokime_protocol::model::PlotKind::FrequencySeries {
                let column = &payload.frame.unwrap().series[0];
                let peak = column
                    .y
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.value.unwrap().total_cmp(&right.value.unwrap()))
                    .unwrap()
                    .0;
                assert!((column.y[peak].value.unwrap() - 1.0).abs() < 1e-10);
                assert_eq!(column.x[peak].kind, Some(model::value::Kind::Number(0.125)));
            } else if matches!(
                kind,
                dokime_protocol::model::PlotKind::Histogram
                    | dokime_protocol::model::PlotKind::BoxPlot
                    | dokime_protocol::model::PlotKind::Contour
                    | dokime_protocol::model::PlotKind::GridChart
            ) {
                assert!(payload.analysis.is_some());
            } else {
                let frame = payload.frame.unwrap();
                assert!(frame.point_count > 0);
                assert!(!frame.signature.is_empty());
            }
        }

        // Keep Grouped Series Paired with Their Labels and Raw Counts
        let grouped = dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::host::plots::SnapshotsListRequest {
                requests: vec![dokime_protocol::host::plots::PlotRequest {
                    plot_type: dokime_protocol::model::PlotKind::Bar as i32,
                    x_key: Some("min".into()),
                    series_by: Some("max".into()),
                    arrangement: Some(dokime_protocol::model::BarArrangement::Grouped as i32),
                    ..request.clone()
                }],
                ..Default::default()
            })
            .await?
            .into_inner();
        let payload = grouped.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(payload.series.len(), 3);
        assert_eq!(payload.series.iter().map(|series| series.raw_points).sum::<u64>(), 32);
        assert_eq!(payload.frame.as_ref().unwrap().series.len(), 3);

        // The Nested Watch Opens at the Same Fixed Selection.
        let mut watch = plots
            .watch(dokime_protocol::host::plots::SnapshotsWatchRequest {
                requests: vec![dokime_protocol::host::plots::PlotRequest {
                    plot_type: dokime_protocol::model::PlotKind::TimeSeries as i32,
                    ..request.clone()
                }],
                ..Default::default()
            })
            .await?
            .into_inner();
        let opening = watch.message().await?.unwrap();
        assert!(opening.opening);
        assert!(!opening.updates.is_empty());
        drop(watch);
        // Refuse Ambiguous Totals and Invalid Pie Options Before Returning a Plot
        let duplicate = dokime_protocol::host::plots::PlotRequest {
            plot_type: dokime_protocol::model::PlotKind::Pie as i32,
            x_key: Some("min".into()),
            ..request.clone()
        };
        assert!(
            dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
                .await?
                .list(dokime_protocol::host::plots::SnapshotsListRequest {
                    requests: vec![duplicate],
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
                .await?
                .list(dokime_protocol::host::plots::SnapshotsListRequest {
                    requests: vec![dokime_protocol::host::plots::PlotRequest {
                        plot_type: dokime_protocol::model::PlotKind::Pie as i32,
                        hole: Some(1.0),
                        ..request
                    }],
                    ..Default::default()
                })
                .await
                .is_err()
        );
        client.close();
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}

#[tokio::test]
async fn sealed_plot_watch_returns_its_query_and_completes() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Store One Frozen Numeric Row.
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::model::Declarations {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::model::StreamDeclaration {
                id: "sensor".into(),
                row: "dokime.host.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "watch-execution".into(),
            cycle: "watch-cycle".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(dokime_protocol::model::Origin {
                kind: Some(model::origin::Kind::Import(dokime_protocol::model::Import::default())),
            }),
            ..Default::default()
        })?;
        store.open_stream("watch-execution", "sensor")?;
        store.sample_batch(
            "watch-execution",
            &[Sample {
                stream: "sensor".into(),
                sequence: 1,
                source_ns: 0,
                row: dokime_protocol::host::plots::PlotDomain { min: 12.0, max: 34.0 }.encode_to_vec(),
                ..Default::default()
            }],
        )?;
        store.close_streams("watch-execution")?;
        store.close(
            "watch-execution",
            dokime_protocol::model::Termination::Completed,
            dokime_protocol::host::ExecutionOutcome::default(),
        )?;
        store.seal("watch-execution")?;

        // Open the Runtime Watch Through the Same Query Path as the Desktop.
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut client = SnapshotsClient::connect(process.url.clone()).await?;
        let mut watch = client
            .watch(SnapshotsWatchRequest {
                requests: vec![dokime_protocol::host::plots::PlotRequest {
                    plot_id: Some("sensor-plot".into()),
                    stream: "sensor".into(),
                    y_keys: vec!["min".into()],
                    plot_type: dokime_protocol::model::PlotKind::TimeSeries as i32,
                    scope: Some(dokime_protocol::model::EvidenceScope {
                        execution_id: Some("watch-execution".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .await?
            .into_inner();
        let opening = watch.message().await?.ok_or("Watch Plot: Missing Opening Query")?;
        assert!(opening.opening);
        assert!(opening.resync_required.is_none());
        assert_eq!(opening.updates.len(), 1);
        let envelope = opening.updates[0].envelope.as_ref().unwrap();
        assert_eq!(envelope.plot_id, "sensor-plot");
        assert_eq!(envelope.diagnostics.as_ref().unwrap().examined_rows, 1);
        let summary = &envelope.payload.as_ref().unwrap().summaries[0];
        assert_eq!(
            (
                summary.count,
                summary.min.as_ref().and_then(|value| value.kind.as_ref()),
                summary.max.as_ref().and_then(|value| value.kind.as_ref())
            ),
            (1, Some(&model::value::Kind::Number(12.0)), Some(&model::value::Kind::Number(12.0)))
        );
        assert!(watch.message().await?.is_none());

        // Close the Host Before Its Disposable Root Removes Credentials.
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
