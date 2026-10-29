from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("identifiers",)
    IDENTIFIERS_FIELD_NUMBER: _ClassVar[int]
    identifiers: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, identifiers: _Optional[_Iterable[str]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("sources",)
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    sources: _containers.RepeatedCompositeFieldContainer[Row3]
    def __init__(self, sources: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("identifier", "adapter", "priority", "enabled")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    ADAPTER_FIELD_NUMBER: _ClassVar[int]
    PRIORITY_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    adapter: str
    priority: int
    enabled: bool
    def __init__(self, identifier: _Optional[str] = ..., adapter: _Optional[str] = ..., priority: _Optional[int] = ..., enabled: _Optional[bool] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
