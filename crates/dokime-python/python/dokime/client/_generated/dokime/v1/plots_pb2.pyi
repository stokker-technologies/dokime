from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import catalog_pb2 as _catalog_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PlotType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_TYPE_UNSPECIFIED: _ClassVar[PlotType]
    PLOT_TYPE_TIME_SERIES: _ClassVar[PlotType]
    PLOT_TYPE_BAR: _ClassVar[PlotType]
    PLOT_TYPE_GROUPED_BAR: _ClassVar[PlotType]
    PLOT_TYPE_STACKED_BAR: _ClassVar[PlotType]
    PLOT_TYPE_PIE: _ClassVar[PlotType]
    PLOT_TYPE_DONUT: _ClassVar[PlotType]
    PLOT_TYPE_PARETO: _ClassVar[PlotType]
    PLOT_TYPE_XY_SCATTER: _ClassVar[PlotType]
    PLOT_TYPE_XY_LINE: _ClassVar[PlotType]
    PLOT_TYPE_XY_INTERVAL: _ClassVar[PlotType]
    PLOT_TYPE_TRAJECTORY: _ClassVar[PlotType]
    PLOT_TYPE_HEATMAP: _ClassVar[PlotType]
    PLOT_TYPE_XY_VALUE_HEATMAP: _ClassVar[PlotType]
    PLOT_TYPE_WIFI_CHANNEL_OCCUPANCY: _ClassVar[PlotType]
    PLOT_TYPE_FREQUENCY_SERIES: _ClassVar[PlotType]
    PLOT_TYPE_HISTOGRAM: _ClassVar[PlotType]
    PLOT_TYPE_BOX_SUMMARY: _ClassVar[PlotType]
    PLOT_TYPE_AGGREGATE_TABLE: _ClassVar[PlotType]

class AxisScale(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AXIS_SCALE_UNSPECIFIED: _ClassVar[AxisScale]
    AXIS_SCALE_LINEAR: _ClassVar[AxisScale]
    AXIS_SCALE_LOG: _ClassVar[AxisScale]

class DownsampleTechnique(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOWNSAMPLE_TECHNIQUE_UNSPECIFIED: _ClassVar[DownsampleTechnique]
    DOWNSAMPLE_TECHNIQUE_RAW: _ClassVar[DownsampleTechnique]
    DOWNSAMPLE_TECHNIQUE_MIN_MAX: _ClassVar[DownsampleTechnique]
    DOWNSAMPLE_TECHNIQUE_LTTB: _ClassVar[DownsampleTechnique]

class FilterOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FILTER_OPERATOR_UNSPECIFIED: _ClassVar[FilterOperator]
    FILTER_OPERATOR_EQ: _ClassVar[FilterOperator]
    FILTER_OPERATOR_NE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_GT: _ClassVar[FilterOperator]
    FILTER_OPERATOR_GTE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_LT: _ClassVar[FilterOperator]
    FILTER_OPERATOR_LTE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_CONTAINS: _ClassVar[FilterOperator]
    FILTER_OPERATOR_EXISTS: _ClassVar[FilterOperator]

class PlotStreamKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_STREAM_KIND_UNSPECIFIED: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_SNAPSHOT: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_APPEND: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_REPLACE: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_ERROR: _ClassVar[PlotStreamKind]

class PlotCartesianMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_CARTESIAN_MODE_UNSPECIFIED: _ClassVar[PlotCartesianMode]
    PLOT_CARTESIAN_MODE_ALIGNED: _ClassVar[PlotCartesianMode]
    PLOT_CARTESIAN_MODE_FACETED: _ClassVar[PlotCartesianMode]

class FftWindow(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FFT_WINDOW_UNSPECIFIED: _ClassVar[FftWindow]
    FFT_WINDOW_HANN: _ClassVar[FftWindow]
    FFT_WINDOW_RECTANGULAR: _ClassVar[FftWindow]

class FftDetrend(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FFT_DETREND_UNSPECIFIED: _ClassVar[FftDetrend]
    FFT_DETREND_NONE: _ClassVar[FftDetrend]
    FFT_DETREND_MEAN: _ClassVar[FftDetrend]

class FrequencyLevelUnit(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FREQUENCY_LEVEL_UNIT_UNSPECIFIED: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_LINEAR: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DB: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DBU: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DBM: _ClassVar[FrequencyLevelUnit]
PLOT_TYPE_UNSPECIFIED: PlotType
PLOT_TYPE_TIME_SERIES: PlotType
PLOT_TYPE_BAR: PlotType
PLOT_TYPE_GROUPED_BAR: PlotType
PLOT_TYPE_STACKED_BAR: PlotType
PLOT_TYPE_PIE: PlotType
PLOT_TYPE_DONUT: PlotType
PLOT_TYPE_PARETO: PlotType
PLOT_TYPE_XY_SCATTER: PlotType
PLOT_TYPE_XY_LINE: PlotType
PLOT_TYPE_XY_INTERVAL: PlotType
PLOT_TYPE_TRAJECTORY: PlotType
PLOT_TYPE_HEATMAP: PlotType
PLOT_TYPE_XY_VALUE_HEATMAP: PlotType
PLOT_TYPE_WIFI_CHANNEL_OCCUPANCY: PlotType
PLOT_TYPE_FREQUENCY_SERIES: PlotType
PLOT_TYPE_HISTOGRAM: PlotType
PLOT_TYPE_BOX_SUMMARY: PlotType
PLOT_TYPE_AGGREGATE_TABLE: PlotType
AXIS_SCALE_UNSPECIFIED: AxisScale
AXIS_SCALE_LINEAR: AxisScale
AXIS_SCALE_LOG: AxisScale
DOWNSAMPLE_TECHNIQUE_UNSPECIFIED: DownsampleTechnique
DOWNSAMPLE_TECHNIQUE_RAW: DownsampleTechnique
DOWNSAMPLE_TECHNIQUE_MIN_MAX: DownsampleTechnique
DOWNSAMPLE_TECHNIQUE_LTTB: DownsampleTechnique
FILTER_OPERATOR_UNSPECIFIED: FilterOperator
FILTER_OPERATOR_EQ: FilterOperator
FILTER_OPERATOR_NE: FilterOperator
FILTER_OPERATOR_GT: FilterOperator
FILTER_OPERATOR_GTE: FilterOperator
FILTER_OPERATOR_LT: FilterOperator
FILTER_OPERATOR_LTE: FilterOperator
FILTER_OPERATOR_CONTAINS: FilterOperator
FILTER_OPERATOR_EXISTS: FilterOperator
PLOT_STREAM_KIND_UNSPECIFIED: PlotStreamKind
PLOT_STREAM_KIND_SNAPSHOT: PlotStreamKind
PLOT_STREAM_KIND_APPEND: PlotStreamKind
PLOT_STREAM_KIND_REPLACE: PlotStreamKind
PLOT_STREAM_KIND_ERROR: PlotStreamKind
PLOT_CARTESIAN_MODE_UNSPECIFIED: PlotCartesianMode
PLOT_CARTESIAN_MODE_ALIGNED: PlotCartesianMode
PLOT_CARTESIAN_MODE_FACETED: PlotCartesianMode
FFT_WINDOW_UNSPECIFIED: FftWindow
FFT_WINDOW_HANN: FftWindow
FFT_WINDOW_RECTANGULAR: FftWindow
FFT_DETREND_UNSPECIFIED: FftDetrend
FFT_DETREND_NONE: FftDetrend
FFT_DETREND_MEAN: FftDetrend
FREQUENCY_LEVEL_UNIT_UNSPECIFIED: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_LINEAR: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DB: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DBU: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DBM: FrequencyLevelUnit

class PlotSnapshotsRequest(_message.Message):
    __slots__ = ("requests",)
    REQUESTS_FIELD_NUMBER: _ClassVar[int]
    requests: _containers.RepeatedCompositeFieldContainer[PlotRequest]
    def __init__(self, requests: _Optional[_Iterable[_Union[PlotRequest, _Mapping]]] = ...) -> None: ...

class PlotSnapshotsReply(_message.Message):
    __slots__ = ("envelopes",)
    ENVELOPES_FIELD_NUMBER: _ClassVar[int]
    envelopes: _containers.RepeatedCompositeFieldContainer[PlotEnvelope]
    def __init__(self, envelopes: _Optional[_Iterable[_Union[PlotEnvelope, _Mapping]]] = ...) -> None: ...

class PlotWatchRequest(_message.Message):
    __slots__ = ("requests",)
    REQUESTS_FIELD_NUMBER: _ClassVar[int]
    requests: _containers.RepeatedCompositeFieldContainer[PlotRequest]
    def __init__(self, requests: _Optional[_Iterable[_Union[PlotRequest, _Mapping]]] = ...) -> None: ...

class PlotRequest(_message.Message):
    __slots__ = ("plot_id", "plot_type", "x_scale", "y_scale", "stream", "y_keys", "marker_record_types", "filters", "x_key", "series_by", "x_expression", "y_expressions", "series_by_expression", "z_stream", "z_field", "unit", "max_points", "window_s", "scope", "cursor", "technique", "z_expression", "fft", "interval", "histogram", "analysis")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_TYPE_FIELD_NUMBER: _ClassVar[int]
    X_SCALE_FIELD_NUMBER: _ClassVar[int]
    Y_SCALE_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    Y_KEYS_FIELD_NUMBER: _ClassVar[int]
    MARKER_RECORD_TYPES_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    X_KEY_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_FIELD_NUMBER: _ClassVar[int]
    X_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    Y_EXPRESSIONS_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    Z_STREAM_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    TECHNIQUE_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    plot_type: PlotType
    x_scale: AxisScale
    y_scale: AxisScale
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[PlotFilter]
    x_key: str
    series_by: str
    x_expression: DataExpression
    y_expressions: _containers.RepeatedCompositeFieldContainer[DataExpression]
    series_by_expression: DataExpression
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: _common_pb2.EvidenceScope
    cursor: PlotCursor
    technique: DownsampleTechnique
    z_expression: DataExpression
    fft: PlotFftOptions
    interval: PlotIntervalOptions
    histogram: PlotHistogramOptions
    analysis: AnalysisSelection
    def __init__(self, plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[PlotType, str]] = ..., x_scale: _Optional[_Union[AxisScale, str]] = ..., y_scale: _Optional[_Union[AxisScale, str]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[PlotFilter, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[DataExpression, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[DataExpression, _Mapping]]] = ..., series_by_expression: _Optional[_Union[DataExpression, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., cursor: _Optional[_Union[PlotCursor, _Mapping]] = ..., technique: _Optional[_Union[DownsampleTechnique, str]] = ..., z_expression: _Optional[_Union[DataExpression, _Mapping]] = ..., fft: _Optional[_Union[PlotFftOptions, _Mapping]] = ..., interval: _Optional[_Union[PlotIntervalOptions, _Mapping]] = ..., histogram: _Optional[_Union[PlotHistogramOptions, _Mapping]] = ..., analysis: _Optional[_Union[AnalysisSelection, _Mapping]] = ...) -> None: ...

class PlotFilter(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: DataExpression
    operator: FilterOperator
    value: _value_pb2.Value
    def __init__(self, expression: _Optional[_Union[DataExpression, _Mapping]] = ..., operator: _Optional[_Union[FilterOperator, str]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ...) -> None: ...

class DataExpression(_message.Message):
    __slots__ = ("telemetry", "parameter", "dimension", "record", "artifact", "transforms", "label")
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    telemetry: TelemetryDataSource
    parameter: ParameterDataSource
    dimension: DimensionDataSource
    record: RecordDataSource
    artifact: ArtifactDataSource
    transforms: _containers.RepeatedCompositeFieldContainer[DataTransform]
    label: str
    def __init__(self, telemetry: _Optional[_Union[TelemetryDataSource, _Mapping]] = ..., parameter: _Optional[_Union[ParameterDataSource, _Mapping]] = ..., dimension: _Optional[_Union[DimensionDataSource, _Mapping]] = ..., record: _Optional[_Union[RecordDataSource, _Mapping]] = ..., artifact: _Optional[_Union[ArtifactDataSource, _Mapping]] = ..., transforms: _Optional[_Iterable[_Union[DataTransform, _Mapping]]] = ..., label: _Optional[str] = ...) -> None: ...

class TelemetryDataSource(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class ParameterDataSource(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class DimensionDataSource(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class RecordDataSource(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class ArtifactDataSource(_message.Message):
    __slots__ = ("artifact",)
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    def __init__(self, artifact: _Optional[str] = ...) -> None: ...

class DataTransform(_message.Message):
    __slots__ = ("count", "mean", "median", "min", "max", "time_between", "telemetry_at_or_before", "detail", "per_case")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    TIME_BETWEEN_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_AT_OR_BEFORE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PER_CASE_FIELD_NUMBER: _ClassVar[int]
    count: EmptyTransform
    mean: EmptyTransform
    median: EmptyTransform
    min: EmptyTransform
    max: EmptyTransform
    time_between: TimeBetweenTransform
    telemetry_at_or_before: TelemetryAtOrBeforeTransform
    detail: DetailTransform
    per_case: EmptyTransform
    def __init__(self, count: _Optional[_Union[EmptyTransform, _Mapping]] = ..., mean: _Optional[_Union[EmptyTransform, _Mapping]] = ..., median: _Optional[_Union[EmptyTransform, _Mapping]] = ..., min: _Optional[_Union[EmptyTransform, _Mapping]] = ..., max: _Optional[_Union[EmptyTransform, _Mapping]] = ..., time_between: _Optional[_Union[TimeBetweenTransform, _Mapping]] = ..., telemetry_at_or_before: _Optional[_Union[TelemetryAtOrBeforeTransform, _Mapping]] = ..., detail: _Optional[_Union[DetailTransform, _Mapping]] = ..., per_case: _Optional[_Union[EmptyTransform, _Mapping]] = ...) -> None: ...

class EmptyTransform(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class DetailTransform(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class TimeBetweenTransform(_message.Message):
    __slots__ = ("previous_record_type",)
    PREVIOUS_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    previous_record_type: str
    def __init__(self, previous_record_type: _Optional[str] = ...) -> None: ...

class TelemetryAtOrBeforeTransform(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class PlotCursor(_message.Message):
    __slots__ = ("telemetry_streams", "record_sequence", "epoch", "selection")
    class TelemetryStreamsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    TELEMETRY_STREAMS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    telemetry_streams: _containers.ScalarMap[str, int]
    record_sequence: int
    epoch: str
    selection: str
    def __init__(self, telemetry_streams: _Optional[_Mapping[str, int]] = ..., record_sequence: _Optional[int] = ..., epoch: _Optional[str] = ..., selection: _Optional[str] = ...) -> None: ...

class PlotEnvelope(_message.Message):
    __slots__ = ("plot_id", "revision", "cursor", "payload", "diagnostics", "warnings")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICS_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    revision: int
    cursor: PlotCursor
    payload: PlotPayload
    diagnostics: PlotPayloadDiagnostics
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, plot_id: _Optional[str] = ..., revision: _Optional[int] = ..., cursor: _Optional[_Union[PlotCursor, _Mapping]] = ..., payload: _Optional[_Union[PlotPayload, _Mapping]] = ..., diagnostics: _Optional[_Union[PlotPayloadDiagnostics, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class PlotStreamFrame(_message.Message):
    __slots__ = ("kind", "envelope")
    KIND_FIELD_NUMBER: _ClassVar[int]
    ENVELOPE_FIELD_NUMBER: _ClassVar[int]
    kind: PlotStreamKind
    envelope: PlotEnvelope
    def __init__(self, kind: _Optional[_Union[PlotStreamKind, str]] = ..., envelope: _Optional[_Union[PlotEnvelope, _Mapping]] = ...) -> None: ...

class PlotPayload(_message.Message):
    __slots__ = ("stream", "x_key", "series_by", "context", "series", "cartesian", "markers", "max_points", "window_s", "generated_at", "x_format", "run_started_at", "cycle_started_at", "analysis", "computed")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    X_KEY_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    CARTESIAN_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    X_FORMAT_FIELD_NUMBER: _ClassVar[int]
    RUN_STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_FIELD_NUMBER: _ClassVar[int]
    COMPUTED_FIELD_NUMBER: _ClassVar[int]
    stream: str
    x_key: str
    series_by: str
    context: _value_pb2.ValueObject
    series: _containers.RepeatedCompositeFieldContainer[PlotSeries]
    cartesian: PlotCartesianFrame
    markers: _containers.RepeatedCompositeFieldContainer[PlotMarker]
    max_points: int
    window_s: float
    generated_at: str
    x_format: _schema_pb2.ValueFormat
    run_started_at: str
    cycle_started_at: str
    analysis: PlotAnalysis
    computed: AnalysisComputation
    def __init__(self, stream: _Optional[str] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., context: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., series: _Optional[_Iterable[_Union[PlotSeries, _Mapping]]] = ..., cartesian: _Optional[_Union[PlotCartesianFrame, _Mapping]] = ..., markers: _Optional[_Iterable[_Union[PlotMarker, _Mapping]]] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., generated_at: _Optional[str] = ..., x_format: _Optional[_Union[_schema_pb2.ValueFormat, _Mapping]] = ..., run_started_at: _Optional[str] = ..., cycle_started_at: _Optional[str] = ..., analysis: _Optional[_Union[PlotAnalysis, _Mapping]] = ..., computed: _Optional[_Union[AnalysisComputation, _Mapping]] = ...) -> None: ...

class PlotCartesianFrame(_message.Message):
    __slots__ = ("mode", "series", "x_domain", "y_domain", "point_count", "signature")
    MODE_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    X_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    Y_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    mode: PlotCartesianMode
    series: _containers.RepeatedCompositeFieldContainer[PlotCartesianSeriesFrame]
    x_domain: PlotDomain
    y_domain: PlotDomain
    point_count: int
    signature: str
    def __init__(self, mode: _Optional[_Union[PlotCartesianMode, str]] = ..., series: _Optional[_Iterable[_Union[PlotCartesianSeriesFrame, _Mapping]]] = ..., x_domain: _Optional[_Union[PlotDomain, _Mapping]] = ..., y_domain: _Optional[_Union[PlotDomain, _Mapping]] = ..., point_count: _Optional[int] = ..., signature: _Optional[str] = ...) -> None: ...

class PlotCartesianSeriesFrame(_message.Message):
    __slots__ = ("x", "y")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    x: _containers.RepeatedScalarFieldContainer[float]
    y: _containers.RepeatedCompositeFieldContainer[NullableNumber]
    def __init__(self, x: _Optional[_Iterable[float]] = ..., y: _Optional[_Iterable[_Union[NullableNumber, _Mapping]]] = ...) -> None: ...

class NullableNumber(_message.Message):
    __slots__ = ("value",)
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: float
    def __init__(self, value: _Optional[float] = ...) -> None: ...

class PlotDomain(_message.Message):
    __slots__ = ("min", "max")
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    min: float
    max: float
    def __init__(self, min: _Optional[float] = ..., max: _Optional[float] = ...) -> None: ...

class PlotPayloadDiagnostics(_message.Message):
    __slots__ = ("series_count", "point_count", "raw_point_count", "downsampled", "delta", "max_points", "window_s")
    SERIES_COUNT_FIELD_NUMBER: _ClassVar[int]
    POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    RAW_POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    DOWNSAMPLED_FIELD_NUMBER: _ClassVar[int]
    DELTA_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    series_count: int
    point_count: int
    raw_point_count: int
    downsampled: bool
    delta: bool
    max_points: int
    window_s: float
    def __init__(self, series_count: _Optional[int] = ..., point_count: _Optional[int] = ..., raw_point_count: _Optional[int] = ..., downsampled: _Optional[bool] = ..., delta: _Optional[bool] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ...) -> None: ...

class PlotSeries(_message.Message):
    __slots__ = ("key", "label", "unit", "points", "raw_points", "downsampled", "format")
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    RAW_POINTS_FIELD_NUMBER: _ClassVar[int]
    DOWNSAMPLED_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    unit: str
    points: _containers.RepeatedCompositeFieldContainer[PlotPoint]
    raw_points: int
    downsampled: bool
    format: _schema_pb2.ValueFormat
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., unit: _Optional[str] = ..., points: _Optional[_Iterable[_Union[PlotPoint, _Mapping]]] = ..., raw_points: _Optional[int] = ..., downsampled: _Optional[bool] = ..., format: _Optional[_Union[_schema_pb2.ValueFormat, _Mapping]] = ...) -> None: ...

class PlotPoint(_message.Message):
    __slots__ = ("x", "y", "z")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    x: _value_pb2.Value
    y: float
    z: float
    def __init__(self, x: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., y: _Optional[float] = ..., z: _Optional[float] = ...) -> None: ...

class PlotMarker(_message.Message):
    __slots__ = ("x", "label", "record_type", "color")
    X_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    x: _value_pb2.Value
    label: str
    record_type: str
    color: str
    def __init__(self, x: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., label: _Optional[str] = ..., record_type: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class PlotFftOptions(_message.Message):
    __slots__ = ("sample_rate_hz", "window", "detrend", "level_unit", "include_dc", "min_frequency_hz", "max_frequency_hz")
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: FftWindow
    detrend: FftDetrend
    level_unit: FrequencyLevelUnit
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[FftWindow, str]] = ..., detrend: _Optional[_Union[FftDetrend, str]] = ..., level_unit: _Optional[_Union[FrequencyLevelUnit, str]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ...) -> None: ...

class PlotIntervalOptions(_message.Message):
    __slots__ = ("bin_width", "sigma", "clip_to_observed")
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...

class PlotHistogramOptions(_message.Message):
    __slots__ = ("bins", "unit")
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

class PlotAnalysis(_message.Message):
    __slots__ = ("histogram", "box_summary", "sigma_interval", "error")
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    BOX_SUMMARY_FIELD_NUMBER: _ClassVar[int]
    SIGMA_INTERVAL_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    histogram: PlotHistogramAnalysis
    box_summary: PlotBoxSummaryAnalysis
    sigma_interval: PlotSigmaIntervalAnalysis
    error: PlotAnalysisError
    def __init__(self, histogram: _Optional[_Union[PlotHistogramAnalysis, _Mapping]] = ..., box_summary: _Optional[_Union[PlotBoxSummaryAnalysis, _Mapping]] = ..., sigma_interval: _Optional[_Union[PlotSigmaIntervalAnalysis, _Mapping]] = ..., error: _Optional[_Union[PlotAnalysisError, _Mapping]] = ...) -> None: ...

class PlotAnalysisError(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class PlotHistogramAnalysis(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotHistogramSeries]
    def __init__(self, series: _Optional[_Iterable[_Union[PlotHistogramSeries, _Mapping]]] = ...) -> None: ...

class PlotHistogramSeries(_message.Message):
    __slots__ = ("key", "bins", "count", "statistics")
    KEY_FIELD_NUMBER: _ClassVar[int]
    BINS_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    STATISTICS_FIELD_NUMBER: _ClassVar[int]
    key: str
    bins: _containers.RepeatedCompositeFieldContainer[PlotHistogramBin]
    count: int
    statistics: PlotHistogramStatistics
    def __init__(self, key: _Optional[str] = ..., bins: _Optional[_Iterable[_Union[PlotHistogramBin, _Mapping]]] = ..., count: _Optional[int] = ..., statistics: _Optional[_Union[PlotHistogramStatistics, _Mapping]] = ...) -> None: ...

class PlotHistogramStatistics(_message.Message):
    __slots__ = ("mean", "median", "std", "rms", "min", "max")
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    STD_FIELD_NUMBER: _ClassVar[int]
    RMS_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    mean: float
    median: float
    std: float
    rms: float
    min: float
    max: float
    def __init__(self, mean: _Optional[float] = ..., median: _Optional[float] = ..., std: _Optional[float] = ..., rms: _Optional[float] = ..., min: _Optional[float] = ..., max: _Optional[float] = ...) -> None: ...

class PlotHistogramBin(_message.Message):
    __slots__ = ("lower", "upper", "count")
    LOWER_FIELD_NUMBER: _ClassVar[int]
    UPPER_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    lower: float
    upper: float
    count: int
    def __init__(self, lower: _Optional[float] = ..., upper: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

class PlotBoxSummaryAnalysis(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotBoxSummary]
    def __init__(self, series: _Optional[_Iterable[_Union[PlotBoxSummary, _Mapping]]] = ...) -> None: ...

class PlotBoxSummary(_message.Message):
    __slots__ = ("key", "count", "min", "q1", "median", "q3", "max", "lower_whisker", "upper_whisker", "outliers", "label")
    KEY_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    Q1_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    Q3_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LOWER_WHISKER_FIELD_NUMBER: _ClassVar[int]
    UPPER_WHISKER_FIELD_NUMBER: _ClassVar[int]
    OUTLIERS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    key: str
    count: int
    min: float
    q1: float
    median: float
    q3: float
    max: float
    lower_whisker: float
    upper_whisker: float
    outliers: _containers.RepeatedScalarFieldContainer[float]
    label: str
    def __init__(self, key: _Optional[str] = ..., count: _Optional[int] = ..., min: _Optional[float] = ..., q1: _Optional[float] = ..., median: _Optional[float] = ..., q3: _Optional[float] = ..., max: _Optional[float] = ..., lower_whisker: _Optional[float] = ..., upper_whisker: _Optional[float] = ..., outliers: _Optional[_Iterable[float]] = ..., label: _Optional[str] = ...) -> None: ...

class PlotSigmaIntervalAnalysis(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotSigmaSeries]
    def __init__(self, series: _Optional[_Iterable[_Union[PlotSigmaSeries, _Mapping]]] = ...) -> None: ...

class PlotSigmaSeries(_message.Message):
    __slots__ = ("key", "bins")
    KEY_FIELD_NUMBER: _ClassVar[int]
    BINS_FIELD_NUMBER: _ClassVar[int]
    key: str
    bins: _containers.RepeatedCompositeFieldContainer[PlotSigmaBin]
    def __init__(self, key: _Optional[str] = ..., bins: _Optional[_Iterable[_Union[PlotSigmaBin, _Mapping]]] = ...) -> None: ...

class PlotSigmaBin(_message.Message):
    __slots__ = ("x", "count", "mean", "lower", "upper")
    X_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    LOWER_FIELD_NUMBER: _ClassVar[int]
    UPPER_FIELD_NUMBER: _ClassVar[int]
    x: float
    count: int
    mean: float
    lower: float
    upper: float
    def __init__(self, x: _Optional[float] = ..., count: _Optional[int] = ..., mean: _Optional[float] = ..., lower: _Optional[float] = ..., upper: _Optional[float] = ...) -> None: ...

class ListAnalysesRequest(_message.Message):
    __slots__ = ("limit", "after_id")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_id: str
    def __init__(self, limit: _Optional[int] = ..., after_id: _Optional[str] = ...) -> None: ...

class ListAnalysesReply(_message.Message):
    __slots__ = ("definitions", "next_after_id")
    DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    definitions: _containers.RepeatedCompositeFieldContainer[AnalysisDefinition]
    next_after_id: str
    def __init__(self, definitions: _Optional[_Iterable[_Union[AnalysisDefinition, _Mapping]]] = ..., next_after_id: _Optional[str] = ...) -> None: ...

class AnalysisDefinition(_message.Message):
    __slots__ = ("id", "version", "label", "description", "parameters", "outputs", "max_samples", "max_output_values")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    MAX_SAMPLES_FIELD_NUMBER: _ClassVar[int]
    MAX_OUTPUT_VALUES_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    label: str
    description: str
    parameters: _containers.RepeatedCompositeFieldContainer[_catalog_pb2.ParameterSchema]
    outputs: _containers.RepeatedCompositeFieldContainer[AnalysisOutput]
    max_samples: int
    max_output_values: int
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., parameters: _Optional[_Iterable[_Union[_catalog_pb2.ParameterSchema, _Mapping]]] = ..., outputs: _Optional[_Iterable[_Union[AnalysisOutput, _Mapping]]] = ..., max_samples: _Optional[int] = ..., max_output_values: _Optional[int] = ...) -> None: ...

class AnalysisOutput(_message.Message):
    __slots__ = ("key", "label", "unit", "axis")
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    unit: str
    axis: AnalysisAxis
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., unit: _Optional[str] = ..., axis: _Optional[_Union[AnalysisAxis, _Mapping]] = ...) -> None: ...

class AnalysisAxis(_message.Message):
    __slots__ = ("key", "unit")
    KEY_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    key: str
    unit: str
    def __init__(self, key: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...

class AnalysisSelection(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: _value_pb2.ValueObject
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class AnalysisComputation(_message.Message):
    __slots__ = ("definition", "selection", "source", "series")
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    definition: AnalysisDefinition
    selection: AnalysisSelection
    source: AnalysisEvidence
    series: _containers.RepeatedCompositeFieldContainer[AnalysisSeries]
    def __init__(self, definition: _Optional[_Union[AnalysisDefinition, _Mapping]] = ..., selection: _Optional[_Union[AnalysisSelection, _Mapping]] = ..., source: _Optional[_Union[AnalysisEvidence, _Mapping]] = ..., series: _Optional[_Iterable[_Union[AnalysisSeries, _Mapping]]] = ...) -> None: ...

class AnalysisEvidence(_message.Message):
    __slots__ = ("epoch", "stream", "scope", "telemetry_streams", "record_sequence", "window_s", "fields", "x_key")
    class TelemetryStreamsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_STREAMS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    X_KEY_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    scope: _common_pb2.EvidenceScope
    telemetry_streams: _containers.ScalarMap[str, int]
    record_sequence: int
    window_s: float
    fields: _containers.RepeatedScalarFieldContainer[str]
    x_key: str
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., telemetry_streams: _Optional[_Mapping[str, int]] = ..., record_sequence: _Optional[int] = ..., window_s: _Optional[float] = ..., fields: _Optional[_Iterable[str]] = ..., x_key: _Optional[str] = ...) -> None: ...

class AnalysisSeries(_message.Message):
    __slots__ = ("key", "label", "origin", "input_samples", "first_axis", "last_axis", "outputs")
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    INPUT_SAMPLES_FIELD_NUMBER: _ClassVar[int]
    FIRST_AXIS_FIELD_NUMBER: _ClassVar[int]
    LAST_AXIS_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    origin: str
    input_samples: int
    first_axis: float
    last_axis: float
    outputs: _containers.RepeatedCompositeFieldContainer[AnalysisValues]
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., origin: _Optional[str] = ..., input_samples: _Optional[int] = ..., first_axis: _Optional[float] = ..., last_axis: _Optional[float] = ..., outputs: _Optional[_Iterable[_Union[AnalysisValues, _Mapping]]] = ...) -> None: ...

class AnalysisValues(_message.Message):
    __slots__ = ("key", "unit", "axis_unit", "axis", "values", "display_indices")
    KEY_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_INDICES_FIELD_NUMBER: _ClassVar[int]
    key: str
    unit: str
    axis_unit: str
    axis: _containers.RepeatedScalarFieldContainer[float]
    values: _containers.RepeatedScalarFieldContainer[float]
    display_indices: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, key: _Optional[str] = ..., unit: _Optional[str] = ..., axis_unit: _Optional[str] = ..., axis: _Optional[_Iterable[float]] = ..., values: _Optional[_Iterable[float]] = ..., display_indices: _Optional[_Iterable[int]] = ...) -> None: ...
