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
    __slots__ = ("app_name", "organization", "title_template", "logos", "themes", "active_theme", "theme_family_locked", "mode_policy", "mode_locked", "source_plugin_id")
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
    organization: Row3
    title_template: str
    logos: _containers.RepeatedCompositeFieldContainer[Row4]
    themes: _containers.RepeatedCompositeFieldContainer[Row5]
    active_theme: str
    theme_family_locked: bool
    mode_policy: Row6
    mode_locked: bool
    source_plugin_id: str
    def __init__(self, app_name: _Optional[str] = ..., organization: _Optional[_Union[Row3, _Mapping]] = ..., title_template: _Optional[str] = ..., logos: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., themes: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., active_theme: _Optional[str] = ..., theme_family_locked: _Optional[bool] = ..., mode_policy: _Optional[_Union[Row6, _Mapping]] = ..., mode_locked: _Optional[bool] = ..., source_plugin_id: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("name", "unit", "website")
    NAME_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    WEBSITE_FIELD_NUMBER: _ClassVar[int]
    name: str
    unit: str
    website: str
    def __init__(self, name: _Optional[str] = ..., unit: _Optional[str] = ..., website: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("variant", "asset", "alt")
    VARIANT_FIELD_NUMBER: _ClassVar[int]
    ASSET_FIELD_NUMBER: _ClassVar[int]
    ALT_FIELD_NUMBER: _ClassVar[int]
    variant: Row7
    asset: Row8
    alt: str
    def __init__(self, variant: _Optional[_Union[Row7, _Mapping]] = ..., asset: _Optional[_Union[Row8, _Mapping]] = ..., alt: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("plugin_id", "theme_id", "label", "description", "stylesheet", "variables", "corner_radii", "light", "dark")
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
    variables: _containers.RepeatedCompositeFieldContainer[Row9]
    corner_radii: Row10
    light: Row11
    dark: Row11
    def __init__(self, plugin_id: _Optional[str] = ..., theme_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., stylesheet: _Optional[str] = ..., variables: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., corner_radii: _Optional[_Union[Row10, _Mapping]] = ..., light: _Optional[_Union[Row11, _Mapping]] = ..., dark: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "UserOrSystem", "ForceSystem", "ForceLight", "ForceDark")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    USERORSYSTEM_FIELD_NUMBER: _ClassVar[int]
    FORCESYSTEM_FIELD_NUMBER: _ClassVar[int]
    FORCELIGHT_FIELD_NUMBER: _ClassVar[int]
    FORCEDARK_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row12
    UserOrSystem: Row13
    ForceSystem: Row14
    ForceLight: Row15
    ForceDark: Row16
    def __init__(self, Unspecified: _Optional[_Union[Row12, _Mapping]] = ..., UserOrSystem: _Optional[_Union[Row13, _Mapping]] = ..., ForceSystem: _Optional[_Union[Row14, _Mapping]] = ..., ForceLight: _Optional[_Union[Row15, _Mapping]] = ..., ForceDark: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("TransparentOnLight", "TransparentOnDark", "ContainedOnLight", "ContainedOnDark")
    TRANSPARENTONLIGHT_FIELD_NUMBER: _ClassVar[int]
    TRANSPARENTONDARK_FIELD_NUMBER: _ClassVar[int]
    CONTAINEDONLIGHT_FIELD_NUMBER: _ClassVar[int]
    CONTAINEDONDARK_FIELD_NUMBER: _ClassVar[int]
    TransparentOnLight: Row17
    TransparentOnDark: Row18
    ContainedOnLight: Row19
    ContainedOnDark: Row20
    def __init__(self, TransparentOnLight: _Optional[_Union[Row17, _Mapping]] = ..., TransparentOnDark: _Optional[_Union[Row18, _Mapping]] = ..., ContainedOnLight: _Optional[_Union[Row19, _Mapping]] = ..., ContainedOnDark: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("reference", "content_type", "size_bytes")
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    reference: Row21
    content_type: str
    size_bytes: int
    def __init__(self, reference: _Optional[_Union[Row21, _Mapping]] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Row10(_message.Message):
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

class Row11(_message.Message):
    __slots__ = ("variables", "plot_palette")
    VARIABLES_FIELD_NUMBER: _ClassVar[int]
    PLOT_PALETTE_FIELD_NUMBER: _ClassVar[int]
    variables: _containers.RepeatedCompositeFieldContainer[Row22]
    plot_palette: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, variables: _Optional[_Iterable[_Union[Row22, _Mapping]]] = ..., plot_palette: _Optional[_Iterable[str]] = ...) -> None: ...

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
    __slots__ = ("plugin_id", "asset_id", "sha256")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    ASSET_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    asset_id: str
    sha256: str
    def __init__(self, plugin_id: _Optional[str] = ..., asset_id: _Optional[str] = ..., sha256: _Optional[str] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
