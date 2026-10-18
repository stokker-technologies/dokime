from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("limit", "after_prompt_type", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_prompt_type: str
    cursor: Row3
    def __init__(self, limit: _Optional[int] = ..., after_prompt_type: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("definitions", "next_after_prompt_type", "cursor")
    DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    definitions: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after_prompt_type: str
    cursor: Row3
    def __init__(self, definitions: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after_prompt_type: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("prompt_type", "schema_version", "title")
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    schema_version: int
    title: str
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., title: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
