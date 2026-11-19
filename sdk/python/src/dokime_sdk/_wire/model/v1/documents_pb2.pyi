from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import expressions_pb2 as _expressions_pb2
from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import plots_pb2 as _plots_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class EvidenceCard(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EVIDENCE_CARD_UNSPECIFIED: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_STEPS: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_CRITERIA: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_TRACEABILITY: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_NOTES: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_RECORDS: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_ARTIFACTS: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_MEDIA: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_HEALTH: _ClassVar[EvidenceCard]
    EVIDENCE_CARD_STREAM_CONTEXT: _ClassVar[EvidenceCard]
EVIDENCE_CARD_UNSPECIFIED: EvidenceCard
EVIDENCE_CARD_STEPS: EvidenceCard
EVIDENCE_CARD_CRITERIA: EvidenceCard
EVIDENCE_CARD_TRACEABILITY: EvidenceCard
EVIDENCE_CARD_NOTES: EvidenceCard
EVIDENCE_CARD_RECORDS: EvidenceCard
EVIDENCE_CARD_ARTIFACTS: EvidenceCard
EVIDENCE_CARD_MEDIA: EvidenceCard
EVIDENCE_CARD_HEALTH: EvidenceCard
EVIDENCE_CARD_STREAM_CONTEXT: EvidenceCard

@final
class Document(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    sections: _containers.RepeatedCompositeFieldContainer[Section]
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., sections: _Optional[_Iterable[_Union[Section, _Mapping]]] = ...) -> None: ...

@final
class Section(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    cards: _containers.RepeatedCompositeFieldContainer[Card]
    sections: _containers.RepeatedCompositeFieldContainer[Section]
    repeat: SectionRepeat
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., cards: _Optional[_Iterable[_Union[Card, _Mapping]]] = ..., sections: _Optional[_Iterable[_Union[Section, _Mapping]]] = ..., repeat: _Optional[_Union[SectionRepeat, _Mapping]] = ...) -> None: ...

@final
class SectionRepeat(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    value: _expressions_pb2.DataExpression
    label_template: str
    order_by: _expressions_pb2.DataExpression
    def __init__(self, value: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., label_template: _Optional[str] = ..., order_by: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ...) -> None: ...

@final
class Card(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    markdown: str
    plot: _plots_pb2.Plot
    table: Table
    evidence: EvidenceCard
    summary: Summary
    layout: CardLayout
    filters: _containers.RepeatedCompositeFieldContainer[_expressions_pb2.DocumentFilter]
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., markdown: _Optional[str] = ..., plot: _Optional[_Union[_plots_pb2.Plot, _Mapping]] = ..., table: _Optional[_Union[Table, _Mapping]] = ..., evidence: _Optional[_Union[EvidenceCard, str]] = ..., summary: _Optional[_Union[Summary, _Mapping]] = ..., layout: _Optional[_Union[CardLayout, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[_expressions_pb2.DocumentFilter, _Mapping]]] = ...) -> None: ...

@final
class CardLayout(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    columns: int
    rows: int
    order: int
    def __init__(self, columns: _Optional[int] = ..., rows: _Optional[int] = ..., order: _Optional[int] = ...) -> None: ...

@final
class Table(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    rows: _containers.RepeatedCompositeFieldContainer[TableRow]
    def __init__(self, rows: _Optional[_Iterable[_Union[TableRow, _Mapping]]] = ...) -> None: ...

@final
class TableRow(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    @final
    class CellsEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
        __slots__: tuple[str, ...]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    CELLS_FIELD_NUMBER: _ClassVar[int]
    cells: _containers.MessageMap[str, _scalar_pb2.Value]
    def __init__(self, cells: _Optional[_Mapping[str, _scalar_pb2.Value]] = ...) -> None: ...

@final
class Summary(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    TILES_FIELD_NUMBER: _ClassVar[int]
    tiles: _containers.RepeatedCompositeFieldContainer[SummaryTile]
    def __init__(self, tiles: _Optional[_Iterable[_Union[SummaryTile, _Mapping]]] = ...) -> None: ...

@final
class SummaryTile(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    value: _expressions_pb2.DataExpression
    unit: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_expressions_pb2.DataExpression, _Mapping]] = ..., unit: _Optional[str] = ...) -> None: ...
