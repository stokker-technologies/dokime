from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cursor", "limit")
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    cursor: str
    limit: int
    def __init__(self, cursor: _Optional[str] = ..., limit: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("sessions", "next_cursor")
    SESSIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    sessions: _containers.RepeatedCompositeFieldContainer[Row3]
    next_cursor: str
    def __init__(self, sessions: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ..., next_cursor: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("id", "peer_address", "principal", "connected_at", "last_seen_at", "disconnected_at", "calls", "outcome", "rejection")
    ID_FIELD_NUMBER: _ClassVar[int]
    PEER_ADDRESS_FIELD_NUMBER: _ClassVar[int]
    PRINCIPAL_FIELD_NUMBER: _ClassVar[int]
    CONNECTED_AT_FIELD_NUMBER: _ClassVar[int]
    LAST_SEEN_AT_FIELD_NUMBER: _ClassVar[int]
    DISCONNECTED_AT_FIELD_NUMBER: _ClassVar[int]
    CALLS_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    REJECTION_FIELD_NUMBER: _ClassVar[int]
    id: str
    peer_address: str
    principal: str
    connected_at: int
    last_seen_at: int
    disconnected_at: int
    calls: int
    outcome: Row4
    rejection: str
    def __init__(self, id: _Optional[str] = ..., peer_address: _Optional[str] = ..., principal: _Optional[str] = ..., connected_at: _Optional[int] = ..., last_seen_at: _Optional[int] = ..., disconnected_at: _Optional[int] = ..., calls: _Optional[int] = ..., outcome: _Optional[_Union[Row4, _Mapping]] = ..., rejection: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Admitted", "Rejected")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row5
    Admitted: Row6
    Rejected: Row7
    def __init__(self, Unspecified: _Optional[_Union[Row5, _Mapping]] = ..., Admitted: _Optional[_Union[Row6, _Mapping]] = ..., Rejected: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row6(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
