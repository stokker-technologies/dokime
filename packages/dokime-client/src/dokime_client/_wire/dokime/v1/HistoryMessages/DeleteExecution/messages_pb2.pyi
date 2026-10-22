from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("checkpoint", "request_identifier")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    request_identifier: str
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., request_identifier: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("request_identifier", "execution_identifier", "local_complete")
    REQUEST_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LOCAL_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    request_identifier: str
    execution_identifier: str
    local_complete: bool
    def __init__(self, request_identifier: _Optional[str] = ..., execution_identifier: _Optional[str] = ..., local_complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row4
    version: bytes
    evidence_through: Row4
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row4, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row5
    sequence: int
    def __init__(self, history: _Optional[_Union[Row5, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
