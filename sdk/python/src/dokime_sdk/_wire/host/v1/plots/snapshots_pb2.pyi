from dokime_sdk._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import expressions_pb2 as _expressions_pb2
from dokime_sdk._wire.model.v1 import plots_pb2 as _plots_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PlotCartesianMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_CARTESIAN_MODE_UNSPECIFIED: _ClassVar[PlotCartesianMode]
    PLOT_CARTESIAN_MODE_ALIGNED: _ClassVar[PlotCartesianMode]
    PLOT_CARTESIAN_MODE_FACETED: _ClassVar[PlotCartesianMode]

class PlotStreamKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_STREAM_KIND_UNSPECIFIED: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_SNAPSHOT: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_REPLACE: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_DELTA: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_STATISTICS: _ClassVar[PlotStreamKind]
    PLOT_STREAM_KIND_ERROR: _ClassVar[PlotStreamKind]
PLOT_CARTESIAN_MODE_UNSPECIFIED: PlotCartesianMode
PLOT_CARTESIAN_MODE_ALIGNED: PlotCartesianMode
PLOT_CARTESIAN_MODE_FACETED: PlotCartesianMode
PLOT_STREAM_KIND_UNSPECIFIED: PlotStreamKind
PLOT_STREAM_KIND_SNAPSHOT: PlotStreamKind
PLOT_STREAM_KIND_REPLACE: PlotStreamKind
PLOT_STREAM_KIND_DELTA: PlotStreamKind
PLOT_STREAM_KIND_STATISTICS: PlotStreamKind
PLOT_STREAM_KIND_ERROR: PlotStreamKind

class SnapshotsListRequest(_message.Message):
    __slots__ = ("requests", "at", "page")
    REQUESTS_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    requests: _containers.RepeatedCompositeFieldContainer[PlotRequest]
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, requests: _Optional[_Iterable[_Union[PlotRequest, _Mapping]]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class SnapshotsListResponse(_message.Message):
    __slots__ = ("envelopes", "resolved_at", "next_page_token")
    ENVELOPES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    envelopes: _containers.RepeatedCompositeFieldContainer[PlotEnvelope]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, envelopes: _Optional[_Iterable[_Union[PlotEnvelope, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

class SnapshotsWatchRequest(_message.Message):
    __slots__ = ("requests", "at", "page")
    REQUESTS_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    requests: _containers.RepeatedCompositeFieldContainer[PlotRequest]
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, requests: _Optional[_Iterable[_Union[PlotRequest, _Mapping]]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class SnapshotsWatchResponse(_message.Message):
    __slots__ = ("updates", "resolved_at", "opening", "resync_required")
    UPDATES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    updates: _containers.RepeatedCompositeFieldContainer[PlotStreamUpdate]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, updates: _Optional[_Iterable[_Union[PlotStreamUpdate, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class ContourGrid(_message.Message):
    __slots__ = ("columns", "rows", "x_edges", "y_edges", "values", "valid")
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    X_EDGES_FIELD_NUMBER: _ClassVar[int]
    Y_EDGES_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    VALID_FIELD_NUMBER: _ClassVar[int]
    columns: int
    rows: int
    x_edges: _containers.RepeatedScalarFieldContainer[float]
    y_edges: _containers.RepeatedScalarFieldContainer[float]
    values: _containers.RepeatedScalarFieldContainer[float]
    valid: _containers.RepeatedScalarFieldContainer[bool]
    def __init__(self, columns: _Optional[int] = ..., rows: _Optional[int] = ..., x_edges: _Optional[_Iterable[float]] = ..., y_edges: _Optional[_Iterable[float]] = ..., values: _Optional[_Iterable[float]] = ..., valid: _Optional[_Iterable[bool]] = ...) -> None: ...

class GridCell(_message.Message):
    __slots__ = ("row", "column", "label")
    ROW_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    row: int
    column: int
    label: str
    def __init__(self, row: _Optional[int] = ..., column: _Optional[int] = ..., label: _Optional[str] = ...) -> None: ...

class GridFrame(_message.Message):
    __slots__ = ("instant", "values", "valid", "highlights")
    INSTANT_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    VALID_FIELD_NUMBER: _ClassVar[int]
    HIGHLIGHTS_FIELD_NUMBER: _ClassVar[int]
    instant: str
    values: _containers.RepeatedScalarFieldContainer[float]
    valid: _containers.RepeatedScalarFieldContainer[bool]
    highlights: _containers.RepeatedCompositeFieldContainer[GridCell]
    def __init__(self, instant: _Optional[str] = ..., values: _Optional[_Iterable[float]] = ..., valid: _Optional[_Iterable[bool]] = ..., highlights: _Optional[_Iterable[_Union[GridCell, _Mapping]]] = ...) -> None: ...

class GridFrames(_message.Message):
    __slots__ = ("rows", "columns", "frames", "summary_label")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    FRAMES_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_LABEL_FIELD_NUMBER: _ClassVar[int]
    rows: _containers.RepeatedCompositeFieldContainer[_plots_pb2.GridAxisEntry]
    columns: _containers.RepeatedCompositeFieldContainer[_plots_pb2.GridAxisEntry]
    frames: _containers.RepeatedCompositeFieldContainer[GridFrame]
    summary_label: str
    def __init__(self, rows: _Optional[_Iterable[_Union[_plots_pb2.GridAxisEntry, _Mapping]]] = ..., columns: _Optional[_Iterable[_Union[_plots_pb2.GridAxisEntry, _Mapping]]] = ..., frames: _Optional[_Iterable[_Union[GridFrame, _Mapping]]] = ..., summary_label: _Optional[str] = ...) -> None: ...

class NullableNumber(_message.Message):
    __slots__ = ("value",)
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: float
    def __init__(self, value: _Optional[float] = ...) -> None: ...

class PlotAnalysis(_message.Message):
    __slots__ = ("result",)
    RESULT_FIELD_NUMBER: _ClassVar[int]
    result: PlotAnalysisResult
    def __init__(self, result: _Optional[_Union[PlotAnalysisResult, _Mapping]] = ...) -> None: ...

class PlotAnalysisError(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class PlotAnalysisResult(_message.Message):
    __slots__ = ("contour", "grid_chart", "histogram", "box_plot", "sigma_interval", "error")
    CONTOUR_FIELD_NUMBER: _ClassVar[int]
    GRID_CHART_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    BOX_PLOT_FIELD_NUMBER: _ClassVar[int]
    SIGMA_INTERVAL_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    contour: ContourGrid
    grid_chart: GridFrames
    histogram: PlotHistogramAnalysis
    box_plot: PlotBoxSummaryAnalysis
    sigma_interval: PlotSigmaIntervalAnalysis
    error: PlotAnalysisError
    def __init__(self, contour: _Optional[_Union[ContourGrid, _Mapping]] = ..., grid_chart: _Optional[_Union[GridFrames, _Mapping]] = ..., histogram: _Optional[_Union[PlotHistogramAnalysis, _Mapping]] = ..., box_plot: _Optional[_Union[PlotBoxSummaryAnalysis, _Mapping]] = ..., sigma_interval: _Optional[_Union[PlotSigmaIntervalAnalysis, _Mapping]] = ..., error: _Optional[_Union[PlotAnalysisError, _Mapping]] = ...) -> None: ...

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
    min: _scalar_pb2.Value
    q1: _scalar_pb2.Value
    median: _scalar_pb2.Value
    q3: _scalar_pb2.Value
    max: _scalar_pb2.Value
    lower_whisker: float
    upper_whisker: float
    outliers: _containers.RepeatedScalarFieldContainer[float]
    label: str
    def __init__(self, key: _Optional[str] = ..., count: _Optional[int] = ..., min: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., q1: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., median: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., q3: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., max: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., lower_whisker: _Optional[float] = ..., upper_whisker: _Optional[float] = ..., outliers: _Optional[_Iterable[float]] = ..., label: _Optional[str] = ...) -> None: ...

class PlotBoxSummaryAnalysis(_message.Message):
    __slots__ = ("series", "total")
    SERIES_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotBoxSummary]
    total: int
    def __init__(self, series: _Optional[_Iterable[_Union[PlotBoxSummary, _Mapping]]] = ..., total: _Optional[int] = ...) -> None: ...

class PlotColumn(_message.Message):
    __slots__ = ("instants", "x", "y", "z", "totals")
    INSTANTS_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    TOTALS_FIELD_NUMBER: _ClassVar[int]
    instants: _containers.RepeatedScalarFieldContainer[str]
    x: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    y: _containers.RepeatedCompositeFieldContainer[NullableNumber]
    z: _containers.RepeatedCompositeFieldContainer[NullableNumber]
    totals: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    def __init__(self, instants: _Optional[_Iterable[str]] = ..., x: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ..., y: _Optional[_Iterable[_Union[NullableNumber, _Mapping]]] = ..., z: _Optional[_Iterable[_Union[NullableNumber, _Mapping]]] = ..., totals: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ...) -> None: ...

class PlotDomain(_message.Message):
    __slots__ = ("min", "max")
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    min: float
    max: float
    def __init__(self, min: _Optional[float] = ..., max: _Optional[float] = ...) -> None: ...

class PlotEnvelope(_message.Message):
    __slots__ = ("plot_id", "revision", "epoch", "selection", "payload", "diagnostics", "warnings")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICS_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    revision: int
    epoch: str
    selection: str
    payload: PlotPayload
    diagnostics: PlotPayloadDiagnostics
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, plot_id: _Optional[str] = ..., revision: _Optional[int] = ..., epoch: _Optional[str] = ..., selection: _Optional[str] = ..., payload: _Optional[_Union[PlotPayload, _Mapping]] = ..., diagnostics: _Optional[_Union[PlotPayloadDiagnostics, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class PlotFrame(_message.Message):
    __slots__ = ("mode", "time", "series", "x_domain", "y_domain", "point_count", "signature")
    MODE_FIELD_NUMBER: _ClassVar[int]
    TIME_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    X_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    Y_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    mode: PlotCartesianMode
    time: _containers.RepeatedScalarFieldContainer[float]
    series: _containers.RepeatedCompositeFieldContainer[PlotColumn]
    x_domain: PlotDomain
    y_domain: PlotDomain
    point_count: int
    signature: str
    def __init__(self, mode: _Optional[_Union[PlotCartesianMode, str]] = ..., time: _Optional[_Iterable[float]] = ..., series: _Optional[_Iterable[_Union[PlotColumn, _Mapping]]] = ..., x_domain: _Optional[_Union[PlotDomain, _Mapping]] = ..., y_domain: _Optional[_Union[PlotDomain, _Mapping]] = ..., point_count: _Optional[int] = ..., signature: _Optional[str] = ...) -> None: ...

class PlotHistogramAnalysis(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotHistogramSeries]
    def __init__(self, series: _Optional[_Iterable[_Union[PlotHistogramSeries, _Mapping]]] = ...) -> None: ...

class PlotHistogramBin(_message.Message):
    __slots__ = ("lower", "upper", "count")
    LOWER_FIELD_NUMBER: _ClassVar[int]
    UPPER_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    lower: float
    upper: float
    count: int
    def __init__(self, lower: _Optional[float] = ..., upper: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

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
    mean: _scalar_pb2.Value
    median: _scalar_pb2.Value
    std: float
    rms: float
    min: _scalar_pb2.Value
    max: _scalar_pb2.Value
    def __init__(self, mean: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., median: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., std: _Optional[float] = ..., rms: _Optional[float] = ..., min: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., max: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...

class PlotMarker(_message.Message):
    __slots__ = ("x", "label", "record_type", "color")
    X_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    x: _scalar_pb2.Value
    label: str
    record_type: str
    color: str
    def __init__(self, x: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., label: _Optional[str] = ..., record_type: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class PlotPayload(_message.Message):
    __slots__ = ("stream", "x_key", "series_by", "context", "series", "frame", "markers", "max_points", "window_s", "generated_at", "x_format", "run_started_at", "cycle_started_at", "analysis", "summaries", "bucket_width_s", "bucket_window", "replaced_buckets")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    X_KEY_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    FRAME_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    X_FORMAT_FIELD_NUMBER: _ClassVar[int]
    RUN_STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_FIELD_NUMBER: _ClassVar[int]
    SUMMARIES_FIELD_NUMBER: _ClassVar[int]
    BUCKET_WIDTH_S_FIELD_NUMBER: _ClassVar[int]
    BUCKET_WINDOW_FIELD_NUMBER: _ClassVar[int]
    REPLACED_BUCKETS_FIELD_NUMBER: _ClassVar[int]
    stream: str
    x_key: str
    series_by: str
    context: _scalar_pb2.Value
    series: _containers.RepeatedCompositeFieldContainer[PlotSeries]
    frame: PlotFrame
    markers: _containers.RepeatedCompositeFieldContainer[PlotMarker]
    max_points: int
    window_s: float
    generated_at: str
    x_format: _record_schemas_pb2.ValueFormat
    run_started_at: str
    cycle_started_at: str
    analysis: PlotAnalysis
    summaries: _containers.RepeatedCompositeFieldContainer[PlotSeriesSummary]
    bucket_width_s: float
    bucket_window: PlotWindow
    replaced_buckets: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, stream: _Optional[str] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., context: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., series: _Optional[_Iterable[_Union[PlotSeries, _Mapping]]] = ..., frame: _Optional[_Union[PlotFrame, _Mapping]] = ..., markers: _Optional[_Iterable[_Union[PlotMarker, _Mapping]]] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., generated_at: _Optional[str] = ..., x_format: _Optional[_Union[_record_schemas_pb2.ValueFormat, _Mapping]] = ..., run_started_at: _Optional[str] = ..., cycle_started_at: _Optional[str] = ..., analysis: _Optional[_Union[PlotAnalysis, _Mapping]] = ..., summaries: _Optional[_Iterable[_Union[PlotSeriesSummary, _Mapping]]] = ..., bucket_width_s: _Optional[float] = ..., bucket_window: _Optional[_Union[PlotWindow, _Mapping]] = ..., replaced_buckets: _Optional[_Iterable[float]] = ...) -> None: ...

class PlotPayloadDiagnostics(_message.Message):
    __slots__ = ("examined_rows", "resolution_nanos", "series_count", "point_count", "raw_point_count", "downsampled", "max_points", "window_s")
    EXAMINED_ROWS_FIELD_NUMBER: _ClassVar[int]
    RESOLUTION_NANOS_FIELD_NUMBER: _ClassVar[int]
    SERIES_COUNT_FIELD_NUMBER: _ClassVar[int]
    POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    RAW_POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    DOWNSAMPLED_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    examined_rows: int
    resolution_nanos: int
    series_count: int
    point_count: int
    raw_point_count: int
    downsampled: bool
    max_points: int
    window_s: float
    def __init__(self, examined_rows: _Optional[int] = ..., resolution_nanos: _Optional[int] = ..., series_count: _Optional[int] = ..., point_count: _Optional[int] = ..., raw_point_count: _Optional[int] = ..., downsampled: _Optional[bool] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ...) -> None: ...

class PlotSeries(_message.Message):
    __slots__ = ("key", "label", "unit", "raw_points", "downsampled", "format")
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    RAW_POINTS_FIELD_NUMBER: _ClassVar[int]
    DOWNSAMPLED_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    unit: str
    raw_points: int
    downsampled: bool
    format: _record_schemas_pb2.ValueFormat
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., unit: _Optional[str] = ..., raw_points: _Optional[int] = ..., downsampled: _Optional[bool] = ..., format: _Optional[_Union[_record_schemas_pb2.ValueFormat, _Mapping]] = ...) -> None: ...

class PlotSeriesSummary(_message.Message):
    __slots__ = ("series_id", "count", "min", "max", "mean", "median", "std", "rms", "window")
    SERIES_ID_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    STD_FIELD_NUMBER: _ClassVar[int]
    RMS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    series_id: str
    count: int
    min: _scalar_pb2.Value
    max: _scalar_pb2.Value
    mean: _scalar_pb2.Value
    median: _scalar_pb2.Value
    std: float
    rms: float
    window: PlotWindow
    def __init__(self, series_id: _Optional[str] = ..., count: _Optional[int] = ..., min: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., max: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., mean: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., median: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., std: _Optional[float] = ..., rms: _Optional[float] = ..., window: _Optional[_Union[PlotWindow, _Mapping]] = ...) -> None: ...

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

class PlotSigmaIntervalAnalysis(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[PlotSigmaSeries]
    def __init__(self, series: _Optional[_Iterable[_Union[PlotSigmaSeries, _Mapping]]] = ...) -> None: ...

class PlotSigmaSeries(_message.Message):
    __slots__ = ("key", "bins", "raw_points")
    KEY_FIELD_NUMBER: _ClassVar[int]
    BINS_FIELD_NUMBER: _ClassVar[int]
    RAW_POINTS_FIELD_NUMBER: _ClassVar[int]
    key: str
    bins: _containers.RepeatedCompositeFieldContainer[PlotSigmaBin]
    raw_points: int
    def __init__(self, key: _Optional[str] = ..., bins: _Optional[_Iterable[_Union[PlotSigmaBin, _Mapping]]] = ..., raw_points: _Optional[int] = ...) -> None: ...

class PlotStreamUpdate(_message.Message):
    __slots__ = ("kind", "envelope")
    KIND_FIELD_NUMBER: _ClassVar[int]
    ENVELOPE_FIELD_NUMBER: _ClassVar[int]
    kind: PlotStreamKind
    envelope: PlotEnvelope
    def __init__(self, kind: _Optional[_Union[PlotStreamKind, str]] = ..., envelope: _Optional[_Union[PlotEnvelope, _Mapping]] = ...) -> None: ...

class PlotRequest(_message.Message):
    __slots__ = ("arrangement", "pareto", "hole", "xy_interval", "grid", "reduction", "render", "y_viewport", "panel_height", "plot_id", "plot_type", "style", "x_scale", "y_scale", "stream", "y_keys", "marker_record_types", "filters", "x_key", "series_by", "x_expression", "y_expressions", "series_by_expression", "z_stream", "z_field", "unit", "max_points", "window_s", "scope", "technique", "z_expression", "fft", "interval", "histogram", "summary_window", "viewport", "panel_width", "at")
    ARRANGEMENT_FIELD_NUMBER: _ClassVar[int]
    PARETO_FIELD_NUMBER: _ClassVar[int]
    HOLE_FIELD_NUMBER: _ClassVar[int]
    XY_INTERVAL_FIELD_NUMBER: _ClassVar[int]
    GRID_FIELD_NUMBER: _ClassVar[int]
    REDUCTION_FIELD_NUMBER: _ClassVar[int]
    RENDER_FIELD_NUMBER: _ClassVar[int]
    Y_VIEWPORT_FIELD_NUMBER: _ClassVar[int]
    PANEL_HEIGHT_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_TYPE_FIELD_NUMBER: _ClassVar[int]
    STYLE_FIELD_NUMBER: _ClassVar[int]
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
    TECHNIQUE_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_WINDOW_FIELD_NUMBER: _ClassVar[int]
    VIEWPORT_FIELD_NUMBER: _ClassVar[int]
    PANEL_WIDTH_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    arrangement: _plots_pb2.BarArrangement
    pareto: bool
    hole: float
    xy_interval: _plots_pb2.XyInterval
    grid: _plots_pb2.GridDefinition
    reduction: _plots_pb2.Reduction
    render: _plots_pb2.ContourRender
    y_viewport: PlotWindow
    panel_height: int
    plot_id: str
    plot_type: _plots_pb2.PlotKind
    style: _plots_pb2.LineStyle
    x_scale: _plots_pb2.AxisScale
    y_scale: _plots_pb2.AxisScale
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    x_key: str
    series_by: str
    x_expression: _expressions_pb2.DataExpression
    y_expressions: _containers.RepeatedCompositeFieldContainer[_expressions_pb2.DataExpression]
    series_by_expression: _expressions_pb2.DataExpression
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: _evidence_pb2.EvidenceScope
    technique: _plots_pb2.Downsample
    z_expression: _expressions_pb2.DataExpression
    fft: _plots_pb2.PlotFftOptions
    interval: _plots_pb2.PlotIntervalOptions
    histogram: _plots_pb2.PlotHistogramOptions
    summary_window: PlotWindow
    viewport: PlotWindow
    panel_width: int
    at: _position_pb2.ReadCut
    def __init__(self, arrangement: _Optional[_Union[_plots_pb2.BarArrangement, str]] = ..., pareto: _Optional[bool] = ..., hole: _Optional[float] = ..., xy_interval: _Optional[_Union[_plots_pb2.XyInterval, str]] = ..., grid: _Optional[_Union[_plots_pb2.GridDefinition, _Mapping]] = ..., reduction: _Optional[_Union[_plots_pb2.Reduction, str]] = ..., render: _Optional[_Union[_plots_pb2.ContourRender, str]] = ..., y_viewport: _Optional[_Union[PlotWindow, _Mapping]] = ..., panel_height: _Optional[int] = ..., plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[_plots_pb2.PlotKind, str]] = ..., style: _Optional[_Union[_plots_pb2.LineStyle, str]] = ..., x_scale: _Optional[_Union[_plots_pb2.AxisScale, str]] = ..., y_scale: _Optional[_Union[_plots_pb2.AxisScale, str]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[_expressions_pb2.DataExpression, _Mapping]]] = ..., series_by_expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., technique: _Optional[_Union[_plots_pb2.Downsample, str]] = ..., z_expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., fft: _Optional[_Union[_plots_pb2.PlotFftOptions, _Mapping]] = ..., interval: _Optional[_Union[_plots_pb2.PlotIntervalOptions, _Mapping]] = ..., histogram: _Optional[_Union[_plots_pb2.PlotHistogramOptions, _Mapping]] = ..., summary_window: _Optional[_Union[PlotWindow, _Mapping]] = ..., viewport: _Optional[_Union[PlotWindow, _Mapping]] = ..., panel_width: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class PlotWindow(_message.Message):
    __slots__ = ("to",)
    FROM_FIELD_NUMBER: _ClassVar[int]
    TO_FIELD_NUMBER: _ClassVar[int]
    to: float
    def __init__(self, to: _Optional[float] = ..., **kwargs) -> None: ...
