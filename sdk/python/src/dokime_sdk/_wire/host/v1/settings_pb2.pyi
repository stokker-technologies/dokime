from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import plots_pb2 as _plots_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Density(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DENSITY_UNSPECIFIED: _ClassVar[Density]
    DENSITY_COMFORTABLE: _ClassVar[Density]
    DENSITY_COMPACT: _ClassVar[Density]
    DENSITY_TOUCH: _ClassVar[Density]

class HostCardTone(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    HOST_CARD_TONE_UNSPECIFIED: _ClassVar[HostCardTone]
    HOST_CARD_TONE_NEUTRAL: _ClassVar[HostCardTone]
    HOST_CARD_TONE_INFO: _ClassVar[HostCardTone]
    HOST_CARD_TONE_SUCCESS: _ClassVar[HostCardTone]
    HOST_CARD_TONE_WARNING: _ClassVar[HostCardTone]
    HOST_CARD_TONE_DANGER: _ClassVar[HostCardTone]

class SettingsCardState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SETTINGS_CARD_STATE_UNSPECIFIED: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_READY: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_DISABLED: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_DISCONNECTED: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_STALE: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_ERROR: _ClassVar[SettingsCardState]
    SETTINGS_CARD_STATE_PERMISSION_DENIED: _ClassVar[SettingsCardState]

class ThemeMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    THEME_MODE_UNSPECIFIED: _ClassVar[ThemeMode]
    THEME_MODE_SYSTEM: _ClassVar[ThemeMode]
    THEME_MODE_LIGHT: _ClassVar[ThemeMode]
    THEME_MODE_DARK: _ClassVar[ThemeMode]
DENSITY_UNSPECIFIED: Density
DENSITY_COMFORTABLE: Density
DENSITY_COMPACT: Density
DENSITY_TOUCH: Density
HOST_CARD_TONE_UNSPECIFIED: HostCardTone
HOST_CARD_TONE_NEUTRAL: HostCardTone
HOST_CARD_TONE_INFO: HostCardTone
HOST_CARD_TONE_SUCCESS: HostCardTone
HOST_CARD_TONE_WARNING: HostCardTone
HOST_CARD_TONE_DANGER: HostCardTone
SETTINGS_CARD_STATE_UNSPECIFIED: SettingsCardState
SETTINGS_CARD_STATE_READY: SettingsCardState
SETTINGS_CARD_STATE_DISABLED: SettingsCardState
SETTINGS_CARD_STATE_DISCONNECTED: SettingsCardState
SETTINGS_CARD_STATE_STALE: SettingsCardState
SETTINGS_CARD_STATE_ERROR: SettingsCardState
SETTINGS_CARD_STATE_PERMISSION_DENIED: SettingsCardState
THEME_MODE_UNSPECIFIED: ThemeMode
THEME_MODE_SYSTEM: ThemeMode
THEME_MODE_LIGHT: ThemeMode
THEME_MODE_DARK: ThemeMode

@final
class SettingsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SettingsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: ApplicationSettings
    revision: bytes
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[ApplicationSettings, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SettingsSetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    value: ApplicationSettings
    expected_revision: bytes
    request_id: str
    def __init__(self, value: _Optional[_Union[ApplicationSettings, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class SettingsSetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: ApplicationSettings
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[ApplicationSettings, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class SettingsSchemaRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SettingsSchemaResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    schema: SettingsFrame
    resolved_at: _position_pb2.ReadCut
    def __init__(self, schema: _Optional[_Union[SettingsFrame, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class ApplicationAppearance(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    THEME_MODE_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    theme_mode: ThemeMode
    density: Density
    def __init__(self, theme_mode: _Optional[_Union[ThemeMode, str]] = ..., density: _Optional[_Union[Density, str]] = ...) -> None: ...

@final
class ApplicationSettings(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    REPORT_FIELD_NUMBER: _ClassVar[int]
    CLIENT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    LISTENER_FIELD_NUMBER: _ClassVar[int]
    appearance: ApplicationAppearance
    report: ReportDefaults
    client: ClientPreferences
    telemetry: TelemetryPreferences
    listener: RemoteListenerSettings
    def __init__(self, appearance: _Optional[_Union[ApplicationAppearance, _Mapping]] = ..., report: _Optional[_Union[ReportDefaults, _Mapping]] = ..., client: _Optional[_Union[ClientPreferences, _Mapping]] = ..., telemetry: _Optional[_Union[TelemetryPreferences, _Mapping]] = ..., listener: _Optional[_Union[RemoteListenerSettings, _Mapping]] = ...) -> None: ...

@final
class BootstrapPaths(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    APPLICATION_FIELD_NUMBER: _ClassVar[int]
    STORAGE_FIELD_NUMBER: _ClassVar[int]
    PLUGINS_FIELD_NUMBER: _ClassVar[int]
    LOGS_FIELD_NUMBER: _ClassVar[int]
    application: str
    storage: str
    plugins: str
    logs: str
    def __init__(self, application: _Optional[str] = ..., storage: _Optional[str] = ..., plugins: _Optional[str] = ..., logs: _Optional[str] = ...) -> None: ...

@final
class CachedRefresh(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    MAX_AGE_MS_FIELD_NUMBER: _ClassVar[int]
    max_age_ms: int
    def __init__(self, max_age_ms: _Optional[int] = ...) -> None: ...

@final
class ClientPreferences(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CONNECT_TIMEOUT_MS_FIELD_NUMBER: _ClassVar[int]
    REQUEST_TIMEOUT_MS_FIELD_NUMBER: _ClassVar[int]
    connect_timeout_ms: int
    request_timeout_ms: int
    def __init__(self, connect_timeout_ms: _Optional[int] = ..., request_timeout_ms: _Optional[int] = ...) -> None: ...

@final
class HostCardBlock(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: HostCardBlockKind
    def __init__(self, kind: _Optional[_Union[HostCardBlockKind, _Mapping]] = ...) -> None: ...

@final
class HostCardBlockKind(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    KEY_VALUE_LIST_FIELD_NUMBER: _ClassVar[int]
    METRICS_FIELD_NUMBER: _ClassVar[int]
    text: str
    status: HostCardStatus
    key_value_list: HostCardKeyValueList
    metrics: HostCardMetrics
    def __init__(self, text: _Optional[str] = ..., status: _Optional[_Union[HostCardStatus, _Mapping]] = ..., key_value_list: _Optional[_Union[HostCardKeyValueList, _Mapping]] = ..., metrics: _Optional[_Union[HostCardMetrics, _Mapping]] = ...) -> None: ...

@final
class HostCardBody(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BLOCKS_FIELD_NUMBER: _ClassVar[int]
    blocks: _containers.RepeatedCompositeFieldContainer[HostCardBlock]
    def __init__(self, blocks: _Optional[_Iterable[_Union[HostCardBlock, _Mapping]]] = ...) -> None: ...

@final
class HostCardKeyValue(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    value: str
    detail: str
    def __init__(self, label: _Optional[str] = ..., value: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

@final
class HostCardKeyValueList(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[HostCardKeyValue]
    def __init__(self, items: _Optional[_Iterable[_Union[HostCardKeyValue, _Mapping]]] = ...) -> None: ...

@final
class HostCardMetric(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    TREND_FIELD_NUMBER: _ClassVar[int]
    label: str
    value: str
    unit: str
    trend: str
    def __init__(self, label: _Optional[str] = ..., value: _Optional[str] = ..., unit: _Optional[str] = ..., trend: _Optional[str] = ...) -> None: ...

@final
class HostCardMetrics(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[HostCardMetric]
    def __init__(self, items: _Optional[_Iterable[_Union[HostCardMetric, _Mapping]]] = ...) -> None: ...

@final
class HostCardStatus(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    tone: HostCardTone
    detail: str
    def __init__(self, label: _Optional[str] = ..., tone: _Optional[_Union[HostCardTone, str]] = ..., detail: _Optional[str] = ...) -> None: ...

@final
class ManualRefresh(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...

@final
class OnDemandRefresh(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...

@final
class RemoteListenerSettings(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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

@final
class ReportDefaults(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SOURCE_BYTES_FIELD_NUMBER: _ClassVar[int]
    VALUE_DEPTH_FIELD_NUMBER: _ClassVar[int]
    TABLE_ROWS_FIELD_NUMBER: _ClassVar[int]
    PLOT_POINTS_FIELD_NUMBER: _ClassVar[int]
    PLOT_DOWNSAMPLE_FIELD_NUMBER: _ClassVar[int]
    source_bytes: int
    value_depth: int
    table_rows: int
    plot_points: int
    plot_downsample: _plots_pb2.Downsample
    def __init__(self, source_bytes: _Optional[int] = ..., value_depth: _Optional[int] = ..., table_rows: _Optional[int] = ..., plot_points: _Optional[int] = ..., plot_downsample: _Optional[_Union[_plots_pb2.Downsample, str]] = ...) -> None: ...

@final
class SettingsCard(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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
    body: HostCardBody
    refresh: SettingsCardRefreshPolicy
    state: SettingsCardState
    order: int
    def __init__(self, plugin_id: _Optional[str] = ..., card_id: _Optional[str] = ..., section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., body: _Optional[_Union[HostCardBody, _Mapping]] = ..., refresh: _Optional[_Union[SettingsCardRefreshPolicy, _Mapping]] = ..., state: _Optional[_Union[SettingsCardState, str]] = ..., order: _Optional[int] = ...) -> None: ...

@final
class SettingsCardRefreshPolicy(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    POLICY_FIELD_NUMBER: _ClassVar[int]
    policy: SettingsCardRefreshPolicyPolicy
    def __init__(self, policy: _Optional[_Union[SettingsCardRefreshPolicyPolicy, _Mapping]] = ...) -> None: ...

@final
class SettingsCardRefreshPolicyPolicy(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    MANUAL_FIELD_NUMBER: _ClassVar[int]
    CACHED_FIELD_NUMBER: _ClassVar[int]
    ON_DEMAND_FIELD_NUMBER: _ClassVar[int]
    manual: ManualRefresh
    cached: CachedRefresh
    on_demand: OnDemandRefresh
    def __init__(self, manual: _Optional[_Union[ManualRefresh, _Mapping]] = ..., cached: _Optional[_Union[CachedRefresh, _Mapping]] = ..., on_demand: _Optional[_Union[OnDemandRefresh, _Mapping]] = ...) -> None: ...

@final
class SettingsFrame(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    BOOTSTRAP_PATHS_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    bootstrap_paths: BootstrapPaths
    sections: _containers.RepeatedCompositeFieldContainer[SettingsSection]
    cards: _containers.RepeatedCompositeFieldContainer[SettingsCard]
    def __init__(self, bootstrap_paths: _Optional[_Union[BootstrapPaths, _Mapping]] = ..., sections: _Optional[_Iterable[_Union[SettingsSection, _Mapping]]] = ..., cards: _Optional[_Iterable[_Union[SettingsCard, _Mapping]]] = ...) -> None: ...

@final
class SettingsSection(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SECTION_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    section_id: str
    label: str
    description: str
    owner: SettingsSectionOwner
    order: int
    def __init__(self, section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., owner: _Optional[_Union[SettingsSectionOwner, _Mapping]] = ..., order: _Optional[int] = ...) -> None: ...

@final
class SettingsSectionOwner(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    APP_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    app: bool
    installed_plugin_id: str
    def __init__(self, app: _Optional[bool] = ..., installed_plugin_id: _Optional[str] = ...) -> None: ...

@final
class TelemetryPreferences(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    interval_ms: int
    def __init__(self, interval_ms: _Optional[int] = ...) -> None: ...
