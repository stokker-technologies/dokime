from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("scope", "document_id")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    scope: Row4
    document_id: str
    def __init__(self, scope: _Optional[_Union[Row4, _Mapping]] = ..., document_id: _Optional[str] = ...) -> None: ...

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
    __slots__ = ("Header", "LayoutNode", "Card", "TableRow", "SummaryGroup", "SummaryTile", "Warning", "Completion", "GroupDefinition", "TileDefinition", "Binding", "RepeatSelection", "RepeatInstance", "SectionIdentity")
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
    def __init__(self, Header: _Optional[_Union[Row6, _Mapping]] = ..., LayoutNode: _Optional[_Union[Row7, _Mapping]] = ..., Card: _Optional[_Union[Row8, _Mapping]] = ..., TableRow: _Optional[_Union[Row9, _Mapping]] = ..., SummaryGroup: _Optional[_Union[Row10, _Mapping]] = ..., SummaryTile: _Optional[_Union[Row11, _Mapping]] = ..., Warning: _Optional[_Union[Row12, _Mapping]] = ..., Completion: _Optional[_Union[Row13, _Mapping]] = ..., GroupDefinition: _Optional[_Union[Row14, _Mapping]] = ..., TileDefinition: _Optional[_Union[Row15, _Mapping]] = ..., Binding: _Optional[_Union[Row16, _Mapping]] = ..., RepeatSelection: _Optional[_Union[Row17, _Mapping]] = ..., RepeatInstance: _Optional[_Union[Row18, _Mapping]] = ..., SectionIdentity: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row20
    def __init__(self, field_0: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row21
    def __init__(self, field_0: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row22
    def __init__(self, field_0: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row23
    def __init__(self, field_0: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row24
    def __init__(self, field_0: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row25
    def __init__(self, field_0: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row26
    def __init__(self, field_0: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row27
    def __init__(self, field_0: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row28
    def __init__(self, field_0: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row31
    def __init__(self, field_0: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("document_id", "metadata", "scope", "frontiers")
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FRONTIERS_FIELD_NUMBER: _ClassVar[int]
    document_id: str
    metadata: Row34
    scope: Row35
    frontiers: Row36
    def __init__(self, document_id: _Optional[str] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., scope: _Optional[_Union[Row35, _Mapping]] = ..., frontiers: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("ordinal", "parent_ordinal", "node")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    PARENT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    NODE_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    parent_ordinal: int
    node: Row37
    def __init__(self, ordinal: _Optional[int] = ..., parent_ordinal: _Optional[int] = ..., node: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("metadata", "filters", "body")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    BODY_FIELD_NUMBER: _ClassVar[int]
    metadata: Row34
    filters: _containers.RepeatedCompositeFieldContainer[Row38]
    body: Row39
    def __init__(self, metadata: _Optional[_Union[Row34, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ..., body: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("card_id", "ordinal", "values")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    values: Row40
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., values: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: Row34
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class Row25(_message.Message):
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
    metadata: Row34
    value: Row40
    stat: Row41
    format: Row42
    presentation: Row43
    unit: str
    state: Row44
    unavailable: str
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., value: _Optional[_Union[Row40, _Mapping]] = ..., stat: _Optional[_Union[Row41, _Mapping]] = ..., format: _Optional[_Union[Row42, _Mapping]] = ..., presentation: _Optional[_Union[Row43, _Mapping]] = ..., unit: _Optional[str] = ..., state: _Optional[_Union[Row44, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("code", "message")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class Row27(_message.Message):
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

class Row28(_message.Message):
    __slots__ = ("card_id", "ordinal", "metadata", "tile_count")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILE_COUNT_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    metadata: Row34
    tile_count: int
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., tile_count: _Optional[int] = ...) -> None: ...

class Row29(_message.Message):
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
    metadata: Row34
    value: Row45
    filters: _containers.RepeatedCompositeFieldContainer[Row38]
    stat: Row41
    format: Row42
    presentation: Row43
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[Row46]
    def __init__(self, card_id: _Optional[str] = ..., group_ordinal: _Optional[int] = ..., ordinal: _Optional[int] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., value: _Optional[_Union[Row45, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ..., stat: _Optional[_Union[Row41, _Mapping]] = ..., format: _Optional[_Union[Row42, _Mapping]] = ..., presentation: _Optional[_Union[Row43, _Mapping]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[Row46, _Mapping]]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("source_id", "metadata", "result")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    metadata: Row34
    result: Row47
    def __init__(self, source_id: _Optional[str] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., result: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("node_id", "instance_count")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    INSTANCE_COUNT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    instance_count: int
    def __init__(self, node_id: _Optional[str] = ..., instance_count: _Optional[int] = ...) -> None: ...

class Row32(_message.Message):
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
    filter: Row38
    def __init__(self, selection_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., suffix: _Optional[str] = ..., node_id: _Optional[str] = ..., filter: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("node_id", "metadata", "unavailable")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: Row34
    unavailable: str
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., unavailable: _Optional[str] = ...) -> None: ...

class Row34(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row48]
    created_at: Row49
    updated_at: Row49
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row48, _Mapping]]] = ..., created_at: _Optional[_Union[Row49, _Mapping]] = ..., updated_at: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
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
    origin: Row50
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("process_epoch", "state_revision", "telemetry", "artifacts_sha256")
    PROCESS_EPOCH_FIELD_NUMBER: _ClassVar[int]
    STATE_REVISION_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_SHA256_FIELD_NUMBER: _ClassVar[int]
    process_epoch: str
    state_revision: int
    telemetry: Row51
    artifacts_sha256: str
    def __init__(self, process_epoch: _Optional[str] = ..., state_revision: _Optional[int] = ..., telemetry: _Optional[_Union[Row51, _Mapping]] = ..., artifacts_sha256: _Optional[str] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("Section", "Placement")
    SECTION_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    Section: Row52
    Placement: Row53
    def __init__(self, Section: _Optional[_Union[Row52, _Mapping]] = ..., Placement: _Optional[_Union[Row53, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: Row45
    operator: Row54
    value: Row40
    def __init__(self, expression: _Optional[_Union[Row45, _Mapping]] = ..., operator: _Optional[_Union[Row54, _Mapping]] = ..., value: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("Markdown", "Table", "Summary", "Owner")
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    Markdown: Row55
    Table: Row56
    Summary: Row57
    Owner: Row58
    def __init__(self, Markdown: _Optional[_Union[Row55, _Mapping]] = ..., Table: _Optional[_Union[Row56, _Mapping]] = ..., Summary: _Optional[_Union[Row57, _Mapping]] = ..., Owner: _Optional[_Union[Row58, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row59]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row59, _Mapping]]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("Unspecified", "Latest")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row60
    Latest: Row61
    def __init__(self, Unspecified: _Optional[_Union[Row60, _Mapping]] = ..., Latest: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("Unspecified", "Auto", "Number", "Percent", "Duration", "Distance", "Status", "Text")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    PERCENT_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row62
    Auto: Row63
    Number: Row64
    Percent: Row65
    Duration: Row66
    Distance: Row67
    Status: Row68
    Text: Row69
    def __init__(self, Unspecified: _Optional[_Union[Row62, _Mapping]] = ..., Auto: _Optional[_Union[Row63, _Mapping]] = ..., Number: _Optional[_Union[Row64, _Mapping]] = ..., Percent: _Optional[_Union[Row65, _Mapping]] = ..., Duration: _Optional[_Union[Row66, _Mapping]] = ..., Distance: _Optional[_Union[Row67, _Mapping]] = ..., Status: _Optional[_Union[Row68, _Mapping]] = ..., Text: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

class Row43(_message.Message):
    __slots__ = ("Unspecified", "Value", "Progress", "Badge")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    BADGE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row70
    Value: Row71
    Progress: Row72
    Badge: Row73
    def __init__(self, Unspecified: _Optional[_Union[Row70, _Mapping]] = ..., Value: _Optional[_Union[Row71, _Mapping]] = ..., Progress: _Optional[_Union[Row72, _Mapping]] = ..., Badge: _Optional[_Union[Row73, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("Unspecified", "Ok", "Warning", "Critical", "Muted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OK_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    CRITICAL_FIELD_NUMBER: _ClassVar[int]
    MUTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row74
    Ok: Row75
    Warning: Row76
    Critical: Row77
    Muted: Row78
    def __init__(self, Unspecified: _Optional[_Union[Row74, _Mapping]] = ..., Ok: _Optional[_Union[Row75, _Mapping]] = ..., Warning: _Optional[_Union[Row76, _Mapping]] = ..., Critical: _Optional[_Union[Row77, _Mapping]] = ..., Muted: _Optional[_Union[Row78, _Mapping]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("source", "transforms", "label")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    source: Row79
    transforms: _containers.RepeatedCompositeFieldContainer[Row80]
    label: str
    def __init__(self, source: _Optional[_Union[Row79, _Mapping]] = ..., transforms: _Optional[_Iterable[_Union[Row80, _Mapping]]] = ..., label: _Optional[str] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: Row54
    value: Row40
    state: Row44
    def __init__(self, operator: _Optional[_Union[Row54, _Mapping]] = ..., value: _Optional[_Union[Row40, _Mapping]] = ..., state: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("Literal", "Summary", "Plot")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    Literal: Row81
    Summary: Row82
    Plot: Row83
    def __init__(self, Literal: _Optional[_Union[Row81, _Mapping]] = ..., Summary: _Optional[_Union[Row82, _Mapping]] = ..., Plot: _Optional[_Union[Row83, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row40
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row84
    def __init__(self, kind: _Optional[_Union[Row84, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("delivered_cursor", "source", "series_ordinal")
    DELIVERED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SERIES_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    delivered_cursor: int
    source: Row85
    series_ordinal: int
    def __init__(self, delivered_cursor: _Optional[int] = ..., source: _Optional[_Union[Row85, _Mapping]] = ..., series_ordinal: _Optional[int] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row86
    def __init__(self, field_0: _Optional[_Union[Row86, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row87
    def __init__(self, field_0: _Optional[_Union[Row87, _Mapping]] = ...) -> None: ...

class Row54(_message.Message):
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
    Unspecified: Row88
    Eq: Row89
    Ne: Row90
    Gt: Row91
    Gte: Row92
    Lt: Row93
    Lte: Row94
    Contains: Row95
    Exists: Row96
    def __init__(self, Unspecified: _Optional[_Union[Row88, _Mapping]] = ..., Eq: _Optional[_Union[Row89, _Mapping]] = ..., Ne: _Optional[_Union[Row90, _Mapping]] = ..., Gt: _Optional[_Union[Row91, _Mapping]] = ..., Gte: _Optional[_Union[Row92, _Mapping]] = ..., Lt: _Optional[_Union[Row93, _Mapping]] = ..., Lte: _Optional[_Union[Row94, _Mapping]] = ..., Contains: _Optional[_Union[Row95, _Mapping]] = ..., Exists: _Optional[_Union[Row96, _Mapping]] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row97
    def __init__(self, field_0: _Optional[_Union[Row97, _Mapping]] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row98
    def __init__(self, field_0: _Optional[_Union[Row98, _Mapping]] = ...) -> None: ...

class Row58(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row99
    def __init__(self, field_0: _Optional[_Union[Row99, _Mapping]] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row100
    def __init__(self, kind: _Optional[_Union[Row100, _Mapping]] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row61(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row62(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row63(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row64(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row65(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row66(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row67(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row68(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Telemetry", "Parameter", "Dimension", "Record", "Artifact")
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    Telemetry: Row101
    Parameter: Row102
    Dimension: Row103
    Record: Row104
    Artifact: Row105
    def __init__(self, Telemetry: _Optional[_Union[Row101, _Mapping]] = ..., Parameter: _Optional[_Union[Row102, _Mapping]] = ..., Dimension: _Optional[_Union[Row103, _Mapping]] = ..., Record: _Optional[_Union[Row104, _Mapping]] = ..., Artifact: _Optional[_Union[Row105, _Mapping]] = ...) -> None: ...

class Row80(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row106
    def __init__(self, kind: _Optional[_Union[Row106, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row98
    def __init__(self, field_0: _Optional[_Union[Row98, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row107
    def __init__(self, field_0: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row108
    Initialization: Row109
    StepAttemptId: Row110
    Cleanup: Row111
    def __init__(self, Default: _Optional[_Union[Row108, _Mapping]] = ..., Initialization: _Optional[_Union[Row109, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row110, _Mapping]] = ..., Cleanup: _Optional[_Union[Row111, _Mapping]] = ...) -> None: ...

class Row85(_message.Message):
    __slots__ = ("SeriesEpoch", "RetainedWindow")
    SERIESEPOCH_FIELD_NUMBER: _ClassVar[int]
    RETAINEDWINDOW_FIELD_NUMBER: _ClassVar[int]
    SeriesEpoch: Row112
    RetainedWindow: Row113
    def __init__(self, SeriesEpoch: _Optional[_Union[Row112, _Mapping]] = ..., RetainedWindow: _Optional[_Union[Row113, _Mapping]] = ...) -> None: ...

class Row86(_message.Message):
    __slots__ = ("node_id", "metadata", "repeat")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: Row34
    repeat: Row114
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[Row34, _Mapping]] = ..., repeat: _Optional[_Union[Row114, _Mapping]] = ...) -> None: ...

class Row87(_message.Message):
    __slots__ = ("card_id", "column_span", "row_span", "order", "density", "rail")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMN_SPAN_FIELD_NUMBER: _ClassVar[int]
    ROW_SPAN_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    RAIL_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    column_span: Row115
    row_span: int
    order: int
    density: Row116
    rail: Row117
    def __init__(self, card_id: _Optional[str] = ..., column_span: _Optional[_Union[Row115, _Mapping]] = ..., row_span: _Optional[int] = ..., order: _Optional[int] = ..., density: _Optional[_Union[Row116, _Mapping]] = ..., rail: _Optional[_Union[Row117, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("row_count",)
    ROW_COUNT_FIELD_NUMBER: _ClassVar[int]
    row_count: int
    def __init__(self, row_count: _Optional[int] = ...) -> None: ...

class Row98(_message.Message):
    __slots__ = ("group_count",)
    GROUP_COUNT_FIELD_NUMBER: _ClassVar[int]
    group_count: int
    def __init__(self, group_count: _Optional[int] = ...) -> None: ...

class Row99(_message.Message):
    __slots__ = ("kind", "plot_id", "plot")
    KIND_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    kind: Row118
    plot_id: str
    plot: Row107
    def __init__(self, kind: _Optional[_Union[Row118, _Mapping]] = ..., plot_id: _Optional[str] = ..., plot: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row100(_message.Message):
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
    Null: Row119
    Bool: Row120
    Integer: Row121
    Unsigned: Row122
    Number: Row123
    Text: Row124
    Datetime: Row125
    Array: Row126
    Object: Row127
    def __init__(self, Null: _Optional[_Union[Row119, _Mapping]] = ..., Bool: _Optional[_Union[Row120, _Mapping]] = ..., Integer: _Optional[_Union[Row121, _Mapping]] = ..., Unsigned: _Optional[_Union[Row122, _Mapping]] = ..., Number: _Optional[_Union[Row123, _Mapping]] = ..., Text: _Optional[_Union[Row124, _Mapping]] = ..., Datetime: _Optional[_Union[Row125, _Mapping]] = ..., Array: _Optional[_Union[Row126, _Mapping]] = ..., Object: _Optional[_Union[Row127, _Mapping]] = ...) -> None: ...

class Row101(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row128
    def __init__(self, field_0: _Optional[_Union[Row128, _Mapping]] = ...) -> None: ...

class Row102(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row129
    def __init__(self, field_0: _Optional[_Union[Row129, _Mapping]] = ...) -> None: ...

class Row103(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row130
    def __init__(self, field_0: _Optional[_Union[Row130, _Mapping]] = ...) -> None: ...

class Row104(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row131
    def __init__(self, field_0: _Optional[_Union[Row131, _Mapping]] = ...) -> None: ...

class Row105(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row132
    def __init__(self, field_0: _Optional[_Union[Row132, _Mapping]] = ...) -> None: ...

class Row106(_message.Message):
    __slots__ = ("Aggregate", "TimeBetween", "TelemetryAtOrBefore", "Detail", "PerCase")
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    TIMEBETWEEN_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYATORBEFORE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PERCASE_FIELD_NUMBER: _ClassVar[int]
    Aggregate: Row133
    TimeBetween: Row134
    TelemetryAtOrBefore: Row135
    Detail: Row136
    PerCase: Row137
    def __init__(self, Aggregate: _Optional[_Union[Row133, _Mapping]] = ..., TimeBetween: _Optional[_Union[Row134, _Mapping]] = ..., TelemetryAtOrBefore: _Optional[_Union[Row135, _Mapping]] = ..., Detail: _Optional[_Union[Row136, _Mapping]] = ..., PerCase: _Optional[_Union[Row137, _Mapping]] = ...) -> None: ...

class Row107(_message.Message):
    __slots__ = ("metadata", "request", "presentation", "z_expression")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row34
    request: Row138
    presentation: Row139
    z_expression: Row45
    def __init__(self, metadata: _Optional[_Union[Row34, _Mapping]] = ..., request: _Optional[_Union[Row138, _Mapping]] = ..., presentation: _Optional[_Union[Row139, _Mapping]] = ..., z_expression: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row108(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row109(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row110(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row111(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row112(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row113(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row114(_message.Message):
    __slots__ = ("value", "label_template", "id_template", "order_by", "filters")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ID_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    value: Row45
    label_template: str
    id_template: str
    order_by: Row45
    filters: _containers.RepeatedCompositeFieldContainer[Row38]
    def __init__(self, value: _Optional[_Union[Row45, _Mapping]] = ..., label_template: _Optional[str] = ..., id_template: _Optional[str] = ..., order_by: _Optional[_Union[Row45, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ...) -> None: ...

class Row115(_message.Message):
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

class Row116(_message.Message):
    __slots__ = ("Unspecified", "Compact", "Normal", "Spacious")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPACT_FIELD_NUMBER: _ClassVar[int]
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    SPACIOUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row140
    Compact: Row141
    Normal: Row142
    Spacious: Row143
    def __init__(self, Unspecified: _Optional[_Union[Row140, _Mapping]] = ..., Compact: _Optional[_Union[Row141, _Mapping]] = ..., Normal: _Optional[_Union[Row142, _Mapping]] = ..., Spacious: _Optional[_Union[Row143, _Mapping]] = ...) -> None: ...

class Row117(_message.Message):
    __slots__ = ("Unspecified", "Main", "Right", "Left")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    MAIN_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row144
    Main: Row145
    Right: Row146
    Left: Row147
    def __init__(self, Unspecified: _Optional[_Union[Row144, _Mapping]] = ..., Main: _Optional[_Union[Row145, _Mapping]] = ..., Right: _Optional[_Union[Row146, _Mapping]] = ..., Left: _Optional[_Union[Row147, _Mapping]] = ...) -> None: ...

class Row118(_message.Message):
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
    Unspecified: Row148
    Text: Row149
    TelemetrySummary: Row150
    Plot: Row151
    Table: Row152
    Steps: Row153
    Criteria: Row154
    Traceability: Row155
    Notes: Row156
    Records: Row157
    Artifacts: Row158
    Media: Row159
    Health: Row160
    TelemetryContext: Row161
    def __init__(self, Unspecified: _Optional[_Union[Row148, _Mapping]] = ..., Text: _Optional[_Union[Row149, _Mapping]] = ..., TelemetrySummary: _Optional[_Union[Row150, _Mapping]] = ..., Plot: _Optional[_Union[Row151, _Mapping]] = ..., Table: _Optional[_Union[Row152, _Mapping]] = ..., Steps: _Optional[_Union[Row153, _Mapping]] = ..., Criteria: _Optional[_Union[Row154, _Mapping]] = ..., Traceability: _Optional[_Union[Row155, _Mapping]] = ..., Notes: _Optional[_Union[Row156, _Mapping]] = ..., Records: _Optional[_Union[Row157, _Mapping]] = ..., Artifacts: _Optional[_Union[Row158, _Mapping]] = ..., Media: _Optional[_Union[Row159, _Mapping]] = ..., Health: _Optional[_Union[Row160, _Mapping]] = ..., TelemetryContext: _Optional[_Union[Row161, _Mapping]] = ...) -> None: ...

class Row119(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row120(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row121(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row122(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row123(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row124(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row125(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row49
    def __init__(self, field_0: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row126(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row162
    def __init__(self, field_0: _Optional[_Union[Row162, _Mapping]] = ...) -> None: ...

class Row127(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row163
    def __init__(self, field_0: _Optional[_Union[Row163, _Mapping]] = ...) -> None: ...

class Row128(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row129(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row130(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row131(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class Row132(_message.Message):
    __slots__ = ("artifact",)
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    def __init__(self, artifact: _Optional[str] = ...) -> None: ...

class Row133(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row164
    def __init__(self, field_0: _Optional[_Union[Row164, _Mapping]] = ...) -> None: ...

class Row134(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row165
    def __init__(self, field_0: _Optional[_Union[Row165, _Mapping]] = ...) -> None: ...

class Row135(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row166
    def __init__(self, field_0: _Optional[_Union[Row166, _Mapping]] = ...) -> None: ...

class Row136(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row167
    def __init__(self, field_0: _Optional[_Union[Row167, _Mapping]] = ...) -> None: ...

class Row137(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row168
    def __init__(self, field_0: _Optional[_Union[Row168, _Mapping]] = ...) -> None: ...

class Row138(_message.Message):
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
    plot_type: Row169
    x_scale: Row170
    y_scale: Row170
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[Row38]
    x_key: str
    series_by: str
    x_expression: Row45
    y_expressions: _containers.RepeatedCompositeFieldContainer[Row45]
    series_by_expression: Row45
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: Row35
    cursor: Row171
    technique: Row172
    z_expression: Row45
    fft: Row173
    interval: Row174
    histogram: Row175
    analysis: Row176
    summary_window: Row177
    def __init__(self, plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[Row169, _Mapping]] = ..., x_scale: _Optional[_Union[Row170, _Mapping]] = ..., y_scale: _Optional[_Union[Row170, _Mapping]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[Row45, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[Row45, _Mapping]]] = ..., series_by_expression: _Optional[_Union[Row45, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[Row35, _Mapping]] = ..., cursor: _Optional[_Union[Row171, _Mapping]] = ..., technique: _Optional[_Union[Row172, _Mapping]] = ..., z_expression: _Optional[_Union[Row45, _Mapping]] = ..., fft: _Optional[_Union[Row173, _Mapping]] = ..., interval: _Optional[_Union[Row174, _Mapping]] = ..., histogram: _Optional[_Union[Row175, _Mapping]] = ..., analysis: _Optional[_Union[Row176, _Mapping]] = ..., summary_window: _Optional[_Union[Row177, _Mapping]] = ...) -> None: ...

class Row139(_message.Message):
    __slots__ = ("style", "fft", "interval", "histogram", "reference_lines", "reference_bands", "live_window_s", "x_format", "y_format", "x_axis", "x_axis_unit")
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
    style: Row178
    fft: Row173
    interval: Row174
    histogram: Row175
    reference_lines: _containers.RepeatedCompositeFieldContainer[Row179]
    reference_bands: _containers.RepeatedCompositeFieldContainer[Row180]
    live_window_s: float
    x_format: Row181
    y_format: Row181
    x_axis: Row182
    x_axis_unit: str
    def __init__(self, style: _Optional[_Union[Row178, _Mapping]] = ..., fft: _Optional[_Union[Row173, _Mapping]] = ..., interval: _Optional[_Union[Row174, _Mapping]] = ..., histogram: _Optional[_Union[Row175, _Mapping]] = ..., reference_lines: _Optional[_Iterable[_Union[Row179, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[Row180, _Mapping]]] = ..., live_window_s: _Optional[float] = ..., x_format: _Optional[_Union[Row181, _Mapping]] = ..., y_format: _Optional[_Union[Row181, _Mapping]] = ..., x_axis: _Optional[_Union[Row182, _Mapping]] = ..., x_axis_unit: _Optional[str] = ...) -> None: ...

class Row140(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row141(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row142(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row143(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row144(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row145(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row146(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row147(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row148(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row163(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row183]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row183, _Mapping]]] = ...) -> None: ...

class Row164(_message.Message):
    __slots__ = ("aggregate",)
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    aggregate: Row184
    def __init__(self, aggregate: _Optional[_Union[Row184, _Mapping]] = ...) -> None: ...

class Row165(_message.Message):
    __slots__ = ("previous_record_type",)
    PREVIOUS_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    previous_record_type: str
    def __init__(self, previous_record_type: _Optional[str] = ...) -> None: ...

class Row166(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row167(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row168(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row169(_message.Message):
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
    Unspecified: Row185
    TimeSeries: Row186
    Bar: Row187
    GroupedBar: Row188
    StackedBar: Row189
    Pie: Row190
    Donut: Row191
    Pareto: Row192
    XyScatter: Row193
    XyLine: Row194
    XyInterval: Row195
    Trajectory: Row196
    Heatmap: Row197
    XyValueHeatmap: Row198
    WifiChannelOccupancy: Row199
    FrequencySeries: Row200
    Histogram: Row201
    BoxSummary: Row202
    AggregateTable: Row203
    def __init__(self, Unspecified: _Optional[_Union[Row185, _Mapping]] = ..., TimeSeries: _Optional[_Union[Row186, _Mapping]] = ..., Bar: _Optional[_Union[Row187, _Mapping]] = ..., GroupedBar: _Optional[_Union[Row188, _Mapping]] = ..., StackedBar: _Optional[_Union[Row189, _Mapping]] = ..., Pie: _Optional[_Union[Row190, _Mapping]] = ..., Donut: _Optional[_Union[Row191, _Mapping]] = ..., Pareto: _Optional[_Union[Row192, _Mapping]] = ..., XyScatter: _Optional[_Union[Row193, _Mapping]] = ..., XyLine: _Optional[_Union[Row194, _Mapping]] = ..., XyInterval: _Optional[_Union[Row195, _Mapping]] = ..., Trajectory: _Optional[_Union[Row196, _Mapping]] = ..., Heatmap: _Optional[_Union[Row197, _Mapping]] = ..., XyValueHeatmap: _Optional[_Union[Row198, _Mapping]] = ..., WifiChannelOccupancy: _Optional[_Union[Row199, _Mapping]] = ..., FrequencySeries: _Optional[_Union[Row200, _Mapping]] = ..., Histogram: _Optional[_Union[Row201, _Mapping]] = ..., BoxSummary: _Optional[_Union[Row202, _Mapping]] = ..., AggregateTable: _Optional[_Union[Row203, _Mapping]] = ...) -> None: ...

class Row170(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Log")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    LOG_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row204
    Linear: Row205
    Log: Row206
    def __init__(self, Unspecified: _Optional[_Union[Row204, _Mapping]] = ..., Linear: _Optional[_Union[Row205, _Mapping]] = ..., Log: _Optional[_Union[Row206, _Mapping]] = ...) -> None: ...

class Row171(_message.Message):
    __slots__ = ("telemetry_streams", "record_sequence", "epoch", "selection")
    TELEMETRY_STREAMS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    telemetry_streams: _containers.RepeatedCompositeFieldContainer[Row207]
    record_sequence: int
    epoch: str
    selection: str
    def __init__(self, telemetry_streams: _Optional[_Iterable[_Union[Row207, _Mapping]]] = ..., record_sequence: _Optional[int] = ..., epoch: _Optional[str] = ..., selection: _Optional[str] = ...) -> None: ...

class Row172(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row208
    Raw: Row209
    MinMax: Row210
    Lttb: Row211
    Auto: Row212
    def __init__(self, Unspecified: _Optional[_Union[Row208, _Mapping]] = ..., Raw: _Optional[_Union[Row209, _Mapping]] = ..., MinMax: _Optional[_Union[Row210, _Mapping]] = ..., Lttb: _Optional[_Union[Row211, _Mapping]] = ..., Auto: _Optional[_Union[Row212, _Mapping]] = ...) -> None: ...

class Row173(_message.Message):
    __slots__ = ("sample_rate_hz", "window", "detrend", "level_unit", "include_dc", "min_frequency_hz", "max_frequency_hz")
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: Row213
    detrend: Row214
    level_unit: Row215
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[Row213, _Mapping]] = ..., detrend: _Optional[_Union[Row214, _Mapping]] = ..., level_unit: _Optional[_Union[Row215, _Mapping]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ...) -> None: ...

class Row174(_message.Message):
    __slots__ = ("bin_width", "sigma", "clip_to_observed")
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...

class Row175(_message.Message):
    __slots__ = ("bins", "unit")
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

class Row176(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: Row40
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row177(_message.Message):
    __slots__ = ("to",)
    FROM_FIELD_NUMBER: _ClassVar[int]
    TO_FIELD_NUMBER: _ClassVar[int]
    to: float
    def __init__(self, to: _Optional[float] = ..., **kwargs) -> None: ...

class Row178(_message.Message):
    __slots__ = ("Unspecified", "Line", "Points", "LinePoints", "Step")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINE_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    LINEPOINTS_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row216
    Line: Row217
    Points: Row218
    LinePoints: Row219
    Step: Row220
    def __init__(self, Unspecified: _Optional[_Union[Row216, _Mapping]] = ..., Line: _Optional[_Union[Row217, _Mapping]] = ..., Points: _Optional[_Union[Row218, _Mapping]] = ..., LinePoints: _Optional[_Union[Row219, _Mapping]] = ..., Step: _Optional[_Union[Row220, _Mapping]] = ...) -> None: ...

class Row179(_message.Message):
    __slots__ = ("axis", "value", "source", "source_scale", "label", "color")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_SCALE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    axis: Row221
    value: float
    source: Row45
    source_scale: float
    label: str
    color: str
    def __init__(self, axis: _Optional[_Union[Row221, _Mapping]] = ..., value: _Optional[float] = ..., source: _Optional[_Union[Row45, _Mapping]] = ..., source_scale: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class Row180(_message.Message):
    __slots__ = ("axis", "min", "max", "label", "color", "center", "tolerance")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    axis: Row221
    min: float
    max: float
    label: str
    color: str
    center: Row45
    tolerance: Row45
    def __init__(self, axis: _Optional[_Union[Row221, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[Row45, _Mapping]] = ..., tolerance: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row181(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row222
    def __init__(self, kind: _Optional[_Union[Row222, _Mapping]] = ...) -> None: ...

class Row182(_message.Message):
    __slots__ = ("Unspecified", "WallTime", "Elapsed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    WALLTIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row223
    WallTime: Row224
    Elapsed: Row225
    def __init__(self, Unspecified: _Optional[_Union[Row223, _Mapping]] = ..., WallTime: _Optional[_Union[Row224, _Mapping]] = ..., Elapsed: _Optional[_Union[Row225, _Mapping]] = ...) -> None: ...

class Row183(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row184(_message.Message):
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
    Percentile: Row226
    def __init__(self, Count: _Optional[_Union[Row2, _Mapping]] = ..., Sum: _Optional[_Union[Row2, _Mapping]] = ..., Mean: _Optional[_Union[Row2, _Mapping]] = ..., Median: _Optional[_Union[Row2, _Mapping]] = ..., Min: _Optional[_Union[Row2, _Mapping]] = ..., Max: _Optional[_Union[Row2, _Mapping]] = ..., Rms: _Optional[_Union[Row2, _Mapping]] = ..., Std: _Optional[_Union[Row2, _Mapping]] = ..., Percentile: _Optional[_Union[Row226, _Mapping]] = ...) -> None: ...

class Row185(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row186(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row187(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Hann", "Rectangular")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    HANN_FIELD_NUMBER: _ClassVar[int]
    RECTANGULAR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row227
    Hann: Row228
    Rectangular: Row229
    def __init__(self, Unspecified: _Optional[_Union[Row227, _Mapping]] = ..., Hann: _Optional[_Union[Row228, _Mapping]] = ..., Rectangular: _Optional[_Union[Row229, _Mapping]] = ...) -> None: ...

class Row214(_message.Message):
    __slots__ = ("Unspecified", "Mean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NONE_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row230
    Mean: Row232
    def __init__(self, Unspecified: _Optional[_Union[Row230, _Mapping]] = ..., Mean: _Optional[_Union[Row232, _Mapping]] = ..., **kwargs) -> None: ...

class Row215(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Db", "Dbu", "Dbm")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    DB_FIELD_NUMBER: _ClassVar[int]
    DBU_FIELD_NUMBER: _ClassVar[int]
    DBM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row233
    Linear: Row234
    Db: Row235
    Dbu: Row236
    Dbm: Row237
    def __init__(self, Unspecified: _Optional[_Union[Row233, _Mapping]] = ..., Linear: _Optional[_Union[Row234, _Mapping]] = ..., Db: _Optional[_Union[Row235, _Mapping]] = ..., Dbu: _Optional[_Union[Row236, _Mapping]] = ..., Dbm: _Optional[_Union[Row237, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "X", "Y")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row238
    X: Row239
    Y: Row240
    def __init__(self, Unspecified: _Optional[_Union[Row238, _Mapping]] = ..., X: _Optional[_Union[Row239, _Mapping]] = ..., Y: _Optional[_Union[Row240, _Mapping]] = ...) -> None: ...

class Row222(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row241
    Fixed: Row242
    Scientific: Row243
    Engineering: Row244
    SiPrefix: Row245
    AbsoluteTime: Row246
    ElapsedSinceRun: Row247
    ElapsedSinceCycle: Row248
    def __init__(self, Normal: _Optional[_Union[Row241, _Mapping]] = ..., Fixed: _Optional[_Union[Row242, _Mapping]] = ..., Scientific: _Optional[_Union[Row243, _Mapping]] = ..., Engineering: _Optional[_Union[Row244, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row245, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row246, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row247, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row248, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("rank",)
    RANK_FIELD_NUMBER: _ClassVar[int]
    rank: int
    def __init__(self, rank: _Optional[int] = ...) -> None: ...

class Row227(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row228(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row249
    def __init__(self, field_0: _Optional[_Union[Row249, _Mapping]] = ...) -> None: ...

class Row242(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row250
    def __init__(self, field_0: _Optional[_Union[Row250, _Mapping]] = ...) -> None: ...

class Row243(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row251
    def __init__(self, field_0: _Optional[_Union[Row251, _Mapping]] = ...) -> None: ...

class Row244(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row252
    def __init__(self, field_0: _Optional[_Union[Row252, _Mapping]] = ...) -> None: ...

class Row245(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row253
    def __init__(self, field_0: _Optional[_Union[Row253, _Mapping]] = ...) -> None: ...

class Row246(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row254
    def __init__(self, field_0: _Optional[_Union[Row254, _Mapping]] = ...) -> None: ...

class Row247(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row255
    def __init__(self, field_0: _Optional[_Union[Row255, _Mapping]] = ...) -> None: ...

class Row248(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row255
    def __init__(self, field_0: _Optional[_Union[Row255, _Mapping]] = ...) -> None: ...

class Row249(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row250(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row251(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row252(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row253(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row254(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row255(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row256
    def __init__(self, unit: _Optional[_Union[Row256, _Mapping]] = ...) -> None: ...

class Row256(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row257
    Seconds: Row258
    Milliseconds: Row259
    Minutes: Row260
    def __init__(self, Unspecified: _Optional[_Union[Row257, _Mapping]] = ..., Seconds: _Optional[_Union[Row258, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row259, _Mapping]] = ..., Minutes: _Optional[_Union[Row260, _Mapping]] = ...) -> None: ...

class Row257(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row258(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row259(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row260(_message.Message):
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
