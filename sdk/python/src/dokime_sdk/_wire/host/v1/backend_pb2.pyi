from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
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

class ThemeModePolicy(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    THEME_MODE_POLICY_UNSPECIFIED: _ClassVar[ThemeModePolicy]
    THEME_MODE_POLICY_USER_OR_SYSTEM: _ClassVar[ThemeModePolicy]
    THEME_MODE_POLICY_FORCE_SYSTEM: _ClassVar[ThemeModePolicy]
    THEME_MODE_POLICY_FORCE_LIGHT: _ClassVar[ThemeModePolicy]
    THEME_MODE_POLICY_FORCE_DARK: _ClassVar[ThemeModePolicy]

class AppearanceFailureKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    APPEARANCE_FAILURE_KIND_UNSPECIFIED: _ClassVar[AppearanceFailureKind]
    APPEARANCE_FAILURE_KIND_ASSET_MISSING: _ClassVar[AppearanceFailureKind]
    APPEARANCE_FAILURE_KIND_ASSET_CHANGED: _ClassVar[AppearanceFailureKind]
    APPEARANCE_FAILURE_KIND_TOO_LARGE: _ClassVar[AppearanceFailureKind]
    APPEARANCE_FAILURE_KIND_INVALID_REFERENCE: _ClassVar[AppearanceFailureKind]
    APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE: _ClassVar[AppearanceFailureKind]
LOGO_VARIANT_UNSPECIFIED: LogoVariant
LOGO_VARIANT_TRANSPARENT_ON_LIGHT: LogoVariant
LOGO_VARIANT_TRANSPARENT_ON_DARK: LogoVariant
LOGO_VARIANT_CONTAINED_ON_LIGHT: LogoVariant
LOGO_VARIANT_CONTAINED_ON_DARK: LogoVariant
THEME_MODE_POLICY_UNSPECIFIED: ThemeModePolicy
THEME_MODE_POLICY_USER_OR_SYSTEM: ThemeModePolicy
THEME_MODE_POLICY_FORCE_SYSTEM: ThemeModePolicy
THEME_MODE_POLICY_FORCE_LIGHT: ThemeModePolicy
THEME_MODE_POLICY_FORCE_DARK: ThemeModePolicy
APPEARANCE_FAILURE_KIND_UNSPECIFIED: AppearanceFailureKind
APPEARANCE_FAILURE_KIND_ASSET_MISSING: AppearanceFailureKind
APPEARANCE_FAILURE_KIND_ASSET_CHANGED: AppearanceFailureKind
APPEARANCE_FAILURE_KIND_TOO_LARGE: AppearanceFailureKind
APPEARANCE_FAILURE_KIND_INVALID_REFERENCE: AppearanceFailureKind
APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE: AppearanceFailureKind

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
    THEMES_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_THEME_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_LOCKED_FIELD_NUMBER: _ClassVar[int]
    MODE_POLICY_FIELD_NUMBER: _ClassVar[int]
    MODE_LOCKED_FIELD_NUMBER: _ClassVar[int]
    SOURCE_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    app_name: str
    organization: OrganizationIdentity
    title_template: str
    logos: _containers.RepeatedCompositeFieldContainer[AppearanceLogo]
    themes: _containers.RepeatedCompositeFieldContainer[ThemeFamily]
    active_theme: str
    theme_family_locked: bool
    mode_policy: ThemeModePolicy
    mode_locked: bool
    source_plugin_id: str
    def __init__(self, app_name: _Optional[str] = ..., organization: _Optional[_Union[OrganizationIdentity, _Mapping]] = ..., title_template: _Optional[str] = ..., logos: _Optional[_Iterable[_Union[AppearanceLogo, _Mapping]]] = ..., themes: _Optional[_Iterable[_Union[ThemeFamily, _Mapping]]] = ..., active_theme: _Optional[str] = ..., theme_family_locked: _Optional[bool] = ..., mode_policy: _Optional[_Union[ThemeModePolicy, str]] = ..., mode_locked: _Optional[bool] = ..., source_plugin_id: _Optional[str] = ...) -> None: ...

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
    ASSET_FIELD_NUMBER: _ClassVar[int]
    ALT_FIELD_NUMBER: _ClassVar[int]
    variant: LogoVariant
    asset: AppearanceAssetMetadata
    alt: str
    def __init__(self, variant: _Optional[_Union[LogoVariant, str]] = ..., asset: _Optional[_Union[AppearanceAssetMetadata, _Mapping]] = ..., alt: _Optional[str] = ...) -> None: ...

@final
class ThemeFamily(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    @final
    class VariablesEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
        __slots__: tuple[str, ...]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    THEME_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    STYLESHEET_FIELD_NUMBER: _ClassVar[int]
    VARIABLES_FIELD_NUMBER: _ClassVar[int]
    CORNER_RADII_FIELD_NUMBER: _ClassVar[int]
    LIGHT_FIELD_NUMBER: _ClassVar[int]
    DARK_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    theme_id: str
    label: str
    description: str
    stylesheet: str
    variables: _containers.ScalarMap[str, str]
    corner_radii: ThemeCornerRadii
    light: ThemeVariant
    dark: ThemeVariant
    def __init__(self, plugin_id: _Optional[str] = ..., theme_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., stylesheet: _Optional[str] = ..., variables: _Optional[_Mapping[str, str]] = ..., corner_radii: _Optional[_Union[ThemeCornerRadii, _Mapping]] = ..., light: _Optional[_Union[ThemeVariant, _Mapping]] = ..., dark: _Optional[_Union[ThemeVariant, _Mapping]] = ...) -> None: ...

@final
class ThemeCornerRadii(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BASE_FIELD_NUMBER: _ClassVar[int]
    SM_FIELD_NUMBER: _ClassVar[int]
    MD_FIELD_NUMBER: _ClassVar[int]
    LG_FIELD_NUMBER: _ClassVar[int]
    XL_FIELD_NUMBER: _ClassVar[int]
    TWO_XL_FIELD_NUMBER: _ClassVar[int]
    THREE_XL_FIELD_NUMBER: _ClassVar[int]
    FOUR_XL_FIELD_NUMBER: _ClassVar[int]
    base: str
    sm: str
    md: str
    lg: str
    xl: str
    two_xl: str
    three_xl: str
    four_xl: str
    def __init__(self, base: _Optional[str] = ..., sm: _Optional[str] = ..., md: _Optional[str] = ..., lg: _Optional[str] = ..., xl: _Optional[str] = ..., two_xl: _Optional[str] = ..., three_xl: _Optional[str] = ..., four_xl: _Optional[str] = ...) -> None: ...

@final
class ThemeVariant(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    @final
    class VariablesEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
        __slots__: tuple[str, ...]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    VARIABLES_FIELD_NUMBER: _ClassVar[int]
    PLOT_PALETTE_FIELD_NUMBER: _ClassVar[int]
    variables: _containers.ScalarMap[str, str]
    plot_palette: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, variables: _Optional[_Mapping[str, str]] = ..., plot_palette: _Optional[_Iterable[str]] = ...) -> None: ...

@final
class AppearanceFailure(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: AppearanceFailureKind
    def __init__(self, kind: _Optional[_Union[AppearanceFailureKind, str]] = ...) -> None: ...

@final
class AppearanceAssetReference(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    ASSET_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    asset_id: str
    sha256: str
    def __init__(self, plugin_id: _Optional[str] = ..., asset_id: _Optional[str] = ..., sha256: _Optional[str] = ...) -> None: ...

@final
class AppearanceAssetMetadata(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    reference: AppearanceAssetReference
    content_type: str
    size_bytes: int
    file: _file_pb2.FileReference
    def __init__(self, reference: _Optional[_Union[AppearanceAssetReference, _Mapping]] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ...) -> None: ...
