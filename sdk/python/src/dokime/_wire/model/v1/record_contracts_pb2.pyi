from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RecordContracts(_message.Message):
    __slots__ = ("records",)
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[RecordContract]
    def __init__(self, records: _Optional[_Iterable[_Union[RecordContract, _Mapping]]] = ...) -> None: ...

class RecordContract(_message.Message):
    __slots__ = ("metadata", "message")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    message: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Listeners(_message.Message):
    __slots__ = ("listeners",)
    LISTENERS_FIELD_NUMBER: _ClassVar[int]
    listeners: _containers.RepeatedCompositeFieldContainer[RecordListener]
    def __init__(self, listeners: _Optional[_Iterable[_Union[RecordListener, _Mapping]]] = ...) -> None: ...

class RecordListener(_message.Message):
    __slots__ = ("operation", "record")
    OPERATION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    operation: str
    record: str
    def __init__(self, operation: _Optional[str] = ..., record: _Optional[str] = ...) -> None: ...
