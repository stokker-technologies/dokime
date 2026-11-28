from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from harmos._generated.proto.host.v1 import status_pb2 as _status_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class StatusGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    BUILD_FIELD_NUMBER: _ClassVar[int]
    status: _status_pb2.HostStatus
    resolved_at: _position_pb2.Position
    build: HostBuild
    def __init__(self, status: _Optional[_Union[_status_pb2.HostStatus, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., build: _Optional[_Union[HostBuild, _Mapping]] = ...) -> None: ...

@final
class StatusWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    BUILD_FIELD_NUMBER: _ClassVar[int]
    status: _status_pb2.HostStatus
    resolved_at: _position_pb2.Position
    build: HostBuild
    def __init__(self, status: _Optional[_Union[_status_pb2.HostStatus, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., build: _Optional[_Union[HostBuild, _Mapping]] = ...) -> None: ...

@final
class HostBuild(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SOURCE_REVISION_FIELD_NUMBER: _ClassVar[int]
    CHANNEL_FIELD_NUMBER: _ClassVar[int]
    source_revision: str
    channel: str
    def __init__(self, source_revision: _Optional[str] = ..., channel: _Optional[str] = ...) -> None: ...
