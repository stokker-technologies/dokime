from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("install_id", "enabled")
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    enabled: bool
    def __init__(self, install_id: _Optional[str] = ..., enabled: _Optional[bool] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("install_id", "sha256", "installed_at", "state", "last_boot", "metadata", "error", "active", "restart_required", "boot_error")
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_AT_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    LAST_BOOT_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    BOOT_ERROR_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    sha256: str
    installed_at: Row3
    state: Row4
    last_boot: Row5
    metadata: Row6
    error: str
    active: bool
    restart_required: bool
    boot_error: str
    def __init__(self, install_id: _Optional[str] = ..., sha256: _Optional[str] = ..., installed_at: _Optional[_Union[Row3, _Mapping]] = ..., state: _Optional[_Union[Row4, _Mapping]] = ..., last_boot: _Optional[_Union[Row5, _Mapping]] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ..., error: _Optional[str] = ..., active: _Optional[bool] = ..., restart_required: _Optional[bool] = ..., boot_error: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Installed", "Disabled", "RemovalPending")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_FIELD_NUMBER: _ClassVar[int]
    DISABLED_FIELD_NUMBER: _ClassVar[int]
    REMOVALPENDING_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row7
    Installed: Row8
    Disabled: Row9
    RemovalPending: Row10
    def __init__(self, Unspecified: _Optional[_Union[Row7, _Mapping]] = ..., Installed: _Optional[_Union[Row8, _Mapping]] = ..., Disabled: _Optional[_Union[Row9, _Mapping]] = ..., RemovalPending: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("at", "result", "failure")
    AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    at: Row3
    result: Row11
    failure: Row12
    def __init__(self, at: _Optional[_Union[Row3, _Mapping]] = ..., result: _Optional[_Union[Row11, _Mapping]] = ..., failure: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("identity", "owners", "size_bytes")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    OWNERS_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    identity: Row13
    owners: _containers.RepeatedScalarFieldContainer[str]
    size_bytes: int
    def __init__(self, identity: _Optional[_Union[Row13, _Mapping]] = ..., owners: _Optional[_Iterable[str]] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Loaded", "Error")
    LOADED_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Loaded: Row14
    Error: Row15
    def __init__(self, Loaded: _Optional[_Union[Row14, _Mapping]] = ..., Error: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("plugin_id", "plugin_name", "message", "missing_secret", "carrier")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    PLUGIN_NAME_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    MISSING_SECRET_FIELD_NUMBER: _ClassVar[int]
    CARRIER_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    plugin_name: str
    message: str
    missing_secret: str
    carrier: bytes
    def __init__(self, plugin_id: _Optional[str] = ..., plugin_name: _Optional[str] = ..., message: _Optional[str] = ..., missing_secret: _Optional[str] = ..., carrier: _Optional[bytes] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("name", "version", "description")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    name: str
    version: str
    description: str
    def __init__(self, name: _Optional[str] = ..., version: _Optional[str] = ..., description: _Optional[str] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
