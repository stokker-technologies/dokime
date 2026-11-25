from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LogoVariant(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LOGO_VARIANT_UNSPECIFIED: _ClassVar[LogoVariant]
    LOGO_VARIANT_TRANSPARENT_ON_LIGHT: _ClassVar[LogoVariant]
    LOGO_VARIANT_TRANSPARENT_ON_DARK: _ClassVar[LogoVariant]
    LOGO_VARIANT_CONTAINED_ON_LIGHT: _ClassVar[LogoVariant]
    LOGO_VARIANT_CONTAINED_ON_DARK: _ClassVar[LogoVariant]
LOGO_VARIANT_UNSPECIFIED: LogoVariant
LOGO_VARIANT_TRANSPARENT_ON_LIGHT: LogoVariant
LOGO_VARIANT_TRANSPARENT_ON_DARK: LogoVariant
LOGO_VARIANT_CONTAINED_ON_LIGHT: LogoVariant
LOGO_VARIANT_CONTAINED_ON_DARK: LogoVariant

@final
class BackendAppearanceRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class BackendAppearanceResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    appearance: AppearanceSnapshot
    resolved_at: _position_pb2.ReadCut
    def __init__(self, appearance: _Optional[_Union[AppearanceSnapshot, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class BackendEnvironmentRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.Position
    def __init__(self, at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class BackendEnvironmentResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BACKEND_PID_FIELD_NUMBER: _ClassVar[int]
    IN_STEP_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    backend_pid: int
    in_step: bool
    resolved_at: _position_pb2.Position
    def __init__(self, backend_pid: _Optional[int] = ..., in_step: _Optional[bool] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class AppearanceSnapshot(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    APP_NAME_FIELD_NUMBER: _ClassVar[int]
    ORGANIZATION_FIELD_NUMBER: _ClassVar[int]
    TITLE_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    LOGOS_FIELD_NUMBER: _ClassVar[int]
    STYLESHEET_FIELD_NUMBER: _ClassVar[int]
    app_name: str
    organization: OrganizationIdentity
    title_template: str
    logos: _containers.RepeatedCompositeFieldContainer[AppearanceLogo]
    stylesheet: str
    def __init__(self, app_name: _Optional[str] = ..., organization: _Optional[_Union[OrganizationIdentity, _Mapping]] = ..., title_template: _Optional[str] = ..., logos: _Optional[_Iterable[_Union[AppearanceLogo, _Mapping]]] = ..., stylesheet: _Optional[str] = ...) -> None: ...

@final
class OrganizationIdentity(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    WEBSITE_FIELD_NUMBER: _ClassVar[int]
    name: str
    unit: str
    website: str
    def __init__(self, name: _Optional[str] = ..., unit: _Optional[str] = ..., website: _Optional[str] = ...) -> None: ...

@final
class AppearanceLogo(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VARIANT_FIELD_NUMBER: _ClassVar[int]
    DATA_URI_FIELD_NUMBER: _ClassVar[int]
    variant: LogoVariant
    data_uri: str
    def __init__(self, variant: _Optional[_Union[LogoVariant, str]] = ..., data_uri: _Optional[str] = ...) -> None: ...
