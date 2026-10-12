import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NotePublication(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    NOTE_PUBLICATION_UNSPECIFIED: _ClassVar[NotePublication]
    NOTE_PUBLICATION_NO_EXISTING_DATASETS: _ClassVar[NotePublication]
    NOTE_PUBLICATION_CONFIRMED: _ClassVar[NotePublication]
    NOTE_PUBLICATION_UNCONFIRMED: _ClassVar[NotePublication]
NOTE_PUBLICATION_UNSPECIFIED: NotePublication
NOTE_PUBLICATION_NO_EXISTING_DATASETS: NotePublication
NOTE_PUBLICATION_CONFIRMED: NotePublication
NOTE_PUBLICATION_UNCONFIRMED: NotePublication

class AnnotationOwner(_message.Message):
    __slots__ = ("cycle_id", "planned_case", "execution")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    PLANNED_CASE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    planned_case: PlannedCaseNoteOwner
    execution: ExecutionNoteOwner
    def __init__(self, cycle_id: _Optional[str] = ..., planned_case: _Optional[_Union[PlannedCaseNoteOwner, _Mapping]] = ..., execution: _Optional[_Union[ExecutionNoteOwner, _Mapping]] = ...) -> None: ...

class PlannedCaseNoteOwner(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class ExecutionNoteOwner(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class AnnotationSelection(_message.Message):
    __slots__ = ("owner", "include_context")
    OWNER_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_CONTEXT_FIELD_NUMBER: _ClassVar[int]
    owner: AnnotationOwner
    include_context: bool
    def __init__(self, owner: _Optional[_Union[AnnotationOwner, _Mapping]] = ..., include_context: _Optional[bool] = ...) -> None: ...

class ListAnnotationsRequest(_message.Message):
    __slots__ = ("selection", "limit", "after_id", "cursor")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    selection: AnnotationSelection
    limit: int
    after_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, selection: _Optional[_Union[AnnotationSelection, _Mapping]] = ..., limit: _Optional[int] = ..., after_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class OperatorNote(_message.Message):
    __slots__ = ("note_id", "scope", "author", "message", "created_at")
    NOTE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AUTHOR_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    note_id: str
    scope: _common_pb2.EvidenceScope
    author: str
    message: str
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, note_id: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., author: _Optional[str] = ..., message: _Optional[str] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ListNotesResponse(_message.Message):
    __slots__ = ("notes", "cursor", "next_after_id")
    NOTES_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    notes: _containers.RepeatedCompositeFieldContainer[OperatorNote]
    cursor: _common_pb2.Cursor
    next_after_id: str
    def __init__(self, notes: _Optional[_Iterable[_Union[OperatorNote, _Mapping]]] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., next_after_id: _Optional[str] = ...) -> None: ...

class CreateNoteRequest(_message.Message):
    __slots__ = ("epoch", "mutation_id", "owner", "message")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    MUTATION_ID_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    mutation_id: str
    owner: AnnotationOwner
    message: str
    def __init__(self, epoch: _Optional[str] = ..., mutation_id: _Optional[str] = ..., owner: _Optional[_Union[AnnotationOwner, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class NoteReceipt(_message.Message):
    __slots__ = ("epoch", "mutation_id", "owner", "note", "checkpoint_targets", "publication", "unconfirmed_targets")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    MUTATION_ID_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    NOTE_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_TARGETS_FIELD_NUMBER: _ClassVar[int]
    PUBLICATION_FIELD_NUMBER: _ClassVar[int]
    UNCONFIRMED_TARGETS_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    mutation_id: str
    owner: AnnotationOwner
    note: OperatorNote
    checkpoint_targets: _containers.RepeatedScalarFieldContainer[str]
    publication: NotePublication
    unconfirmed_targets: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, epoch: _Optional[str] = ..., mutation_id: _Optional[str] = ..., owner: _Optional[_Union[AnnotationOwner, _Mapping]] = ..., note: _Optional[_Union[OperatorNote, _Mapping]] = ..., checkpoint_targets: _Optional[_Iterable[str]] = ..., publication: _Optional[_Union[NotePublication, str]] = ..., unconfirmed_targets: _Optional[_Iterable[str]] = ...) -> None: ...

class HealthItem(_message.Message):
    __slots__ = ("health_item_id", "scope", "severity", "message", "detail", "observed_at")
    HEALTH_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    health_item_id: str
    scope: _common_pb2.EvidenceScope
    severity: _common_pb2.Severity
    message: str
    detail: str
    observed_at: _timestamp_pb2.Timestamp
    def __init__(self, health_item_id: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., severity: _Optional[_Union[_common_pb2.Severity, str]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ..., observed_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ListHealthResponse(_message.Message):
    __slots__ = ("items", "cursor", "next_after_id")
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[HealthItem]
    cursor: _common_pb2.Cursor
    next_after_id: str
    def __init__(self, items: _Optional[_Iterable[_Union[HealthItem, _Mapping]]] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., next_after_id: _Optional[str] = ...) -> None: ...
