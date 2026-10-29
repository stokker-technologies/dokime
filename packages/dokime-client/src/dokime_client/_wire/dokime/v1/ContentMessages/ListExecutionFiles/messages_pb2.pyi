from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "limit", "after_path", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_PATH_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    limit: int
    after_path: str
    cursor: Row3
    def __init__(self, execution_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_path: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("files", "next_after_path", "cursor")
    FILES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_PATH_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    files: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after_path: str
    cursor: Row3
    def __init__(self, files: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after_path: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "execution_id", "metadata_sha256")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_SHA256_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    execution_id: str
    metadata_sha256: bytes
    def __init__(self, epoch: _Optional[str] = ..., execution_id: _Optional[str] = ..., metadata_sha256: _Optional[bytes] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "size_bytes", "created_at")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: Row5
    state: Row6
    size_bytes: int
    created_at: Row7
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row5, _Mapping]] = ..., state: _Optional[_Union[Row6, _Mapping]] = ..., size_bytes: _Optional[int] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row8
    Text: Row9
    Binary: Row10
    Media: Row11
    Unknown: Row12
    def __init__(self, Unspecified: _Optional[_Union[Row8, _Mapping]] = ..., Text: _Optional[_Union[Row9, _Mapping]] = ..., Binary: _Optional[_Union[Row10, _Mapping]] = ..., Media: _Optional[_Union[Row11, _Mapping]] = ..., Unknown: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Streaming", "Finalized", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    FINALIZED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row13
    Streaming: Row14
    Finalized: Row15
    Interrupted: Row16
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Streaming: _Optional[_Union[Row14, _Mapping]] = ..., Finalized: _Optional[_Union[Row15, _Mapping]] = ..., Interrupted: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row12(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
