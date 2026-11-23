/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Plot Requests and Presentation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result};

use dokime_protocol::host::PlotPresentation;
use dokime_protocol::host::PlotReferenceBand;
use dokime_protocol::host::PlotReferenceLine;
use dokime_protocol::host::PlotReferenceValue;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::model::DataExpression;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::Plot;
use dokime_protocol::model::PlotFftOptions;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::PlotHistogramOptions;
use dokime_protocol::model::PlotKind;
use dokime_storage::{ExecutionCut, Store};

use dokime_protocol::model::value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn request(plot: &Plot, scope: EvidenceScope) -> Result<PlotRequest> {
    let request = PlotRequest {
        plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
        plot_type: plot.kind,
        style: plot.style,
        x_scale: plot.x_scale,
        y_scale: plot.y_scale,
        scope: Some(scope),
        x_expression: plot.x.clone(),
        y_expressions: plot.y.clone(),
        z_expression: plot.z.clone(),
        series_by_expression: plot.series_by.clone(),
        filters: plot
            .filters
            .iter()
            .map(|filter| PlotFilter {
                expression: filter.expression.clone(),
                operator: filter.operator,
                value: filter.value.clone(),
            })
            .collect(),
        marker_record_types: plot.marker_records.clone(),
        unit: Some(plot.unit.clone()),
        max_points: plot.display_points.map(u64::from),
        histogram: plot.histogram_bins.map(|bins| PlotHistogramOptions { bins: bins as u64, ..Default::default() }),
        fft: (plot.kind == PlotKind::FrequencySeries as i32).then_some(PlotFftOptions {
            sample_rate_hz: plot.sample_rate_hz,
            input_samples: plot.input_samples,
            ..Default::default()
        }),
        arrangement: plot.arrangement,
        pareto: plot.pareto,
        hole: plot.hole,
        xy_interval: plot.interval,
        reduction: plot.reduction,
        render: plot.render,
        grid: plot.grid.clone(),
        cell_expression: plot.cell.clone(),
        facet_expression: plot.facet.clone(),
        ..Default::default()
    };
    request.validate_heat_strip().map_err(Error::Invalid)?;
    Ok(request)
}

pub fn presentation(store: &Store, cut: &ExecutionCut, plot: &Plot) -> Result<PlotPresentation> {
    let value = |expression: Option<&DataExpression>| -> Result<Option<PlotReferenceValue>> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let value = store.document_value(cut, expression)?;
        let value = match value.kind {
            Some(value::Kind::Number(value)) => value,
            Some(value::Kind::Integer(value)) => value as f64,
            Some(value::Kind::UnsignedInteger(value)) => value as f64,
            _ => return Err(Error::Invalid("Render Plot Reference: Select a Numeric Scalar".into())),
        };
        Ok(Some(PlotReferenceValue { constant: Some(value), expression: None }))
    };
    let mut reference_lines = Vec::new();
    for line in &plot.reference_lines {
        reference_lines.push(PlotReferenceLine {
            value: value(line.value.as_ref())?,
            label: Some(line.label.clone()),
            color: line.color.clone(),
        });
    }
    let mut reference_bands = Vec::new();
    for band in &plot.reference_bands {
        reference_bands.push(PlotReferenceBand {
            min: value(band.min.as_ref())?,
            max: value(band.max.as_ref())?,
            center: value(band.center.as_ref())?,
            tolerance: value(band.tolerance.as_ref())?,
            label: Some(band.label.clone()),
            color: band.color.clone(),
        });
    }
    Ok(PlotPresentation {
        style: plot.style,
        reference_lines,
        reference_bands,
        fft: (plot.kind == PlotKind::FrequencySeries as i32).then_some(PlotFftOptions {
            sample_rate_hz: plot.sample_rate_hz,
            input_samples: plot.input_samples,
            ..Default::default()
        }),
        histogram: plot.histogram_bins.map(|bins| PlotHistogramOptions { bins: bins as u64, ..Default::default() }),
        ..Default::default()
    })
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::request;
    use crate::host::plots::snapshots::list::snapshots_at;
    use dokime_protocol::host::evidence::RecordsAppendRequest;
    use dokime_protocol::host::plots::plot_analysis_result;
    use dokime_protocol::host::{ExecutionOutcome, UnevaluatedOutcome, execution_outcome};
    use dokime_protocol::model::{
        Aggregate, AggregateExpression, DataExpression, DocumentFilter, EvidenceScope, FieldReference, FilterOperator, Import, Metadata,
        Origin, Plot, PlotFilter, PlotKind, RecordContract, RecordContracts, Termination, TransformExpression, Value, data_expression,
        origin, transform_expression, value,
    };
    use dokime_storage::{Evidence, Execution, Sample, Store};
    use harmos_protocol::model::{Declarations, StreamDeclaration};
    use prost::Message;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, field_descriptor_proto::Type};

    #[derive(Clone, PartialEq, Message)]
    struct Row {
        #[prost(uint64, tag = "1")]
        value: u64,
    }

    #[derive(Clone, PartialEq, Message)]
    struct Status {
        #[prost(string, tag = "1")]
        event: String,
    }

    fn fixture() -> (tempfile::TempDir, Store, DataExpression) {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let contracts = RecordContracts {
            records: ["connected", "disconnected"]
                .into_iter()
                .map(|identifier| RecordContract {
                    metadata: Some(Metadata {
                        identifier: identifier.into(),
                        ..Default::default()
                    }),
                    message: "plots.Row".into(),
                })
                .collect(),
        };
        let mut catalog = Declarations {
            types: Some(FileDescriptorSet {
                file: vec![FileDescriptorProto {
                    name: Some("plots.proto".into()),
                    package: Some("plots".into()),
                    syntax: Some("proto3".into()),
                    message_type: vec![DescriptorProto {
                        name: Some("Row".into()),
                        field: vec![FieldDescriptorProto {
                            name: Some("value".into()),
                            number: Some(1),
                            r#type: Some(Type::Uint64 as i32),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }),
            sections: vec![Default::default()],
            ..Default::default()
        };
        catalog.sections[0].r#type = "dokime.model.v1.RecordContracts".into();
        catalog.sections[0].payload = contracts.encode_to_vec();
        store
            .create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                catalog: catalog.encode_to_vec(),
                ..Default::default()
            })
            .unwrap();
        for (record, seconds, value) in [
            ("connected", 1, 1),
            ("disconnected", 2, 1),
            ("connected", 4, 1),
            ("disconnected", 86406, 1),
            ("connected", 86410, 1),
            ("disconnected", 172812, 2),
            ("connected", 172814, 2),
            ("disconnected", 259216, 2),
            ("connected", 259222, 2),
        ] {
            store
                .append(
                    "execution",
                    Evidence {
                        kind: format!("record:{record}"),
                        source_ns: seconds * 1_000_000_000,
                        payload: RecordsAppendRequest {
                            payload: Row { value }.encode_to_vec(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Transform(Box::new(TransformExpression {
                input: Some(Box::new(DataExpression {
                    kind: Some(data_expression::Kind::Record(FieldReference {
                        identifier: "connected".into(),
                        field: String::new(),
                    })),
                    ..Default::default()
                })),
                kind: Some(transform_expression::Kind::TimeBetween("disconnected".into())),
            }))),
            ..Default::default()
        };
        (root, store, expression)
    }

    #[tokio::test]
    async fn telemetry_at_status_filters_preserve_record_rows_live_and_sealed() {
        let (_root, store, _) = fixture();
        let mut execution = store.cut("execution").unwrap().execution;
        execution.identifier = "sampled-execution".into();
        let mut catalog = Declarations::decode(execution.catalog.as_slice()).unwrap();
        catalog.types.as_mut().unwrap().file[0].message_type.push(DescriptorProto {
            name: Some("Status".into()),
            field: vec![FieldDescriptorProto {
                name: Some("event".into()),
                number: Some(1),
                r#type: Some(Type::String as i32),
                ..Default::default()
            }],
            ..Default::default()
        });
        let contracts = RecordContracts {
            records: vec![RecordContract {
                metadata: Some(Metadata {
                    identifier: "status".into(),
                    ..Default::default()
                }),
                message: "plots.Status".into(),
            }],
        };
        catalog.sections[0].payload = contracts.encode_to_vec();
        catalog.streams.push(StreamDeclaration {
            id: "readings".into(),
            row: "plots.Row".into(),
            ..Default::default()
        });
        execution.catalog = catalog.encode_to_vec();
        store.create(&execution).unwrap();
        store.open_stream("sampled-execution", "readings").unwrap();
        for (seconds, value) in [(1, 10), (2, 20), (2, 25), (3, 30)] {
            store
                .sample(
                    "sampled-execution",
                    Sample {
                        stream: "readings".into(),
                        source_ns: seconds * 1_000_000_000,
                        row: Row { value }.encode_to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        store.flush_samples().unwrap();
        for (seconds, event) in
            [(0, "disconnected"), (1, "connected"), (2, "disconnected"), (2, "connected"), (4, "disconnected"), (5, "connected")]
        {
            store
                .append(
                    "sampled-execution",
                    Evidence {
                        kind: "record:status".into(),
                        source_ns: seconds * 1_000_000_000,
                        payload: RecordsAppendRequest {
                            payload: Status { event: event.into() }.encode_to_vec(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        let sampled = FieldReference::new("readings").field("value").at("status");
        for sealed in [false, true] {
            if sealed {
                store.close_streams("sampled-execution").unwrap();
                store
                    .close(
                        "sampled-execution",
                        Termination::Completed,
                        ExecutionOutcome {
                            attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome::default())),
                        },
                    )
                    .unwrap();
                store.seal("sampled-execution", Default::default()).unwrap();
            }
            let cut = store.cut("sampled-execution").unwrap();
            let details = store.document_value(&cut, &sampled.clone().detail("event").count()).unwrap();
            assert_eq!(details.kind, Some(value::Kind::Integer(5)));
            for (event, distances) in [("disconnected", vec![25.0, 30.0]), ("connected", vec![10.0, 25.0, 30.0])] {
                let filter = DocumentFilter {
                    expression: Some(sampled.clone().detail("event")),
                    operator: FilterOperator::Eq as i32,
                    value: Some(Value {
                        kind: Some(value::Kind::Text(event.into())),
                    }),
                };
                let plot = Plot {
                    kind: PlotKind::Histogram as i32,
                    x: Some(sampled.clone()),
                    histogram_bins: Some(2),
                    filters: vec![filter.clone()],
                    ..Default::default()
                };
                let scope = EvidenceScope {
                    execution_id: Some("sampled-execution".into()),
                    ..Default::default()
                };
                let envelopes = snapshots_at(&store, &cut, vec![request(&plot, scope).unwrap()], 128, 0, "epoch").await.unwrap();
                let Some(plot_analysis_result::Kind::Histogram(histogram)) =
                    envelopes[0].payload.as_ref().unwrap().analysis.as_ref().and_then(|a| a.result.as_ref()).and_then(|r| r.kind.as_ref())
                else {
                    panic!("Missing Histogram")
                };
                assert_eq!(histogram.series[0].count, distances.len() as u64);
                assert_eq!(histogram.series[0].bins.iter().map(|bin| bin.count).sum::<u64>(), distances.len() as u64);
                let filtered = [PlotFilter {
                    expression: filter.expression,
                    operator: filter.operator,
                    value: filter.value,
                }];
                let sum = store.document_value_filtered(&cut, &sampled.clone().sum(), &filtered).unwrap();
                assert_eq!(sum.kind, Some(value::Kind::UnsignedInteger(distances.iter().sum::<f64>() as u64)));
                let plot = Plot {
                    kind: PlotKind::TimeSeries as i32,
                    y: vec![sampled.clone().per_case()],
                    filters: plot.filters,
                    ..Default::default()
                };
                let scope = EvidenceScope {
                    execution_id: Some("sampled-execution".into()),
                    ..Default::default()
                };
                let envelopes = snapshots_at(&store, &cut, vec![request(&plot, scope).unwrap()], 128, 0, "epoch").await.unwrap();
                let frame = envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
                let values = frame.series[0].y.iter().filter_map(|value| value.value).collect::<Vec<_>>();
                assert_eq!(values, distances);
            }
        }
    }

    #[tokio::test]
    async fn record_only_plots_render_intervals_without_a_stream() {
        let (_root, store, expression) = fixture();
        let cut = store.cut("execution").unwrap();
        assert!(store.stream_names("execution").unwrap().is_empty());
        let scope = EvidenceScope {
            execution_id: Some("execution".into()),
            ..Default::default()
        };
        let histogram = Plot {
            kind: PlotKind::Histogram as i32,
            x: Some(expression.clone()),
            histogram_bins: Some(2),
            ..Default::default()
        };
        let temporal = Plot {
            kind: PlotKind::TimeSeries as i32,
            y: vec![expression.clone()],
            ..Default::default()
        };
        let boxes = Plot {
            kind: PlotKind::BoxPlot as i32,
            y: vec![expression.clone()],
            ..Default::default()
        };
        let categories = Plot {
            kind: PlotKind::Bar as i32,
            x: Some(DataExpression {
                kind: Some(data_expression::Kind::Record(FieldReference {
                    identifier: "connected".into(),
                    field: "value".into(),
                })),
                ..Default::default()
            }),
            y: vec![DataExpression {
                kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                    input: Some(Box::new(expression)),
                    kind: Aggregate::Sum as i32,
                    ..Default::default()
                }))),
                ..Default::default()
            }],
            ..Default::default()
        };
        let requests = [&histogram, &temporal, &boxes, &categories].into_iter().map(|plot| request(plot, scope.clone()).unwrap()).collect();
        let envelopes = snapshots_at(&store, &cut, requests, 128, 0, "epoch").await.unwrap();
        let payload = envelopes[0].payload.as_ref().unwrap();
        let Some(plot_analysis_result::Kind::Histogram(histogram)) =
            payload.analysis.as_ref().and_then(|a| a.result.as_ref()).and_then(|r| r.kind.as_ref())
        else {
            panic!("missing histogram")
        };
        assert_eq!(envelopes[0].diagnostics.as_ref().unwrap().raw_point_count, 4);
        assert_eq!(histogram.series[0].count, 4);
        let bins = histogram.series[0].bins.iter().map(|bin| (bin.lower, bin.upper, bin.count)).collect::<Vec<_>>();
        assert_eq!(bins, vec![(2.0, 4.0, 2), (4.0, 6.0, 2)]);
        let frame = envelopes[1].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(frame.time, vec![4.0, 86410.0, 172814.0, 259222.0]);
        assert_eq!(frame.series[0].y.iter().map(|value| value.value).collect::<Vec<_>>(), vec![Some(2.0), Some(4.0), Some(2.0), Some(6.0)]);
        assert_eq!(envelopes[2].diagnostics.as_ref().unwrap().raw_point_count, 4);
        let frame = envelopes[3].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(frame.series[0].y.iter().map(|value| value.value).collect::<Vec<_>>(), vec![Some(6.0), Some(8.0)]);
    }

    #[tokio::test]
    async fn stream_document_plots_match_stream_column_requests() {
        let (_root, store, _) = fixture();
        let mut execution = store.cut("execution").unwrap().execution;
        execution.identifier = "stream-execution".into();
        let mut catalog = Declarations::decode(execution.catalog.as_slice()).unwrap();
        catalog.streams.push(StreamDeclaration {
            id: "readings".into(),
            row: "plots.Row".into(),
            ..Default::default()
        });
        execution.catalog = catalog.encode_to_vec();
        store.create(&execution).unwrap();
        store.open_stream("stream-execution", "readings").unwrap();
        for (index, value) in [2, 4, 2, 6].into_iter().enumerate() {
            store
                .sample(
                    "stream-execution",
                    Sample {
                        stream: "readings".into(),
                        source_ns: index as i64 * 1_000_000_000,
                        row: Row { value }.encode_to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        store.flush_samples().unwrap();
        let cut = store.cut("stream-execution").unwrap();
        let plot = Plot {
            kind: PlotKind::TimeSeries as i32,
            y: vec![FieldReference::new("readings").field("value")],
            ..Default::default()
        };
        let request = request(
            &plot,
            EvidenceScope {
                execution_id: Some("stream-execution".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let mut columns = request.clone();
        columns.stream = "readings".into();
        columns.y_expressions.clear();
        columns.y_keys = vec!["value".into()];
        let envelopes = snapshots_at(&store, &cut, vec![request, columns], 128, 0, "epoch").await.unwrap();
        assert_eq!(envelopes[0].payload, envelopes[1].payload);
        let frame = envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(frame.time, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(frame.series[0].y.iter().map(|value| value.value).collect::<Vec<_>>(), vec![Some(2.0), Some(4.0), Some(2.0), Some(6.0)]);
    }

    #[test]
    fn plots_without_stored_bindings_are_refused() {
        let (_root, store, _) = fixture();
        let cut = store.cut("execution").unwrap();
        let plot = Plot {
            y: vec![DataExpression {
                kind: Some(data_expression::Kind::Literal(Value { kind: Some(value::Kind::Number(2.0)) })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let request = request(&plot, EvidenceScope::default()).unwrap();
        assert!(store.plot_selection(&cut, &request).is_err());
        assert!(store.plot_selection(&cut, &Default::default()).is_err());
    }
    #[tokio::test]
    async fn heat_strip_sample_snapshots_retain_history_and_freeze_the_captured_cut() {
        let (_root, store, _) = fixture();
        let input = DataExpression {
            kind: Some(data_expression::Kind::Record(FieldReference {
                identifier: "connected".into(),
                field: "value".into(),
            })),
            ..Default::default()
        };
        let plot = dokime_protocol::builders::Plot::heat_strip(input)
            .cells(DataExpression {
                kind: Some(data_expression::Kind::Literal(Value {
                    kind: Some(value::Kind::UnsignedInteger(6)),
                })),
                ..Default::default()
            })
            .message;
        let binding = request(
            &plot,
            EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let cut = store.cut("execution").unwrap();
        let captured = snapshots_at(&store, &cut, vec![binding.clone()], 128, 0, "epoch").await.unwrap();
        let frame = captured[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(frame.series.len(), 1);
        assert_eq!(
            frame.series[0].x,
            vec![
                Value {
                    kind: Some(value::Kind::UnsignedInteger(6))
                };
                5
            ]
        );
        assert_eq!(
            frame.series[0].y.iter().map(|number| number.value).collect::<Vec<_>>(),
            vec![Some(1.0), Some(1.0), Some(1.0), Some(2.0), Some(2.0)]
        );
        assert_eq!(
            frame.series[0].instants,
            [1, 4, 86410, 172814, 259222].map(|seconds| prost_types::Timestamp { seconds, nanos: 0 }.to_string())
        );
        store
            .append(
                "execution",
                Evidence {
                    kind: "record:connected".into(),
                    source_ns: 300000_000_000_000,
                    payload: RecordsAppendRequest {
                        payload: Row { value: 3 }.encode_to_vec(),
                        ..Default::default()
                    }
                    .encode_to_vec(),
                    ..Default::default()
                },
            )
            .unwrap();
        let latest = store.cut("execution").unwrap();
        let live = snapshots_at(&store, &latest, vec![binding.clone()], 128, 0, "epoch").await.unwrap();
        let live = live[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(live.series[0].y.len(), 6);
        assert_eq!(live.series[0].y[5].value, Some(3.0));
        let frozen = snapshots_at(&store, &cut, vec![binding.clone()], 128, 0, "epoch").await.unwrap();
        assert_eq!(frozen[0].payload, captured[0].payload);
        let mut window = binding;
        window.viewport = Some(dokime_protocol::host::plots::PlotWindow { from: 0.0, to: 10.0 });
        let selected = snapshots_at(&store, &latest, vec![window], 128, 0, "epoch").await.unwrap();
        let selected = selected[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(selected.series[0].instants, [1, 4].map(|seconds| prost_types::Timestamp { seconds, nanos: 0 }.to_string()));
    }
}
