from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("scope", "document_id", "structure_only", "card_ids", "page")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    STRUCTURE_ONLY_FIELD_NUMBER: _ClassVar[int]
    CARD_IDS_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: Row4
    document_id: str
    structure_only: bool
    card_ids: _containers.RepeatedScalarFieldContainer[str]
    page: str
    def __init__(self, scope: _Optional[_Union[Row4, _Mapping]] = ..., document_id: _Optional[str] = ..., structure_only: _Optional[bool] = ..., card_ids: _Optional[_Iterable[str]] = ..., page: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("frame",)
    FRAME_FIELD_NUMBER: _ClassVar[int]
    frame: Row5
    def __init__(self, frame: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Header", "LayoutNode", "Card", "TableRow", "SummaryGroup", "SummaryTile", "Warning", "Completion", "GroupDefinition", "TileDefinition", "Binding", "RepeatSelection", "RepeatInstance", "SectionIdentity", "StructureComplete", "CardComplete", "ExpansionComplete", "EvidenceCut", "SectionRemoval", "ArtifactCut", "Page")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    LAYOUTNODE_FIELD_NUMBER: _ClassVar[int]
    CARD_FIELD_NUMBER: _ClassVar[int]
    TABLEROW_FIELD_NUMBER: _ClassVar[int]
    SUMMARYGROUP_FIELD_NUMBER: _ClassVar[int]
    SUMMARYTILE_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_FIELD_NUMBER: _ClassVar[int]
    GROUPDEFINITION_FIELD_NUMBER: _ClassVar[int]
    TILEDEFINITION_FIELD_NUMBER: _ClassVar[int]
    BINDING_FIELD_NUMBER: _ClassVar[int]
    REPEATSELECTION_FIELD_NUMBER: _ClassVar[int]
    REPEATINSTANCE_FIELD_NUMBER: _ClassVar[int]
    SECTIONIDENTITY_FIELD_NUMBER: _ClassVar[int]
    STRUCTURECOMPLETE_FIELD_NUMBER: _ClassVar[int]
    CARDCOMPLETE_FIELD_NUMBER: _ClassVar[int]
    EXPANSIONCOMPLETE_FIELD_NUMBER: _ClassVar[int]
    EVIDENCECUT_FIELD_NUMBER: _ClassVar[int]
    SECTIONREMOVAL_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTCUT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    Header: Row6
    LayoutNode: Row7
    Card: Row8
    TableRow: Row9
    SummaryGroup: Row10
    SummaryTile: Row11
    Warning: Row12
    Completion: Row13
    GroupDefinition: Row14
    TileDefinition: Row15
    Binding: Row16
    RepeatSelection: Row17
    RepeatInstance: Row18
    SectionIdentity: Row19
    StructureComplete: Row20
    CardComplete: Row21
    ExpansionComplete: Row22
    EvidenceCut: Row23
    SectionRemoval: Row24
    ArtifactCut: Row25
    Page: Row26
    def __init__(self, Header: _Optional[_Union[Row6, _Mapping]] = ..., LayoutNode: _Optional[_Union[Row7, _Mapping]] = ..., Card: _Optional[_Union[Row8, _Mapping]] = ..., TableRow: _Optional[_Union[Row9, _Mapping]] = ..., SummaryGroup: _Optional[_Union[Row10, _Mapping]] = ..., SummaryTile: _Optional[_Union[Row11, _Mapping]] = ..., Warning: _Optional[_Union[Row12, _Mapping]] = ..., Completion: _Optional[_Union[Row13, _Mapping]] = ..., GroupDefinition: _Optional[_Union[Row14, _Mapping]] = ..., TileDefinition: _Optional[_Union[Row15, _Mapping]] = ..., Binding: _Optional[_Union[Row16, _Mapping]] = ..., RepeatSelection: _Optional[_Union[Row17, _Mapping]] = ..., RepeatInstance: _Optional[_Union[Row18, _Mapping]] = ..., SectionIdentity: _Optional[_Union[Row19, _Mapping]] = ..., StructureComplete: _Optional[_Union[Row20, _Mapping]] = ..., CardComplete: _Optional[_Union[Row21, _Mapping]] = ..., ExpansionComplete: _Optional[_Union[Row22, _Mapping]] = ..., EvidenceCut: _Optional[_Union[Row23, _Mapping]] = ..., SectionRemoval: _Optional[_Union[Row24, _Mapping]] = ..., ArtifactCut: _Optional[_Union[Row25, _Mapping]] = ..., Page: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row27
    def __init__(self, field_0: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row28
    def __init__(self, field_0: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row31
    def __init__(self, field_0: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row35
    def __init__(self, field_0: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row36
    def __init__(self, field_0: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row37
    def __init__(self, field_0: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row38
    def __init__(self, field_0: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row39
    def __init__(self, field_0: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row40
    def __init__(self, field_0: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row41
    def __init__(self, field_0: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row42
    def __init__(self, field_0: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("document_id", "metadata", "scope", "frontiers")
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FRONTIERS_FIELD_NUMBER: _ClassVar[int]
    document_id: str
    metadata: Row43
    scope: Row44
    frontiers: Row45
    def __init__(self, document_id: _Optional[str] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., scope: _Optional[_Union[Row44, _Mapping]] = ..., frontiers: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("ordinal", "parent_ordinal", "node")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    PARENT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    NODE_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    parent_ordinal: int
    node: Row46
    def __init__(self, ordinal: _Optional[int] = ..., parent_ordinal: _Optional[int] = ..., node: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("metadata", "filters", "body")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    BODY_FIELD_NUMBER: _ClassVar[int]
    metadata: Row43
    filters: _containers.RepeatedCompositeFieldContainer[Row47]
    body: Row48
    def __init__(self, metadata: _Optional[_Union[Row43, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row47, _Mapping]]] = ..., body: _Optional[_Union[Row48, _Mapping]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("card_id", "ordinal", "values")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    values: Row49
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., values: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: Row43
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class Row32(_message.Message):
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
    metadata: Row43
    value: Row49
    stat: Row50
    format: Row51
    presentation: Row52
    unit: str
    state: Row53
    unavailable: str
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., value: _Optional[_Union[Row49, _Mapping]] = ..., stat: _Optional[_Union[Row50, _Mapping]] = ..., format: _Optional[_Union[Row51, _Mapping]] = ..., presentation: _Optional[_Union[Row52, _Mapping]] = ..., unit: _Optional[str] = ..., state: _Optional[_Union[Row53, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("code", "message")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class Row34(_message.Message):
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

class Row35(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: Row43
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class Row36(_message.Message):
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
    metadata: Row43
    value: Row54
    filters: _containers.RepeatedCompositeFieldContainer[Row47]
    stat: Row50
    format: Row51
    presentation: Row52
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[Row55]
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., value: _Optional[_Union[Row54, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row47, _Mapping]]] = ..., stat: _Optional[_Union[Row50, _Mapping]] = ..., format: _Optional[_Union[Row51, _Mapping]] = ..., presentation: _Optional[_Union[Row52, _Mapping]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[Row55, _Mapping]]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("source_id", "metadata", "result")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    metadata: Row43
    result: Row56
    def __init__(self, source_id: _Optional[str] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., result: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("node_id", "instance_count", "pending")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    INSTANCE_COUNT_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    instance_count: int
    pending: bool
    def __init__(self, node_id: _Optional[str] = ..., instance_count: _Optional[int] = ..., pending: _Optional[bool] = ...) -> None: ...

class Row39(_message.Message):
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
    filter: Row47
    def __init__(self, selection_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., suffix: _Optional[str] = ..., node_id: _Optional[str] = ..., filter: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("node_id", "metadata", "unavailable")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: Row43
    unavailable: str
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("cut", "telemetry", "cacheable")
    CUT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    CACHEABLE_FIELD_NUMBER: _ClassVar[int]
    cut: Row57
    telemetry: bool
    cacheable: bool
    def __init__(self, cut: _Optional[_Union[Row57, _Mapping]] = ..., telemetry: _Optional[bool] = ..., cacheable: _Optional[bool] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("current", "next")
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    current: str
    next: str
    def __init__(self, current: _Optional[str] = ..., next: _Optional[str] = ...) -> None: ...

class Row43(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row58]
    created_at: Row59
    updated_at: Row59
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row58, _Mapping]]] = ..., created_at: _Optional[_Union[Row59, _Mapping]] = ..., updated_at: _Optional[_Union[Row59, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
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
    origin: Row60
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row60, _Mapping]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("process_epoch", "state_revision", "telemetry", "artifacts_sha256", "evidence")
    PROCESS_EPOCH_FIELD_NUMBER: _ClassVar[int]
    STATE_REVISION_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_SHA256_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_FIELD_NUMBER: _ClassVar[int]
    process_epoch: str
    state_revision: int
    telemetry: Row57
    artifacts_sha256: str
    evidence: Row57
    def __init__(self, process_epoch: _Optional[str] = ..., state_revision: _Optional[int] = ..., telemetry: _Optional[_Union[Row57, _Mapping]] = ..., artifacts_sha256: _Optional[str] = ..., evidence: _Optional[_Union[Row57, _Mapping]] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("Section", "Placement")
    SECTION_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    Section: Row61
    Placement: Row62
    def __init__(self, Section: _Optional[_Union[Row61, _Mapping]] = ..., Placement: _Optional[_Union[Row62, _Mapping]] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: Row54
    operator: Row63
    value: Row49
    def __init__(self, expression: _Optional[_Union[Row54, _Mapping]] = ..., operator: _Optional[_Union[Row63, _Mapping]] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("Markdown", "Table", "Summary", "Owner")
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    Markdown: Row64
    Table: Row65
    Summary: Row66
    Owner: Row67
    def __init__(self, Markdown: _Optional[_Union[Row64, _Mapping]] = ..., Table: _Optional[_Union[Row65, _Mapping]] = ..., Summary: _Optional[_Union[Row66, _Mapping]] = ..., Owner: _Optional[_Union[Row67, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row68]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row68, _Mapping]]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("Unspecified", "Latest")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row69
    Latest: Row70
    def __init__(self, Unspecified: _Optional[_Union[Row69, _Mapping]] = ..., Latest: _Optional[_Union[Row70, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("Unspecified", "Auto", "Number", "Percent", "Duration", "Distance", "Status", "Text")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    PERCENT_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row71
    Auto: Row72
    Number: Row73
    Percent: Row74
    Duration: Row75
    Distance: Row76
    Status: Row77
    Text: Row78
    def __init__(self, Unspecified: _Optional[_Union[Row71, _Mapping]] = ..., Auto: _Optional[_Union[Row72, _Mapping]] = ..., Number: _Optional[_Union[Row73, _Mapping]] = ..., Percent: _Optional[_Union[Row74, _Mapping]] = ..., Duration: _Optional[_Union[Row75, _Mapping]] = ..., Distance: _Optional[_Union[Row76, _Mapping]] = ..., Status: _Optional[_Union[Row77, _Mapping]] = ..., Text: _Optional[_Union[Row78, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("Unspecified", "Value", "Progress", "Badge")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    BADGE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row79
    Value: Row80
    Progress: Row81
    Badge: Row82
    def __init__(self, Unspecified: _Optional[_Union[Row79, _Mapping]] = ..., Value: _Optional[_Union[Row80, _Mapping]] = ..., Progress: _Optional[_Union[Row81, _Mapping]] = ..., Badge: _Optional[_Union[Row82, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("Unspecified", "Ok", "Warning", "Critical", "Muted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OK_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    CRITICAL_FIELD_NUMBER: _ClassVar[int]
    MUTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row83
    Ok: Row84
    Warning: Row85
    Critical: Row86
    Muted: Row87
    def __init__(self, Unspecified: _Optional[_Union[Row83, _Mapping]] = ..., Ok: _Optional[_Union[Row84, _Mapping]] = ..., Warning: _Optional[_Union[Row85, _Mapping]] = ..., Critical: _Optional[_Union[Row86, _Mapping]] = ..., Muted: _Optional[_Union[Row87, _Mapping]] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("source", "transforms", "label")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    source: Row88
    transforms: _containers.RepeatedCompositeFieldContainer[Row89]
    label: str
    def __init__(self, source: _Optional[_Union[Row88, _Mapping]] = ..., transforms: _Optional[_Iterable[_Union[Row89, _Mapping]]] = ..., label: _Optional[str] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: Row63
    value: Row49
    state: Row53
    def __init__(self, operator: _Optional[_Union[Row63, _Mapping]] = ..., value: _Optional[_Union[Row49, _Mapping]] = ..., state: _Optional[_Union[Row53, _Mapping]] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("Literal", "Summary", "Plot", "Pending", "Failure")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    Literal: Row90
    Summary: Row91
    Plot: Row92
    Pending: Row93
    Failure: Row94
    def __init__(self, Literal: _Optional[_Union[Row90, _Mapping]] = ..., Summary: _Optional[_Union[Row91, _Mapping]] = ..., Plot: _Optional[_Union[Row92, _Mapping]] = ..., Pending: _Optional[_Union[Row93, _Mapping]] = ..., Failure: _Optional[_Union[Row94, _Mapping]] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ("store", "epoch", "position")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    position: int
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row58(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row49
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row95
    def __init__(self, kind: _Optional[_Union[Row95, _Mapping]] = ...) -> None: ...

class Row61(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row96
    def __init__(self, field_0: _Optional[_Union[Row96, _Mapping]] = ...) -> None: ...

class Row62(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row97
    def __init__(self, field_0: _Optional[_Union[Row97, _Mapping]] = ...) -> None: ...

class Row63(_message.Message):
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
    Unspecified: Row98
    Eq: Row99
    Ne: Row100
    Gt: Row101
    Gte: Row102
    Lt: Row103
    Lte: Row104
    Contains: Row105
    Exists: Row106
    def __init__(self, Unspecified: _Optional[_Union[Row98, _Mapping]] = ..., Eq: _Optional[_Union[Row99, _Mapping]] = ..., Ne: _Optional[_Union[Row100, _Mapping]] = ..., Gt: _Optional[_Union[Row101, _Mapping]] = ..., Gte: _Optional[_Union[Row102, _Mapping]] = ..., Lt: _Optional[_Union[Row103, _Mapping]] = ..., Lte: _Optional[_Union[Row104, _Mapping]] = ..., Contains: _Optional[_Union[Row105, _Mapping]] = ..., Exists: _Optional[_Union[Row106, _Mapping]] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row65(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row107
    def __init__(self, field_0: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row108
    def __init__(self, field_0: _Optional[_Union[Row108, _Mapping]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row109
    def __init__(self, field_0: _Optional[_Union[Row109, _Mapping]] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row110
    def __init__(self, kind: _Optional[_Union[Row110, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row79(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row80(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row81(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row82(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row83(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row84(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row85(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row86(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row87(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row88(_message.Message):
    __slots__ = ("Telemetry", "Parameter", "Dimension", "Record", "Artifact")
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    Telemetry: Row111
    Parameter: Row112
    Dimension: Row113
    Record: Row114
    Artifact: Row115
    def __init__(self, Telemetry: _Optional[_Union[Row111, _Mapping]] = ..., Parameter: _Optional[_Union[Row112, _Mapping]] = ..., Dimension: _Optional[_Union[Row113, _Mapping]] = ..., Record: _Optional[_Union[Row114, _Mapping]] = ..., Artifact: _Optional[_Union[Row115, _Mapping]] = ...) -> None: ...

class Row89(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row116
    def __init__(self, kind: _Optional[_Union[Row116, _Mapping]] = ...) -> None: ...

class Row90(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row91(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row108
    def __init__(self, field_0: _Optional[_Union[Row108, _Mapping]] = ...) -> None: ...

class Row92(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row117
    def __init__(self, field_0: _Optional[_Union[Row117, _Mapping]] = ...) -> None: ...

class Row93(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row94(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row118
    def __init__(self, field_0: _Optional[_Union[Row118, _Mapping]] = ...) -> None: ...

class Row95(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row119
    Initialization: Row120
    StepAttemptId: Row121
    Cleanup: Row122
    def __init__(self, Default: _Optional[_Union[Row119, _Mapping]] = ..., Initialization: _Optional[_Union[Row120, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row121, _Mapping]] = ..., Cleanup: _Optional[_Union[Row122, _Mapping]] = ...) -> None: ...

class Row96(_message.Message):
    __slots__ = ("node_id", "metadata", "repeat")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: Row43
    repeat: Row123
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[Row43, _Mapping]] = ..., repeat: _Optional[_Union[Row123, _Mapping]] = ...) -> None: ...

class Row97(_message.Message):
    __slots__ = ("card_id", "column_span", "row_span", "order", "density", "rail")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMN_SPAN_FIELD_NUMBER: _ClassVar[int]
    ROW_SPAN_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    RAIL_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    column_span: Row124
    row_span: int
    order: int
    density: Row125
    rail: Row126
    def __init__(self, card_id: _Optional[str] = ..., column_span: _Optional[_Union[Row124, _Mapping]] = ..., row_span: _Optional[int] = ..., order: _Optional[int] = ..., density: _Optional[_Union[Row125, _Mapping]] = ..., rail: _Optional[_Union[Row126, _Mapping]] = ...) -> None: ...

class Row98(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row99(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row100(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row101(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row102(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row103(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row104(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row105(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row106(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row107(_message.Message):
    __slots__ = ("row_count",)
    ROW_COUNT_FIELD_NUMBER: _ClassVar[int]
    row_count: int
    def __init__(self, row_count: _Optional[int] = ...) -> None: ...

class Row108(_message.Message):
    __slots__ = ("group_count",)
    GROUP_COUNT_FIELD_NUMBER: _ClassVar[int]
    group_count: int
    def __init__(self, group_count: _Optional[int] = ...) -> None: ...

class Row109(_message.Message):
    __slots__ = ("kind", "plot_id", "plot")
    KIND_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    kind: Row127
    plot_id: str
    plot: Row117
    def __init__(self, kind: _Optional[_Union[Row127, _Mapping]] = ..., plot_id: _Optional[str] = ..., plot: _Optional[_Union[Row117, _Mapping]] = ...) -> None: ...

class Row110(_message.Message):
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
    Null: Row128
    Bool: Row129
    Integer: Row130
    Unsigned: Row131
    Number: Row132
    Text: Row133
    Datetime: Row134
    Array: Row135
    Object: Row136
    def __init__(self, Null: _Optional[_Union[Row128, _Mapping]] = ..., Bool: _Optional[_Union[Row129, _Mapping]] = ..., Integer: _Optional[_Union[Row130, _Mapping]] = ..., Unsigned: _Optional[_Union[Row131, _Mapping]] = ..., Number: _Optional[_Union[Row132, _Mapping]] = ..., Text: _Optional[_Union[Row133, _Mapping]] = ..., Datetime: _Optional[_Union[Row134, _Mapping]] = ..., Array: _Optional[_Union[Row135, _Mapping]] = ..., Object: _Optional[_Union[Row136, _Mapping]] = ...) -> None: ...

class Row111(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row137
    def __init__(self, field_0: _Optional[_Union[Row137, _Mapping]] = ...) -> None: ...

class Row112(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row138
    def __init__(self, field_0: _Optional[_Union[Row138, _Mapping]] = ...) -> None: ...

class Row113(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row139
    def __init__(self, field_0: _Optional[_Union[Row139, _Mapping]] = ...) -> None: ...

class Row114(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row140
    def __init__(self, field_0: _Optional[_Union[Row140, _Mapping]] = ...) -> None: ...

class Row115(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row141
    def __init__(self, field_0: _Optional[_Union[Row141, _Mapping]] = ...) -> None: ...

class Row116(_message.Message):
    __slots__ = ("Aggregate", "TimeBetween", "Detail", "PerCase")
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    TIMEBETWEEN_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PERCASE_FIELD_NUMBER: _ClassVar[int]
    Aggregate: Row142
    TimeBetween: Row143
    Detail: Row144
    PerCase: Row145
    def __init__(self, Aggregate: _Optional[_Union[Row142, _Mapping]] = ..., TimeBetween: _Optional[_Union[Row143, _Mapping]] = ..., Detail: _Optional[_Union[Row144, _Mapping]] = ..., PerCase: _Optional[_Union[Row145, _Mapping]] = ...) -> None: ...

class Row117(_message.Message):
    __slots__ = ("metadata", "request", "presentation", "z_expression")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row43
    request: Row146
    presentation: Row147
    z_expression: Row54
    def __init__(self, metadata: _Optional[_Union[Row43, _Mapping]] = ..., request: _Optional[_Union[Row146, _Mapping]] = ..., presentation: _Optional[_Union[Row147, _Mapping]] = ..., z_expression: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row118(_message.Message):
    __slots__ = ("code", "message", "budget")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    BUDGET_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    budget: Row148
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ..., budget: _Optional[_Union[Row148, _Mapping]] = ...) -> None: ...

class Row119(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row120(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row121(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row122(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row123(_message.Message):
    __slots__ = ("value", "label_template", "id_template", "order_by", "filters")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ID_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    value: Row54
    label_template: str
    id_template: str
    order_by: Row54
    filters: _containers.RepeatedCompositeFieldContainer[Row47]
    def __init__(self, value: _Optional[_Union[Row54, _Mapping]] = ..., label_template: _Optional[str] = ..., id_template: _Optional[str] = ..., order_by: _Optional[_Union[Row54, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row47, _Mapping]]] = ...) -> None: ...

class Row124(_message.Message):
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

class Row125(_message.Message):
    __slots__ = ("Unspecified", "Compact", "Normal", "Spacious")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPACT_FIELD_NUMBER: _ClassVar[int]
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    SPACIOUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row149
    Compact: Row150
    Normal: Row151
    Spacious: Row152
    def __init__(self, Unspecified: _Optional[_Union[Row149, _Mapping]] = ..., Compact: _Optional[_Union[Row150, _Mapping]] = ..., Normal: _Optional[_Union[Row151, _Mapping]] = ..., Spacious: _Optional[_Union[Row152, _Mapping]] = ...) -> None: ...

class Row126(_message.Message):
    __slots__ = ("Unspecified", "Main", "Right", "Left")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    MAIN_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row153
    Main: Row154
    Right: Row155
    Left: Row156
    def __init__(self, Unspecified: _Optional[_Union[Row153, _Mapping]] = ..., Main: _Optional[_Union[Row154, _Mapping]] = ..., Right: _Optional[_Union[Row155, _Mapping]] = ..., Left: _Optional[_Union[Row156, _Mapping]] = ...) -> None: ...

class Row127(_message.Message):
    __slots__ = ("Unspecified", "Text", "TelemetrySummary", "Plot", "Table", "Steps", "Criteria", "Traceability", "Notes", "Records", "Artifacts", "Media", "Health", "TelemetryContext")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYSUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYCONTEXT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row157
    Text: Row158
    TelemetrySummary: Row159
    Plot: Row160
    Table: Row161
    Steps: Row162
    Criteria: Row163
    Traceability: Row164
    Notes: Row165
    Records: Row166
    Artifacts: Row167
    Media: Row168
    Health: Row169
    TelemetryContext: Row170
    def __init__(self, Unspecified: _Optional[_Union[Row157, _Mapping]] = ..., Text: _Optional[_Union[Row158, _Mapping]] = ..., TelemetrySummary: _Optional[_Union[Row159, _Mapping]] = ..., Plot: _Optional[_Union[Row160, _Mapping]] = ..., Table: _Optional[_Union[Row161, _Mapping]] = ..., Steps: _Optional[_Union[Row162, _Mapping]] = ..., Criteria: _Optional[_Union[Row163, _Mapping]] = ..., Traceability: _Optional[_Union[Row164, _Mapping]] = ..., Notes: _Optional[_Union[Row165, _Mapping]] = ..., Records: _Optional[_Union[Row166, _Mapping]] = ..., Artifacts: _Optional[_Union[Row167, _Mapping]] = ..., Media: _Optional[_Union[Row168, _Mapping]] = ..., Health: _Optional[_Union[Row169, _Mapping]] = ..., TelemetryContext: _Optional[_Union[Row170, _Mapping]] = ...) -> None: ...

class Row128(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row129(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row130(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row131(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row132(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row133(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row134(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row59
    def __init__(self, field_0: _Optional[_Union[Row59, _Mapping]] = ...) -> None: ...

class Row135(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row171
    def __init__(self, field_0: _Optional[_Union[Row171, _Mapping]] = ...) -> None: ...

class Row136(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row172
    def __init__(self, field_0: _Optional[_Union[Row172, _Mapping]] = ...) -> None: ...

class Row137(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row138(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row139(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row140(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class Row141(_message.Message):
    __slots__ = ("artifact",)
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    def __init__(self, artifact: _Optional[str] = ...) -> None: ...

class Row142(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row173
    def __init__(self, field_0: _Optional[_Union[Row173, _Mapping]] = ...) -> None: ...

class Row143(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row174
    def __init__(self, field_0: _Optional[_Union[Row174, _Mapping]] = ...) -> None: ...

class Row144(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row175
    def __init__(self, field_0: _Optional[_Union[Row175, _Mapping]] = ...) -> None: ...

class Row145(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row176
    def __init__(self, field_0: _Optional[_Union[Row176, _Mapping]] = ...) -> None: ...

class Row146(_message.Message):
    __slots__ = ("plot_id", "plot_type", "x_scale", "y_scale", "stream", "y_keys", "marker_record_types", "filters", "x_key", "series_by", "x_expression", "y_expressions", "series_by_expression", "z_stream", "z_field", "unit", "max_points", "window_s", "scope", "technique", "z_expression", "fft", "interval", "histogram", "analysis", "summary_window")
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
    TECHNIQUE_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_WINDOW_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    plot_type: Row177
    x_scale: Row178
    y_scale: Row178
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[Row47]
    x_key: str
    series_by: str
    x_expression: Row54
    y_expressions: _containers.RepeatedCompositeFieldContainer[Row54]
    series_by_expression: Row54
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: Row44
    technique: Row179
    z_expression: Row54
    fft: Row180
    interval: Row181
    histogram: Row182
    analysis: Row183
    summary_window: Row184
    def __init__(self, plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[Row177, _Mapping]] = ..., x_scale: _Optional[_Union[Row178, _Mapping]] = ..., y_scale: _Optional[_Union[Row178, _Mapping]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[Row47, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[Row54, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[Row54, _Mapping]]] = ..., series_by_expression: _Optional[_Union[Row54, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[Row44, _Mapping]] = ..., technique: _Optional[_Union[Row179, _Mapping]] = ..., z_expression: _Optional[_Union[Row54, _Mapping]] = ..., fft: _Optional[_Union[Row180, _Mapping]] = ..., interval: _Optional[_Union[Row181, _Mapping]] = ..., histogram: _Optional[_Union[Row182, _Mapping]] = ..., analysis: _Optional[_Union[Row183, _Mapping]] = ..., summary_window: _Optional[_Union[Row184, _Mapping]] = ...) -> None: ...

class Row147(_message.Message):
    __slots__ = ("style", "fft", "interval", "histogram", "reference_lines", "reference_bands", "x_format", "y_format", "x_axis", "x_axis_unit")
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
    style: Row185
    fft: Row180
    interval: Row181
    histogram: Row182
    reference_lines: _containers.RepeatedCompositeFieldContainer[Row186]
    reference_bands: _containers.RepeatedCompositeFieldContainer[Row187]
    x_format: Row188
    y_format: Row188
    x_axis: Row189
    x_axis_unit: str
    def __init__(self, style: _Optional[_Union[Row185, _Mapping]] = ..., fft: _Optional[_Union[Row180, _Mapping]] = ..., interval: _Optional[_Union[Row181, _Mapping]] = ..., histogram: _Optional[_Union[Row182, _Mapping]] = ..., reference_lines: _Optional[_Iterable[_Union[Row186, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[Row187, _Mapping]]] = ..., x_format: _Optional[_Union[Row188, _Mapping]] = ..., y_format: _Optional[_Union[Row188, _Mapping]] = ..., x_axis: _Optional[_Union[Row189, _Mapping]] = ..., x_axis_unit: _Optional[str] = ...) -> None: ...

class Row148(_message.Message):
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

class Row149(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row150(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row151(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row152(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row153(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row154(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row155(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row156(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row157(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row158(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row159(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row160(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row161(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row162(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row163(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row164(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row165(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row166(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row167(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row168(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row169(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row170(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row171(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row172(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row190]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row190, _Mapping]]] = ...) -> None: ...

class Row173(_message.Message):
    __slots__ = ("aggregate",)
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    aggregate: Row191
    def __init__(self, aggregate: _Optional[_Union[Row191, _Mapping]] = ...) -> None: ...

class Row174(_message.Message):
    __slots__ = ("previous_record_type",)
    PREVIOUS_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    previous_record_type: str
    def __init__(self, previous_record_type: _Optional[str] = ...) -> None: ...

class Row175(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row176(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row177(_message.Message):
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
    Unspecified: Row192
    TimeSeries: Row193
    Bar: Row194
    GroupedBar: Row195
    StackedBar: Row196
    Pie: Row197
    Donut: Row198
    Pareto: Row199
    XyScatter: Row200
    XyLine: Row201
    XyInterval: Row202
    Trajectory: Row203
    Heatmap: Row204
    XyValueHeatmap: Row205
    WifiChannelOccupancy: Row206
    FrequencySeries: Row207
    Histogram: Row208
    BoxSummary: Row209
    AggregateTable: Row210
    def __init__(self, Unspecified: _Optional[_Union[Row192, _Mapping]] = ..., TimeSeries: _Optional[_Union[Row193, _Mapping]] = ..., Bar: _Optional[_Union[Row194, _Mapping]] = ..., GroupedBar: _Optional[_Union[Row195, _Mapping]] = ..., StackedBar: _Optional[_Union[Row196, _Mapping]] = ..., Pie: _Optional[_Union[Row197, _Mapping]] = ..., Donut: _Optional[_Union[Row198, _Mapping]] = ..., Pareto: _Optional[_Union[Row199, _Mapping]] = ..., XyScatter: _Optional[_Union[Row200, _Mapping]] = ..., XyLine: _Optional[_Union[Row201, _Mapping]] = ..., XyInterval: _Optional[_Union[Row202, _Mapping]] = ..., Trajectory: _Optional[_Union[Row203, _Mapping]] = ..., Heatmap: _Optional[_Union[Row204, _Mapping]] = ..., XyValueHeatmap: _Optional[_Union[Row205, _Mapping]] = ..., WifiChannelOccupancy: _Optional[_Union[Row206, _Mapping]] = ..., FrequencySeries: _Optional[_Union[Row207, _Mapping]] = ..., Histogram: _Optional[_Union[Row208, _Mapping]] = ..., BoxSummary: _Optional[_Union[Row209, _Mapping]] = ..., AggregateTable: _Optional[_Union[Row210, _Mapping]] = ...) -> None: ...

class Row178(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Log")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    LOG_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row211
    Linear: Row212
    Log: Row213
    def __init__(self, Unspecified: _Optional[_Union[Row211, _Mapping]] = ..., Linear: _Optional[_Union[Row212, _Mapping]] = ..., Log: _Optional[_Union[Row213, _Mapping]] = ...) -> None: ...

class Row179(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row214
    Raw: Row215
    MinMax: Row216
    Lttb: Row217
    Auto: Row218
    def __init__(self, Unspecified: _Optional[_Union[Row214, _Mapping]] = ..., Raw: _Optional[_Union[Row215, _Mapping]] = ..., MinMax: _Optional[_Union[Row216, _Mapping]] = ..., Lttb: _Optional[_Union[Row217, _Mapping]] = ..., Auto: _Optional[_Union[Row218, _Mapping]] = ...) -> None: ...

class Row180(_message.Message):
    __slots__ = ("sample_rate_hz", "window", "detrend", "level_unit", "include_dc", "min_frequency_hz", "max_frequency_hz")
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: Row219
    detrend: Row220
    level_unit: Row221
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[Row219, _Mapping]] = ..., detrend: _Optional[_Union[Row220, _Mapping]] = ..., level_unit: _Optional[_Union[Row221, _Mapping]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ...) -> None: ...

class Row181(_message.Message):
    __slots__ = ("bin_width", "sigma", "clip_to_observed")
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...

class Row182(_message.Message):
    __slots__ = ("bins", "unit")
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

class Row183(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: Row49
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row184(_message.Message):
    __slots__ = ("to",)
    FROM_FIELD_NUMBER: _ClassVar[int]
    TO_FIELD_NUMBER: _ClassVar[int]
    to: float
    def __init__(self, to: _Optional[float] = ..., **kwargs) -> None: ...

class Row185(_message.Message):
    __slots__ = ("Unspecified", "Line", "Points", "LinePoints", "Step")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINE_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    LINEPOINTS_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row222
    Line: Row223
    Points: Row224
    LinePoints: Row225
    Step: Row226
    def __init__(self, Unspecified: _Optional[_Union[Row222, _Mapping]] = ..., Line: _Optional[_Union[Row223, _Mapping]] = ..., Points: _Optional[_Union[Row224, _Mapping]] = ..., LinePoints: _Optional[_Union[Row225, _Mapping]] = ..., Step: _Optional[_Union[Row226, _Mapping]] = ...) -> None: ...

class Row186(_message.Message):
    __slots__ = ("axis", "value", "source", "source_scale", "label", "color")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_SCALE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    axis: Row227
    value: float
    source: Row54
    source_scale: float
    label: str
    color: str
    def __init__(self, axis: _Optional[_Union[Row227, _Mapping]] = ..., value: _Optional[float] = ..., source: _Optional[_Union[Row54, _Mapping]] = ..., source_scale: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class Row187(_message.Message):
    __slots__ = ("axis", "min", "max", "label", "color", "center", "tolerance")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    axis: Row227
    min: float
    max: float
    label: str
    color: str
    center: Row54
    tolerance: Row54
    def __init__(self, axis: _Optional[_Union[Row227, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[Row54, _Mapping]] = ..., tolerance: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row188(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row228
    def __init__(self, kind: _Optional[_Union[Row228, _Mapping]] = ...) -> None: ...

class Row189(_message.Message):
    __slots__ = ("Unspecified", "WallTime", "Elapsed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    WALLTIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row229
    WallTime: Row230
    Elapsed: Row231
    def __init__(self, Unspecified: _Optional[_Union[Row229, _Mapping]] = ..., WallTime: _Optional[_Union[Row230, _Mapping]] = ..., Elapsed: _Optional[_Union[Row231, _Mapping]] = ...) -> None: ...

class Row190(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row191(_message.Message):
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
    Percentile: Row232
    def __init__(self, Count: _Optional[_Union[Row2, _Mapping]] = ..., Sum: _Optional[_Union[Row2, _Mapping]] = ..., Mean: _Optional[_Union[Row2, _Mapping]] = ..., Median: _Optional[_Union[Row2, _Mapping]] = ..., Min: _Optional[_Union[Row2, _Mapping]] = ..., Max: _Optional[_Union[Row2, _Mapping]] = ..., Rms: _Optional[_Union[Row2, _Mapping]] = ..., Std: _Optional[_Union[Row2, _Mapping]] = ..., Percentile: _Optional[_Union[Row232, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row198(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row199(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row200(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row201(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row207(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row208(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row209(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row210(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Unspecified", "Hann", "Rectangular")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    HANN_FIELD_NUMBER: _ClassVar[int]
    RECTANGULAR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row233
    Hann: Row234
    Rectangular: Row235
    def __init__(self, Unspecified: _Optional[_Union[Row233, _Mapping]] = ..., Hann: _Optional[_Union[Row234, _Mapping]] = ..., Rectangular: _Optional[_Union[Row235, _Mapping]] = ...) -> None: ...

class Row220(_message.Message):
    __slots__ = ("Unspecified", "Mean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NONE_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row236
    Mean: Row238
    def __init__(self, Unspecified: _Optional[_Union[Row236, _Mapping]] = ..., Mean: _Optional[_Union[Row238, _Mapping]] = ..., **kwargs) -> None: ...

class Row221(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Db", "Dbu", "Dbm")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    DB_FIELD_NUMBER: _ClassVar[int]
    DBU_FIELD_NUMBER: _ClassVar[int]
    DBM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row239
    Linear: Row240
    Db: Row241
    Dbu: Row242
    Dbm: Row243
    def __init__(self, Unspecified: _Optional[_Union[Row239, _Mapping]] = ..., Linear: _Optional[_Union[Row240, _Mapping]] = ..., Db: _Optional[_Union[Row241, _Mapping]] = ..., Dbu: _Optional[_Union[Row242, _Mapping]] = ..., Dbm: _Optional[_Union[Row243, _Mapping]] = ...) -> None: ...

class Row222(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row223(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row224(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row225(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row226(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row227(_message.Message):
    __slots__ = ("Unspecified", "X", "Y")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row244
    X: Row245
    Y: Row246
    def __init__(self, Unspecified: _Optional[_Union[Row244, _Mapping]] = ..., X: _Optional[_Union[Row245, _Mapping]] = ..., Y: _Optional[_Union[Row246, _Mapping]] = ...) -> None: ...

class Row228(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row247
    Fixed: Row248
    Scientific: Row249
    Engineering: Row250
    SiPrefix: Row251
    AbsoluteTime: Row252
    ElapsedSinceRun: Row253
    ElapsedSinceCycle: Row254
    def __init__(self, Normal: _Optional[_Union[Row247, _Mapping]] = ..., Fixed: _Optional[_Union[Row248, _Mapping]] = ..., Scientific: _Optional[_Union[Row249, _Mapping]] = ..., Engineering: _Optional[_Union[Row250, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row251, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row252, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row253, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row254, _Mapping]] = ...) -> None: ...

class Row229(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row230(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row231(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row232(_message.Message):
    __slots__ = ("rank",)
    RANK_FIELD_NUMBER: _ClassVar[int]
    rank: int
    def __init__(self, rank: _Optional[int] = ...) -> None: ...

class Row233(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row234(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row235(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row236(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row237(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row238(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row239(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row240(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row241(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row242(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row243(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row255
    def __init__(self, field_0: _Optional[_Union[Row255, _Mapping]] = ...) -> None: ...

class Row248(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row256
    def __init__(self, field_0: _Optional[_Union[Row256, _Mapping]] = ...) -> None: ...

class Row249(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row257
    def __init__(self, field_0: _Optional[_Union[Row257, _Mapping]] = ...) -> None: ...

class Row250(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row258
    def __init__(self, field_0: _Optional[_Union[Row258, _Mapping]] = ...) -> None: ...

class Row251(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row259
    def __init__(self, field_0: _Optional[_Union[Row259, _Mapping]] = ...) -> None: ...

class Row252(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row260
    def __init__(self, field_0: _Optional[_Union[Row260, _Mapping]] = ...) -> None: ...

class Row253(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row261
    def __init__(self, field_0: _Optional[_Union[Row261, _Mapping]] = ...) -> None: ...

class Row254(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row261
    def __init__(self, field_0: _Optional[_Union[Row261, _Mapping]] = ...) -> None: ...

class Row255(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row256(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row257(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row258(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row259(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row260(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row261(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row262
    def __init__(self, unit: _Optional[_Union[Row262, _Mapping]] = ...) -> None: ...

class Row262(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row263
    Seconds: Row264
    Milliseconds: Row265
    Minutes: Row266
    def __init__(self, Unspecified: _Optional[_Union[Row263, _Mapping]] = ..., Seconds: _Optional[_Union[Row264, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row265, _Mapping]] = ..., Minutes: _Optional[_Union[Row266, _Mapping]] = ...) -> None: ...

class Row263(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row264(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row265(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row266(_message.Message):
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
