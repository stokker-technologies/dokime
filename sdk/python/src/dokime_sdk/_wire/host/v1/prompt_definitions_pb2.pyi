from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import prompts_pb2 as _prompts_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class PromptDefinitionsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    NAME_FIELD_NUMBER: _ClassVar[int]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    parent: str
    schema_version: int
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., parent: _Optional[str] = ..., schema_version: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PromptDefinitionsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    VALUE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: _prompts_pb2.Prompt
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[_prompts_pb2.Prompt, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PromptDefinitionsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    parent: str
    at: _position_pb2.ReadCut
    def __init__(self, parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PromptDefinitionsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    VALUES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_prompts_pb2.Prompt]
    resolved_at: _position_pb2.ReadCut
    def __init__(self, values: _Optional[_Iterable[_Union[_prompts_pb2.Prompt, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...
