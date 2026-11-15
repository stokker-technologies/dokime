import datetime

from dokime._wire.model.v1 import evidence_pb2 as _evidence_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NotesGetRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class NotesGetResponse(_message.Message):
    __slots__ = ("value", "revision", "note", "resolved_at")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    NOTE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: NoteWritable
    revision: bytes
    note: OperatorNote
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[NoteWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., note: _Optional[_Union[OperatorNote, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class NotesSetRequest(_message.Message):
    __slots__ = ("name", "value", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: NoteWritable
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[NoteWritable, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class NotesSetResponse(_message.Message):
    __slots__ = ("value", "revision", "receipt")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: NoteWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[NoteWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class NotesListRequest(_message.Message):
    __slots__ = ("selection", "at", "page")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: AnnotationSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[AnnotationSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class NotesListResponse(_message.Message):
    __slots__ = ("notes", "resolved_at", "next_page_token")
    NOTES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    notes: _containers.RepeatedCompositeFieldContainer[OperatorNote]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, notes: _Optional[_Iterable[_Union[OperatorNote, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

class NotesWatchRequest(_message.Message):
    __slots__ = ("selection", "at", "page")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: AnnotationSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[AnnotationSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class NotesWatchResponse(_message.Message):
    __slots__ = ("notes", "resolved_at", "opening", "resync_required")
    NOTES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    notes: _containers.RepeatedCompositeFieldContainer[OperatorNote]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, notes: _Optional[_Iterable[_Union[OperatorNote, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class NotesCreateRequest(_message.Message):
    __slots__ = ("owner", "value", "request_id")
    OWNER_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    owner: AnnotationOwner
    value: NoteWritable
    request_id: str
    def __init__(self, owner: _Optional[_Union[AnnotationOwner, _Mapping]] = ..., value: _Optional[_Union[NoteWritable, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

class NotesCreateResponse(_message.Message):
    __slots__ = ("note", "revision", "receipt")
    NOTE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    note: OperatorNote
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, note: _Optional[_Union[OperatorNote, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class OperatorNote(_message.Message):
    __slots__ = ("note_id", "scope", "author", "message", "created_at")
    NOTE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AUTHOR_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    note_id: str
    scope: _evidence_pb2.EvidenceScope
    author: str
    message: str
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, note_id: _Optional[str] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., author: _Optional[str] = ..., message: _Optional[str] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class NoteWritable(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class AnnotationOwner(_message.Message):
    __slots__ = ("owner",)
    OWNER_FIELD_NUMBER: _ClassVar[int]
    owner: AnnotationOwnerOwner
    def __init__(self, owner: _Optional[_Union[AnnotationOwnerOwner, _Mapping]] = ...) -> None: ...

class AnnotationOwnerOwner(_message.Message):
    __slots__ = ("cycle_id", "planned_case", "execution")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    PLANNED_CASE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    planned_case: PlannedCaseNoteOwner
    execution: ExecutionNoteOwner
    def __init__(self, cycle_id: _Optional[str] = ..., planned_case: _Optional[_Union[PlannedCaseNoteOwner, _Mapping]] = ..., execution: _Optional[_Union[ExecutionNoteOwner, _Mapping]] = ...) -> None: ...

class AnnotationSelection(_message.Message):
    __slots__ = ("owner", "include_context")
    OWNER_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_CONTEXT_FIELD_NUMBER: _ClassVar[int]
    owner: AnnotationOwner
    include_context: bool
    def __init__(self, owner: _Optional[_Union[AnnotationOwner, _Mapping]] = ..., include_context: _Optional[bool] = ...) -> None: ...

class ExecutionNoteOwner(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class PlannedCaseNoteOwner(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...
