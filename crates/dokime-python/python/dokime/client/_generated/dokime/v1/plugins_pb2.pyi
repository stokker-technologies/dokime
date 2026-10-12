import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PluginInstallationState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLUGIN_INSTALLATION_STATE_UNSPECIFIED: _ClassVar[PluginInstallationState]
    PLUGIN_INSTALLATION_STATE_STAGED: _ClassVar[PluginInstallationState]
    PLUGIN_INSTALLATION_STATE_INSTALLED: _ClassVar[PluginInstallationState]
    PLUGIN_INSTALLATION_STATE_DISABLED: _ClassVar[PluginInstallationState]
    PLUGIN_INSTALLATION_STATE_REMOVAL_PENDING: _ClassVar[PluginInstallationState]
PLUGIN_INSTALLATION_STATE_UNSPECIFIED: PluginInstallationState
PLUGIN_INSTALLATION_STATE_STAGED: PluginInstallationState
PLUGIN_INSTALLATION_STATE_INSTALLED: PluginInstallationState
PLUGIN_INSTALLATION_STATE_DISABLED: PluginInstallationState
PLUGIN_INSTALLATION_STATE_REMOVAL_PENDING: PluginInstallationState

class InstallationCursor(_message.Message):
    __slots__ = ("manifest_sha256", "after_install_id")
    MANIFEST_SHA256_FIELD_NUMBER: _ClassVar[int]
    AFTER_INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    manifest_sha256: str
    after_install_id: str
    def __init__(self, manifest_sha256: _Optional[str] = ..., after_install_id: _Optional[str] = ...) -> None: ...

class ListInstallationsRequest(_message.Message):
    __slots__ = ("page_size", "after")
    PAGE_SIZE_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    page_size: int
    after: InstallationCursor
    def __init__(self, page_size: _Optional[int] = ..., after: _Optional[_Union[InstallationCursor, _Mapping]] = ...) -> None: ...

class InstallationPage(_message.Message):
    __slots__ = ("installations", "next", "total_count", "manifest_sha256")
    INSTALLATIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    MANIFEST_SHA256_FIELD_NUMBER: _ClassVar[int]
    installations: _containers.RepeatedCompositeFieldContainer[PluginInstallation]
    next: InstallationCursor
    total_count: int
    manifest_sha256: str
    def __init__(self, installations: _Optional[_Iterable[_Union[PluginInstallation, _Mapping]]] = ..., next: _Optional[_Union[InstallationCursor, _Mapping]] = ..., total_count: _Optional[int] = ..., manifest_sha256: _Optional[str] = ...) -> None: ...

class GetInstallationRequest(_message.Message):
    __slots__ = ("install_id",)
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    def __init__(self, install_id: _Optional[str] = ...) -> None: ...

class ConsentPluginRequest(_message.Message):
    __slots__ = ("install_id", "expected_sha256")
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_SHA256_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    expected_sha256: str
    def __init__(self, install_id: _Optional[str] = ..., expected_sha256: _Optional[str] = ...) -> None: ...

class SetPluginEnabledRequest(_message.Message):
    __slots__ = ("install_id", "enabled")
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    enabled: bool
    def __init__(self, install_id: _Optional[str] = ..., enabled: _Optional[bool] = ...) -> None: ...

class RemovePluginRequest(_message.Message):
    __slots__ = ("install_id",)
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    def __init__(self, install_id: _Optional[str] = ...) -> None: ...

class RemovePluginReply(_message.Message):
    __slots__ = ("installation",)
    INSTALLATION_FIELD_NUMBER: _ClassVar[int]
    installation: PluginInstallation
    def __init__(self, installation: _Optional[_Union[PluginInstallation, _Mapping]] = ...) -> None: ...

class StagePluginFrame(_message.Message):
    __slots__ = ("header", "chunk")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    header: StagePluginHeader
    chunk: StagePluginChunk
    def __init__(self, header: _Optional[_Union[StagePluginHeader, _Mapping]] = ..., chunk: _Optional[_Union[StagePluginChunk, _Mapping]] = ...) -> None: ...

class StagePluginHeader(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class StagePluginChunk(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class PluginIdentity(_message.Message):
    __slots__ = ("name", "version", "description")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    name: str
    version: str
    description: str
    def __init__(self, name: _Optional[str] = ..., version: _Optional[str] = ..., description: _Optional[str] = ...) -> None: ...

class PluginEnvironmentReference(_message.Message):
    __slots__ = ("owner", "setting", "variable")
    OWNER_FIELD_NUMBER: _ClassVar[int]
    SETTING_FIELD_NUMBER: _ClassVar[int]
    VARIABLE_FIELD_NUMBER: _ClassVar[int]
    owner: str
    setting: str
    variable: str
    def __init__(self, owner: _Optional[str] = ..., setting: _Optional[str] = ..., variable: _Optional[str] = ...) -> None: ...

class PluginMetadata(_message.Message):
    __slots__ = ("identity", "owners", "size_bytes", "environment_references")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    OWNERS_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    ENVIRONMENT_REFERENCES_FIELD_NUMBER: _ClassVar[int]
    identity: PluginIdentity
    owners: _containers.RepeatedScalarFieldContainer[str]
    size_bytes: int
    environment_references: _containers.RepeatedCompositeFieldContainer[PluginEnvironmentReference]
    def __init__(self, identity: _Optional[_Union[PluginIdentity, _Mapping]] = ..., owners: _Optional[_Iterable[str]] = ..., size_bytes: _Optional[int] = ..., environment_references: _Optional[_Iterable[_Union[PluginEnvironmentReference, _Mapping]]] = ...) -> None: ...

class PluginLastBoot(_message.Message):
    __slots__ = ("at", "loaded", "error")
    AT_FIELD_NUMBER: _ClassVar[int]
    LOADED_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    at: _timestamp_pb2.Timestamp
    loaded: bool
    error: str
    def __init__(self, at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., loaded: _Optional[bool] = ..., error: _Optional[str] = ...) -> None: ...

class PluginInstallation(_message.Message):
    __slots__ = ("install_id", "sha256", "staged_at", "consented_at", "state", "last_boot", "metadata", "error", "active", "restart_required", "boot_error")
    INSTALL_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    STAGED_AT_FIELD_NUMBER: _ClassVar[int]
    CONSENTED_AT_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    LAST_BOOT_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    BOOT_ERROR_FIELD_NUMBER: _ClassVar[int]
    install_id: str
    sha256: str
    staged_at: _timestamp_pb2.Timestamp
    consented_at: _timestamp_pb2.Timestamp
    state: PluginInstallationState
    last_boot: PluginLastBoot
    metadata: PluginMetadata
    error: str
    active: bool
    restart_required: bool
    boot_error: str
    def __init__(self, install_id: _Optional[str] = ..., sha256: _Optional[str] = ..., staged_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., consented_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., state: _Optional[_Union[PluginInstallationState, str]] = ..., last_boot: _Optional[_Union[PluginLastBoot, _Mapping]] = ..., metadata: _Optional[_Union[PluginMetadata, _Mapping]] = ..., error: _Optional[str] = ..., active: _Optional[bool] = ..., restart_required: _Optional[bool] = ..., boot_error: _Optional[str] = ...) -> None: ...

class AdmittedPluginCursor(_message.Message):
    __slots__ = ("epoch", "revision", "after_plugin_id")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    AFTER_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    revision: int
    after_plugin_id: str
    def __init__(self, epoch: _Optional[str] = ..., revision: _Optional[int] = ..., after_plugin_id: _Optional[str] = ...) -> None: ...

class ListAdmittedPluginsRequest(_message.Message):
    __slots__ = ("page_size", "after")
    PAGE_SIZE_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    page_size: int
    after: AdmittedPluginCursor
    def __init__(self, page_size: _Optional[int] = ..., after: _Optional[_Union[AdmittedPluginCursor, _Mapping]] = ...) -> None: ...

class AdmittedPluginPage(_message.Message):
    __slots__ = ("plugins", "next", "total_count", "epoch", "revision")
    PLUGINS_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    plugins: _containers.RepeatedCompositeFieldContainer[AdmittedPlugin]
    next: AdmittedPluginCursor
    total_count: int
    epoch: str
    revision: int
    def __init__(self, plugins: _Optional[_Iterable[_Union[AdmittedPlugin, _Mapping]]] = ..., next: _Optional[_Union[AdmittedPluginCursor, _Mapping]] = ..., total_count: _Optional[int] = ..., epoch: _Optional[str] = ..., revision: _Optional[int] = ...) -> None: ...

class AdmittedBundleOwner(_message.Message):
    __slots__ = ("package", "installation_id", "content_digest")
    PACKAGE_FIELD_NUMBER: _ClassVar[int]
    INSTALLATION_ID_FIELD_NUMBER: _ClassVar[int]
    CONTENT_DIGEST_FIELD_NUMBER: _ClassVar[int]
    package: str
    installation_id: str
    content_digest: str
    def __init__(self, package: _Optional[str] = ..., installation_id: _Optional[str] = ..., content_digest: _Optional[str] = ...) -> None: ...

class AdmittedEmbeddedOwner(_message.Message):
    __slots__ = ("generation",)
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    generation: str
    def __init__(self, generation: _Optional[str] = ...) -> None: ...

class AdmittedPlugin(_message.Message):
    __slots__ = ("plugin_id", "label", "version", "enabled", "host", "bundle", "embedded")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    HOST_FIELD_NUMBER: _ClassVar[int]
    BUNDLE_FIELD_NUMBER: _ClassVar[int]
    EMBEDDED_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    label: str
    version: str
    enabled: bool
    host: bool
    bundle: AdmittedBundleOwner
    embedded: AdmittedEmbeddedOwner
    def __init__(self, plugin_id: _Optional[str] = ..., label: _Optional[str] = ..., version: _Optional[str] = ..., enabled: _Optional[bool] = ..., host: _Optional[bool] = ..., bundle: _Optional[_Union[AdmittedBundleOwner, _Mapping]] = ..., embedded: _Optional[_Union[AdmittedEmbeddedOwner, _Mapping]] = ...) -> None: ...
