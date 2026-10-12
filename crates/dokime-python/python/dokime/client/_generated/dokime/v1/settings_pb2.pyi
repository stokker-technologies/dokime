from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ThemeMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    THEME_MODE_UNSPECIFIED: _ClassVar[ThemeMode]
    THEME_MODE_SYSTEM: _ClassVar[ThemeMode]
    THEME_MODE_LIGHT: _ClassVar[ThemeMode]
    THEME_MODE_DARK: _ClassVar[ThemeMode]

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

class CredentialBackendKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CREDENTIAL_BACKEND_KIND_UNSPECIFIED: _ClassVar[CredentialBackendKind]
    CREDENTIAL_BACKEND_KIND_OS_KEYRING: _ClassVar[CredentialBackendKind]
    CREDENTIAL_BACKEND_KIND_MEMORY: _ClassVar[CredentialBackendKind]
    CREDENTIAL_BACKEND_KIND_LOCAL_FILE: _ClassVar[CredentialBackendKind]
THEME_MODE_UNSPECIFIED: ThemeMode
THEME_MODE_SYSTEM: ThemeMode
THEME_MODE_LIGHT: ThemeMode
THEME_MODE_DARK: ThemeMode
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
CREDENTIAL_BACKEND_KIND_UNSPECIFIED: CredentialBackendKind
CREDENTIAL_BACKEND_KIND_OS_KEYRING: CredentialBackendKind
CREDENTIAL_BACKEND_KIND_MEMORY: CredentialBackendKind
CREDENTIAL_BACKEND_KIND_LOCAL_FILE: CredentialBackendKind

class ReadApplicationSettingsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class UpdateAppearanceRequest(_message.Message):
    __slots__ = ("appearance",)
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    appearance: AppearancePreference
    def __init__(self, appearance: _Optional[_Union[AppearancePreference, _Mapping]] = ...) -> None: ...

class UpdateDataRootRequest(_message.Message):
    __slots__ = ("data_root",)
    DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    data_root: str
    def __init__(self, data_root: _Optional[str] = ...) -> None: ...

class AppearancePreference(_message.Message):
    __slots__ = ("theme_mode", "theme_family", "density")
    THEME_MODE_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    theme_mode: ThemeMode
    theme_family: str
    density: Density
    def __init__(self, theme_mode: _Optional[_Union[ThemeMode, str]] = ..., theme_family: _Optional[str] = ..., density: _Optional[_Union[Density, str]] = ...) -> None: ...

class ApplicationSettings(_message.Message):
    __slots__ = ("appearance", "application_root", "configured_data_root", "default_data_root", "effective_data_root", "restart_required")
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    APPLICATION_ROOT_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    EFFECTIVE_DATA_ROOT_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    appearance: AppearancePreference
    application_root: str
    configured_data_root: str
    default_data_root: str
    effective_data_root: str
    restart_required: bool
    def __init__(self, appearance: _Optional[_Union[AppearancePreference, _Mapping]] = ..., application_root: _Optional[str] = ..., configured_data_root: _Optional[str] = ..., default_data_root: _Optional[str] = ..., effective_data_root: _Optional[str] = ..., restart_required: _Optional[bool] = ...) -> None: ...

class ReadStoragePolicyRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class UpdateStoragePolicyRequest(_message.Message):
    __slots__ = ("expected_revision", "policy")
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    POLICY_FIELD_NUMBER: _ClassVar[int]
    expected_revision: int
    policy: StoragePolicy
    def __init__(self, expected_revision: _Optional[int] = ..., policy: _Optional[_Union[StoragePolicy, _Mapping]] = ...) -> None: ...

class StoragePolicy(_message.Message):
    __slots__ = ("save_interval_ms", "compaction_interval_ms", "local_retention_ms")
    SAVE_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    COMPACTION_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    LOCAL_RETENTION_MS_FIELD_NUMBER: _ClassVar[int]
    save_interval_ms: int
    compaction_interval_ms: int
    local_retention_ms: int
    def __init__(self, save_interval_ms: _Optional[int] = ..., compaction_interval_ms: _Optional[int] = ..., local_retention_ms: _Optional[int] = ...) -> None: ...

class StoragePolicyView(_message.Message):
    __slots__ = ("revision", "configured", "effective", "restart_required")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_FIELD_NUMBER: _ClassVar[int]
    EFFECTIVE_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    revision: int
    configured: StoragePolicy
    effective: StoragePolicy
    restart_required: bool
    def __init__(self, revision: _Optional[int] = ..., configured: _Optional[_Union[StoragePolicy, _Mapping]] = ..., effective: _Optional[_Union[StoragePolicy, _Mapping]] = ..., restart_required: _Optional[bool] = ...) -> None: ...

class GetSettingsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ListStorageSourcesRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReorderStorageSourcesRequest(_message.Message):
    __slots__ = ("identifiers",)
    IDENTIFIERS_FIELD_NUMBER: _ClassVar[int]
    identifiers: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, identifiers: _Optional[_Iterable[str]] = ...) -> None: ...

class SetStorageSourceEnabledRequest(_message.Message):
    __slots__ = ("identifier", "enabled")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    enabled: bool
    def __init__(self, identifier: _Optional[str] = ..., enabled: _Optional[bool] = ...) -> None: ...

class StorageSource(_message.Message):
    __slots__ = ("identifier", "adapter", "priority", "enabled")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    ADAPTER_FIELD_NUMBER: _ClassVar[int]
    PRIORITY_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    adapter: str
    priority: int
    enabled: bool
    def __init__(self, identifier: _Optional[str] = ..., adapter: _Optional[str] = ..., priority: _Optional[int] = ..., enabled: _Optional[bool] = ...) -> None: ...

class StorageSources(_message.Message):
    __slots__ = ("sources",)
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    sources: _containers.RepeatedCompositeFieldContainer[StorageSource]
    def __init__(self, sources: _Optional[_Iterable[_Union[StorageSource, _Mapping]]] = ...) -> None: ...

class SettingsFrame(_message.Message):
    __slots__ = ("sections", "cards")
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    sections: _containers.RepeatedCompositeFieldContainer[SettingsSection]
    cards: _containers.RepeatedCompositeFieldContainer[SettingsCard]
    def __init__(self, sections: _Optional[_Iterable[_Union[SettingsSection, _Mapping]]] = ..., cards: _Optional[_Iterable[_Union[SettingsCard, _Mapping]]] = ...) -> None: ...

class SettingsSection(_message.Message):
    __slots__ = ("section_id", "label", "description", "app", "installed_plugin_id", "order")
    SECTION_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    APP_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    section_id: str
    label: str
    description: str
    app: bool
    installed_plugin_id: str
    order: int
    def __init__(self, section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., app: _Optional[bool] = ..., installed_plugin_id: _Optional[str] = ..., order: _Optional[int] = ...) -> None: ...

class SettingsCard(_message.Message):
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
    body: HostCardBody
    refresh: SettingsCardRefreshPolicy
    state: SettingsCardState
    order: int
    def __init__(self, plugin_id: _Optional[str] = ..., card_id: _Optional[str] = ..., section_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., body: _Optional[_Union[HostCardBody, _Mapping]] = ..., refresh: _Optional[_Union[SettingsCardRefreshPolicy, _Mapping]] = ..., state: _Optional[_Union[SettingsCardState, str]] = ..., order: _Optional[int] = ...) -> None: ...

class HostCardBody(_message.Message):
    __slots__ = ("blocks",)
    BLOCKS_FIELD_NUMBER: _ClassVar[int]
    blocks: _containers.RepeatedCompositeFieldContainer[HostCardBlock]
    def __init__(self, blocks: _Optional[_Iterable[_Union[HostCardBlock, _Mapping]]] = ...) -> None: ...

class HostCardBlock(_message.Message):
    __slots__ = ("text", "status", "key_value_list", "metrics")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    KEY_VALUE_LIST_FIELD_NUMBER: _ClassVar[int]
    METRICS_FIELD_NUMBER: _ClassVar[int]
    text: str
    status: HostCardStatus
    key_value_list: HostCardKeyValueList
    metrics: HostCardMetrics
    def __init__(self, text: _Optional[str] = ..., status: _Optional[_Union[HostCardStatus, _Mapping]] = ..., key_value_list: _Optional[_Union[HostCardKeyValueList, _Mapping]] = ..., metrics: _Optional[_Union[HostCardMetrics, _Mapping]] = ...) -> None: ...

class HostCardStatus(_message.Message):
    __slots__ = ("label", "tone", "detail")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    tone: HostCardTone
    detail: str
    def __init__(self, label: _Optional[str] = ..., tone: _Optional[_Union[HostCardTone, str]] = ..., detail: _Optional[str] = ...) -> None: ...

class HostCardKeyValueList(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[HostCardKeyValue]
    def __init__(self, items: _Optional[_Iterable[_Union[HostCardKeyValue, _Mapping]]] = ...) -> None: ...

class HostCardKeyValue(_message.Message):
    __slots__ = ("label", "value", "detail")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    label: str
    value: str
    detail: str
    def __init__(self, label: _Optional[str] = ..., value: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class HostCardMetrics(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[HostCardMetric]
    def __init__(self, items: _Optional[_Iterable[_Union[HostCardMetric, _Mapping]]] = ...) -> None: ...

class HostCardMetric(_message.Message):
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

class SettingsCardRefreshPolicy(_message.Message):
    __slots__ = ("manual", "cached", "on_demand")
    MANUAL_FIELD_NUMBER: _ClassVar[int]
    CACHED_FIELD_NUMBER: _ClassVar[int]
    ON_DEMAND_FIELD_NUMBER: _ClassVar[int]
    manual: ManualRefresh
    cached: CachedRefresh
    on_demand: OnDemandRefresh
    def __init__(self, manual: _Optional[_Union[ManualRefresh, _Mapping]] = ..., cached: _Optional[_Union[CachedRefresh, _Mapping]] = ..., on_demand: _Optional[_Union[OnDemandRefresh, _Mapping]] = ...) -> None: ...

class ManualRefresh(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CachedRefresh(_message.Message):
    __slots__ = ("max_age_ms",)
    MAX_AGE_MS_FIELD_NUMBER: _ClassVar[int]
    max_age_ms: int
    def __init__(self, max_age_ms: _Optional[int] = ...) -> None: ...

class OnDemandRefresh(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class GetPluginSettingsRequest(_message.Message):
    __slots__ = ("plugin_id",)
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    def __init__(self, plugin_id: _Optional[str] = ...) -> None: ...

class UpdatePluginSettingsRequest(_message.Message):
    __slots__ = ("plugin_id", "values")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    values: _value_pb2.ValueObject
    def __init__(self, plugin_id: _Optional[str] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class UpdatePluginCredentialRequest(_message.Message):
    __slots__ = ("plugin_id", "credential_id", "secret")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    CREDENTIAL_ID_FIELD_NUMBER: _ClassVar[int]
    SECRET_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    credential_id: str
    secret: str
    def __init__(self, plugin_id: _Optional[str] = ..., credential_id: _Optional[str] = ..., secret: _Optional[str] = ...) -> None: ...

class PluginRuntimeSettings(_message.Message):
    __slots__ = ("settings", "values", "credential_bindings", "discovered")
    SETTINGS_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    CREDENTIAL_BINDINGS_FIELD_NUMBER: _ClassVar[int]
    DISCOVERED_FIELD_NUMBER: _ClassVar[int]
    settings: SettingsCatalog
    values: _value_pb2.ValueObject
    credential_bindings: PluginRuntimeCredentialBindings
    discovered: _value_pb2.ValueObject
    def __init__(self, settings: _Optional[_Union[SettingsCatalog, _Mapping]] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., credential_bindings: _Optional[_Union[PluginRuntimeCredentialBindings, _Mapping]] = ..., discovered: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class SettingsCatalog(_message.Message):
    __slots__ = ("contributions",)
    CONTRIBUTIONS_FIELD_NUMBER: _ClassVar[int]
    contributions: _containers.RepeatedCompositeFieldContainer[SettingContribution]
    def __init__(self, contributions: _Optional[_Iterable[_Union[SettingContribution, _Mapping]]] = ...) -> None: ...

class SettingContribution(_message.Message):
    __slots__ = ("boolean", "text_area", "path", "number", "multiple_choice", "secret_reference")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    TEXT_AREA_FIELD_NUMBER: _ClassVar[int]
    PATH_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_CHOICE_FIELD_NUMBER: _ClassVar[int]
    SECRET_REFERENCE_FIELD_NUMBER: _ClassVar[int]
    boolean: BooleanSetting
    text_area: TextAreaSetting
    path: PathSetting
    number: NumberSetting
    multiple_choice: MultipleChoiceSetting
    secret_reference: SecretSetting
    def __init__(self, boolean: _Optional[_Union[BooleanSetting, _Mapping]] = ..., text_area: _Optional[_Union[TextAreaSetting, _Mapping]] = ..., path: _Optional[_Union[PathSetting, _Mapping]] = ..., number: _Optional[_Union[NumberSetting, _Mapping]] = ..., multiple_choice: _Optional[_Union[MultipleChoiceSetting, _Mapping]] = ..., secret_reference: _Optional[_Union[SecretSetting, _Mapping]] = ...) -> None: ...

class BooleanSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[bool] = ...) -> None: ...

class TextAreaSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., required: _Optional[bool] = ...) -> None: ...

class PathSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., required: _Optional[bool] = ...) -> None: ...

class NumberSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default", "min", "max", "step")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: float
    min: float
    max: float
    step: float
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[float] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

class MultipleChoiceSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default", "choices", "provider")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    PROVIDER_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    choices: _containers.RepeatedCompositeFieldContainer[SettingChoice]
    provider: str
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., choices: _Optional[_Iterable[_Union[SettingChoice, _Mapping]]] = ..., provider: _Optional[str] = ...) -> None: ...

class SettingChoice(_message.Message):
    __slots__ = ("value", "title")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    value: str
    title: str
    def __init__(self, value: _Optional[str] = ..., title: _Optional[str] = ...) -> None: ...

class SecretSetting(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: SecretReference
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[_Union[SecretReference, _Mapping]] = ..., required: _Optional[bool] = ...) -> None: ...

class SecretReference(_message.Message):
    __slots__ = ("environment", "stored_credential")
    ENVIRONMENT_FIELD_NUMBER: _ClassVar[int]
    STORED_CREDENTIAL_FIELD_NUMBER: _ClassVar[int]
    environment: str
    stored_credential: str
    def __init__(self, environment: _Optional[str] = ..., stored_credential: _Optional[str] = ...) -> None: ...

class PluginRuntimeCredentialBindings(_message.Message):
    __slots__ = ("bindings", "missing_required")
    class BindingsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: RedactedCredentialBinding
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[RedactedCredentialBinding, _Mapping]] = ...) -> None: ...
    BINDINGS_FIELD_NUMBER: _ClassVar[int]
    MISSING_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    bindings: _containers.MessageMap[str, RedactedCredentialBinding]
    missing_required: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, bindings: _Optional[_Mapping[str, RedactedCredentialBinding]] = ..., missing_required: _Optional[_Iterable[str]] = ...) -> None: ...

class RedactedCredentialBinding(_message.Message):
    __slots__ = ("credential_id", "backend", "configured")
    CREDENTIAL_ID_FIELD_NUMBER: _ClassVar[int]
    BACKEND_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_FIELD_NUMBER: _ClassVar[int]
    credential_id: str
    backend: CredentialBackendKind
    configured: bool
    def __init__(self, credential_id: _Optional[str] = ..., backend: _Optional[_Union[CredentialBackendKind, str]] = ..., configured: _Optional[bool] = ...) -> None: ...
