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
async fn client_queries_plot_kinds() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Retain Numeric Coordinates, Categories and a Known Sampled Signal
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: ["cells", "wave"]
                .into_iter()
                .map(|id| harmos_protocol::v1::StreamDeclaration {
                    id: id.into(),
                    row: "dokime.v1.PlotDomain".into(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "kinds".into(),
            cycle: "plot-kinds".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
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
                        v1::PlotDomain {
                            min: (index % 2) as f64,
                            max: (index % 3) as f64,
                        }
                    } else {
                        v1::PlotDomain {
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
        store.close("kinds", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("kinds")?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut plots = v1::plots_client::PlotsClient::new(channel);
        let scope = Some(v1::EvidenceScope {
            execution_id: Some("kinds".into()),
            ..Default::default()
        });
        let request = v1::PlotRequest {
            scope,
            stream: "cells".into(),
            y_keys: vec!["max".into()],
            max_points: Some(512),
            ..Default::default()
        };
        let reference = |field: &str| v1::DataExpression {
            kind: Some(v1::data_expression::Kind::Stream(v1::FieldReference {
                identifier: "cells".into(),
                field: field.into(),
            })),
            ..Default::default()
        };

        // Query the Ten Landed Kinds Through the Real Client Service
        for kind in [
            v1::PlotKind::TimeSeries,
            v1::PlotKind::Bar,
            v1::PlotKind::Pie,
            v1::PlotKind::Xy,
            v1::PlotKind::Trajectory,
            v1::PlotKind::FrequencySeries,
            v1::PlotKind::Histogram,
            v1::PlotKind::Contour,
            v1::PlotKind::GridChart,
            v1::PlotKind::BoxPlot,
        ] {
            let mut selected = v1::PlotRequest {
                plot_type: kind as i32,
                plot_id: Some(kind.as_str_name().into()),
                ..request.clone()
            };
            match kind {
                v1::PlotKind::Bar | v1::PlotKind::Xy | v1::PlotKind::Trajectory => selected.x_key = Some("min".into()),
                v1::PlotKind::Pie => {
                    selected.x_key = Some("min".into());
                    selected.y_keys.clear();
                    selected.y_expressions.push(v1::DataExpression {
                        kind: Some(v1::data_expression::Kind::Aggregate(Box::new(v1::AggregateExpression {
                            input: Some(Box::new(reference("max"))),
                            kind: v1::Aggregate::Sum as i32,
                            ..Default::default()
                        }))),
                        ..Default::default()
                    });
                }
                v1::PlotKind::Contour => {
                    selected.x_key = Some("min".into());
                    selected.z_field = Some("max".into());
                    selected.y_keys = vec!["max".into()];
                }
                v1::PlotKind::GridChart => {
                    selected.grid = Some(v1::GridDefinition {
                        row: Some(reference("min")),
                        column: Some(reference("max")),
                        ..Default::default()
                    });
                    selected.reduction = Some(v1::Reduction::Mean as i32);
                }
                v1::PlotKind::FrequencySeries => {
                    selected.stream = "wave".into();
                    selected.y_keys = vec!["min".into()];
                    selected.fft = Some(v1::PlotFftOptions {
                        sample_rate_hz: Some(1.0),
                        window: v1::FftWindow::Rectangular as i32,
                        detrend: v1::FftDetrend::None as i32,
                        ..Default::default()
                    });
                }
                _ => (),
            }
            let envelope =
                plots.query_snapshots(v1::PlotsQuerySnapshotsRequest { requests: vec![selected] }).await?.into_inner().envelopes.remove(0);
            assert_eq!(envelope.diagnostics.as_ref().unwrap().examined_rows, 32, "{kind:?}");
            let payload = envelope.payload.unwrap();
            if kind == v1::PlotKind::FrequencySeries {
                let column = &payload.frame.unwrap().series[0];
                let peak = column
                    .y
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.value.unwrap().total_cmp(&right.value.unwrap()))
                    .unwrap()
                    .0;
                assert!((column.y[peak].value.unwrap() - 1.0).abs() < 1e-10);
                assert_eq!(column.x[peak].kind, Some(v1::value::Kind::Number(0.125)));
            } else if matches!(kind, v1::PlotKind::Histogram | v1::PlotKind::BoxPlot | v1::PlotKind::Contour | v1::PlotKind::GridChart) {
                assert!(payload.analysis.is_some());
            } else {
                let frame = payload.frame.unwrap();
                assert!(frame.point_count > 0);
                assert!(!frame.signature.is_empty());
            }
        }

        // Keep Grouped Series Paired with Their Labels and Raw Counts
        let grouped = plots
            .query_snapshots(v1::PlotsQuerySnapshotsRequest {
                requests: vec![v1::PlotRequest {
                    plot_type: v1::PlotKind::Bar as i32,
                    x_key: Some("min".into()),
                    series_by: Some("max".into()),
                    arrangement: Some(v1::BarArrangement::Grouped as i32),
                    ..request.clone()
                }],
            })
            .await?
            .into_inner();
        let payload = grouped.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(payload.series.len(), 3);
        assert_eq!(payload.series.iter().map(|series| series.raw_points).sum::<u64>(), 32);
        assert_eq!(payload.frame.as_ref().unwrap().series.len(), 3);

        // Invoke Query and Watch Through Their Run Envelopes
        plots.list_analyses(v1::PlotsListAnalysesRequest::default()).await?;
        let mut run = plots
            .run_query_snapshots(tokio_stream::iter([v1::PlotsQuerySnapshotsRunInput {
                value: Some(v1::plots_query_snapshots_run_input::Value::Request(v1::PlotsQuerySnapshotsRequest {
                    requests: vec![v1::PlotRequest {
                        plot_type: v1::PlotKind::TimeSeries as i32,
                        ..request.clone()
                    }],
                })),
            }]))
            .await?
            .into_inner();
        assert!(matches!(run.message().await?.unwrap().value, Some(v1::plots_query_snapshots_run_output::Value::Response(_))));
        assert!(run.message().await?.is_none());
        let mut run = plots
            .run_watch(tokio_stream::iter([v1::PlotsWatchRunInput {
                value: Some(v1::plots_watch_run_input::Value::Request(v1::PlotsWatchRequest {
                    requests: vec![v1::PlotRequest {
                        plot_type: v1::PlotKind::TimeSeries as i32,
                        ..request.clone()
                    }],
                })),
            }]))
            .await?
            .into_inner();
        let mut terminal = false;
        while let Some(frame) = run.message().await? {
            terminal |= matches!(frame.value, Some(v1::plots_watch_run_output::Value::Response(_)));
        }
        assert!(terminal);

        // Refuse Ambiguous Totals and Invalid Pie Options Before Returning a Plot
        let duplicate = v1::PlotRequest {
            plot_type: v1::PlotKind::Pie as i32,
            x_key: Some("min".into()),
            ..request.clone()
        };
        assert!(plots.query_snapshots(v1::PlotsQuerySnapshotsRequest { requests: vec![duplicate] }).await.is_err());
        assert!(
            plots
                .query_snapshots(v1::PlotsQuerySnapshotsRequest {
                    requests: vec![v1::PlotRequest {
                        plot_type: v1::PlotKind::Pie as i32,
                        hole: Some(1.0),
                        ..request
                    }]
                })
                .await
                .is_err()
        );
        client.close();
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
