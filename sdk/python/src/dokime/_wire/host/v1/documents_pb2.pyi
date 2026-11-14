from dokime._wire.host.v1.plots import snapshots_pb2 as _snapshots_pb2
from dokime._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from dokime._wire.model.v1 import documents_pb2 as _documents_pb2
from dokime._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime._wire.model.v1 import expressions_pb2 as _expressions_pb2
from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime._wire.model.v1 import plots_pb2 as _plots_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
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

class DocumentSummaryStat(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_STAT_UNSPECIFIED: _ClassVar[DocumentSummaryStat]
    DOCUMENT_SUMMARY_STAT_LATEST: _ClassVar[DocumentSummaryStat]

class DocumentSummaryState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DOCUMENT_SUMMARY_STATE_UNSPECIFIED: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_OK: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_WARNING: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_CRITICAL: _ClassVar[DocumentSummaryState]
    DOCUMENT_SUMMARY_STATE_MUTED: _ClassVar[DocumentSummaryState]

class PlotAxisKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLOT_AXIS_KIND_UNSPECIFIED: _ClassVar[PlotAxisKind]
    PLOT_AXIS_KIND_WALL_TIME: _ClassVar[PlotAxisKind]
    PLOT_AXIS_KIND_ELAPSED: _ClassVar[PlotAxisKind]
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
DOCUMENT_SUMMARY_STAT_UNSPECIFIED: DocumentSummaryStat
DOCUMENT_SUMMARY_STAT_LATEST: DocumentSummaryStat
DOCUMENT_SUMMARY_STATE_UNSPECIFIED: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_OK: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_WARNING: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_CRITICAL: DocumentSummaryState
DOCUMENT_SUMMARY_STATE_MUTED: DocumentSummaryState
PLOT_AXIS_KIND_UNSPECIFIED: PlotAxisKind
PLOT_AXIS_KIND_WALL_TIME: PlotAxisKind
PLOT_AXIS_KIND_ELAPSED: PlotAxisKind

class DocumentsGetRequest(_message.Message):
    __slots__ = ("scope", "document_id", "structure_only", "card_ids", "page_token", "max_bytes", "max_frames", "at")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    STRUCTURE_ONLY_FIELD_NUMBER: _ClassVar[int]
    CARD_IDS_FIELD_NUMBER: _ClassVar[int]
    PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_FRAMES_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: DocumentScope
    document_id: str
    structure_only: bool
    card_ids: _containers.RepeatedScalarFieldContainer[str]
    page_token: str
    max_bytes: int
    max_frames: int
    at: _position_pb2.ReadCut
    def __init__(self, scope: _Optional[_Union[DocumentScope, _Mapping]] = ..., document_id: _Optional[str] = ..., structure_only: _Optional[bool] = ..., card_ids: _Optional[_Iterable[str]] = ..., page_token: _Optional[str] = ..., max_bytes: _Optional[int] = ..., max_frames: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class DocumentsGetResponse(_message.Message):
    __slots__ = ("frames", "next_page_token", "resolved_at")
    FRAMES_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    frames: _containers.RepeatedCompositeFieldContainer[DocumentFrame]
    next_page_token: str
    resolved_at: _position_pb2.ReadCut
    def __init__(self, frames: _Optional[_Iterable[_Union[DocumentFrame, _Mapping]]] = ..., next_page_token: _Optional[str] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class DocumentsWatchRequest(_message.Message):
    __slots__ = ("scope", "document_id", "structure_only", "card_ids", "page_token", "max_bytes", "max_frames", "at")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    STRUCTURE_ONLY_FIELD_NUMBER: _ClassVar[int]
    CARD_IDS_FIELD_NUMBER: _ClassVar[int]
    PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_FRAMES_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: DocumentScope
    document_id: str
    structure_only: bool
    card_ids: _containers.RepeatedScalarFieldContainer[str]
    page_token: str
    max_bytes: int
    max_frames: int
    at: _position_pb2.ReadCut
    def __init__(self, scope: _Optional[_Union[DocumentScope, _Mapping]] = ..., document_id: _Optional[str] = ..., structure_only: _Optional[bool] = ..., card_ids: _Optional[_Iterable[str]] = ..., page_token: _Optional[str] = ..., max_bytes: _Optional[int] = ..., max_frames: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class DocumentsWatchResponse(_message.Message):
    __slots__ = ("frames", "resolved_at", "opening", "resync_required")
    FRAMES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    frames: _containers.RepeatedCompositeFieldContainer[DocumentFrame]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, frames: _Optional[_Iterable[_Union[DocumentFrame, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class DocumentsAppendPartsRequest(_message.Message):
    __slots__ = ("execution", "document", "section", "subsection", "card", "request_id")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_FIELD_NUMBER: _ClassVar[int]
    SECTION_FIELD_NUMBER: _ClassVar[int]
    SUBSECTION_FIELD_NUMBER: _ClassVar[int]
    CARD_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    execution: str
    document: str
    section: str
    subsection: _documents_pb2.Section
    card: _documents_pb2.Card
    request_id: str
    def __init__(self, execution: _Optional[str] = ..., document: _Optional[str] = ..., section: _Optional[str] = ..., subsection: _Optional[_Union[_documents_pb2.Section, _Mapping]] = ..., card: _Optional[_Union[_documents_pb2.Card, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

class DocumentsAppendPartsResponse(_message.Message):
    __slots__ = ("receipt",)
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class DocumentBinding(_message.Message):
    __slots__ = ("source_id", "metadata", "result")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    metadata: _metadata_pb2.Metadata
    result: DocumentBindingResult
    def __init__(self, source_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., result: _Optional[_Union[DocumentBindingResult, _Mapping]] = ...) -> None: ...

class DocumentBindingResult(_message.Message):
    __slots__ = ("literal", "summary", "plot", "pending", "failure")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    literal: bool
    summary: DocumentSummary
    plot: DocumentPlotDefinition
    pending: bool
    failure: DocumentCardFailure
    def __init__(self, literal: _Optional[bool] = ..., summary: _Optional[_Union[DocumentSummary, _Mapping]] = ..., plot: _Optional[_Union[DocumentPlotDefinition, _Mapping]] = ..., pending: _Optional[bool] = ..., failure: _Optional[_Union[DocumentCardFailure, _Mapping]] = ...) -> None: ...

class DocumentCard(_message.Message):
    __slots__ = ("metadata", "filters", "body")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    BODY_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    body: DocumentCardBody
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ..., body: _Optional[_Union[DocumentCardBody, _Mapping]] = ...) -> None: ...

class DocumentCardBody(_message.Message):
    __slots__ = ("markdown", "table", "summary", "owner")
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    markdown: str
    table: DocumentTable
    summary: DocumentSummary
    owner: DocumentOwnerReference
    def __init__(self, markdown: _Optional[str] = ..., table: _Optional[_Union[DocumentTable, _Mapping]] = ..., summary: _Optional[_Union[DocumentSummary, _Mapping]] = ..., owner: _Optional[_Union[DocumentOwnerReference, _Mapping]] = ...) -> None: ...

class DocumentCardBudget(_message.Message):
    __slots__ = ("name", "limit", "reached", "card_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    REACHED_FIELD_NUMBER: _ClassVar[int]
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    limit: int
    reached: int
    card_id: str
    def __init__(self, name: _Optional[str] = ..., limit: _Optional[int] = ..., reached: _Optional[int] = ..., card_id: _Optional[str] = ...) -> None: ...

class DocumentCardFailure(_message.Message):
    __slots__ = ("code", "message", "budget")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    BUDGET_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    budget: DocumentCardBudget
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ..., budget: _Optional[_Union[DocumentCardBudget, _Mapping]] = ...) -> None: ...

class DocumentCompletion(_message.Message):
    __slots__ = ("frame_count", "card_count", "layout_node_count", "binding_count", "repeat_count", "section_count", "partial")
    FRAME_COUNT_FIELD_NUMBER: _ClassVar[int]
    CARD_COUNT_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_NODE_COUNT_FIELD_NUMBER: _ClassVar[int]
    BINDING_COUNT_FIELD_NUMBER: _ClassVar[int]
    REPEAT_COUNT_FIELD_NUMBER: _ClassVar[int]
    SECTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    PARTIAL_FIELD_NUMBER: _ClassVar[int]
    frame_count: int
    card_count: int
    layout_node_count: int
    binding_count: int
    repeat_count: int
    section_count: int
    partial: bool
    def __init__(self, frame_count: _Optional[int] = ..., card_count: _Optional[int] = ..., layout_node_count: _Optional[int] = ..., binding_count: _Optional[int] = ..., repeat_count: _Optional[int] = ..., section_count: _Optional[int] = ..., partial: _Optional[bool] = ...) -> None: ...

class DocumentEvidenceCut(_message.Message):
    __slots__ = ("cut", "telemetry", "cacheable")
    CUT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    CACHEABLE_FIELD_NUMBER: _ClassVar[int]
    cut: DocumentTelemetryCut
    telemetry: bool
    cacheable: bool
    def __init__(self, cut: _Optional[_Union[DocumentTelemetryCut, _Mapping]] = ..., telemetry: _Optional[bool] = ..., cacheable: _Optional[bool] = ...) -> None: ...

class DocumentFrame(_message.Message):
    __slots__ = ("frame",)
    FRAME_FIELD_NUMBER: _ClassVar[int]
    frame: DocumentFrameFrame
    def __init__(self, frame: _Optional[_Union[DocumentFrameFrame, _Mapping]] = ...) -> None: ...

class DocumentFrameFrame(_message.Message):
    __slots__ = ("header", "layout_node", "card", "table_row", "summary_group", "summary_tile", "warning", "completion", "group_definition", "tile_definition", "binding", "repeat_selection", "repeat_instance", "section_identity", "structure_complete", "card_complete", "expansion_complete", "evidence_cut", "section_removal", "artifact_cut", "page")
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
    STRUCTURE_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CARD_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    EXPANSION_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_CUT_FIELD_NUMBER: _ClassVar[int]
    SECTION_REMOVAL_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_CUT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
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
    structure_complete: DocumentCompletion
    card_complete: str
    expansion_complete: bool
    evidence_cut: DocumentEvidenceCut
    section_removal: str
    artifact_cut: str
    page: DocumentPage
    def __init__(self, header: _Optional[_Union[DocumentHeader, _Mapping]] = ..., layout_node: _Optional[_Union[DocumentLayoutNode, _Mapping]] = ..., card: _Optional[_Union[DocumentCard, _Mapping]] = ..., table_row: _Optional[_Union[DocumentTableRow, _Mapping]] = ..., summary_group: _Optional[_Union[DocumentSummaryGroup, _Mapping]] = ..., summary_tile: _Optional[_Union[DocumentSummaryTile, _Mapping]] = ..., warning: _Optional[_Union[DocumentWarning, _Mapping]] = ..., completion: _Optional[_Union[DocumentCompletion, _Mapping]] = ..., group_definition: _Optional[_Union[DocumentSummaryGroupDefinition, _Mapping]] = ..., tile_definition: _Optional[_Union[DocumentSummaryTileDefinition, _Mapping]] = ..., binding: _Optional[_Union[DocumentBinding, _Mapping]] = ..., repeat_selection: _Optional[_Union[DocumentRepeatSelection, _Mapping]] = ..., repeat_instance: _Optional[_Union[DocumentRepeatInstance, _Mapping]] = ..., section_identity: _Optional[_Union[DocumentSectionIdentity, _Mapping]] = ..., structure_complete: _Optional[_Union[DocumentCompletion, _Mapping]] = ..., card_complete: _Optional[str] = ..., expansion_complete: _Optional[bool] = ..., evidence_cut: _Optional[_Union[DocumentEvidenceCut, _Mapping]] = ..., section_removal: _Optional[str] = ..., artifact_cut: _Optional[str] = ..., page: _Optional[_Union[DocumentPage, _Mapping]] = ...) -> None: ...

class DocumentFrontiers(_message.Message):
    __slots__ = ("process_epoch", "state_revision", "telemetry", "artifacts_sha256", "evidence")
    PROCESS_EPOCH_FIELD_NUMBER: _ClassVar[int]
    STATE_REVISION_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_SHA256_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_FIELD_NUMBER: _ClassVar[int]
    process_epoch: str
    state_revision: int
    telemetry: DocumentTelemetryCut
    artifacts_sha256: str
    evidence: DocumentTelemetryCut
    def __init__(self, process_epoch: _Optional[str] = ..., state_revision: _Optional[int] = ..., telemetry: _Optional[_Union[DocumentTelemetryCut, _Mapping]] = ..., artifacts_sha256: _Optional[str] = ..., evidence: _Optional[_Union[DocumentTelemetryCut, _Mapping]] = ...) -> None: ...

class DocumentHeader(_message.Message):
    __slots__ = ("document_id", "metadata", "scope", "frontiers")
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FRONTIERS_FIELD_NUMBER: _ClassVar[int]
    document_id: str
    metadata: _metadata_pb2.Metadata
    scope: _evidence_pb2.EvidenceScope
    frontiers: DocumentFrontiers
    def __init__(self, document_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., frontiers: _Optional[_Union[DocumentFrontiers, _Mapping]] = ...) -> None: ...

class DocumentLayoutNode(_message.Message):
    __slots__ = ("ordinal", "parent_ordinal", "node")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    PARENT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    NODE_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    parent_ordinal: int
    node: DocumentLayoutNodeNode
    def __init__(self, ordinal: _Optional[int] = ..., parent_ordinal: _Optional[int] = ..., node: _Optional[_Union[DocumentLayoutNodeNode, _Mapping]] = ...) -> None: ...

class DocumentLayoutNodeNode(_message.Message):
    __slots__ = ("section", "placement")
    SECTION_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    section: DocumentSection
    placement: DocumentPlacement
    def __init__(self, section: _Optional[_Union[DocumentSection, _Mapping]] = ..., placement: _Optional[_Union[DocumentPlacement, _Mapping]] = ...) -> None: ...

class DocumentOwnerReference(_message.Message):
    __slots__ = ("kind", "plot_id", "plot")
    KIND_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    kind: DocumentCardKind
    plot_id: str
    plot: DocumentPlotDefinition
    def __init__(self, kind: _Optional[_Union[DocumentCardKind, str]] = ..., plot_id: _Optional[str] = ..., plot: _Optional[_Union[DocumentPlotDefinition, _Mapping]] = ...) -> None: ...

class DocumentPage(_message.Message):
    __slots__ = ("current", "next")
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    current: str
    next: str
    def __init__(self, current: _Optional[str] = ..., next: _Optional[str] = ...) -> None: ...

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

class DocumentPlotDefinition(_message.Message):
    __slots__ = ("metadata", "request", "presentation", "z_expression")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    request: _snapshots_pb2.PlotRequest
    presentation: PlotPresentation
    z_expression: _expressions_pb2.DataExpression
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., request: _Optional[_Union[_snapshots_pb2.PlotRequest, _Mapping]] = ..., presentation: _Optional[_Union[PlotPresentation, _Mapping]] = ..., z_expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ...) -> None: ...

class DocumentRepeat(_message.Message):
    __slots__ = ("value", "label_template", "id_template", "order_by", "filters")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ID_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    value: _expressions_pb2.DataExpression
    label_template: str
    id_template: str
    order_by: _expressions_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    def __init__(self, value: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., label_template: _Optional[str] = ..., id_template: _Optional[str] = ..., order_by: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ...) -> None: ...

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

class DocumentRepeatSelection(_message.Message):
    __slots__ = ("node_id", "instance_count", "pending")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    INSTANCE_COUNT_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    instance_count: int
    pending: bool
    def __init__(self, node_id: _Optional[str] = ..., instance_count: _Optional[int] = ..., pending: _Optional[bool] = ...) -> None: ...

class DocumentSection(_message.Message):
    __slots__ = ("node_id", "metadata", "repeat")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: _metadata_pb2.Metadata
    repeat: DocumentRepeat
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., repeat: _Optional[_Union[DocumentRepeat, _Mapping]] = ...) -> None: ...

class DocumentSectionIdentity(_message.Message):
    __slots__ = ("node_id", "metadata", "unavailable")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: _metadata_pb2.Metadata
    unavailable: str
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

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

class DocumentSummary(_message.Message):
    __slots__ = ("group_count",)
    GROUP_COUNT_FIELD_NUMBER: _ClassVar[int]
    group_count: int
    def __init__(self, group_count: _Optional[int] = ...) -> None: ...

class DocumentSummaryCondition(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: _expressions_pb2.FilterOperator
    value: _scalar_pb2.Value
    state: DocumentSummaryState
    def __init__(self, operator: _Optional[_Union[_expressions_pb2.FilterOperator, str]] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., state: _Optional[_Union[DocumentSummaryState, str]] = ...) -> None: ...

class DocumentSummaryGroup(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: _metadata_pb2.Metadata
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class DocumentSummaryGroupDefinition(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: _metadata_pb2.Metadata
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

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
    metadata: _metadata_pb2.Metadata
    value: _scalar_pb2.Value
    stat: DocumentSummaryStat
    format: DocumentSummaryFormat
    presentation: DocumentSummaryPresentation
    unit: str
    state: DocumentSummaryState
    unavailable: str
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., stat: _Optional[_Union[DocumentSummaryStat, str]] = ..., format: _Optional[_Union[DocumentSummaryFormat, str]] = ..., presentation: _Optional[_Union[DocumentSummaryPresentation, str]] = ..., unit: _Optional[str] = ..., state: _Optional[_Union[DocumentSummaryState, str]] = ..., unavailable: _Optional[str] = ...) -> None: ...

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
    metadata: _metadata_pb2.Metadata
    value: _expressions_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[_plots_pb2.PlotFilter]
    stat: DocumentSummaryStat
    format: DocumentSummaryFormat
    presentation: DocumentSummaryPresentation
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[DocumentSummaryCondition]
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_plots_pb2.PlotFilter, _Mapping]]] = ..., stat: _Optional[_Union[DocumentSummaryStat, str]] = ..., format: _Optional[_Union[DocumentSummaryFormat, str]] = ..., presentation: _Optional[_Union[DocumentSummaryPresentation, str]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[DocumentSummaryCondition, _Mapping]]] = ...) -> None: ...

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
    values: _documents_pb2.TableRow
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., values: _Optional[_Union[_documents_pb2.TableRow, _Mapping]] = ...) -> None: ...

class DocumentTelemetryCut(_message.Message):
    __slots__ = ("store", "epoch", "position")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    position: int
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class DocumentWarning(_message.Message):
    __slots__ = ("code", "message")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class PlotPresentation(_message.Message):
    __slots__ = ("style", "fft", "interval", "histogram", "reference_lines", "reference_bands", "x_format", "y_format", "x_axis", "x_axis_unit", "scale")
    STYLE_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_LINES_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_BANDS_FIELD_NUMBER: _ClassVar[int]
    X_FORMAT_FIELD_NUMBER: _ClassVar[int]
    Y_FORMAT_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    SCALE_FIELD_NUMBER: _ClassVar[int]
    style: _plots_pb2.LineStyle
    fft: _plots_pb2.PlotFftOptions
    interval: _plots_pb2.PlotIntervalOptions
    histogram: _plots_pb2.PlotHistogramOptions
    reference_lines: _containers.RepeatedCompositeFieldContainer[PlotReferenceLine]
    reference_bands: _containers.RepeatedCompositeFieldContainer[PlotReferenceBand]
    x_format: _record_schemas_pb2.ValueFormat
    y_format: _record_schemas_pb2.ValueFormat
    x_axis: PlotAxisKind
    x_axis_unit: str
    scale: float
    def __init__(self, style: _Optional[_Union[_plots_pb2.LineStyle, str]] = ..., fft: _Optional[_Union[_plots_pb2.PlotFftOptions, _Mapping]] = ..., interval: _Optional[_Union[_plots_pb2.PlotIntervalOptions, _Mapping]] = ..., histogram: _Optional[_Union[_plots_pb2.PlotHistogramOptions, _Mapping]] = ..., reference_lines: _Optional[_Iterable[_Union[PlotReferenceLine, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[PlotReferenceBand, _Mapping]]] = ..., x_format: _Optional[_Union[_record_schemas_pb2.ValueFormat, _Mapping]] = ..., y_format: _Optional[_Union[_record_schemas_pb2.ValueFormat, _Mapping]] = ..., x_axis: _Optional[_Union[PlotAxisKind, str]] = ..., x_axis_unit: _Optional[str] = ..., scale: _Optional[float] = ...) -> None: ...

class PlotReferenceBand(_message.Message):
    __slots__ = ("min", "max", "label", "color", "center", "tolerance")
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    min: PlotReferenceValue
    max: PlotReferenceValue
    label: str
    color: str
    center: PlotReferenceValue
    tolerance: PlotReferenceValue
    def __init__(self, min: _Optional[_Union[PlotReferenceValue, _Mapping]] = ..., max: _Optional[_Union[PlotReferenceValue, _Mapping]] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[PlotReferenceValue, _Mapping]] = ..., tolerance: _Optional[_Union[PlotReferenceValue, _Mapping]] = ...) -> None: ...

class PlotReferenceLine(_message.Message):
    __slots__ = ("value", "label", "color")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    value: PlotReferenceValue
    label: str
    color: str
    def __init__(self, value: _Optional[_Union[PlotReferenceValue, _Mapping]] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class PlotReferenceValue(_message.Message):
    __slots__ = ("constant", "expression")
    CONSTANT_FIELD_NUMBER: _ClassVar[int]
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    constant: float
    expression: _expressions_pb2.DataExpression
    def __init__(self, constant: _Optional[float] = ..., expression: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ...) -> None: ...

class DocumentScope(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...
