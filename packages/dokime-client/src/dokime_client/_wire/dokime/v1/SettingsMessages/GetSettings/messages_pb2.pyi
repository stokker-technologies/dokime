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
    __slots__ = ("bootstrap_paths", "sections", "cards")
    BOOTSTRAP_PATHS_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    bootstrap_paths: Row3
    sections: _containers.RepeatedCompositeFieldContainer[Row4]
    cards: _containers.RepeatedCompositeFieldContainer[Row5]
    def __init__(self, bootstrap_paths: _Optional[_Union[Row3, _Mapping]] = ..., sections: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., cards: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("application", "storage", "plugins", "logs")
    APPLICATION_FIELD_NUMBER: _ClassVar[int]
    STORAGE_FIELD_NUMBER: _ClassVar[int]
    PLUGINS_FIELD_NUMBER: _ClassVar[int]
    LOGS_FIELD_NUMBER: _ClassVar[int]
    application: str
    storage: str
    plugins: str
    logs: str
    def __init__(self, application: _Optional[str] = ..., storage: _Optional[str] = ..., plugins: _Optional[str] = ..., logs: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("section_id", "label", "description", "owner", "order")
    SECTION_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    section_id: str
    label: str
    description: str
    owner: Row6
    order: int
    def __init__(self, section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., owner: _Optional[_Union[Row6, _Mapping]] = ..., order: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("plugin_id", "card_id", "section_id", "label", "description", "body", "refresh", "state", "order")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    SECTION_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    BODY_FIELD_NUMBER: _ClassVar[int]
    REFRESH_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    card_id: str
    section_id: str
    label: str
    description: str
    body: Row7
    refresh: Row8
    state: Row9
    order: int
    def __init__(self, plugin_id: _Optional[str] = ..., card_id: _Optional[str] = ..., section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., body: _Optional[_Union[Row7, _Mapping]] = ..., refresh: _Optional[_Union[Row8, _Mapping]] = ..., state: _Optional[_Union[Row9, _Mapping]] = ..., order: _Optional[int] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("App", "InstalledPluginId")
    APP_FIELD_NUMBER: _ClassVar[int]
    INSTALLEDPLUGINID_FIELD_NUMBER: _ClassVar[int]
    App: Row10
    InstalledPluginId: Row11
    def __init__(self, App: _Optional[_Union[Row10, _Mapping]] = ..., InstalledPluginId: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("blocks",)
    BLOCKS_FIELD_NUMBER: _ClassVar[int]
    blocks: _containers.RepeatedCompositeFieldContainer[Row12]
    def __init__(self, blocks: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("policy",)
    POLICY_FIELD_NUMBER: _ClassVar[int]
    policy: Row13
    def __init__(self, policy: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Unspecified", "Ready", "Disabled", "Disconnected", "Stale", "Error", "PermissionDenied")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    DISABLED_FIELD_NUMBER: _ClassVar[int]
    DISCONNECTED_FIELD_NUMBER: _ClassVar[int]
    STALE_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    PERMISSIONDENIED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row14
    Ready: Row15
    Disabled: Row16
    Disconnected: Row17
    Stale: Row18
    Error: Row19
    PermissionDenied: Row20
    def __init__(self, Unspecified: _Optional[_Union[Row14, _Mapping]] = ..., Ready: _Optional[_Union[Row15, _Mapping]] = ..., Disabled: _Optional[_Union[Row16, _Mapping]] = ..., Disconnected: _Optional[_Union[Row17, _Mapping]] = ..., Stale: _Optional[_Union[Row18, _Mapping]] = ..., Error: _Optional[_Union[Row19, _Mapping]] = ..., PermissionDenied: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row21
    def __init__(self, kind: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Manual", "Cached", "OnDemand")
    MANUAL_FIELD_NUMBER: _ClassVar[int]
    CACHED_FIELD_NUMBER: _ClassVar[int]
    ONDEMAND_FIELD_NUMBER: _ClassVar[int]
    Manual: Row22
    Cached: Row23
    OnDemand: Row24
    def __init__(self, Manual: _Optional[_Union[Row22, _Mapping]] = ..., Cached: _Optional[_Union[Row23, _Mapping]] = ..., OnDemand: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Text", "Status", "KeyValueList", "Metrics")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    KEYVALUELIST_FIELD_NUMBER: _ClassVar[int]
    METRICS_FIELD_NUMBER: _ClassVar[int]
    Text: Row25
    Status: Row26
    KeyValueList: Row27
    Metrics: Row28
    def __init__(self, Text: _Optional[_Union[Row25, _Mapping]] = ..., Status: _Optional[_Union[Row26, _Mapping]] = ..., KeyValueList: _Optional[_Union[Row27, _Mapping]] = ..., Metrics: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row31
    def __init__(self, field_0: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ("max_age_ms",)
    MAX_AGE_MS_FIELD_NUMBER: _ClassVar[int]
    max_age_ms: int
    def __init__(self, max_age_ms: _Optional[int] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ("label", "tone", "detail")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    tone: Row35
    detail: str
    def __init__(self, label: _Optional[str] = ..., tone: _Optional[_Union[Row35, _Mapping]] = ..., detail: _Optional[str] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[Row36]
    def __init__(self, items: _Optional[_Iterable[_Union[Row36, _Mapping]]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[Row37]
    def __init__(self, items: _Optional[_Iterable[_Union[Row37, _Mapping]]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("Unspecified", "Neutral", "Info", "Success", "Warning", "Danger")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NEUTRAL_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row38
    Neutral: Row39
    Info: Row40
    Success: Row41
    Warning: Row42
    Danger: Row43
    def __init__(self, Unspecified: _Optional[_Union[Row38, _Mapping]] = ..., Neutral: _Optional[_Union[Row39, _Mapping]] = ..., Info: _Optional[_Union[Row40, _Mapping]] = ..., Success: _Optional[_Union[Row41, _Mapping]] = ..., Warning: _Optional[_Union[Row42, _Mapping]] = ..., Danger: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("label", "value", "detail")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    value: str
    detail: str
    def __init__(self, label: _Optional[str] = ..., value: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("label", "value", "unit", "trend")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    TREND_FIELD_NUMBER: _ClassVar[int]
    label: str
    value: str
    unit: str
    trend: str
    def __init__(self, label: _Optional[str] = ..., value: _Optional[str] = ..., unit: _Optional[str] = ..., trend: _Optional[str] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row39(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
