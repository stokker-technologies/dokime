from dokime._wire.model.v1 import secrets_pb2 as _secrets_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SecretsListRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsListResponse(_message.Message):
    __slots__ = ("bindings", "revision", "resolved_at")
    BINDINGS_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    bindings: _containers.RepeatedCompositeFieldContainer[SecretBinding]
    revision: bytes
    resolved_at: _position_pb2.ReadCut
    def __init__(self, bindings: _Optional[_Iterable[_Union[SecretBinding, _Mapping]]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsSetRequest(_message.Message):
    __slots__ = ("name", "bindings", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    BINDINGS_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    bindings: _containers.RepeatedCompositeFieldContainer[SecretBindingValue]
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., bindings: _Optional[_Iterable[_Union[SecretBindingValue, _Mapping]]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class SecretsSetResponse(_message.Message):
    __slots__ = ("bindings", "revision", "receipt")
    BINDINGS_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    bindings: _containers.RepeatedCompositeFieldContainer[SecretBinding]
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, bindings: _Optional[_Iterable[_Union[SecretBinding, _Mapping]]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class SecretsDeleteRequest(_message.Message):
    __slots__ = ("name", "identifiers", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIERS_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    identifiers: _containers.RepeatedScalarFieldContainer[str]
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., identifiers: _Optional[_Iterable[str]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class SecretsDeleteResponse(_message.Message):
    __slots__ = ("receipt",)
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class SecretBindingValue(_message.Message):
    __slots__ = ("identifier", "secret")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    SECRET_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    secret: str
    def __init__(self, identifier: _Optional[str] = ..., secret: _Optional[str] = ...) -> None: ...

class SecretBinding(_message.Message):
    __slots__ = ("declaration", "secret", "available")
    DECLARATION_FIELD_NUMBER: _ClassVar[int]
    SECRET_FIELD_NUMBER: _ClassVar[int]
    AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    declaration: _secrets_pb2.SecretDeclaration
    secret: str
    available: bool
    def __init__(self, declaration: _Optional[_Union[_secrets_pb2.SecretDeclaration, _Mapping]] = ..., secret: _Optional[str] = ..., available: _Optional[bool] = ...) -> None: ...
