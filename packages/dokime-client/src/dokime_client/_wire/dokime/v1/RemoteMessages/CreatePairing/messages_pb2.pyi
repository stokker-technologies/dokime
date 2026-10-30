from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("label",)
    LABEL_FIELD_NUMBER: _ClassVar[int]
    label: str
    def __init__(self, label: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("client", "pairing")
    CLIENT_FIELD_NUMBER: _ClassVar[int]
    PAIRING_FIELD_NUMBER: _ClassVar[int]
    client: Row3
    pairing: Row4
    def __init__(self, client: _Optional[_Union[Row3, _Mapping]] = ..., pairing: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("identifier", "label", "created_at", "last_seen_at")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    LAST_SEEN_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    created_at: int
    last_seen_at: int
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., created_at: _Optional[int] = ..., last_seen_at: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("version", "endpoints", "backend_identifier", "label", "ca_certificate_pem", "access_key")
    VERSION_FIELD_NUMBER: _ClassVar[int]
    ENDPOINTS_FIELD_NUMBER: _ClassVar[int]
    BACKEND_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    CA_CERTIFICATE_PEM_FIELD_NUMBER: _ClassVar[int]
    ACCESS_KEY_FIELD_NUMBER: _ClassVar[int]
    version: int
    endpoints: _containers.RepeatedScalarFieldContainer[str]
    backend_identifier: str
    label: str
    ca_certificate_pem: str
    access_key: str
    def __init__(self, version: _Optional[int] = ..., endpoints: _Optional[_Iterable[str]] = ..., backend_identifier: _Optional[str] = ..., label: _Optional[str] = ..., ca_certificate_pem: _Optional[str] = ..., access_key: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
