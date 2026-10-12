from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

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

class GetAppearanceRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class AppearanceSnapshot(_message.Message):
    __slots__ = ("app_name", "organization", "title_template", "logo", "themes", "active_theme", "theme_family_locked", "mode_policy", "mode_locked", "source_plugin_id")
    APP_NAME_FIELD_NUMBER: _ClassVar[int]
    ORGANIZATION_FIELD_NUMBER: _ClassVar[int]
    TITLE_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    LOGO_FIELD_NUMBER: _ClassVar[int]
    THEMES_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_THEME_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_LOCKED_FIELD_NUMBER: _ClassVar[int]
    MODE_POLICY_FIELD_NUMBER: _ClassVar[int]
    MODE_LOCKED_FIELD_NUMBER: _ClassVar[int]
    SOURCE_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    app_name: str
    organization: OrganizationIdentity
    title_template: str
    logo: AppearanceLogo
    themes: _containers.RepeatedCompositeFieldContainer[ThemeFamily]
    active_theme: str
    theme_family_locked: bool
    mode_policy: ThemeModePolicy
    mode_locked: bool
    source_plugin_id: str
    def __init__(self, app_name: _Optional[str] = ..., organization: _Optional[_Union[OrganizationIdentity, _Mapping]] = ..., title_template: _Optional[str] = ..., logo: _Optional[_Union[AppearanceLogo, _Mapping]] = ..., themes: _Optional[_Iterable[_Union[ThemeFamily, _Mapping]]] = ..., active_theme: _Optional[str] = ..., theme_family_locked: _Optional[bool] = ..., mode_policy: _Optional[_Union[ThemeModePolicy, str]] = ..., mode_locked: _Optional[bool] = ..., source_plugin_id: _Optional[str] = ...) -> None: ...

class OrganizationIdentity(_message.Message):
    __slots__ = ("name", "unit", "website")
    NAME_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    WEBSITE_FIELD_NUMBER: _ClassVar[int]
    name: str
    unit: str
    website: str
    def __init__(self, name: _Optional[str] = ..., unit: _Optional[str] = ..., website: _Optional[str] = ...) -> None: ...

class AppearanceLogo(_message.Message):
    __slots__ = ("asset", "alt")
    ASSET_FIELD_NUMBER: _ClassVar[int]
    ALT_FIELD_NUMBER: _ClassVar[int]
    asset: AppearanceAssetMetadata
    alt: str
    def __init__(self, asset: _Optional[_Union[AppearanceAssetMetadata, _Mapping]] = ..., alt: _Optional[str] = ...) -> None: ...

class AppearanceAssetReference(_message.Message):
    __slots__ = ("plugin_id", "asset_id", "sha256")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    ASSET_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    asset_id: str
    sha256: str
    def __init__(self, plugin_id: _Optional[str] = ..., asset_id: _Optional[str] = ..., sha256: _Optional[str] = ...) -> None: ...

class AppearanceAssetMetadata(_message.Message):
    __slots__ = ("reference", "content_type", "size_bytes")
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    reference: AppearanceAssetReference
    content_type: str
    size_bytes: int
    def __init__(self, reference: _Optional[_Union[AppearanceAssetReference, _Mapping]] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class ThemeFamily(_message.Message):
    __slots__ = ("plugin_id", "theme_id", "label", "description", "stylesheet", "variables", "corner_radii", "light", "dark")
    class VariablesEntry(_message.Message):
        __slots__ = ("key", "value")
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

class ThemeCornerRadii(_message.Message):
    __slots__ = ("base", "sm", "md", "lg", "xl", "two_xl", "three_xl", "four_xl")
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

class ThemeVariant(_message.Message):
    __slots__ = ("variables", "plot_palette")
    class VariablesEntry(_message.Message):
        __slots__ = ("key", "value")
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

class AppearanceFailure(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: AppearanceFailureKind
    def __init__(self, kind: _Optional[_Union[AppearanceFailureKind, str]] = ...) -> None: ...
