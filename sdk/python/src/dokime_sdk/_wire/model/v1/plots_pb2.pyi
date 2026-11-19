from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import expressions_pb2 as _expressions_pb2
from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PlotStyle(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_STYLE_UNSPECIFIED: _ClassVar[PlotStyle]
    PLOT_STYLE_LINE: _ClassVar[PlotStyle]
    PLOT_STYLE_POINTS: _ClassVar[PlotStyle]
    PLOT_STYLE_LINE_POINTS: _ClassVar[PlotStyle]
    PLOT_STYLE_STEP: _ClassVar[PlotStyle]

class AxisScale(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AXIS_SCALE_UNSPECIFIED: _ClassVar[AxisScale]
    AXIS_SCALE_LINEAR: _ClassVar[AxisScale]
    AXIS_SCALE_LOG: _ClassVar[AxisScale]

class Reduction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    REDUCTION_UNSPECIFIED: _ClassVar[Reduction]
    REDUCTION_MEAN: _ClassVar[Reduction]
    REDUCTION_MIN: _ClassVar[Reduction]
    REDUCTION_MAX: _ClassVar[Reduction]

class ContourRender(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CONTOUR_RENDER_UNSPECIFIED: _ClassVar[ContourRender]
    CONTOUR_RENDER_CELLS: _ClassVar[ContourRender]
    CONTOUR_RENDER_BANDS: _ClassVar[ContourRender]

class BarArrangement(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    BAR_ARRANGEMENT_UNSPECIFIED: _ClassVar[BarArrangement]
    BAR_ARRANGEMENT_GROUPED: _ClassVar[BarArrangement]
    BAR_ARRANGEMENT_STACKED: _ClassVar[BarArrangement]

class XyInterval(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    XY_INTERVAL_UNSPECIFIED: _ClassVar[XyInterval]
    XY_INTERVAL_SIGMA: _ClassVar[XyInterval]

class PlotKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_KIND_UNSPECIFIED: _ClassVar[PlotKind]
    PLOT_KIND_TIME_SERIES: _ClassVar[PlotKind]
    PLOT_KIND_BAR: _ClassVar[PlotKind]
    PLOT_KIND_PIE: _ClassVar[PlotKind]
    PLOT_KIND_XY: _ClassVar[PlotKind]
    PLOT_KIND_TRAJECTORY: _ClassVar[PlotKind]
    PLOT_KIND_FREQUENCY_SERIES: _ClassVar[PlotKind]
    PLOT_KIND_HISTOGRAM: _ClassVar[PlotKind]
    PLOT_KIND_CONTOUR: _ClassVar[PlotKind]
    PLOT_KIND_GRID_CHART: _ClassVar[PlotKind]
    PLOT_KIND_BOX_PLOT: _ClassVar[PlotKind]

class Downsample(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOWNSAMPLE_UNSPECIFIED: _ClassVar[Downsample]
    DOWNSAMPLE_RAW: _ClassVar[Downsample]
    DOWNSAMPLE_MIN_MAX: _ClassVar[Downsample]
    DOWNSAMPLE_LTTB: _ClassVar[Downsample]
    DOWNSAMPLE_AUTO: _ClassVar[Downsample]

class FftDetrend(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FFT_DETREND_UNSPECIFIED: _ClassVar[FftDetrend]
    FFT_DETREND_NONE: _ClassVar[FftDetrend]
    FFT_DETREND_MEAN: _ClassVar[FftDetrend]

class FftWindow(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FFT_WINDOW_UNSPECIFIED: _ClassVar[FftWindow]
    FFT_WINDOW_HANN: _ClassVar[FftWindow]
    FFT_WINDOW_RECTANGULAR: _ClassVar[FftWindow]

class FrequencyLevelUnit(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FREQUENCY_LEVEL_UNIT_UNSPECIFIED: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_LINEAR: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DB: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DBU: _ClassVar[FrequencyLevelUnit]
    FREQUENCY_LEVEL_UNIT_DBM: _ClassVar[FrequencyLevelUnit]

class LineStyle(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LINE_STYLE_UNSPECIFIED: _ClassVar[LineStyle]
    LINE_STYLE_LINE: _ClassVar[LineStyle]
    LINE_STYLE_POINTS: _ClassVar[LineStyle]
    LINE_STYLE_LINE_POINTS: _ClassVar[LineStyle]
    LINE_STYLE_STEP: _ClassVar[LineStyle]
PLOT_STYLE_UNSPECIFIED: PlotStyle
PLOT_STYLE_LINE: PlotStyle
PLOT_STYLE_POINTS: PlotStyle
PLOT_STYLE_LINE_POINTS: PlotStyle
PLOT_STYLE_STEP: PlotStyle
AXIS_SCALE_UNSPECIFIED: AxisScale
AXIS_SCALE_LINEAR: AxisScale
AXIS_SCALE_LOG: AxisScale
REDUCTION_UNSPECIFIED: Reduction
REDUCTION_MEAN: Reduction
REDUCTION_MIN: Reduction
REDUCTION_MAX: Reduction
CONTOUR_RENDER_UNSPECIFIED: ContourRender
CONTOUR_RENDER_CELLS: ContourRender
CONTOUR_RENDER_BANDS: ContourRender
BAR_ARRANGEMENT_UNSPECIFIED: BarArrangement
BAR_ARRANGEMENT_GROUPED: BarArrangement
BAR_ARRANGEMENT_STACKED: BarArrangement
XY_INTERVAL_UNSPECIFIED: XyInterval
XY_INTERVAL_SIGMA: XyInterval
PLOT_KIND_UNSPECIFIED: PlotKind
PLOT_KIND_TIME_SERIES: PlotKind
PLOT_KIND_BAR: PlotKind
PLOT_KIND_PIE: PlotKind
PLOT_KIND_XY: PlotKind
PLOT_KIND_TRAJECTORY: PlotKind
PLOT_KIND_FREQUENCY_SERIES: PlotKind
PLOT_KIND_HISTOGRAM: PlotKind
PLOT_KIND_CONTOUR: PlotKind
PLOT_KIND_GRID_CHART: PlotKind
PLOT_KIND_BOX_PLOT: PlotKind
DOWNSAMPLE_UNSPECIFIED: Downsample
DOWNSAMPLE_RAW: Downsample
DOWNSAMPLE_MIN_MAX: Downsample
DOWNSAMPLE_LTTB: Downsample
DOWNSAMPLE_AUTO: Downsample
FFT_DETREND_UNSPECIFIED: FftDetrend
FFT_DETREND_NONE: FftDetrend
FFT_DETREND_MEAN: FftDetrend
FFT_WINDOW_UNSPECIFIED: FftWindow
FFT_WINDOW_HANN: FftWindow
FFT_WINDOW_RECTANGULAR: FftWindow
FREQUENCY_LEVEL_UNIT_UNSPECIFIED: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_LINEAR: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DB: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DBU: FrequencyLevelUnit
FREQUENCY_LEVEL_UNIT_DBM: FrequencyLevelUnit
LINE_STYLE_UNSPECIFIED: LineStyle
LINE_STYLE_LINE: LineStyle
LINE_STYLE_POINTS: LineStyle
LINE_STYLE_LINE_POINTS: LineStyle
LINE_STYLE_STEP: LineStyle

@final
class Plot(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    MARKER_RECORDS_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_BINS_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_POINTS_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_LINES_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_BANDS_FIELD_NUMBER: _ClassVar[int]
    ARRANGEMENT_FIELD_NUMBER: _ClassVar[int]
    PARETO_FIELD_NUMBER: _ClassVar[int]
    HOLE_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    REDUCTION_FIELD_NUMBER: _ClassVar[int]
    RENDER_FIELD_NUMBER: _ClassVar[int]
    GRID_FIELD_NUMBER: _ClassVar[int]
    STYLE_FIELD_NUMBER: _ClassVar[int]
    X_SCALE_FIELD_NUMBER: _ClassVar[int]
    Y_SCALE_FIELD_NUMBER: _ClassVar[int]
    INPUT_SAMPLES_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    kind: PlotKind
    x: _expressions_pb2.DataExpression
    y: _containers.RepeatedCompositeFieldContainer[_expressions_pb2.DataExpression]
    z: _expressions_pb2.DataExpression
    series_by: _expressions_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[_expressions_pb2.DocumentFilter]
    unit: str
    marker_records: _containers.RepeatedScalarFieldContainer[str]
    histogram_bins: int
    sample_rate_hz: float
    display_points: int
    reference_lines: _containers.RepeatedCompositeFieldContainer[ReferenceLine]
    reference_bands: _containers.RepeatedCompositeFieldContainer[ReferenceBand]
    arrangement: BarArrangement
    pareto: bool
    hole: float
    interval: XyInterval
    reduction: Reduction
    render: ContourRender
    grid: GridChart
    style: PlotStyle
    x_scale: AxisScale
    y_scale: AxisScale
    input_samples: int
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., kind: _Optional[_Union[PlotKind, str]] = ..., x: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., y: _Optional[_Iterable[_Union[_expressions_pb2.DataExpression, _Mapping]]] = ..., z: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., series_by: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_expressions_pb2.DocumentFilter, _Mapping]]] = ..., unit: _Optional[str] = ..., marker_records: _Optional[_Iterable[str]] = ..., histogram_bins: _Optional[int] = ..., sample_rate_hz: _Optional[float] = ..., display_points: _Optional[int] = ..., reference_lines: _Optional[_Iterable[_Union[ReferenceLine, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[ReferenceBand, _Mapping]]] = ..., arrangement: _Optional[_Union[BarArrangement, str]] = ..., pareto: _Optional[bool] = ..., hole: _Optional[float] = ..., interval: _Optional[_Union[XyInterval, str]] = ..., reduction: _Optional[_Union[Reduction, str]] = ..., render: _Optional[_Union[ContourRender, str]] = ..., grid: _Optional[_Union[GridChart, _Mapping]] = ..., style: _Optional[_Union[PlotStyle, str]] = ..., x_scale: _Optional[_Union[AxisScale, str]] = ..., y_scale: _Optional[_Union[AxisScale, str]] = ..., input_samples: _Optional[int] = ...) -> None: ...

@final
class GridAxisEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    KEY_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    GROUP_FIELD_NUMBER: _ClassVar[int]
    key: str
    label: str
    order: int
    group: str
    def __init__(self, key: _Optional[str] = ..., label: _Optional[str] = ..., order: _Optional[int] = ..., group: _Optional[str] = ...) -> None: ...

@final
class GridHighlight(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ROW_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    row: _expressions_pb2.DataExpression
    column: _expressions_pb2.DataExpression
    label: str
    def __init__(self, row: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., column: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., label: _Optional[str] = ...) -> None: ...

@final
class GridChart(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ROW_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    FRAME_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    HIGHLIGHTS_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_LABEL_FIELD_NUMBER: _ClassVar[int]
    row: _expressions_pb2.DataExpression
    column: _expressions_pb2.DataExpression
    frame: _expressions_pb2.DataExpression
    rows: _containers.RepeatedCompositeFieldContainer[GridAxisEntry]
    columns: _containers.RepeatedCompositeFieldContainer[GridAxisEntry]
    highlights: _containers.RepeatedCompositeFieldContainer[GridHighlight]
    summary_label: str
    def __init__(self, row: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., column: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., frame: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., rows: _Optional[_Iterable[_Union[GridAxisEntry, _Mapping]]] = ..., columns: _Optional[_Iterable[_Union[GridAxisEntry, _Mapping]]] = ..., highlights: _Optional[_Iterable[_Union[GridHighlight, _Mapping]]] = ..., summary_label: _Optional[str] = ...) -> None: ...

@final
class ReferenceLine(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    value: _expressions_pb2.DataExpression
    label: str
    color: str
    def __init__(self, value: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

@final
class ReferenceBand(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    min: _expressions_pb2.DataExpression
    max: _expressions_pb2.DataExpression
    label: str
    color: str
    center: _expressions_pb2.DataExpression
    tolerance: _expressions_pb2.DataExpression
    def __init__(self, min: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., max: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., tolerance: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ...) -> None: ...

@final
class GridDefinition(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ROW_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    FRAME_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    HIGHLIGHTS_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_LABEL_FIELD_NUMBER: _ClassVar[int]
    row: _expressions_pb2.DataExpression
    column: _expressions_pb2.DataExpression
    frame: _expressions_pb2.DataExpression
    rows: _containers.RepeatedCompositeFieldContainer[GridAxisEntry]
    columns: _containers.RepeatedCompositeFieldContainer[GridAxisEntry]
    highlights: _containers.RepeatedCompositeFieldContainer[GridHighlight]
    summary_label: str
    def __init__(self, row: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., column: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., frame: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., rows: _Optional[_Iterable[_Union[GridAxisEntry, _Mapping]]] = ..., columns: _Optional[_Iterable[_Union[GridAxisEntry, _Mapping]]] = ..., highlights: _Optional[_Iterable[_Union[GridHighlight, _Mapping]]] = ..., summary_label: _Optional[str] = ...) -> None: ...

@final
class PlotFftOptions(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    INPUT_SAMPLES_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: FftWindow
    detrend: FftDetrend
    level_unit: FrequencyLevelUnit
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    input_samples: int
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[FftWindow, str]] = ..., detrend: _Optional[_Union[FftDetrend, str]] = ..., level_unit: _Optional[_Union[FrequencyLevelUnit, str]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ..., input_samples: _Optional[int] = ...) -> None: ...

@final
class PlotFilter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: _expressions_pb2.DataExpression
    operator: _expressions_pb2.FilterOperator
    value: _scalar_pb2.Value
    def __init__(self, expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., operator: _Optional[_Union[_expressions_pb2.FilterOperator, str]] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...

@final
class PlotHistogramOptions(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

@final
class PlotIntervalOptions(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...
