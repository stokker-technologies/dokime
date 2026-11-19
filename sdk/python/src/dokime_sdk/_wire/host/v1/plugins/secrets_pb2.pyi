from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import secrets_pb2 as _secrets_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class SecretsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SecretsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SECRETS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    secrets: _containers.RepeatedCompositeFieldContainer[SecretRequirement]
    resolved_at: _position_pb2.ReadCut
    def __init__(self, secrets: _Optional[_Iterable[_Union[SecretRequirement, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SecretRequirement(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    DECLARATION_FIELD_NUMBER: _ClassVar[int]
    AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    declaration: _secrets_pb2.SecretDeclaration
    available: bool
    def __init__(self, declaration: _Optional[_Union[_secrets_pb2.SecretDeclaration, _Mapping]] = ..., available: _Optional[bool] = ...) -> None: ...
