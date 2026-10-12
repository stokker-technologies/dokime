from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import plots_pb2 as _plots_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class DocumentCardKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_CARD_KIND_UNSPECIFIED: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_TEXT: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_TELEMETRY_SUMMARY: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_PLOT: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_TABLE: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_STEPS: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_CRITERIA: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_TRACEABILITY: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_NOTES: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_RECORDS: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_ARTIFACTS: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_MEDIA: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_HEALTH: _ClassVar[DocumentCardKind]
    DOCUMENT_CARD_KIND_TELEMETRY_CONTEXT: _ClassVar[DocumentCardKind]

class DocumentDensity(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_DENSITY_UNSPECIFIED: _ClassVar[DocumentDensity]
    DOCUMENT_DENSITY_COMPACT: _ClassVar[DocumentDensity]
    DOCUMENT_DENSITY_NORMAL: _ClassVar[DocumentDensity]
    DOCUMENT_DENSITY_SPACIOUS: _ClassVar[DocumentDensity]

class DocumentRail(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_RAIL_UNSPECIFIED: _ClassVar[DocumentRail]
    DOCUMENT_RAIL_MAIN: _ClassVar[DocumentRail]
    DOCUMENT_RAIL_RIGHT: _ClassVar[DocumentRail]
    DOCUMENT_RAIL_LEFT: _ClassVar[DocumentRail]

class PlotAxisKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_AXIS_KIND_UNSPECIFIED: _ClassVar[PlotAxisKind]
    PLOT_AXIS_KIND_WALL_TIME: _ClassVar[PlotAxisKind]
    PLOT_AXIS_KIND_ELAPSED: _ClassVar[PlotAxisKind]

class PlotStyle(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_STYLE_UNSPECIFIED: _ClassVar[PlotStyle]
    PLOT_STYLE_LINE: _ClassVar[PlotStyle]
    PLOT_STYLE_POINTS: _ClassVar[PlotStyle]
    PLOT_STYLE_LINE_POINTS: _ClassVar[PlotStyle]
    PLOT_STYLE_STEP: _ClassVar[PlotStyle]

class ReferenceAxis(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    REFERENCE_AXIS_UNSPECIFIED: _ClassVar[ReferenceAxis]
    REFERENCE_AXIS_X: _ClassVar[ReferenceAxis]
    REFERENCE_AXIS_Y: _ClassVar[ReferenceAxis]

class DocumentSummaryStat(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_STAT_UNSPECIFIED: _ClassVar[DocumentSummaryStat]
    DOCUMENT_SUMMARY_STAT_LATEST: _ClassVar[DocumentSummaryStat]

class DocumentSummaryFormat(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_FORMAT_UNSPECIFIED: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_AUTO: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_NUMBER: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_PERCENT: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_DURATION: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_DISTANCE: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_STATUS: _ClassVar[DocumentSummaryFormat]
    DOCUMENT_SUMMARY_FORMAT_TEXT: _ClassVar[DocumentSummaryFormat]

class DocumentSummaryPresentation(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_PRESENTATION_UNSPECIFIED: _ClassVar[DocumentSummaryPresentation]
    DOCUMENT_SUMMARY_PRESENTATION_VALUE: _ClassVar[DocumentSummaryPresentation]
    DOCUMENT_SUMMARY_PRESENTATION_PROGRESS: _ClassVar[DocumentSummaryPresentation]
    DOCUMENT_SUMMARY_PRESENTATION_BADGE: _ClassVar[DocumentSummaryPresentation]

class DocumentSummaryState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_STATE_UNSPECIFIED: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_OK: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_WARNING: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_CRITICAL: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_MUTED: _ClassVar[DocumentSummaryState]
DOCUMENT_CARD_KIND_UNSPECIFIED: DocumentCardKind
DOCUMENT_CARD_KIND_TEXT: DocumentCardKind
DOCUMENT_CARD_KIND_TELEMETRY_SUMMARY: DocumentCardKind
DOCUMENT_CARD_KIND_PLOT: DocumentCardKind
DOCUMENT_CARD_KIND_TABLE: DocumentCardKind
DOCUMENT_CARD_KIND_STEPS: DocumentCardKind
DOCUMENT_CARD_KIND_CRITERIA: DocumentCardKind
DOCUMENT_CARD_KIND_TRACEABILITY: DocumentCardKind
DOCUMENT_CARD_KIND_NOTES: DocumentCardKind
DOCUMENT_CARD_KIND_RECORDS: DocumentCardKind
DOCUMENT_CARD_KIND_ARTIFACTS: DocumentCardKind
DOCUMENT_CARD_KIND_MEDIA: DocumentCardKind
DOCUMENT_CARD_KIND_HEALTH: DocumentCardKind
DOCUMENT_CARD_KIND_TELEMETRY_CONTEXT: DocumentCardKind
DOCUMENT_DENSITY_UNSPECIFIED: DocumentDensity
DOCUMENT_DENSITY_COMPACT: DocumentDensity
DOCUMENT_DENSITY_NORMAL: DocumentDensity
DOCUMENT_DENSITY_SPACIOUS: DocumentDensity
DOCUMENT_RAIL_UNSPECIFIED: DocumentRail
DOCUMENT_RAIL_MAIN: DocumentRail
DOCUMENT_RAIL_RIGHT: DocumentRail
DOCUMENT_RAIL_LEFT: DocumentRail
PLOT_AXIS_KIND_UNSPECIFIED: PlotAxisKind
PLOT_AXIS_KIND_WALL_TIME: PlotAxisKind
PLOT_AXIS_KIND_ELAPSED: PlotAxisKind
PLOT_STYLE_UNSPECIFIED: PlotStyle
PLOT_STYLE_LINE: PlotStyle
PLOT_STYLE_POINTS: PlotStyle
PLOT_STYLE_LINE_POINTS: PlotStyle
PLOT_STYLE_STEP: PlotStyle
REFERENCE_AXIS_UNSPECIFIED: ReferenceAxis
REFERENCE_AXIS_X: ReferenceAxis
REFERENCE_AXIS_Y: ReferenceAxis
DOCUMENT_SUMMARY_STAT_UNSPECIFIED: DocumentSummaryStat
DOCUMENT_SUMMARY_STAT_LATEST: DocumentSummaryStat
DOCUMENT_SUMMARY_FORMAT_UNSPECIFIED: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_AUTO: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_NUMBER: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_PERCENT: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_DURATION: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_DISTANCE: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_STATUS: DocumentSummaryFormat
DOCUMENT_SUMMARY_FORMAT_TEXT: DocumentSummaryFormat
DOCUMENT_SUMMARY_PRESENTATION_UNSPECIFIED: DocumentSummaryPresentation
DOCUMENT_SUMMARY_PRESENTATION_VALUE: DocumentSummaryPresentation
DOCUMENT_SUMMARY_PRESENTATION_PROGRESS: DocumentSummaryPresentation
DOCUMENT_SUMMARY_PRESENTATION_BADGE: DocumentSummaryPresentation
DOCUMENT_SUMMARY_STATE_UNSPECIFIED: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_OK: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_WARNING: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_CRITICAL: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_MUTED: DocumentSummaryState

class DocumentScope(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class CaptureCurrentDocumentRequest(_message.Message):
    __slots__ = ("scope", "document_id")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    scope: DocumentScope
    document_id: str
    def __init__(self, scope: _Optional[_Union[DocumentScope, _Mapping]] = ..., document_id: _Optional[str] = ...) -> None: ...

class DocumentFrontiers(_message.Message):
    __slots__ = ("process_epoch", "state_revision", "telemetry", "artifacts_sha256")
    PROCESS_EPOCH_FIELD_NUMBER: _ClassVar[int]
    STATE_REVISION_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_SHA256_FIELD_NUMBER: _ClassVar[int]
    process_epoch: str
    state_revision: int
    telemetry: DocumentTelemetryCut
    artifacts_sha256: str
    def __init__(self, process_epoch: _Optional[str] = ..., state_revision: _Optional[int] = ..., telemetry: _Optional[_Union[DocumentTelemetryCut, _Mapping]] = ..., artifacts_sha256: _Optional[str] = ...) -> None: ...

class DocumentTelemetryCut(_message.Message):
    __slots__ = ("delivered_cursor", "series_epoch", "retained_window", "series_ordinal")
    DELIVERED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    SERIES_EPOCH_FIELD_NUMBER: _ClassVar[int]
    RETAINED_WINDOW_FIELD_NUMBER: _ClassVar[int]
    SERIES_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    delivered_cursor: int
    series_epoch: int
    retained_window: bool
    series_ordinal: int
    def __init__(self, delivered_cursor: _Optional[int] = ..., series_epoch: _Optional[int] = ..., retained_window: _Optional[bool] = ..., series_ordinal: _Optional[int] = ...) -> None: ...

class DocumentHeader(_message.Message):
    __slots__ = ("document_id", "metadata", "scope", "frontiers")
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FRONTIERS_FIELD_NUMBER: _ClassVar[int]
    document_id: str
    metadata: _common_pb2.Metadata
    scope: _common_pb2.EvidenceScope
    frontiers: DocumentFrontiers
    def __init__(self, document_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., frontiers: _Optional[_Union[DocumentFrontiers, _Mapping]] = ...) -> None: ...

class DocumentLayoutNode(_message.Message):
    __slots__ = ("ordinal", "parent_ordinal", "section", "placement")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    PARENT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SECTION_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    parent_ordinal: int
    section: DocumentSection
    placement: DocumentPlacement
    def __init__(self, ordinal: _Optional[int] = ..., parent_ordinal: _Optional[int] = ..., section: _Optional[_Union[DocumentSection, _Mapping]] = ..., placement: _Optional[_Union[DocumentPlacement, _Mapping]] = ...) -> None: ...

class DocumentSection(_message.Message):
    __slots__ = ("node_id", "metadata", "repeat")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: _common_pb2.Metadata
    repeat: DocumentRepeat
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., repeat: _Optional[_Union[DocumentRepeat, _Mapping]] = ...) -> None: ...

class DocumentRepeat(_message.Message):
    __slots__ = ("value", "label_template", "id_template", "order_by", "filters")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ID_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    value: _plots_pb2.DataExpression
    label_template: str
    id_template: str
    order_by: _plots_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    def __init__(self, value: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., label_template: _Optional[str] = ..., id_template: _Optional[str] = ..., order_by: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ...) -> None: ...

class DocumentPlacement(_message.Message):
    __slots__ = ("card_id", "column_span", "row_span", "order", "density", "rail")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMN_SPAN_FIELD_NUMBER: _ClassVar[int]
    ROW_SPAN_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    RAIL_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    column_span: DocumentSpan
    row_span: int
    order: int
    density: DocumentDensity
    rail: DocumentRail
    def __init__(self, card_id: _Optional[str] = ..., column_span: _Optional[_Union[DocumentSpan, _Mapping]] = ..., row_span: _Optional[int] = ..., order: _Optional[int] = ..., density: _Optional[_Union[DocumentDensity, str]] = ..., rail: _Optional[_Union[DocumentRail, str]] = ...) -> None: ...

class DocumentSpan(_message.Message):
    __slots__ = ("base", "sm", "md", "lg", "xl")
    BASE_FIELD_NUMBER: _ClassVar[int]
    SM_FIELD_NUMBER: _ClassVar[int]
    MD_FIELD_NUMBER: _ClassVar[int]
    LG_FIELD_NUMBER: _ClassVar[int]
    XL_FIELD_NUMBER: _ClassVar[int]
    base: int
    sm: int
    md: int
    lg: int
    xl: int
    def __init__(self, base: _Optional[int] = ..., sm: _Optional[int] = ..., md: _Optional[int] = ..., lg: _Optional[int] = ..., xl: _Optional[int] = ...) -> None: ...

class DocumentCard(_message.Message):
    __slots__ = ("metadata", "filters", "markdown", "table", "summary", "owner")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    markdown: str
    table: DocumentTable
    summary: DocumentSummary
    owner: DocumentOwnerReference
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ..., markdown: _Optional[str] = ..., table: _Optional[_Union[DocumentTable, _Mapping]] = ..., summary: _Optional[_Union[DocumentSummary, _Mapping]] = ..., owner: _Optional[_Union[DocumentOwnerReference, _Mapping]] = ...) -> None: ...

class DocumentOwnerReference(_message.Message):
    __slots__ = ("kind", "plot_id", "plot")
    KIND_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    kind: DocumentCardKind
    plot_id: str
    plot: DocumentPlotDefinition
    def __init__(self, kind: _Optional[_Union[DocumentCardKind, str]] = ..., plot_id: _Optional[str] = ..., plot: _Optional[_Union[DocumentPlotDefinition, _Mapping]] = ...) -> None: ...

class DocumentPlotDefinition(_message.Message):
    __slots__ = ("metadata", "request", "presentation", "z_expression")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    request: _plots_pb2.PlotRequest
    presentation: PlotPresentation
    z_expression: _plots_pb2.DataExpression
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., request: _Optional[_Union[_plots_pb2.PlotRequest, _Mapping]] = ..., presentation: _Optional[_Union[PlotPresentation, _Mapping]] = ..., z_expression: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ...) -> None: ...

class PlotPresentation(_message.Message):
    __slots__ = ("group", "style", "fft", "interval", "histogram", "reference_lines", "reference_bands", "live_window_s", "x_format", "y_format", "x_axis", "x_axis_unit")
    GROUP_FIELD_NUMBER: _ClassVar[int]
    STYLE_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_LINES_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_BANDS_FIELD_NUMBER: _ClassVar[int]
    LIVE_WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    X_FORMAT_FIELD_NUMBER: _ClassVar[int]
    Y_FORMAT_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    group: str
    style: PlotStyle
    fft: _plots_pb2.PlotFftOptions
    interval: _plots_pb2.PlotIntervalOptions
    histogram: _plots_pb2.PlotHistogramOptions
    reference_lines: _containers.RepeatedCompositeFieldContainer[PlotReferenceLine]
    reference_bands: _containers.RepeatedCompositeFieldContainer[PlotReferenceBand]
    live_window_s: float
    x_format: _schema_pb2.ValueFormat
    y_format: _schema_pb2.ValueFormat
    x_axis: PlotAxisKind
    x_axis_unit: str
    def __init__(self, group: _Optional[str] = ..., style: _Optional[_Union[PlotStyle, str]] = ..., fft: _Optional[_Union[_plots_pb2.PlotFftOptions, _Mapping]] = ..., interval: _Optional[_Union[_plots_pb2.PlotIntervalOptions, _Mapping]] = ..., histogram: _Optional[_Union[_plots_pb2.PlotHistogramOptions, _Mapping]] = ..., reference_lines: _Optional[_Iterable[_Union[PlotReferenceLine, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[PlotReferenceBand, _Mapping]]] = ..., live_window_s: _Optional[float] = ..., x_format: _Optional[_Union[_schema_pb2.ValueFormat, _Mapping]] = ..., y_format: _Optional[_Union[_schema_pb2.ValueFormat, _Mapping]] = ..., x_axis: _Optional[_Union[PlotAxisKind, str]] = ..., x_axis_unit: _Optional[str] = ...) -> None: ...

class PlotReferenceLine(_message.Message):
    __slots__ = ("axis", "value", "source", "source_scale", "label", "color")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_SCALE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    axis: ReferenceAxis
    value: float
    source: _plots_pb2.DataExpression
    source_scale: float
    label: str
    color: str
    def __init__(self, axis: _Optional[_Union[ReferenceAxis, str]] = ..., value: _Optional[float] = ..., source: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., source_scale: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class PlotReferenceBand(_message.Message):
    __slots__ = ("axis", "min", "max", "label", "color", "center", "tolerance")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    axis: ReferenceAxis
    min: float
    max: float
    label: str
    color: str
    center: _plots_pb2.DataExpression
    tolerance: _plots_pb2.DataExpression
    def __init__(self, axis: _Optional[_Union[ReferenceAxis, str]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., tolerance: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ...) -> None: ...

class DocumentTable(_message.Message):
    __slots__ = ("row_count",)
    ROW_COUNT_FIELD_NUMBER: _ClassVar[int]
    row_count: int
    def __init__(self, row_count: _Optional[int] = ...) -> None: ...

class DocumentTableRow(_message.Message):
    __slots__ = ("card_id", "ordinal", "values")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    values: _value_pb2.ValueObject
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class DocumentSummary(_message.Message):
    __slots__ = ("group_count",)
    GROUP_COUNT_FIELD_NUMBER: _ClassVar[int]
    group_count: int
    def __init__(self, group_count: _Optional[int] = ...) -> None: ...

class DocumentSummaryGroup(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: _common_pb2.Metadata
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class DocumentSummaryTile(_message.Message):
    __slots__ = ("card_id", "group_ordinal", "ordinal", "metadata", "value", "stat", "format", "presentation", "unit", "state", "unavailable")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    GROUP_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STAT_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    group_ordinal: int
    ordinal: int
    metadata: _common_pb2.Metadata
    value: _value_pb2.Value
    stat: DocumentSummaryStat
    format: DocumentSummaryFormat
    presentation: DocumentSummaryPresentation
    unit: str
    state: DocumentSummaryState
    unavailable: str
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., stat: _Optional[_Union[DocumentSummaryStat, str]] = ..., format: _Optional[_Union[DocumentSummaryFormat, str]] = ..., presentation: _Optional[_Union[DocumentSummaryPresentation, str]] = ..., unit: _Optional[str] = ..., state: _Optional[_Union[DocumentSummaryState, str]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class DocumentWarning(_message.Message):
    __slots__ = ("code", "message")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class DocumentSummaryGroupDefinition(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: _common_pb2.Metadata
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class DocumentSummaryTileDefinition(_message.Message):
    __slots__ = ("card_id", "group_ordinal", "ordinal", "metadata", "value", "filters", "stat", "format", "presentation", "unit", "conditions")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    GROUP_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    STAT_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    group_ordinal: int
    ordinal: int
    metadata: _common_pb2.Metadata
    value: _plots_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    stat: DocumentSummaryStat
    format: DocumentSummaryFormat
    presentation: DocumentSummaryPresentation
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[DocumentSummaryCondition]
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ..., stat: _Optional[_Union[DocumentSummaryStat, str]] = ..., format: _Optional[_Union[DocumentSummaryFormat, str]] = ..., presentation: _Optional[_Union[DocumentSummaryPresentation, str]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[DocumentSummaryCondition, _Mapping]]] = ...) -> None: ...

class DocumentSummaryCondition(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: _plots_pb2.FilterOperator
    value: _value_pb2.Value
    state: DocumentSummaryState
    def __init__(self, operator: _Optional[_Union[_plots_pb2.FilterOperator, str]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., state: _Optional[_Union[DocumentSummaryState, str]] = ...) -> None: ...

class DocumentBinding(_message.Message):
    __slots__ = ("source_id", "metadata", "literal", "summary", "plot")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    metadata: _common_pb2.Metadata
    literal: bool
    summary: DocumentSummary
    plot: DocumentPlotDefinition
    def __init__(self, source_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., literal: _Optional[bool] = ..., summary: _Optional[_Union[DocumentSummary, _Mapping]] = ..., plot: _Optional[_Union[DocumentPlotDefinition, _Mapping]] = ...) -> None: ...

class DocumentRepeatSelection(_message.Message):
    __slots__ = ("node_id", "instance_count")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    INSTANCE_COUNT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    instance_count: int
    def __init__(self, node_id: _Optional[str] = ..., instance_count: _Optional[int] = ...) -> None: ...

class DocumentRepeatInstance(_message.Message):
    __slots__ = ("selection_id", "ordinal", "suffix", "node_id", "filter")
    SELECTION_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SUFFIX_FIELD_NUMBER: _ClassVar[int]
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    selection_id: str
    ordinal: int
    suffix: str
    node_id: str
    filter: _plots_pb2.PlotFilter
    def __init__(self, selection_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., suffix: _Optional[str] = ..., node_id: _Optional[str] = ..., filter: _Optional[_Union[_plots_pb2.PlotFilter, _Mapping]] = ...) -> None: ...

class DocumentSectionIdentity(_message.Message):
    __slots__ = ("node_id", "metadata", "unavailable")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: _common_pb2.Metadata
    unavailable: str
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class DocumentCompletion(_message.Message):
    __slots__ = ("frame_count", "card_count", "layout_node_count", "binding_count", "repeat_count", "section_count")
    FRAME_COUNT_FIELD_NUMBER: _ClassVar[int]
    CARD_COUNT_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_NODE_COUNT_FIELD_NUMBER: _ClassVar[int]
    BINDING_COUNT_FIELD_NUMBER: _ClassVar[int]
    REPEAT_COUNT_FIELD_NUMBER: _ClassVar[int]
    SECTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    frame_count: int
    card_count: int
    layout_node_count: int
    binding_count: int
    repeat_count: int
    section_count: int
    def __init__(self, frame_count: _Optional[int] = ..., card_count: _Optional[int] = ..., layout_node_count: _Optional[int] = ..., binding_count: _Optional[int] = ..., repeat_count: _Optional[int] = ..., section_count: _Optional[int] = ...) -> None: ...

class DocumentFrame(_message.Message):
    __slots__ = ("header", "layout_node", "card", "table_row", "summary_group", "summary_tile", "warning", "completion", "group_definition", "tile_definition", "binding", "repeat_selection", "repeat_instance", "section_identity")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_NODE_FIELD_NUMBER: _ClassVar[int]
    CARD_FIELD_NUMBER: _ClassVar[int]
    TABLE_ROW_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_GROUP_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_TILE_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_FIELD_NUMBER: _ClassVar[int]
    GROUP_DEFINITION_FIELD_NUMBER: _ClassVar[int]
    TILE_DEFINITION_FIELD_NUMBER: _ClassVar[int]
    BINDING_FIELD_NUMBER: _ClassVar[int]
    REPEAT_SELECTION_FIELD_NUMBER: _ClassVar[int]
    REPEAT_INSTANCE_FIELD_NUMBER: _ClassVar[int]
    SECTION_IDENTITY_FIELD_NUMBER: _ClassVar[int]
    header: DocumentHeader
    layout_node: DocumentLayoutNode
    card: DocumentCard
    table_row: DocumentTableRow
    summary_group: DocumentSummaryGroup
    summary_tile: DocumentSummaryTile
    warning: DocumentWarning
    completion: DocumentCompletion
    group_definition: DocumentSummaryGroupDefinition
    tile_definition: DocumentSummaryTileDefinition
    binding: DocumentBinding
    repeat_selection: DocumentRepeatSelection
    repeat_instance: DocumentRepeatInstance
    section_identity: DocumentSectionIdentity
    def __init__(self, header: _Optional[_Union[DocumentHeader, _Mapping]] = ..., layout_node: _Optional[_Union[DocumentLayoutNode, _Mapping]] = ..., card: _Optional[_Union[DocumentCard, _Mapping]] = ..., table_row: _Optional[_Union[DocumentTableRow, _Mapping]] = ..., summary_group: _Optional[_Union[DocumentSummaryGroup, _Mapping]] = ..., summary_tile: _Optional[_Union[DocumentSummaryTile, _Mapping]] = ..., warning: _Optional[_Union[DocumentWarning, _Mapping]] = ..., completion: _Optional[_Union[DocumentCompletion, _Mapping]] = ..., group_definition: _Optional[_Union[DocumentSummaryGroupDefinition, _Mapping]] = ..., tile_definition: _Optional[_Union[DocumentSummaryTileDefinition, _Mapping]] = ..., binding: _Optional[_Union[DocumentBinding, _Mapping]] = ..., repeat_selection: _Optional[_Union[DocumentRepeatSelection, _Mapping]] = ..., repeat_instance: _Optional[_Union[DocumentRepeatInstance, _Mapping]] = ..., section_identity: _Optional[_Union[DocumentSectionIdentity, _Mapping]] = ...) -> None: ...
