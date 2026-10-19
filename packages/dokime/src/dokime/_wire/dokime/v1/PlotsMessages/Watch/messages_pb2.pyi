from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("requests",)
    REQUESTS_FIELD_NUMBER: _ClassVar[int]
    requests: _containers.RepeatedCompositeFieldContainer[Row4]
    def __init__(self, requests: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("kind", "envelope")
    KIND_FIELD_NUMBER: _ClassVar[int]
    ENVELOPE_FIELD_NUMBER: _ClassVar[int]
    kind: Row5
    envelope: Row6
    def __init__(self, kind: _Optional[_Union[Row5, _Mapping]] = ..., envelope: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("plot_id", "plot_type", "x_scale", "y_scale", "stream", "y_keys", "marker_record_types", "filters", "x_key", "series_by", "x_expression", "y_expressions", "series_by_expression", "z_stream", "z_field", "unit", "max_points", "window_s", "scope", "cursor", "technique", "z_expression", "fft", "interval", "histogram", "analysis", "summary_window")
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
    SUMMARY_WINDOW_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    plot_type: Row7
    x_scale: Row8
    y_scale: Row8
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[Row9]
    x_key: str
    series_by: str
    x_expression: Row10
    y_expressions: _containers.RepeatedCompositeFieldContainer[Row10]
    series_by_expression: Row10
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: Row11
    cursor: Row12
    technique: Row13
    z_expression: Row10
    fft: Row14
    interval: Row15
    histogram: Row16
    analysis: Row17
    summary_window: Row18
    def __init__(self, plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[Row7, _Mapping]] = ..., x_scale: _Optional[_Union[Row8, _Mapping]] = ..., y_scale: _Optional[_Union[Row8, _Mapping]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[Row10, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., series_by_expression: _Optional[_Union[Row10, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[Row11, _Mapping]] = ..., cursor: _Optional[_Union[Row12, _Mapping]] = ..., technique: _Optional[_Union[Row13, _Mapping]] = ..., z_expression: _Optional[_Union[Row10, _Mapping]] = ..., fft: _Optional[_Union[Row14, _Mapping]] = ..., interval: _Optional[_Union[Row15, _Mapping]] = ..., histogram: _Optional[_Union[Row16, _Mapping]] = ..., analysis: _Optional[_Union[Row17, _Mapping]] = ..., summary_window: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "Snapshot", "Append", "Replace", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    APPEND_FIELD_NUMBER: _ClassVar[int]
    REPLACE_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row19
    Snapshot: Row20
    Append: Row21
    Replace: Row22
    Error: Row23
    def __init__(self, Unspecified: _Optional[_Union[Row19, _Mapping]] = ..., Snapshot: _Optional[_Union[Row20, _Mapping]] = ..., Append: _Optional[_Union[Row21, _Mapping]] = ..., Replace: _Optional[_Union[Row22, _Mapping]] = ..., Error: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("plot_id", "revision", "cursor", "payload", "diagnostics", "warnings")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICS_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    revision: int
    cursor: Row12
    payload: Row24
    diagnostics: Row25
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, plot_id: _Optional[str] = ..., revision: _Optional[int] = ..., cursor: _Optional[_Union[Row12, _Mapping]] = ..., payload: _Optional[_Union[Row24, _Mapping]] = ..., diagnostics: _Optional[_Union[Row25, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "TimeSeries", "Bar", "GroupedBar", "StackedBar", "Pie", "Donut", "Pareto", "XyScatter", "XyLine", "XyInterval", "Trajectory", "Heatmap", "XyValueHeatmap", "WifiChannelOccupancy", "FrequencySeries", "Histogram", "BoxSummary", "AggregateTable")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TIMESERIES_FIELD_NUMBER: _ClassVar[int]
    BAR_FIELD_NUMBER: _ClassVar[int]
    GROUPEDBAR_FIELD_NUMBER: _ClassVar[int]
    STACKEDBAR_FIELD_NUMBER: _ClassVar[int]
    PIE_FIELD_NUMBER: _ClassVar[int]
    DONUT_FIELD_NUMBER: _ClassVar[int]
    PARETO_FIELD_NUMBER: _ClassVar[int]
    XYSCATTER_FIELD_NUMBER: _ClassVar[int]
    XYLINE_FIELD_NUMBER: _ClassVar[int]
    XYINTERVAL_FIELD_NUMBER: _ClassVar[int]
    TRAJECTORY_FIELD_NUMBER: _ClassVar[int]
    HEATMAP_FIELD_NUMBER: _ClassVar[int]
    XYVALUEHEATMAP_FIELD_NUMBER: _ClassVar[int]
    WIFICHANNELOCCUPANCY_FIELD_NUMBER: _ClassVar[int]
    FREQUENCYSERIES_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    BOXSUMMARY_FIELD_NUMBER: _ClassVar[int]
    AGGREGATETABLE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row26
    TimeSeries: Row27
    Bar: Row28
    GroupedBar: Row29
    StackedBar: Row30
    Pie: Row31
    Donut: Row32
    Pareto: Row33
    XyScatter: Row34
    XyLine: Row35
    XyInterval: Row36
    Trajectory: Row37
    Heatmap: Row38
    XyValueHeatmap: Row39
    WifiChannelOccupancy: Row40
    FrequencySeries: Row41
    Histogram: Row42
    BoxSummary: Row43
    AggregateTable: Row44
    def __init__(self, Unspecified: _Optional[_Union[Row26, _Mapping]] = ..., TimeSeries: _Optional[_Union[Row27, _Mapping]] = ..., Bar: _Optional[_Union[Row28, _Mapping]] = ..., GroupedBar: _Optional[_Union[Row29, _Mapping]] = ..., StackedBar: _Optional[_Union[Row30, _Mapping]] = ..., Pie: _Optional[_Union[Row31, _Mapping]] = ..., Donut: _Optional[_Union[Row32, _Mapping]] = ..., Pareto: _Optional[_Union[Row33, _Mapping]] = ..., XyScatter: _Optional[_Union[Row34, _Mapping]] = ..., XyLine: _Optional[_Union[Row35, _Mapping]] = ..., XyInterval: _Optional[_Union[Row36, _Mapping]] = ..., Trajectory: _Optional[_Union[Row37, _Mapping]] = ..., Heatmap: _Optional[_Union[Row38, _Mapping]] = ..., XyValueHeatmap: _Optional[_Union[Row39, _Mapping]] = ..., WifiChannelOccupancy: _Optional[_Union[Row40, _Mapping]] = ..., FrequencySeries: _Optional[_Union[Row41, _Mapping]] = ..., Histogram: _Optional[_Union[Row42, _Mapping]] = ..., BoxSummary: _Optional[_Union[Row43, _Mapping]] = ..., AggregateTable: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Log")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    LOG_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row45
    Linear: Row46
    Log: Row47
    def __init__(self, Unspecified: _Optional[_Union[Row45, _Mapping]] = ..., Linear: _Optional[_Union[Row46, _Mapping]] = ..., Log: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: Row10
    operator: Row48
    value: Row49
    def __init__(self, expression: _Optional[_Union[Row10, _Mapping]] = ..., operator: _Optional[_Union[Row48, _Mapping]] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("source", "transforms", "label")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    source: Row50
    transforms: _containers.RepeatedCompositeFieldContainer[Row51]
    label: str
    def __init__(self, source: _Optional[_Union[Row50, _Mapping]] = ..., transforms: _Optional[_Iterable[_Union[Row51, _Mapping]]] = ..., label: _Optional[str] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row52
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row52, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("telemetry_streams", "record_sequence", "epoch", "selection")
    TELEMETRY_STREAMS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    telemetry_streams: _containers.RepeatedCompositeFieldContainer[Row53]
    record_sequence: int
    epoch: str
    selection: str
    def __init__(self, telemetry_streams: _Optional[_Iterable[_Union[Row53, _Mapping]]] = ..., record_sequence: _Optional[int] = ..., epoch: _Optional[str] = ..., selection: _Optional[str] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row54
    Raw: Row55
    MinMax: Row56
    Lttb: Row57
    Auto: Row58
    def __init__(self, Unspecified: _Optional[_Union[Row54, _Mapping]] = ..., Raw: _Optional[_Union[Row55, _Mapping]] = ..., MinMax: _Optional[_Union[Row56, _Mapping]] = ..., Lttb: _Optional[_Union[Row57, _Mapping]] = ..., Auto: _Optional[_Union[Row58, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("sample_rate_hz", "window", "detrend", "level_unit", "include_dc", "min_frequency_hz", "max_frequency_hz")
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: Row59
    detrend: Row60
    level_unit: Row61
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[Row59, _Mapping]] = ..., detrend: _Optional[_Union[Row60, _Mapping]] = ..., level_unit: _Optional[_Union[Row61, _Mapping]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("bin_width", "sigma", "clip_to_observed")
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("bins", "unit")
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: Row49
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("to",)
    FROM_FIELD_NUMBER: _ClassVar[int]
    TO_FIELD_NUMBER: _ClassVar[int]
    to: float
    def __init__(self, to: _Optional[float] = ..., **kwargs) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ("stream", "x_key", "series_by", "context", "series", "cartesian", "markers", "max_points", "window_s", "generated_at", "x_format", "run_started_at", "cycle_started_at", "analysis", "computed", "summaries")
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
    SUMMARIES_FIELD_NUMBER: _ClassVar[int]
    stream: str
    x_key: str
    series_by: str
    context: Row49
    series: _containers.RepeatedCompositeFieldContainer[Row62]
    cartesian: Row63
    markers: _containers.RepeatedCompositeFieldContainer[Row64]
    max_points: int
    window_s: float
    generated_at: str
    x_format: Row65
    run_started_at: str
    cycle_started_at: str
    analysis: Row66
    computed: Row67
    summaries: _containers.RepeatedCompositeFieldContainer[Row68]
    def __init__(self, stream: _Optional[str] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., context: _Optional[_Union[Row49, _Mapping]] = ..., series: _Optional[_Iterable[_Union[Row62, _Mapping]]] = ..., cartesian: _Optional[_Union[Row63, _Mapping]] = ..., markers: _Optional[_Iterable[_Union[Row64, _Mapping]]] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., generated_at: _Optional[str] = ..., x_format: _Optional[_Union[Row65, _Mapping]] = ..., run_started_at: _Optional[str] = ..., cycle_started_at: _Optional[str] = ..., analysis: _Optional[_Union[Row66, _Mapping]] = ..., computed: _Optional[_Union[Row67, _Mapping]] = ..., summaries: _Optional[_Iterable[_Union[Row68, _Mapping]]] = ...) -> None: ...

class Row25(_message.Message):
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

class Row26(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row37(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row38(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row39(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ("Unspecified", "Eq", "Ne", "Gt", "Gte", "Lt", "Lte", "Contains", "Exists")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    NE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    CONTAINS_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row69
    Eq: Row70
    Ne: Row71
    Gt: Row72
    Gte: Row73
    Lt: Row74
    Lte: Row75
    Contains: Row76
    Exists: Row77
    def __init__(self, Unspecified: _Optional[_Union[Row69, _Mapping]] = ..., Eq: _Optional[_Union[Row70, _Mapping]] = ..., Ne: _Optional[_Union[Row71, _Mapping]] = ..., Gt: _Optional[_Union[Row72, _Mapping]] = ..., Gte: _Optional[_Union[Row73, _Mapping]] = ..., Lt: _Optional[_Union[Row74, _Mapping]] = ..., Lte: _Optional[_Union[Row75, _Mapping]] = ..., Contains: _Optional[_Union[Row76, _Mapping]] = ..., Exists: _Optional[_Union[Row77, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row78]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row78, _Mapping]]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("Telemetry", "Parameter", "Dimension", "Record", "Artifact")
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    Telemetry: Row79
    Parameter: Row80
    Dimension: Row81
    Record: Row82
    Artifact: Row83
    def __init__(self, Telemetry: _Optional[_Union[Row79, _Mapping]] = ..., Parameter: _Optional[_Union[Row80, _Mapping]] = ..., Dimension: _Optional[_Union[Row81, _Mapping]] = ..., Record: _Optional[_Union[Row82, _Mapping]] = ..., Artifact: _Optional[_Union[Row83, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row84
    def __init__(self, kind: _Optional[_Union[Row84, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row85
    def __init__(self, kind: _Optional[_Union[Row85, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row55(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row56(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row57(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row58(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row59(_message.Message):
    __slots__ = ("Unspecified", "Hann", "Rectangular")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    HANN_FIELD_NUMBER: _ClassVar[int]
    RECTANGULAR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row86
    Hann: Row87
    Rectangular: Row88
    def __init__(self, Unspecified: _Optional[_Union[Row86, _Mapping]] = ..., Hann: _Optional[_Union[Row87, _Mapping]] = ..., Rectangular: _Optional[_Union[Row88, _Mapping]] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("Unspecified", "Mean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NONE_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row89
    Mean: Row91
    def __init__(self, Unspecified: _Optional[_Union[Row89, _Mapping]] = ..., Mean: _Optional[_Union[Row91, _Mapping]] = ..., **kwargs) -> None: ...

class Row61(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Db", "Dbu", "Dbm")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    DB_FIELD_NUMBER: _ClassVar[int]
    DBU_FIELD_NUMBER: _ClassVar[int]
    DBM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row92
    Linear: Row93
    Db: Row94
    Dbu: Row95
    Dbm: Row96
    def __init__(self, Unspecified: _Optional[_Union[Row92, _Mapping]] = ..., Linear: _Optional[_Union[Row93, _Mapping]] = ..., Db: _Optional[_Union[Row94, _Mapping]] = ..., Dbu: _Optional[_Union[Row95, _Mapping]] = ..., Dbm: _Optional[_Union[Row96, _Mapping]] = ...) -> None: ...

class Row62(_message.Message):
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
    points: _containers.RepeatedCompositeFieldContainer[Row97]
    raw_points: int
    downsampled: bool
    format: Row65
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., unit: _Optional[str] = ..., points: _Optional[_Iterable[_Union[Row97, _Mapping]]] = ..., raw_points: _Optional[int] = ..., downsampled: _Optional[bool] = ..., format: _Optional[_Union[Row65, _Mapping]] = ...) -> None: ...

class Row63(_message.Message):
    __slots__ = ("mode", "series", "x_domain", "y_domain", "point_count", "signature")
    MODE_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    X_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    Y_DOMAIN_FIELD_NUMBER: _ClassVar[int]
    POINT_COUNT_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    mode: Row98
    series: _containers.RepeatedCompositeFieldContainer[Row99]
    x_domain: Row100
    y_domain: Row100
    point_count: int
    signature: str
    def __init__(self, mode: _Optional[_Union[Row98, _Mapping]] = ..., series: _Optional[_Iterable[_Union[Row99, _Mapping]]] = ..., x_domain: _Optional[_Union[Row100, _Mapping]] = ..., y_domain: _Optional[_Union[Row100, _Mapping]] = ..., point_count: _Optional[int] = ..., signature: _Optional[str] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ("x", "label", "record_type", "color")
    X_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    x: Row49
    label: str
    record_type: str
    color: str
    def __init__(self, x: _Optional[_Union[Row49, _Mapping]] = ..., label: _Optional[str] = ..., record_type: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class Row65(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row101
    def __init__(self, kind: _Optional[_Union[Row101, _Mapping]] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("result",)
    RESULT_FIELD_NUMBER: _ClassVar[int]
    result: Row102
    def __init__(self, result: _Optional[_Union[Row102, _Mapping]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("definition", "selection", "source", "series")
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SERIES_FIELD_NUMBER: _ClassVar[int]
    definition: Row103
    selection: Row17
    source: Row104
    series: _containers.RepeatedCompositeFieldContainer[Row105]
    def __init__(self, definition: _Optional[_Union[Row103, _Mapping]] = ..., selection: _Optional[_Union[Row17, _Mapping]] = ..., source: _Optional[_Union[Row104, _Mapping]] = ..., series: _Optional[_Iterable[_Union[Row105, _Mapping]]] = ...) -> None: ...

class Row68(_message.Message):
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
    min: float
    max: float
    mean: float
    median: float
    std: float
    rms: float
    window: Row18
    def __init__(self, series_id: _Optional[str] = ..., count: _Optional[int] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., mean: _Optional[float] = ..., median: _Optional[float] = ..., std: _Optional[float] = ..., rms: _Optional[float] = ..., window: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row70(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row71(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row72(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row73(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row74(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row75(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row76(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row77(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row78(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row106
    def __init__(self, kind: _Optional[_Union[Row106, _Mapping]] = ...) -> None: ...

class Row79(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row107
    def __init__(self, field_0: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row80(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row108
    def __init__(self, field_0: _Optional[_Union[Row108, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row109
    def __init__(self, field_0: _Optional[_Union[Row109, _Mapping]] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row110
    def __init__(self, field_0: _Optional[_Union[Row110, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row111
    def __init__(self, field_0: _Optional[_Union[Row111, _Mapping]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("Aggregate", "TimeBetween", "TelemetryAtOrBefore", "Detail", "PerCase")
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    TIMEBETWEEN_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYATORBEFORE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PERCASE_FIELD_NUMBER: _ClassVar[int]
    Aggregate: Row112
    TimeBetween: Row113
    TelemetryAtOrBefore: Row114
    Detail: Row115
    PerCase: Row116
    def __init__(self, Aggregate: _Optional[_Union[Row112, _Mapping]] = ..., TimeBetween: _Optional[_Union[Row113, _Mapping]] = ..., TelemetryAtOrBefore: _Optional[_Union[Row114, _Mapping]] = ..., Detail: _Optional[_Union[Row115, _Mapping]] = ..., PerCase: _Optional[_Union[Row116, _Mapping]] = ...) -> None: ...

class Row85(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row117
    Initialization: Row118
    StepAttemptId: Row119
    Cleanup: Row120
    def __init__(self, Default: _Optional[_Union[Row117, _Mapping]] = ..., Initialization: _Optional[_Union[Row118, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row119, _Mapping]] = ..., Cleanup: _Optional[_Union[Row120, _Mapping]] = ...) -> None: ...

class Row86(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row87(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row88(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row89(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row90(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row91(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row92(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row93(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row94(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row95(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row96(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row97(_message.Message):
    __slots__ = ("x", "y", "z")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    x: Row49
    y: float
    z: float
    def __init__(self, x: _Optional[_Union[Row49, _Mapping]] = ..., y: _Optional[float] = ..., z: _Optional[float] = ...) -> None: ...

class Row98(_message.Message):
    __slots__ = ("Unspecified", "Aligned", "Faceted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ALIGNED_FIELD_NUMBER: _ClassVar[int]
    FACETED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row121
    Aligned: Row122
    Faceted: Row123
    def __init__(self, Unspecified: _Optional[_Union[Row121, _Mapping]] = ..., Aligned: _Optional[_Union[Row122, _Mapping]] = ..., Faceted: _Optional[_Union[Row123, _Mapping]] = ...) -> None: ...

class Row99(_message.Message):
    __slots__ = ("x", "y")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    x: _containers.RepeatedScalarFieldContainer[float]
    y: _containers.RepeatedCompositeFieldContainer[Row124]
    def __init__(self, x: _Optional[_Iterable[float]] = ..., y: _Optional[_Iterable[_Union[Row124, _Mapping]]] = ...) -> None: ...

class Row100(_message.Message):
    __slots__ = ("min", "max")
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    min: float
    max: float
    def __init__(self, min: _Optional[float] = ..., max: _Optional[float] = ...) -> None: ...

class Row101(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row125
    Fixed: Row126
    Scientific: Row127
    Engineering: Row128
    SiPrefix: Row129
    AbsoluteTime: Row130
    ElapsedSinceRun: Row131
    ElapsedSinceCycle: Row132
    def __init__(self, Normal: _Optional[_Union[Row125, _Mapping]] = ..., Fixed: _Optional[_Union[Row126, _Mapping]] = ..., Scientific: _Optional[_Union[Row127, _Mapping]] = ..., Engineering: _Optional[_Union[Row128, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row129, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row130, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row131, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row132, _Mapping]] = ...) -> None: ...

class Row102(_message.Message):
    __slots__ = ("Histogram", "BoxSummary", "SigmaInterval", "Error", "WifiOccupancy")
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    BOXSUMMARY_FIELD_NUMBER: _ClassVar[int]
    SIGMAINTERVAL_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    WIFIOCCUPANCY_FIELD_NUMBER: _ClassVar[int]
    Histogram: Row133
    BoxSummary: Row134
    SigmaInterval: Row135
    Error: Row136
    WifiOccupancy: Row137
    def __init__(self, Histogram: _Optional[_Union[Row133, _Mapping]] = ..., BoxSummary: _Optional[_Union[Row134, _Mapping]] = ..., SigmaInterval: _Optional[_Union[Row135, _Mapping]] = ..., Error: _Optional[_Union[Row136, _Mapping]] = ..., WifiOccupancy: _Optional[_Union[Row137, _Mapping]] = ...) -> None: ...

class Row103(_message.Message):
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
    parameters: _containers.RepeatedCompositeFieldContainer[Row138]
    outputs: _containers.RepeatedCompositeFieldContainer[Row139]
    max_samples: int
    max_output_values: int
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., parameters: _Optional[_Iterable[_Union[Row138, _Mapping]]] = ..., outputs: _Optional[_Iterable[_Union[Row139, _Mapping]]] = ..., max_samples: _Optional[int] = ..., max_output_values: _Optional[int] = ...) -> None: ...

class Row104(_message.Message):
    __slots__ = ("epoch", "stream", "scope", "telemetry_streams", "record_sequence", "window_s", "fields", "x_key")
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
    scope: Row11
    telemetry_streams: _containers.RepeatedCompositeFieldContainer[Row140]
    record_sequence: int
    window_s: float
    fields: _containers.RepeatedScalarFieldContainer[str]
    x_key: str
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., scope: _Optional[_Union[Row11, _Mapping]] = ..., telemetry_streams: _Optional[_Iterable[_Union[Row140, _Mapping]]] = ..., record_sequence: _Optional[int] = ..., window_s: _Optional[float] = ..., fields: _Optional[_Iterable[str]] = ..., x_key: _Optional[str] = ...) -> None: ...

class Row105(_message.Message):
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
    outputs: _containers.RepeatedCompositeFieldContainer[Row141]
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., origin: _Optional[str] = ..., input_samples: _Optional[int] = ..., first_axis: _Optional[float] = ..., last_axis: _Optional[float] = ..., outputs: _Optional[_Iterable[_Union[Row141, _Mapping]]] = ...) -> None: ...

class Row106(_message.Message):
    __slots__ = ("Null", "Bool", "Integer", "Unsigned", "Number", "Text", "Datetime", "Array", "Object")
    NULL_FIELD_NUMBER: _ClassVar[int]
    BOOL_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    Null: Row142
    Bool: Row143
    Integer: Row144
    Unsigned: Row145
    Number: Row146
    Text: Row147
    Datetime: Row148
    Array: Row149
    Object: Row150
    def __init__(self, Null: _Optional[_Union[Row142, _Mapping]] = ..., Bool: _Optional[_Union[Row143, _Mapping]] = ..., Integer: _Optional[_Union[Row144, _Mapping]] = ..., Unsigned: _Optional[_Union[Row145, _Mapping]] = ..., Number: _Optional[_Union[Row146, _Mapping]] = ..., Text: _Optional[_Union[Row147, _Mapping]] = ..., Datetime: _Optional[_Union[Row148, _Mapping]] = ..., Array: _Optional[_Union[Row149, _Mapping]] = ..., Object: _Optional[_Union[Row150, _Mapping]] = ...) -> None: ...

class Row107(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row108(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row109(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row110(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class Row111(_message.Message):
    __slots__ = ("artifact",)
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    def __init__(self, artifact: _Optional[str] = ...) -> None: ...

class Row112(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row151
    def __init__(self, field_0: _Optional[_Union[Row151, _Mapping]] = ...) -> None: ...

class Row113(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row152
    def __init__(self, field_0: _Optional[_Union[Row152, _Mapping]] = ...) -> None: ...

class Row114(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row153
    def __init__(self, field_0: _Optional[_Union[Row153, _Mapping]] = ...) -> None: ...

class Row115(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row154
    def __init__(self, field_0: _Optional[_Union[Row154, _Mapping]] = ...) -> None: ...

class Row116(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row155
    def __init__(self, field_0: _Optional[_Union[Row155, _Mapping]] = ...) -> None: ...

class Row117(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row118(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row119(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row120(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row121(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row122(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row123(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row124(_message.Message):
    __slots__ = ("value",)
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: float
    def __init__(self, value: _Optional[float] = ...) -> None: ...

class Row125(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row156
    def __init__(self, field_0: _Optional[_Union[Row156, _Mapping]] = ...) -> None: ...

class Row126(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row157
    def __init__(self, field_0: _Optional[_Union[Row157, _Mapping]] = ...) -> None: ...

class Row127(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row158
    def __init__(self, field_0: _Optional[_Union[Row158, _Mapping]] = ...) -> None: ...

class Row128(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row159
    def __init__(self, field_0: _Optional[_Union[Row159, _Mapping]] = ...) -> None: ...

class Row129(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row160
    def __init__(self, field_0: _Optional[_Union[Row160, _Mapping]] = ...) -> None: ...

class Row130(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row161
    def __init__(self, field_0: _Optional[_Union[Row161, _Mapping]] = ...) -> None: ...

class Row131(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row162
    def __init__(self, field_0: _Optional[_Union[Row162, _Mapping]] = ...) -> None: ...

class Row132(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row162
    def __init__(self, field_0: _Optional[_Union[Row162, _Mapping]] = ...) -> None: ...

class Row133(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row163
    def __init__(self, field_0: _Optional[_Union[Row163, _Mapping]] = ...) -> None: ...

class Row134(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row164
    def __init__(self, field_0: _Optional[_Union[Row164, _Mapping]] = ...) -> None: ...

class Row135(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row165
    def __init__(self, field_0: _Optional[_Union[Row165, _Mapping]] = ...) -> None: ...

class Row136(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row166
    def __init__(self, field_0: _Optional[_Union[Row166, _Mapping]] = ...) -> None: ...

class Row137(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row167
    def __init__(self, field_0: _Optional[_Union[Row167, _Mapping]] = ...) -> None: ...

class Row138(_message.Message):
    __slots__ = ("parameter_id", "metadata", "group_id", "value_type", "unit", "required", "runtime_writable", "expression", "selection", "extensions", "list", "min", "max", "expansion")
    PARAMETER_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_WRITABLE_FIELD_NUMBER: _ClassVar[int]
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    EXTENSIONS_FIELD_NUMBER: _ClassVar[int]
    LIST_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    EXPANSION_FIELD_NUMBER: _ClassVar[int]
    parameter_id: str
    metadata: Row168
    group_id: str
    value_type: Row169
    unit: Row170
    required: bool
    runtime_writable: bool
    expression: Row171
    selection: Row172
    extensions: _containers.RepeatedCompositeFieldContainer[Row173]
    list: Row174
    min: float
    max: float
    expansion: Row175
    def __init__(self, parameter_id: _Optional[str] = ..., metadata: _Optional[_Union[Row168, _Mapping]] = ..., group_id: _Optional[str] = ..., value_type: _Optional[_Union[Row169, _Mapping]] = ..., unit: _Optional[_Union[Row170, _Mapping]] = ..., required: _Optional[bool] = ..., runtime_writable: _Optional[bool] = ..., expression: _Optional[_Union[Row171, _Mapping]] = ..., selection: _Optional[_Union[Row172, _Mapping]] = ..., extensions: _Optional[_Iterable[_Union[Row173, _Mapping]]] = ..., list: _Optional[_Union[Row174, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., expansion: _Optional[_Union[Row175, _Mapping]] = ...) -> None: ...

class Row139(_message.Message):
    __slots__ = ("key", "label", "unit", "axis")
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    unit: str
    axis: Row176
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., unit: _Optional[str] = ..., axis: _Optional[_Union[Row176, _Mapping]] = ...) -> None: ...

class Row140(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row141(_message.Message):
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

class Row142(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row143(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row144(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row145(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row146(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row147(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row148(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row177
    def __init__(self, field_0: _Optional[_Union[Row177, _Mapping]] = ...) -> None: ...

class Row149(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row178
    def __init__(self, field_0: _Optional[_Union[Row178, _Mapping]] = ...) -> None: ...

class Row150(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row179
    def __init__(self, field_0: _Optional[_Union[Row179, _Mapping]] = ...) -> None: ...

class Row151(_message.Message):
    __slots__ = ("aggregate",)
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    aggregate: Row180
    def __init__(self, aggregate: _Optional[_Union[Row180, _Mapping]] = ...) -> None: ...

class Row152(_message.Message):
    __slots__ = ("previous_record_type",)
    PREVIOUS_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    previous_record_type: str
    def __init__(self, previous_record_type: _Optional[str] = ...) -> None: ...

class Row153(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row154(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row155(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row156(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row157(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row158(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row159(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row160(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row161(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row162(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row181
    def __init__(self, unit: _Optional[_Union[Row181, _Mapping]] = ...) -> None: ...

class Row163(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[Row182]
    def __init__(self, series: _Optional[_Iterable[_Union[Row182, _Mapping]]] = ...) -> None: ...

class Row164(_message.Message):
    __slots__ = ("series", "total")
    SERIES_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[Row183]
    total: int
    def __init__(self, series: _Optional[_Iterable[_Union[Row183, _Mapping]]] = ..., total: _Optional[int] = ...) -> None: ...

class Row165(_message.Message):
    __slots__ = ("series",)
    SERIES_FIELD_NUMBER: _ClassVar[int]
    series: _containers.RepeatedCompositeFieldContainer[Row184]
    def __init__(self, series: _Optional[_Iterable[_Union[Row184, _Mapping]]] = ...) -> None: ...

class Row166(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class Row167(_message.Message):
    __slots__ = ("channels", "occupied_channels", "bands", "scans")
    CHANNELS_FIELD_NUMBER: _ClassVar[int]
    OCCUPIED_CHANNELS_FIELD_NUMBER: _ClassVar[int]
    BANDS_FIELD_NUMBER: _ClassVar[int]
    SCANS_FIELD_NUMBER: _ClassVar[int]
    channels: _containers.RepeatedScalarFieldContainer[int]
    occupied_channels: _containers.RepeatedScalarFieldContainer[int]
    bands: _containers.RepeatedCompositeFieldContainer[Row185]
    scans: _containers.RepeatedCompositeFieldContainer[Row186]
    def __init__(self, channels: _Optional[_Iterable[int]] = ..., occupied_channels: _Optional[_Iterable[int]] = ..., bands: _Optional[_Iterable[_Union[Row185, _Mapping]]] = ..., scans: _Optional[_Iterable[_Union[Row186, _Mapping]]] = ...) -> None: ...

class Row168(_message.Message):
    __slots__ = ("identifier", "label", "description", "tags", "attributes", "created_at", "updated_at")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    tags: _containers.RepeatedScalarFieldContainer[str]
    attributes: _containers.RepeatedCompositeFieldContainer[Row187]
    created_at: Row177
    updated_at: Row177
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row187, _Mapping]]] = ..., created_at: _Optional[_Union[Row177, _Mapping]] = ..., updated_at: _Optional[_Union[Row177, _Mapping]] = ...) -> None: ...

class Row169(_message.Message):
    __slots__ = ("Unspecified", "Boolean", "Integer", "Number", "Text", "File", "Object", "Array", "Datetime")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row188
    Boolean: Row189
    Integer: Row190
    Number: Row191
    Text: Row192
    File: Row193
    Object: Row194
    Array: Row195
    Datetime: Row196
    def __init__(self, Unspecified: _Optional[_Union[Row188, _Mapping]] = ..., Boolean: _Optional[_Union[Row189, _Mapping]] = ..., Integer: _Optional[_Union[Row190, _Mapping]] = ..., Number: _Optional[_Union[Row191, _Mapping]] = ..., Text: _Optional[_Union[Row192, _Mapping]] = ..., File: _Optional[_Union[Row193, _Mapping]] = ..., Object: _Optional[_Union[Row194, _Mapping]] = ..., Array: _Optional[_Union[Row195, _Mapping]] = ..., Datetime: _Optional[_Union[Row196, _Mapping]] = ...) -> None: ...

class Row170(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row168
    family: Row197
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row168, _Mapping]] = ..., family: _Optional[_Union[Row197, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row171(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row198]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row198, _Mapping]]] = ...) -> None: ...

class Row172(_message.Message):
    __slots__ = ("choices",)
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    choices: _containers.RepeatedCompositeFieldContainer[Row49]
    def __init__(self, choices: _Optional[_Iterable[_Union[Row49, _Mapping]]] = ...) -> None: ...

class Row173(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row49
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row174(_message.Message):
    __slots__ = ("element_type", "min_items", "max_items")
    ELEMENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    element_type: Row169
    min_items: int
    max_items: int
    def __init__(self, element_type: _Optional[_Union[Row169, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class Row175(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row199
    def __init__(self, kind: _Optional[_Union[Row199, _Mapping]] = ...) -> None: ...

class Row176(_message.Message):
    __slots__ = ("key", "unit")
    KEY_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    key: str
    unit: str
    def __init__(self, key: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...

class Row177(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row178(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row179(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row200]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row200, _Mapping]]] = ...) -> None: ...

class Row180(_message.Message):
    __slots__ = ("Count", "Sum", "Mean", "Median", "Min", "Max", "Rms", "Std", "Percentile")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    SUM_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    RMS_FIELD_NUMBER: _ClassVar[int]
    STD_FIELD_NUMBER: _ClassVar[int]
    PERCENTILE_FIELD_NUMBER: _ClassVar[int]
    Count: Row2
    Sum: Row2
    Mean: Row2
    Median: Row2
    Min: Row2
    Max: Row2
    Rms: Row2
    Std: Row2
    Percentile: Row201
    def __init__(self, Count: _Optional[_Union[Row2, _Mapping]] = ..., Sum: _Optional[_Union[Row2, _Mapping]] = ..., Mean: _Optional[_Union[Row2, _Mapping]] = ..., Median: _Optional[_Union[Row2, _Mapping]] = ..., Min: _Optional[_Union[Row2, _Mapping]] = ..., Max: _Optional[_Union[Row2, _Mapping]] = ..., Rms: _Optional[_Union[Row2, _Mapping]] = ..., Std: _Optional[_Union[Row2, _Mapping]] = ..., Percentile: _Optional[_Union[Row201, _Mapping]] = ...) -> None: ...

class Row181(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row202
    Seconds: Row203
    Milliseconds: Row204
    Minutes: Row205
    def __init__(self, Unspecified: _Optional[_Union[Row202, _Mapping]] = ..., Seconds: _Optional[_Union[Row203, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row204, _Mapping]] = ..., Minutes: _Optional[_Union[Row205, _Mapping]] = ...) -> None: ...

class Row182(_message.Message):
    __slots__ = ("key", "bins", "count", "statistics")
    KEY_FIELD_NUMBER: _ClassVar[int]
    BINS_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    STATISTICS_FIELD_NUMBER: _ClassVar[int]
    key: str
    bins: _containers.RepeatedCompositeFieldContainer[Row206]
    count: int
    statistics: Row207
    def __init__(self, key: _Optional[str] = ..., bins: _Optional[_Iterable[_Union[Row206, _Mapping]]] = ..., count: _Optional[int] = ..., statistics: _Optional[_Union[Row207, _Mapping]] = ...) -> None: ...

class Row183(_message.Message):
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

class Row184(_message.Message):
    __slots__ = ("key", "bins", "raw_points")
    KEY_FIELD_NUMBER: _ClassVar[int]
    BINS_FIELD_NUMBER: _ClassVar[int]
    RAW_POINTS_FIELD_NUMBER: _ClassVar[int]
    key: str
    bins: _containers.RepeatedCompositeFieldContainer[Row208]
    raw_points: int
    def __init__(self, key: _Optional[str] = ..., bins: _Optional[_Iterable[_Union[Row208, _Mapping]]] = ..., raw_points: _Optional[int] = ...) -> None: ...

class Row185(_message.Message):
    __slots__ = ("band", "occupied_channels")
    BAND_FIELD_NUMBER: _ClassVar[int]
    OCCUPIED_CHANNELS_FIELD_NUMBER: _ClassVar[int]
    band: str
    occupied_channels: int
    def __init__(self, band: _Optional[str] = ..., occupied_channels: _Optional[int] = ...) -> None: ...

class Row186(_message.Message):
    __slots__ = ("time", "channels", "bands", "occupied_channels", "total_networks", "peak_count")
    TIME_FIELD_NUMBER: _ClassVar[int]
    CHANNELS_FIELD_NUMBER: _ClassVar[int]
    BANDS_FIELD_NUMBER: _ClassVar[int]
    OCCUPIED_CHANNELS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_NETWORKS_FIELD_NUMBER: _ClassVar[int]
    PEAK_COUNT_FIELD_NUMBER: _ClassVar[int]
    time: str
    channels: _containers.RepeatedCompositeFieldContainer[Row209]
    bands: _containers.RepeatedCompositeFieldContainer[Row210]
    occupied_channels: int
    total_networks: int
    peak_count: int
    def __init__(self, time: _Optional[str] = ..., channels: _Optional[_Iterable[_Union[Row209, _Mapping]]] = ..., bands: _Optional[_Iterable[_Union[Row210, _Mapping]]] = ..., occupied_channels: _Optional[int] = ..., total_networks: _Optional[int] = ..., peak_count: _Optional[int] = ...) -> None: ...

class Row187(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row49
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row188(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row189(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row190(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row191(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row192(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row193(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row194(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row195(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row196(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row197(_message.Message):
    __slots__ = ("Unspecified", "Count", "Distance", "Duration", "Frequency", "Power", "Temperature", "Voltage", "Ratio", "Custom")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    FREQUENCY_FIELD_NUMBER: _ClassVar[int]
    POWER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_FIELD_NUMBER: _ClassVar[int]
    VOLTAGE_FIELD_NUMBER: _ClassVar[int]
    RATIO_FIELD_NUMBER: _ClassVar[int]
    CUSTOM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row211
    Count: Row212
    Distance: Row213
    Duration: Row214
    Frequency: Row215
    Power: Row216
    Temperature: Row217
    Voltage: Row218
    Ratio: Row219
    Custom: Row220
    def __init__(self, Unspecified: _Optional[_Union[Row211, _Mapping]] = ..., Count: _Optional[_Union[Row212, _Mapping]] = ..., Distance: _Optional[_Union[Row213, _Mapping]] = ..., Duration: _Optional[_Union[Row214, _Mapping]] = ..., Frequency: _Optional[_Union[Row215, _Mapping]] = ..., Power: _Optional[_Union[Row216, _Mapping]] = ..., Temperature: _Optional[_Union[Row217, _Mapping]] = ..., Voltage: _Optional[_Union[Row218, _Mapping]] = ..., Ratio: _Optional[_Union[Row219, _Mapping]] = ..., Custom: _Optional[_Union[Row220, _Mapping]] = ...) -> None: ...

class Row198(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row221
    def __init__(self, kind: _Optional[_Union[Row221, _Mapping]] = ...) -> None: ...

class Row199(_message.Message):
    __slots__ = ("Single", "Cartesian", "Zip")
    SINGLE_FIELD_NUMBER: _ClassVar[int]
    CARTESIAN_FIELD_NUMBER: _ClassVar[int]
    ZIP_FIELD_NUMBER: _ClassVar[int]
    Single: Row222
    Cartesian: Row223
    Zip: Row224
    def __init__(self, Single: _Optional[_Union[Row222, _Mapping]] = ..., Cartesian: _Optional[_Union[Row223, _Mapping]] = ..., Zip: _Optional[_Union[Row224, _Mapping]] = ...) -> None: ...

class Row200(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row201(_message.Message):
    __slots__ = ("rank",)
    RANK_FIELD_NUMBER: _ClassVar[int]
    rank: int
    def __init__(self, rank: _Optional[int] = ...) -> None: ...

class Row202(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row203(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row204(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row205(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row206(_message.Message):
    __slots__ = ("lower", "upper", "count")
    LOWER_FIELD_NUMBER: _ClassVar[int]
    UPPER_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    lower: float
    upper: float
    count: int
    def __init__(self, lower: _Optional[float] = ..., upper: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

class Row207(_message.Message):
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

class Row208(_message.Message):
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

class Row209(_message.Message):
    __slots__ = ("channel", "count")
    CHANNEL_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    channel: int
    count: int
    def __init__(self, channel: _Optional[int] = ..., count: _Optional[int] = ...) -> None: ...

class Row210(_message.Message):
    __slots__ = ("band", "occupied_channels", "total_networks", "peak_count")
    BAND_FIELD_NUMBER: _ClassVar[int]
    OCCUPIED_CHANNELS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_NETWORKS_FIELD_NUMBER: _ClassVar[int]
    PEAK_COUNT_FIELD_NUMBER: _ClassVar[int]
    band: str
    occupied_channels: int
    total_networks: int
    peak_count: int
    def __init__(self, band: _Optional[str] = ..., occupied_channels: _Optional[int] = ..., total_networks: _Optional[int] = ..., peak_count: _Optional[int] = ...) -> None: ...

class Row211(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row212(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row213(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row214(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row215(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row216(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row217(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row218(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row219(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row220(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row221(_message.Message):
    __slots__ = ("Literal", "Array", "Range", "Arange", "Linspace", "Reference", "DynamicDefault", "Unary", "Binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    ARANGE_FIELD_NUMBER: _ClassVar[int]
    LINSPACE_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    DYNAMICDEFAULT_FIELD_NUMBER: _ClassVar[int]
    UNARY_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    Literal: Row225
    Array: Row226
    Range: Row227
    Arange: Row228
    Linspace: Row229
    Reference: Row230
    DynamicDefault: Row231
    Unary: Row232
    Binary: Row233
    def __init__(self, Literal: _Optional[_Union[Row225, _Mapping]] = ..., Array: _Optional[_Union[Row226, _Mapping]] = ..., Range: _Optional[_Union[Row227, _Mapping]] = ..., Arange: _Optional[_Union[Row228, _Mapping]] = ..., Linspace: _Optional[_Union[Row229, _Mapping]] = ..., Reference: _Optional[_Union[Row230, _Mapping]] = ..., DynamicDefault: _Optional[_Union[Row231, _Mapping]] = ..., Unary: _Optional[_Union[Row232, _Mapping]] = ..., Binary: _Optional[_Union[Row233, _Mapping]] = ...) -> None: ...

class Row222(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row223(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row224(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row225(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row49
    def __init__(self, field_0: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row226(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row234
    def __init__(self, field_0: _Optional[_Union[Row234, _Mapping]] = ...) -> None: ...

class Row227(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row235
    def __init__(self, field_0: _Optional[_Union[Row235, _Mapping]] = ...) -> None: ...

class Row228(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row236
    def __init__(self, field_0: _Optional[_Union[Row236, _Mapping]] = ...) -> None: ...

class Row229(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row237
    def __init__(self, field_0: _Optional[_Union[Row237, _Mapping]] = ...) -> None: ...

class Row230(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row231(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row238
    def __init__(self, field_0: _Optional[_Union[Row238, _Mapping]] = ...) -> None: ...

class Row232(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row239
    def __init__(self, field_0: _Optional[_Union[Row239, _Mapping]] = ...) -> None: ...

class Row233(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row240
    def __init__(self, field_0: _Optional[_Union[Row240, _Mapping]] = ...) -> None: ...

class Row234(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, values: _Optional[_Iterable[int]] = ...) -> None: ...

class Row235(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: int
    stop: int
    step: int
    def __init__(self, start: _Optional[int] = ..., stop: _Optional[int] = ..., step: _Optional[int] = ...) -> None: ...

class Row236(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    step: float
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

class Row237(_message.Message):
    __slots__ = ("start", "stop", "count")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    count: int
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

class Row238(_message.Message):
    __slots__ = ("key", "fallback")
    KEY_FIELD_NUMBER: _ClassVar[int]
    FALLBACK_FIELD_NUMBER: _ClassVar[int]
    key: Row241
    fallback: int
    def __init__(self, key: _Optional[_Union[Row241, _Mapping]] = ..., fallback: _Optional[int] = ...) -> None: ...

class Row239(_message.Message):
    __slots__ = ("op", "value")
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    op: Row242
    value: int
    def __init__(self, op: _Optional[_Union[Row242, _Mapping]] = ..., value: _Optional[int] = ...) -> None: ...

class Row240(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: Row243
    left: int
    right: int
    def __init__(self, op: _Optional[_Union[Row243, _Mapping]] = ..., left: _Optional[int] = ..., right: _Optional[int] = ...) -> None: ...

class Row241(_message.Message):
    __slots__ = ("Unspecified", "OperatorName", "OperatorEmail")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OPERATORNAME_FIELD_NUMBER: _ClassVar[int]
    OPERATOREMAIL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row244
    OperatorName: Row245
    OperatorEmail: Row246
    def __init__(self, Unspecified: _Optional[_Union[Row244, _Mapping]] = ..., OperatorName: _Optional[_Union[Row245, _Mapping]] = ..., OperatorEmail: _Optional[_Union[Row246, _Mapping]] = ...) -> None: ...

class Row242(_message.Message):
    __slots__ = ("Unspecified", "Negate")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NEGATE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row247
    Negate: Row248
    def __init__(self, Unspecified: _Optional[_Union[Row247, _Mapping]] = ..., Negate: _Optional[_Union[Row248, _Mapping]] = ...) -> None: ...

class Row243(_message.Message):
    __slots__ = ("Unspecified", "Add", "Subtract", "Multiply", "Divide")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ADD_FIELD_NUMBER: _ClassVar[int]
    SUBTRACT_FIELD_NUMBER: _ClassVar[int]
    MULTIPLY_FIELD_NUMBER: _ClassVar[int]
    DIVIDE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row249
    Add: Row250
    Subtract: Row251
    Multiply: Row252
    Divide: Row253
    def __init__(self, Unspecified: _Optional[_Union[Row249, _Mapping]] = ..., Add: _Optional[_Union[Row250, _Mapping]] = ..., Subtract: _Optional[_Union[Row251, _Mapping]] = ..., Multiply: _Optional[_Union[Row252, _Mapping]] = ..., Divide: _Optional[_Union[Row253, _Mapping]] = ...) -> None: ...

class Row244(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row245(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row246(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row247(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row248(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row249(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row250(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row251(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row252(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row253(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
