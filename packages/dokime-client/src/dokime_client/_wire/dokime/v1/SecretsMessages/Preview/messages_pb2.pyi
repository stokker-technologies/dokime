from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    def __init__(self, data: _Optional[bytes] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("host_id", "host_label", "generation", "entries")
    HOST_ID_FIELD_NUMBER: _ClassVar[int]
    HOST_LABEL_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    host_id: str
    host_label: str
    generation: int
    entries: _containers.RepeatedCompositeFieldContainer[Row3]
    def __init__(self, host_id: _Optional[str] = ..., host_label: _Optional[str] = ..., generation: _Optional[int] = ..., entries: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("metadata", "exists")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    metadata: Row4
    exists: bool
    def __init__(self, metadata: _Optional[_Union[Row4, _Mapping]] = ..., exists: _Optional[bool] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("identifier", "label", "description")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
