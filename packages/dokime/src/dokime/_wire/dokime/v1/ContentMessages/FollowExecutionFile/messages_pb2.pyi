from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "relative_path", "identity", "offset", "max_bytes")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    relative_path: str
    identity: Row4
    offset: int
    max_bytes: int
    def __init__(self, execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ..., identity: _Optional[_Union[Row4, _Mapping]] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row5
    def __init__(self, payload: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("history", "file_id", "execution_id", "relative_path")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    FILE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    history: Row6
    file_id: str
    execution_id: str
    relative_path: str
    def __init__(self, history: _Optional[_Union[Row6, _Mapping]] = ..., file_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Metadata", "Chunk", "Terminal")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    TERMINAL_FIELD_NUMBER: _ClassVar[int]
    Metadata: Row7
    Chunk: Row8
    Terminal: Row9
    def __init__(self, Metadata: _Optional[_Union[Row7, _Mapping]] = ..., Chunk: _Optional[_Union[Row8, _Mapping]] = ..., Terminal: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row11
    def __init__(self, field_0: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row12
    def __init__(self, field_0: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("identity", "content_type", "kind", "phase", "available_length", "offset")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    AVAILABLE_LENGTH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    identity: Row4
    content_type: str
    kind: Row13
    phase: Row14
    available_length: int
    offset: int
    def __init__(self, identity: _Optional[_Union[Row4, _Mapping]] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row13, _Mapping]] = ..., phase: _Optional[_Union[Row14, _Mapping]] = ..., available_length: _Optional[int] = ..., offset: _Optional[int] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("phase", "length", "sealed")
    PHASE_FIELD_NUMBER: _ClassVar[int]
    LENGTH_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    phase: Row14
    length: int
    sealed: Row15
    def __init__(self, phase: _Optional[_Union[Row14, _Mapping]] = ..., length: _Optional[int] = ..., sealed: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row16
    Text: Row17
    Binary: Row18
    Media: Row19
    Unknown: Row20
    def __init__(self, Unspecified: _Optional[_Union[Row16, _Mapping]] = ..., Text: _Optional[_Union[Row17, _Mapping]] = ..., Binary: _Optional[_Union[Row18, _Mapping]] = ..., Media: _Optional[_Union[Row19, _Mapping]] = ..., Unknown: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("Unspecified", "Producing", "Finished", "Abandoned")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PRODUCING_FIELD_NUMBER: _ClassVar[int]
    FINISHED_FIELD_NUMBER: _ClassVar[int]
    ABANDONED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row21
    Producing: Row22
    Finished: Row23
    Abandoned: Row24
    def __init__(self, Unspecified: _Optional[_Union[Row21, _Mapping]] = ..., Producing: _Optional[_Union[Row22, _Mapping]] = ..., Finished: _Optional[_Union[Row23, _Mapping]] = ..., Abandoned: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("identity", "position", "length", "sha256")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    LENGTH_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    identity: Row4
    position: Row25
    length: int
    sha256: str
    def __init__(self, identity: _Optional[_Union[Row4, _Mapping]] = ..., position: _Optional[_Union[Row25, _Mapping]] = ..., length: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

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

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row25(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row6
    sequence: int
    def __init__(self, history: _Optional[_Union[Row6, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
