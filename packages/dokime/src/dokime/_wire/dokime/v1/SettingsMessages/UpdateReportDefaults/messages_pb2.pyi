from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("report",)
    REPORT_FIELD_NUMBER: _ClassVar[int]
    report: Row3
    def __init__(self, report: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("appearance", "report", "application_root", "configured_data_root", "default_data_root", "effective_data_root", "restart_required")
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    REPORT_FIELD_NUMBER: _ClassVar[int]
    APPLICATION_ROOT_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    EFFECTIVE_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    appearance: Row4
    report: Row3
    application_root: str
    configured_data_root: str
    default_data_root: str
    effective_data_root: str
    restart_required: bool
    def __init__(self, appearance: _Optional[_Union[Row4, _Mapping]] = ..., report: _Optional[_Union[Row3, _Mapping]] = ..., application_root: _Optional[str] = ..., configured_data_root: _Optional[str] = ..., default_data_root: _Optional[str] = ..., effective_data_root: _Optional[str] = ..., restart_required: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("plot_points", "plot_downsample")
    PLOT_POINTS_FIELD_NUMBER: _ClassVar[int]
    PLOT_DOWNSAMPLE_FIELD_NUMBER: _ClassVar[int]
    plot_points: int
    plot_downsample: Row5
    def __init__(self, plot_points: _Optional[int] = ..., plot_downsample: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("theme_mode", "theme_family", "density")
    THEME_MODE_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    theme_mode: Row6
    theme_family: str
    density: Row7
    def __init__(self, theme_mode: _Optional[_Union[Row6, _Mapping]] = ..., theme_family: _Optional[str] = ..., density: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row8
    Raw: Row9
    MinMax: Row10
    Lttb: Row11
    Auto: Row12
    def __init__(self, Unspecified: _Optional[_Union[Row8, _Mapping]] = ..., Raw: _Optional[_Union[Row9, _Mapping]] = ..., MinMax: _Optional[_Union[Row10, _Mapping]] = ..., Lttb: _Optional[_Union[Row11, _Mapping]] = ..., Auto: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "System", "Light", "Dark")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SYSTEM_FIELD_NUMBER: _ClassVar[int]
    LIGHT_FIELD_NUMBER: _ClassVar[int]
    DARK_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row13
    System: Row14
    Light: Row15
    Dark: Row16
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., System: _Optional[_Union[Row14, _Mapping]] = ..., Light: _Optional[_Union[Row15, _Mapping]] = ..., Dark: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "Comfortable", "Compact", "Touch")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMFORTABLE_FIELD_NUMBER: _ClassVar[int]
    COMPACT_FIELD_NUMBER: _ClassVar[int]
    TOUCH_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row17
    Comfortable: Row18
    Compact: Row19
    Touch: Row20
    def __init__(self, Unspecified: _Optional[_Union[Row17, _Mapping]] = ..., Comfortable: _Optional[_Union[Row18, _Mapping]] = ..., Compact: _Optional[_Union[Row19, _Mapping]] = ..., Touch: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
