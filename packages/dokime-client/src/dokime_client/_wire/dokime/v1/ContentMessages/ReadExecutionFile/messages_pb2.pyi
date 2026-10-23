from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "relative_path", "offset", "max_bytes")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    relative_path: str
    offset: int
    max_bytes: int
    def __init__(self, execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "offset", "observed_size_bytes", "data")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: Row3
    state: Row4
    offset: int
    observed_size_bytes: int
    data: bytes
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row3, _Mapping]] = ..., state: _Optional[_Union[Row4, _Mapping]] = ..., offset: _Optional[int] = ..., observed_size_bytes: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row5
    Text: Row6
    Binary: Row7
    Media: Row8
    Unknown: Row9
    def __init__(self, Unspecified: _Optional[_Union[Row5, _Mapping]] = ..., Text: _Optional[_Union[Row6, _Mapping]] = ..., Binary: _Optional[_Union[Row7, _Mapping]] = ..., Media: _Optional[_Union[Row8, _Mapping]] = ..., Unknown: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Streaming", "Finalized", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    FINALIZED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Streaming: Row11
    Finalized: Row12
    Interrupted: Row13
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Streaming: _Optional[_Union[Row11, _Mapping]] = ..., Finalized: _Optional[_Union[Row12, _Mapping]] = ..., Interrupted: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row6(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
