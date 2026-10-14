from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("backend_id", "epoch", "methods", "limits")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    METHODS_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    methods: _containers.RepeatedScalarFieldContainer[str]
    limits: Row3
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., methods: _Optional[_Iterable[str]] = ..., limits: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("max_request_bytes", "max_response_bytes", "max_content_bytes", "max_chunk_bytes", "max_page_items", "max_stream_buffer_items")
    MAX_REQUEST_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_RESPONSE_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_CONTENT_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_PAGE_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_STREAM_BUFFER_ITEMS_FIELD_NUMBER: _ClassVar[int]
    max_request_bytes: int
    max_response_bytes: int
    max_content_bytes: int
    max_chunk_bytes: int
    max_page_items: int
    max_stream_buffer_items: int
    def __init__(self, max_request_bytes: _Optional[int] = ..., max_response_bytes: _Optional[int] = ..., max_content_bytes: _Optional[int] = ..., max_chunk_bytes: _Optional[int] = ..., max_page_items: _Optional[int] = ..., max_stream_buffer_items: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
