from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class RecordContracts(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[RecordContract]
    def __init__(self, records: _Optional[_Iterable[_Union[RecordContract, _Mapping]]] = ...) -> None: ...

@final
class RecordContract(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    METADATA_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    message: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

@final
class Listeners(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    LISTENERS_FIELD_NUMBER: _ClassVar[int]
    listeners: _containers.RepeatedCompositeFieldContainer[RecordListener]
    def __init__(self, listeners: _Optional[_Iterable[_Union[RecordListener, _Mapping]]] = ...) -> None: ...

@final
class RecordListener(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    OPERATION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    operation: str
    record: str
    def __init__(self, operation: _Optional[str] = ..., record: _Optional[str] = ...) -> None: ...
