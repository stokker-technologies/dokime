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
use dokime::Aggregate;
use dokime::AggregateExpression;
use dokime::BarArrangement;
use dokime::Client;
use dokime::DataExpression;
use dokime::EvidenceScope;
use dokime::FftDetrend;
use dokime::FftWindow;
use dokime::FieldReference;
use dokime::GridDefinition;
use dokime::Import;
use dokime::Origin;
use dokime::PlotFftOptions;
use dokime::PlotKind;
use dokime::Reduction;
use dokime::Termination;
use dokime::client::ExecutionOutcome;
use dokime::client::PlotsPlotDomain;
use dokime::client::PlotsPlotRequest;
use dokime::encoding::Message;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::model::data_expression::Kind as DataExpressionKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_storage::Execution;
use dokime_storage::Sample;
use dokime_storage::Store;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::StreamDeclaration;
use prost_types::FileDescriptorSet;
use std::error::Error;
use std::f64::consts::TAU;
use tonic::transport::Endpoint;

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

#[test]
fn client_queries_plot_kinds() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Retain Numeric Coordinates, Categories and a Known Sampled Signal
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            let catalog = Declarations {
                types: Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),
                streams: ["cells", "wave"]
                    .into_iter()
                    .map(|id| StreamDeclaration {
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
                origin: Some(Origin {
                    kind: Some(OriginKind::Import(Import::default())),
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
                            PlotsPlotDomain {
                                min: (index % 2) as f64,
                                max: (index % 3) as f64,
                            }
                        } else {
                            PlotsPlotDomain {
                                min: (TAU * index as f64 / 8.0).sin(),
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
            store.close("kinds", Termination::Completed, ExecutionOutcome::default())?;
            store.seal("kinds")?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let plots = Client::from_channel(channel, super::support::client_settings())?.plots.snapshots.clone();
            let scope = Some(EvidenceScope {
                execution_id: Some("kinds".into()),
                ..Default::default()
            });
            let request = PlotsPlotRequest {
                scope,
                stream: "cells".into(),
                y_keys: vec!["max".into()],
                max_points: Some(512),
                ..Default::default()
            };
            let reference = |field: &str| DataExpression {
                kind: Some(DataExpressionKind::Stream(FieldReference {
                    identifier: "cells".into(),
                    field: field.into(),
                })),
                ..Default::default()
            };

            // Query the Ten Landed Kinds Through the Real Client Service
            for kind in [
                PlotKind::TimeSeries,
                PlotKind::Bar,
                PlotKind::Pie,
                PlotKind::Xy,
                PlotKind::Trajectory,
                PlotKind::FrequencySeries,
                PlotKind::Histogram,
                PlotKind::Contour,
                PlotKind::GridChart,
                PlotKind::BoxPlot,
            ] {
                let mut selected = PlotsPlotRequest {
                    plot_type: kind as i32,
                    plot_id: Some(kind.as_str_name().into()),
                    ..request.clone()
                };
                match kind {
                    PlotKind::Bar | PlotKind::Xy | PlotKind::Trajectory => selected.x_key = Some("min".into()),
                    PlotKind::Pie => {
                        selected.x_key = Some("min".into());
                        selected.y_keys.clear();
                        selected.y_expressions.push(DataExpression {
                            kind: Some(DataExpressionKind::Aggregate(Box::new(AggregateExpression {
                                input: Some(Box::new(reference("max"))),
                                kind: Aggregate::Sum as i32,
                                ..Default::default()
                            }))),
                            ..Default::default()
                        });
                    }
                    PlotKind::Contour => {
                        selected.x_key = Some("min".into());
                        selected.z_field = Some("max".into());
                        selected.y_keys = vec!["max".into()];
                    }
                    PlotKind::GridChart => {
                        selected.grid = Some(GridDefinition {
                            row: Some(reference("min")),
                            column: Some(reference("max")),
                            ..Default::default()
                        });
                        selected.reduction = Some(Reduction::Mean as i32);
                    }
                    PlotKind::FrequencySeries => {
                        selected.stream = "wave".into();
                        selected.y_keys = vec!["min".into()];
                        selected.fft = Some(PlotFftOptions {
                            sample_rate_hz: Some(1.0),
                            window: FftWindow::Rectangular as i32,
                            detrend: FftDetrend::None as i32,
                            ..Default::default()
                        });
                    }
                    _ => (),
                }
                let envelope =
                    Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                        .plots
                        .snapshots
                        .clone()
                        .list(vec![selected])
                        .await?
                        .envelopes
                        .remove(0);
                assert_eq!(envelope.diagnostics.as_ref().unwrap().examined_rows, 32, "{kind:?}");
                let payload = envelope.payload.unwrap();
                if kind == PlotKind::FrequencySeries {
                    let column = &payload.frame.unwrap().series[0];
                    let peak = column
                        .y
                        .iter()
                        .enumerate()
                        .max_by(|(_, left), (_, right)| left.value.unwrap().total_cmp(&right.value.unwrap()))
                        .unwrap()
                        .0;
                    assert!((column.y[peak].value.unwrap() - 1.0).abs() < 1e-10);
                    assert_eq!(column.x[peak].kind, Some(ValueKind::Number(0.125)));
                } else if matches!(kind, PlotKind::Histogram | PlotKind::BoxPlot | PlotKind::Contour | PlotKind::GridChart) {
                    assert!(payload.analysis.is_some());
                } else {
                    let frame = payload.frame.unwrap();
                    assert!(frame.point_count > 0);
                    assert!(!frame.signature.is_empty());
                }
            }

            // Keep Grouped Series Paired with Their Labels and Raw Counts
            let grouped =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plots
                    .snapshots
                    .clone()
                    .list(vec![PlotsPlotRequest {
                        plot_type: PlotKind::Bar as i32,
                        x_key: Some("min".into()),
                        series_by: Some("max".into()),
                        arrangement: Some(BarArrangement::Grouped as i32),
                        ..request.clone()
                    }])
                    .await?;
            let payload = grouped.envelopes[0].payload.as_ref().unwrap();
            assert_eq!(payload.series.len(), 3);
            assert_eq!(payload.series.iter().map(|series| series.raw_points).sum::<u64>(), 32);
            assert_eq!(payload.frame.as_ref().unwrap().series.len(), 3);

            // The Nested Watch Opens at the Same Fixed Selection.
            let mut watch = plots
                .watch(vec![PlotsPlotRequest {
                    plot_type: PlotKind::TimeSeries as i32,
                    ..request.clone()
                }])
                .await?;
            let opening = watch.next().await?.unwrap();
            assert!(opening.opening);
            assert!(!opening.updates.is_empty());
            drop(watch);
            // Refuse Ambiguous Totals and Invalid Pie Options Before Returning a Plot
            let duplicate = PlotsPlotRequest {
                plot_type: PlotKind::Pie as i32,
                x_key: Some("min".into()),
                ..request.clone()
            };
            assert!(
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plots
                    .snapshots
                    .clone()
                    .list(vec![duplicate])
                    .await
                    .is_err()
            );
            assert!(
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plots
                    .snapshots
                    .clone()
                    .list(vec![PlotsPlotRequest {
                        plot_type: PlotKind::Pie as i32,
                        hole: Some(1.0),
                        ..request
                    }])
                    .await
                    .is_err()
            );

            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    })
}

#[test]
fn sealed_plot_watch_returns_its_query_and_completes() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Store One Frozen Numeric Row.
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            let catalog = Declarations {
                types: Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),
                streams: vec![StreamDeclaration {
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
                origin: Some(Origin {
                    kind: Some(OriginKind::Import(Import::default())),
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
                    row: PlotsPlotDomain { min: 12.0, max: 34.0 }.encode_to_vec(),
                    ..Default::default()
                }],
            )?;
            store.close_streams("watch-execution")?;
            store.close("watch-execution", Termination::Completed, ExecutionOutcome::default())?;
            store.seal("watch-execution")?;

            // Open the Runtime Watch Through the Same Query Path as the Desktop.
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let client =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plots
                    .snapshots
                    .clone();
            let mut watch = client
                .watch(vec![PlotsPlotRequest {
                    plot_id: Some("sensor-plot".into()),
                    stream: "sensor".into(),
                    y_keys: vec!["min".into()],
                    plot_type: PlotKind::TimeSeries as i32,
                    scope: Some(EvidenceScope {
                        execution_id: Some("watch-execution".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }])
                .await?;
            let opening = watch.next().await?.ok_or("Watch Plot: Missing Opening Query")?;
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
                (1, Some(&ValueKind::Number(12.0)), Some(&ValueKind::Number(12.0)))
            );
            assert!(watch.next().await?.is_none());

            // Close the Host Before Its Disposable Root Removes Credentials.
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    })
}
