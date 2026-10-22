from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "relative_path")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    relative_path: str
    def __init__(self, execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row4
    def __init__(self, payload: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Metadata", "Chunk", "Complete")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    Metadata: Row5
    Chunk: Row6
    Complete: Row7
    def __init__(self, Metadata: _Optional[_Union[Row5, _Mapping]] = ..., Chunk: _Optional[_Union[Row6, _Mapping]] = ..., Complete: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "observed_size_bytes")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: Row11
    state: Row12
    observed_size_bytes: int
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row11, _Mapping]] = ..., state: _Optional[_Union[Row12, _Mapping]] = ..., observed_size_bytes: _Optional[int] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row13
    Text: Row14
    Binary: Row15
    Media: Row16
    Unknown: Row17
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Text: _Optional[_Union[Row14, _Mapping]] = ..., Binary: _Optional[_Union[Row15, _Mapping]] = ..., Media: _Optional[_Union[Row16, _Mapping]] = ..., Unknown: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Unspecified", "Streaming", "Finalized", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    FINALIZED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row18
    Streaming: Row19
    Finalized: Row20
    Interrupted: Row21
    def __init__(self, Unspecified: _Optional[_Union[Row18, _Mapping]] = ..., Streaming: _Optional[_Union[Row19, _Mapping]] = ..., Finalized: _Optional[_Union[Row20, _Mapping]] = ..., Interrupted: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

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

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
