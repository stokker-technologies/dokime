from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("backend_id", "epoch", "api", "product_version", "source_revision", "sdk_revision", "started_at", "display_name")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    API_FIELD_NUMBER: _ClassVar[int]
    PRODUCT_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_REVISION_FIELD_NUMBER: _ClassVar[int]
    SDK_REVISION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    api: Row3
    product_version: str
    source_revision: str
    sdk_revision: str
    started_at: Row4
    display_name: str
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., api: _Optional[_Union[Row3, _Mapping]] = ..., product_version: _Optional[str] = ..., source_revision: _Optional[str] = ..., sdk_revision: _Optional[str] = ..., started_at: _Optional[_Union[Row4, _Mapping]] = ..., display_name: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("package", "version", "descriptor_sha256")
    PACKAGE_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTOR_SHA256_FIELD_NUMBER: _ClassVar[int]
    package: str
    version: str
    descriptor_sha256: bytes
    def __init__(self, package: _Optional[str] = ..., version: _Optional[str] = ..., descriptor_sha256: _Optional[bytes] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
