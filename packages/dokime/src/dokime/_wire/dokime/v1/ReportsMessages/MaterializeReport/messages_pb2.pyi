from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("subject", "at")
    SUBJECT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    subject: Row3
    at: Row4
    def __init__(self, subject: _Optional[_Union[Row3, _Mapping]] = ..., at: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("report", "materialized_input")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    MATERIALIZED_INPUT_FIELD_NUMBER: _ClassVar[int]
    report: Row5
    materialized_input: Row5
    def __init__(self, report: _Optional[_Union[Row5, _Mapping]] = ..., materialized_input: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Execution", "Cycle")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    Execution: Row6
    Cycle: Row7
    def __init__(self, Execution: _Optional[_Union[Row6, _Mapping]] = ..., Cycle: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row8
    sequence: int
    def __init__(self, history: _Optional[_Union[Row8, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("position", "metadata", "length", "sha256")
    POSITION_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    LENGTH_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    position: Row4
    metadata: Row9
    length: int
    sha256: bytes
    def __init__(self, position: _Optional[_Union[Row4, _Mapping]] = ..., metadata: _Optional[_Union[Row9, _Mapping]] = ..., length: _Optional[int] = ..., sha256: _Optional[bytes] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("id", "scope", "media_type", "expected_length", "expected_sha256")
    ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    MEDIA_TYPE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_LENGTH_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_SHA256_FIELD_NUMBER: _ClassVar[int]
    id: str
    scope: str
    media_type: str
    expected_length: int
    expected_sha256: bytes
    def __init__(self, id: _Optional[str] = ..., scope: _Optional[str] = ..., media_type: _Optional[str] = ..., expected_length: _Optional[int] = ..., expected_sha256: _Optional[bytes] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
