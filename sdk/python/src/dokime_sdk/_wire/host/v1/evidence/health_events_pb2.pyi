from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._wire.host.v1.evidence import notes_pb2 as _notes_pb2
from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class HealthEventsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: _notes_pb2.AnnotationSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[_notes_pb2.AnnotationSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class HealthEventsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    events: _containers.RepeatedCompositeFieldContainer[HealthItem]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, events: _Optional[_Iterable[_Union[HealthItem, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

@final
class HealthEventsWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: _notes_pb2.AnnotationSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[_notes_pb2.AnnotationSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class HealthEventsWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    events: _containers.RepeatedCompositeFieldContainer[HealthItem]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, events: _Optional[_Iterable[_Union[HealthItem, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class HealthItem(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    HEALTH_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    health_item_id: str
    scope: _evidence_pb2.EvidenceScope
    severity: _evidence_pb2.Severity
    message: str
    detail: str
    observed_at: _timestamp_pb2.Timestamp
    def __init__(self, health_item_id: _Optional[str] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., severity: _Optional[_Union[_evidence_pb2.Severity, str]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ..., observed_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...
