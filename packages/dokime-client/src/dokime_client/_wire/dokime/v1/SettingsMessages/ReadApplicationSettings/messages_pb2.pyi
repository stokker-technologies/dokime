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
    __slots__ = ("revision", "settings")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    SETTINGS_FIELD_NUMBER: _ClassVar[int]
    revision: int
    settings: Row3
    def __init__(self, revision: _Optional[int] = ..., settings: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("appearance", "report", "client", "telemetry", "listener")
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    REPORT_FIELD_NUMBER: _ClassVar[int]
    CLIENT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    LISTENER_FIELD_NUMBER: _ClassVar[int]
    appearance: Row4
    report: Row5
    client: Row6
    telemetry: Row7
    listener: Row8
    def __init__(self, appearance: _Optional[_Union[Row4, _Mapping]] = ..., report: _Optional[_Union[Row5, _Mapping]] = ..., client: _Optional[_Union[Row6, _Mapping]] = ..., telemetry: _Optional[_Union[Row7, _Mapping]] = ..., listener: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("theme_mode", "theme_family", "density")
    THEME_MODE_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    theme_mode: Row9
    theme_family: str
    density: Row10
    def __init__(self, theme_mode: _Optional[_Union[Row9, _Mapping]] = ..., theme_family: _Optional[str] = ..., density: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("source_bytes", "value_depth", "table_rows", "plot_points", "plot_downsample")
    SOURCE_BYTES_FIELD_NUMBER: _ClassVar[int]
    VALUE_DEPTH_FIELD_NUMBER: _ClassVar[int]
    TABLE_ROWS_FIELD_NUMBER: _ClassVar[int]
    PLOT_POINTS_FIELD_NUMBER: _ClassVar[int]
    PLOT_DOWNSAMPLE_FIELD_NUMBER: _ClassVar[int]
    source_bytes: int
    value_depth: int
    table_rows: int
    plot_points: int
    plot_downsample: Row11
    def __init__(self, source_bytes: _Optional[int] = ..., value_depth: _Optional[int] = ..., table_rows: _Optional[int] = ..., plot_points: _Optional[int] = ..., plot_downsample: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("connect_timeout_ms", "request_timeout_ms", "endpoint_stagger_ms")
    CONNECT_TIMEOUT_MS_FIELD_NUMBER: _ClassVar[int]
    REQUEST_TIMEOUT_MS_FIELD_NUMBER: _ClassVar[int]
    ENDPOINT_STAGGER_MS_FIELD_NUMBER: _ClassVar[int]
    connect_timeout_ms: int
    request_timeout_ms: int
    endpoint_stagger_ms: int
    def __init__(self, connect_timeout_ms: _Optional[int] = ..., request_timeout_ms: _Optional[int] = ..., endpoint_stagger_ms: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("interval_ms",)
    INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    interval_ms: int
    def __init__(self, interval_ms: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
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

class Row9(_message.Message):
    __slots__ = ("System", "Light", "Dark")
    SYSTEM_FIELD_NUMBER: _ClassVar[int]
    LIGHT_FIELD_NUMBER: _ClassVar[int]
    DARK_FIELD_NUMBER: _ClassVar[int]
    System: Row12
    Light: Row13
    Dark: Row14
    def __init__(self, System: _Optional[_Union[Row12, _Mapping]] = ..., Light: _Optional[_Union[Row13, _Mapping]] = ..., Dark: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Comfortable", "Compact", "Touch")
    COMFORTABLE_FIELD_NUMBER: _ClassVar[int]
    COMPACT_FIELD_NUMBER: _ClassVar[int]
    TOUCH_FIELD_NUMBER: _ClassVar[int]
    Comfortable: Row15
    Compact: Row16
    Touch: Row17
    def __init__(self, Comfortable: _Optional[_Union[Row15, _Mapping]] = ..., Compact: _Optional[_Union[Row16, _Mapping]] = ..., Touch: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row18
    Raw: Row19
    MinMax: Row20
    Lttb: Row21
    Auto: Row22
    def __init__(self, Unspecified: _Optional[_Union[Row18, _Mapping]] = ..., Raw: _Optional[_Union[Row19, _Mapping]] = ..., MinMax: _Optional[_Union[Row20, _Mapping]] = ..., Lttb: _Optional[_Union[Row21, _Mapping]] = ..., Auto: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
