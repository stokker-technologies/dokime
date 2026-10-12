from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ExecutionCheckpoint(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: StorageCursor
    version: bytes
    evidence_through: StorageCursor
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[StorageCursor, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[StorageCursor, _Mapping]] = ...) -> None: ...

class StorageCursor(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: StoreHistory
    sequence: int
    def __init__(self, history: _Optional[_Union[StoreHistory, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class StoreHistory(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...
