from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("settings", "listening_address", "certificate_pem", "key_configured", "error")
    SETTINGS_FIELD_NUMBER: _ClassVar[int]
    LISTENING_ADDRESS_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_PEM_FIELD_NUMBER: _ClassVar[int]
    KEY_CONFIGURED_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    settings: Row3
    listening_address: str
    certificate_pem: str
    key_configured: bool
    error: str
    def __init__(self, settings: _Optional[_Union[Row3, _Mapping]] = ..., listening_address: _Optional[str] = ..., certificate_pem: _Optional[str] = ..., key_configured: _Optional[bool] = ..., error: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("enabled", "address", "tls_enabled", "authentication_enabled", "certificate_names")
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    ADDRESS_FIELD_NUMBER: _ClassVar[int]
    TLS_ENABLED_FIELD_NUMBER: _ClassVar[int]
    AUTHENTICATION_ENABLED_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_NAMES_FIELD_NUMBER: _ClassVar[int]
    enabled: bool
    address: str
    tls_enabled: bool
    authentication_enabled: bool
    certificate_names: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, enabled: _Optional[bool] = ..., address: _Optional[str] = ..., tls_enabled: _Optional[bool] = ..., authentication_enabled: _Optional[bool] = ..., certificate_names: _Optional[_Iterable[str]] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
