from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("checkpoint", "relative_path", "offset", "max_bytes")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    relative_path: str
    offset: int
    max_bytes: int
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., relative_path: _Optional[str] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("checkpoint", "content")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    content: Row4
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., content: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row5
    version: bytes
    evidence_through: Row5
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row5, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
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
    kind: Row6
    state: Row7
    offset: int
    observed_size_bytes: int
    data: bytes
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row6, _Mapping]] = ..., state: _Optional[_Union[Row7, _Mapping]] = ..., offset: _Optional[int] = ..., observed_size_bytes: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row8
    sequence: int
    def __init__(self, history: _Optional[_Union[Row8, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row9
    Text: Row10
    Binary: Row11
    Media: Row12
    Unknown: Row13
    def __init__(self, Unspecified: _Optional[_Union[Row9, _Mapping]] = ..., Text: _Optional[_Union[Row10, _Mapping]] = ..., Binary: _Optional[_Union[Row11, _Mapping]] = ..., Media: _Optional[_Union[Row12, _Mapping]] = ..., Unknown: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "Streaming", "Finalized", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    FINALIZED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row14
    Streaming: Row15
    Finalized: Row16
    Interrupted: Row17
    def __init__(self, Unspecified: _Optional[_Union[Row14, _Mapping]] = ..., Streaming: _Optional[_Union[Row15, _Mapping]] = ..., Finalized: _Optional[_Union[Row16, _Mapping]] = ..., Interrupted: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

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

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
