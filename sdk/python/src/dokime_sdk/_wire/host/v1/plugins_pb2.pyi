from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import description_pb2 as _description_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PluginState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PLUGIN_STATE_UNSPECIFIED: _ClassVar[PluginState]
    PLUGIN_STATE_CONNECTING: _ClassVar[PluginState]
    PLUGIN_STATE_READY: _ClassVar[PluginState]
    PLUGIN_STATE_UNAVAILABLE: _ClassVar[PluginState]
    PLUGIN_STATE_STOPPED: _ClassVar[PluginState]
    PLUGIN_STATE_STOPPING: _ClassVar[PluginState]
    PLUGIN_STATE_OFFLINE: _ClassVar[PluginState]
PLUGIN_STATE_UNSPECIFIED: PluginState
PLUGIN_STATE_CONNECTING: PluginState
PLUGIN_STATE_READY: PluginState
PLUGIN_STATE_UNAVAILABLE: PluginState
PLUGIN_STATE_STOPPED: PluginState
PLUGIN_STATE_STOPPING: PluginState
PLUGIN_STATE_OFFLINE: PluginState

@final
class PluginsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PluginsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    resolved_at: _position_pb2.ReadCut
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PluginsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PluginsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[PluginEntry]
    resolved_at: _position_pb2.ReadCut
    def __init__(self, entries: _Optional[_Iterable[_Union[PluginEntry, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PluginsWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class PluginsWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    CHANGE_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    resolved_at: _position_pb2.ReadCut
    snapshot: PluginSnapshot
    change: PluginChange
    resync_required: str
    def __init__(self, resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., snapshot: _Optional[_Union[PluginSnapshot, _Mapping]] = ..., change: _Optional[_Union[PluginChange, _Mapping]] = ..., resync_required: _Optional[str] = ...) -> None: ...

@final
class PluginsCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    SDK_VERSION_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    name: str
    placement: PluginPlacement
    value: PluginValue
    origin: str
    sdk_version: str
    def __init__(self, request_id: _Optional[str] = ..., name: _Optional[str] = ..., placement: _Optional[_Union[PluginPlacement, _Mapping]] = ..., value: _Optional[_Union[PluginValue, _Mapping]] = ..., origin: _Optional[str] = ..., sdk_version: _Optional[str] = ...) -> None: ...

@final
class PluginsCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    receipt: _receipt_pb2.Receipt
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class PluginsSetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: PluginValue
    expected_revision: bytes
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[PluginValue, _Mapping]] = ..., expected_revision: _Optional[bytes] = ...) -> None: ...

@final
class PluginsSetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    receipt: _receipt_pb2.Receipt
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class PluginsDeleteRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    name: str
    expected_revision: bytes
    def __init__(self, name: _Optional[str] = ..., expected_revision: _Optional[bytes] = ...) -> None: ...

@final
class PluginsDeleteResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class PluginsRestartRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    name: str
    expected_revision: bytes
    def __init__(self, name: _Optional[str] = ..., expected_revision: _Optional[bytes] = ...) -> None: ...

@final
class PluginsRestartResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ...) -> None: ...

@final
class PluginsReplaceRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    placement: PluginPlacement
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., placement: _Optional[_Union[PluginPlacement, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class PluginsReplaceResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    receipt: _receipt_pb2.Receipt
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class PluginValue(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    enabled: bool
    def __init__(self, enabled: _Optional[bool] = ...) -> None: ...

@final
class PluginDescription(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    DECLARATIONS_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    SDK_FIELD_NUMBER: _ClassVar[int]
    SDK_VERSION_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: str
    started_at: _timestamp_pb2.Timestamp
    declarations: _description_pb2.Declarations
    revision: int
    sdk: str
    sdk_version: str
    def __init__(self, id: _Optional[str] = ..., version: _Optional[str] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., declarations: _Optional[_Union[_description_pb2.Declarations, _Mapping]] = ..., revision: _Optional[int] = ..., sdk: _Optional[str] = ..., sdk_version: _Optional[str] = ...) -> None: ...

@final
class PluginRemote(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    URL_FIELD_NUMBER: _ClassVar[int]
    url: str
    def __init__(self, url: _Optional[str] = ...) -> None: ...

@final
class PluginManaged(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    content: _file_pb2.FileReference
    sha256: bytes
    def __init__(self, content: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., sha256: _Optional[bytes] = ...) -> None: ...

@final
class PluginGuest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...

@final
class PluginPlacement(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    MANAGED_FIELD_NUMBER: _ClassVar[int]
    REMOTE_FIELD_NUMBER: _ClassVar[int]
    GUEST_FIELD_NUMBER: _ClassVar[int]
    managed: PluginManaged
    remote: PluginRemote
    guest: PluginGuest
    def __init__(self, managed: _Optional[_Union[PluginManaged, _Mapping]] = ..., remote: _Optional[_Union[PluginRemote, _Mapping]] = ..., guest: _Optional[_Union[PluginGuest, _Mapping]] = ...) -> None: ...

@final
class PluginProcess(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTABLE_DIGEST_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    id: int
    executable_digest: bytes
    started_at: _timestamp_pb2.Timestamp
    def __init__(self, id: _Optional[int] = ..., executable_digest: _Optional[bytes] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

@final
class PluginEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DECLARATIONS_DIGEST_FIELD_NUMBER: _ClassVar[int]
    PROCESS_FIELD_NUMBER: _ClassVar[int]
    ACTIVATION_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    name: str
    placement: PluginPlacement
    value: PluginValue
    revision: bytes
    state: PluginState
    failure: _failure_pb2.Failure
    description: PluginDescription
    declarations_digest: bytes
    process: PluginProcess
    activation: int
    origin: str
    def __init__(self, name: _Optional[str] = ..., placement: _Optional[_Union[PluginPlacement, _Mapping]] = ..., value: _Optional[_Union[PluginValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., state: _Optional[_Union[PluginState, str]] = ..., failure: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ..., description: _Optional[_Union[PluginDescription, _Mapping]] = ..., declarations_digest: _Optional[bytes] = ..., process: _Optional[_Union[PluginProcess, _Mapping]] = ..., activation: _Optional[int] = ..., origin: _Optional[str] = ...) -> None: ...

@final
class PluginSnapshot(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[PluginEntry]
    def __init__(self, entries: _Optional[_Iterable[_Union[PluginEntry, _Mapping]]] = ...) -> None: ...

@final
class PluginChange(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    DELETED_FIELD_NUMBER: _ClassVar[int]
    entry: PluginEntry
    deleted: str
    def __init__(self, entry: _Optional[_Union[PluginEntry, _Mapping]] = ..., deleted: _Optional[str] = ...) -> None: ...
