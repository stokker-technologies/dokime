from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from google.protobuf import descriptor_pb2 as _descriptor_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class SchemasGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    page: _page_pb2.PageRequest
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class SchemasGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    DIGEST_FIELD_NUMBER: _ClassVar[int]
    TYPES_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    digest: bytes
    types: _descriptor_pb2.FileDescriptorSet
    records: _containers.RepeatedCompositeFieldContainer[_record_schemas_pb2.RecordSchema]
    streams: _containers.RepeatedCompositeFieldContainer[_record_schemas_pb2.TelemetrySchema]
    resolved_at: _position_pb2.Position
    next_page_token: str
    def __init__(self, digest: _Optional[bytes] = ..., types: _Optional[_Union[_descriptor_pb2.FileDescriptorSet, _Mapping]] = ..., records: _Optional[_Iterable[_Union[_record_schemas_pb2.RecordSchema, _Mapping]]] = ..., streams: _Optional[_Iterable[_Union[_record_schemas_pb2.TelemetrySchema, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...
