import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Readiness(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    READINESS_UNSPECIFIED: _ClassVar[Readiness]
    READINESS_STARTING: _ClassVar[Readiness]
    READINESS_READY: _ClassVar[Readiness]
    READINESS_DEGRADED: _ClassVar[Readiness]
    READINESS_STOPPING: _ClassVar[Readiness]

class DiagnosticCheckStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DIAGNOSTIC_CHECK_STATUS_UNSPECIFIED: _ClassVar[DiagnosticCheckStatus]
    DIAGNOSTIC_CHECK_STATUS_OK: _ClassVar[DiagnosticCheckStatus]
    DIAGNOSTIC_CHECK_STATUS_WARNING: _ClassVar[DiagnosticCheckStatus]
    DIAGNOSTIC_CHECK_STATUS_FAILED: _ClassVar[DiagnosticCheckStatus]

class StartupDoctorCheckKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STARTUP_DOCTOR_CHECK_KEY_UNSPECIFIED: _ClassVar[StartupDoctorCheckKey]
    STARTUP_DOCTOR_CHECK_KEY_STORAGE_ROOT: _ClassVar[StartupDoctorCheckKey]
    STARTUP_DOCTOR_CHECK_KEY_SHUTDOWN_STATE: _ClassVar[StartupDoctorCheckKey]

class DiagnosticPathKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DIAGNOSTIC_PATH_KEY_UNSPECIFIED: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_HUB_LOG: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_STARTUP_DOCTOR: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_DIAGNOSTICS_SNAPSHOT: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_COLLECT_BUNDLE: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_CYCLE_LOG_PATTERN: _ClassVar[DiagnosticPathKey]
    DIAGNOSTIC_PATH_KEY_EXECUTION_LOG_PATTERN: _ClassVar[DiagnosticPathKey]

class DiagnosticPathScope(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DIAGNOSTIC_PATH_SCOPE_UNSPECIFIED: _ClassVar[DiagnosticPathScope]
    DIAGNOSTIC_PATH_SCOPE_ROOT: _ClassVar[DiagnosticPathScope]
    DIAGNOSTIC_PATH_SCOPE_CYCLE_PATTERN: _ClassVar[DiagnosticPathScope]
    DIAGNOSTIC_PATH_SCOPE_EXECUTION_PATTERN: _ClassVar[DiagnosticPathScope]

class DiagnosticFileFormat(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DIAGNOSTIC_FILE_FORMAT_UNSPECIFIED: _ClassVar[DiagnosticFileFormat]
    DIAGNOSTIC_FILE_FORMAT_JSON: _ClassVar[DiagnosticFileFormat]
    DIAGNOSTIC_FILE_FORMAT_JSONL: _ClassVar[DiagnosticFileFormat]
    DIAGNOSTIC_FILE_FORMAT_ZIP: _ClassVar[DiagnosticFileFormat]

class InstalledPluginPhase(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    INSTALLED_PLUGIN_PHASE_UNSPECIFIED: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_ENTRYPOINT: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_ACTION: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_SIDECAR: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_PROVIDER: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_INITIALIZE: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_RUN: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_FINALIZE: _ClassVar[InstalledPluginPhase]
    INSTALLED_PLUGIN_PHASE_INTERRUPT: _ClassVar[InstalledPluginPhase]

class LogLevel(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LOG_LEVEL_UNSPECIFIED: _ClassVar[LogLevel]
    LOG_LEVEL_TRACE: _ClassVar[LogLevel]
    LOG_LEVEL_DEBUG: _ClassVar[LogLevel]
    LOG_LEVEL_INFO: _ClassVar[LogLevel]
    LOG_LEVEL_WARN: _ClassVar[LogLevel]
    LOG_LEVEL_ERROR: _ClassVar[LogLevel]
READINESS_UNSPECIFIED: Readiness
READINESS_STARTING: Readiness
READINESS_READY: Readiness
READINESS_DEGRADED: Readiness
READINESS_STOPPING: Readiness
DIAGNOSTIC_CHECK_STATUS_UNSPECIFIED: DiagnosticCheckStatus
DIAGNOSTIC_CHECK_STATUS_OK: DiagnosticCheckStatus
DIAGNOSTIC_CHECK_STATUS_WARNING: DiagnosticCheckStatus
DIAGNOSTIC_CHECK_STATUS_FAILED: DiagnosticCheckStatus
STARTUP_DOCTOR_CHECK_KEY_UNSPECIFIED: StartupDoctorCheckKey
STARTUP_DOCTOR_CHECK_KEY_STORAGE_ROOT: StartupDoctorCheckKey
STARTUP_DOCTOR_CHECK_KEY_SHUTDOWN_STATE: StartupDoctorCheckKey
DIAGNOSTIC_PATH_KEY_UNSPECIFIED: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_HUB_LOG: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_STARTUP_DOCTOR: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_DIAGNOSTICS_SNAPSHOT: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_COLLECT_BUNDLE: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_CYCLE_LOG_PATTERN: DiagnosticPathKey
DIAGNOSTIC_PATH_KEY_EXECUTION_LOG_PATTERN: DiagnosticPathKey
DIAGNOSTIC_PATH_SCOPE_UNSPECIFIED: DiagnosticPathScope
DIAGNOSTIC_PATH_SCOPE_ROOT: DiagnosticPathScope
DIAGNOSTIC_PATH_SCOPE_CYCLE_PATTERN: DiagnosticPathScope
DIAGNOSTIC_PATH_SCOPE_EXECUTION_PATTERN: DiagnosticPathScope
DIAGNOSTIC_FILE_FORMAT_UNSPECIFIED: DiagnosticFileFormat
DIAGNOSTIC_FILE_FORMAT_JSON: DiagnosticFileFormat
DIAGNOSTIC_FILE_FORMAT_JSONL: DiagnosticFileFormat
DIAGNOSTIC_FILE_FORMAT_ZIP: DiagnosticFileFormat
INSTALLED_PLUGIN_PHASE_UNSPECIFIED: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_ENTRYPOINT: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_ACTION: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_SIDECAR: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_PROVIDER: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_INITIALIZE: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_RUN: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_FINALIZE: InstalledPluginPhase
INSTALLED_PLUGIN_PHASE_INTERRUPT: InstalledPluginPhase
LOG_LEVEL_UNSPECIFIED: LogLevel
LOG_LEVEL_TRACE: LogLevel
LOG_LEVEL_DEBUG: LogLevel
LOG_LEVEL_INFO: LogLevel
LOG_LEVEL_WARN: LogLevel
LOG_LEVEL_ERROR: LogLevel

class GetIdentityRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class GetHealthRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class GetCapabilitiesRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class GetDiagnosticsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CollectDiagnosticsRequest(_message.Message):
    __slots__ = ("max_log_tail_bytes",)
    MAX_LOG_TAIL_BYTES_FIELD_NUMBER: _ClassVar[int]
    max_log_tail_bytes: int
    def __init__(self, max_log_tail_bytes: _Optional[int] = ...) -> None: ...

class WatchLogsRequest(_message.Message):
    __slots__ = ("min_level", "replay_lines")
    MIN_LEVEL_FIELD_NUMBER: _ClassVar[int]
    REPLAY_LINES_FIELD_NUMBER: _ClassVar[int]
    min_level: LogLevel
    replay_lines: int
    def __init__(self, min_level: _Optional[_Union[LogLevel, str]] = ..., replay_lines: _Optional[int] = ...) -> None: ...

class BackendIdentity(_message.Message):
    __slots__ = ("backend_id", "epoch", "api", "product_version", "source_revision", "sdk_revision", "started_at", "display_name")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    API_FIELD_NUMBER: _ClassVar[int]
    PRODUCT_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_REVISION_FIELD_NUMBER: _ClassVar[int]
    SDK_REVISION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    api: ApiIdentity
    product_version: str
    source_revision: str
    sdk_revision: str
    started_at: _timestamp_pb2.Timestamp
    display_name: str
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., api: _Optional[_Union[ApiIdentity, _Mapping]] = ..., product_version: _Optional[str] = ..., source_revision: _Optional[str] = ..., sdk_revision: _Optional[str] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., display_name: _Optional[str] = ...) -> None: ...

class ApiIdentity(_message.Message):
    __slots__ = ("package", "version", "descriptor_sha256")
    PACKAGE_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTOR_SHA256_FIELD_NUMBER: _ClassVar[int]
    package: str
    version: str
    descriptor_sha256: bytes
    def __init__(self, package: _Optional[str] = ..., version: _Optional[str] = ..., descriptor_sha256: _Optional[bytes] = ...) -> None: ...

class BackendHealth(_message.Message):
    __slots__ = ("backend_id", "epoch", "readiness", "observed_at", "message")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    READINESS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    readiness: Readiness
    observed_at: _timestamp_pb2.Timestamp
    message: str
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., readiness: _Optional[_Union[Readiness, str]] = ..., observed_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class BackendCapabilities(_message.Message):
    __slots__ = ("backend_id", "epoch", "methods", "limits")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    METHODS_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    methods: _containers.RepeatedScalarFieldContainer[str]
    limits: BackendLimits
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., methods: _Optional[_Iterable[str]] = ..., limits: _Optional[_Union[BackendLimits, _Mapping]] = ...) -> None: ...

class BackendLimits(_message.Message):
    __slots__ = ("max_request_bytes", "max_response_bytes", "max_content_bytes", "max_chunk_bytes", "max_page_items", "max_stream_buffer_items")
    MAX_REQUEST_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_RESPONSE_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_CONTENT_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_PAGE_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_STREAM_BUFFER_ITEMS_FIELD_NUMBER: _ClassVar[int]
    max_request_bytes: int
    max_response_bytes: int
    max_content_bytes: int
    max_chunk_bytes: int
    max_page_items: int
    max_stream_buffer_items: int
    def __init__(self, max_request_bytes: _Optional[int] = ..., max_response_bytes: _Optional[int] = ..., max_content_bytes: _Optional[int] = ..., max_chunk_bytes: _Optional[int] = ..., max_page_items: _Optional[int] = ..., max_stream_buffer_items: _Optional[int] = ...) -> None: ...

class DiagnosticsResponse(_message.Message):
    __slots__ = ("server", "runtime")
    SERVER_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_FIELD_NUMBER: _ClassVar[int]
    server: BackendDiagnostics
    runtime: RuntimeDiagnostics
    def __init__(self, server: _Optional[_Union[BackendDiagnostics, _Mapping]] = ..., runtime: _Optional[_Union[RuntimeDiagnostics, _Mapping]] = ...) -> None: ...

class BackendDiagnostics(_message.Message):
    __slots__ = ("version", "process")
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PROCESS_FIELD_NUMBER: _ClassVar[int]
    version: str
    process: ProcessDiagnostics
    def __init__(self, version: _Optional[str] = ..., process: _Optional[_Union[ProcessDiagnostics, _Mapping]] = ...) -> None: ...

class ProcessDiagnostics(_message.Message):
    __slots__ = ("pid", "cpu_percent", "memory_bytes", "virtual_memory_bytes")
    PID_FIELD_NUMBER: _ClassVar[int]
    CPU_PERCENT_FIELD_NUMBER: _ClassVar[int]
    MEMORY_BYTES_FIELD_NUMBER: _ClassVar[int]
    VIRTUAL_MEMORY_BYTES_FIELD_NUMBER: _ClassVar[int]
    pid: int
    cpu_percent: float
    memory_bytes: int
    virtual_memory_bytes: int
    def __init__(self, pid: _Optional[int] = ..., cpu_percent: _Optional[float] = ..., memory_bytes: _Optional[int] = ..., virtual_memory_bytes: _Optional[int] = ...) -> None: ...

class RuntimeDiagnostics(_message.Message):
    __slots__ = ("captured_at", "session_id", "identity", "revision", "shutting_down", "storage_root", "hub_metadata", "startup_doctor", "path_conventions", "counts", "installed_plugin_errors")
    CAPTURED_AT_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    SHUTTING_DOWN_FIELD_NUMBER: _ClassVar[int]
    STORAGE_ROOT_FIELD_NUMBER: _ClassVar[int]
    HUB_METADATA_FIELD_NUMBER: _ClassVar[int]
    STARTUP_DOCTOR_FIELD_NUMBER: _ClassVar[int]
    PATH_CONVENTIONS_FIELD_NUMBER: _ClassVar[int]
    COUNTS_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_PLUGIN_ERRORS_FIELD_NUMBER: _ClassVar[int]
    captured_at: _timestamp_pb2.Timestamp
    session_id: str
    identity: RuntimeIdentity
    revision: int
    shutting_down: bool
    storage_root: str
    hub_metadata: _common_pb2.Metadata
    startup_doctor: StartupDoctorSnapshot
    path_conventions: _containers.RepeatedCompositeFieldContainer[DiagnosticPathConvention]
    counts: DiagnosticsCounts
    installed_plugin_errors: _containers.RepeatedCompositeFieldContainer[InstalledPluginErrorRecord]
    def __init__(self, captured_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., session_id: _Optional[str] = ..., identity: _Optional[_Union[RuntimeIdentity, _Mapping]] = ..., revision: _Optional[int] = ..., shutting_down: _Optional[bool] = ..., storage_root: _Optional[str] = ..., hub_metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., startup_doctor: _Optional[_Union[StartupDoctorSnapshot, _Mapping]] = ..., path_conventions: _Optional[_Iterable[_Union[DiagnosticPathConvention, _Mapping]]] = ..., counts: _Optional[_Union[DiagnosticsCounts, _Mapping]] = ..., installed_plugin_errors: _Optional[_Iterable[_Union[InstalledPluginErrorRecord, _Mapping]]] = ...) -> None: ...

class RuntimeIdentity(_message.Message):
    __slots__ = ("node", "actor")
    NODE_FIELD_NUMBER: _ClassVar[int]
    ACTOR_FIELD_NUMBER: _ClassVar[int]
    node: RuntimeNodeIdentity
    actor: RuntimeActorIdentity
    def __init__(self, node: _Optional[_Union[RuntimeNodeIdentity, _Mapping]] = ..., actor: _Optional[_Union[RuntimeActorIdentity, _Mapping]] = ...) -> None: ...

class RuntimeNodeIdentity(_message.Message):
    __slots__ = ("node_id", "hostname_label", "created_at", "updated_at")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    HOSTNAME_LABEL_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    hostname_label: str
    created_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    def __init__(self, node_id: _Optional[str] = ..., hostname_label: _Optional[str] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class RuntimeActorIdentity(_message.Message):
    __slots__ = ("actor_id", "display_name")
    ACTOR_ID_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    actor_id: str
    display_name: str
    def __init__(self, actor_id: _Optional[str] = ..., display_name: _Optional[str] = ...) -> None: ...

class StartupDoctorSnapshot(_message.Message):
    __slots__ = ("checked_at", "status", "checks")
    CHECKED_AT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    CHECKS_FIELD_NUMBER: _ClassVar[int]
    checked_at: _timestamp_pb2.Timestamp
    status: DiagnosticCheckStatus
    checks: _containers.RepeatedCompositeFieldContainer[StartupDoctorCheck]
    def __init__(self, checked_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., status: _Optional[_Union[DiagnosticCheckStatus, str]] = ..., checks: _Optional[_Iterable[_Union[StartupDoctorCheck, _Mapping]]] = ...) -> None: ...

class StartupDoctorCheck(_message.Message):
    __slots__ = ("key", "status", "message", "detail")
    KEY_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    key: StartupDoctorCheckKey
    status: DiagnosticCheckStatus
    message: str
    detail: str
    def __init__(self, key: _Optional[_Union[StartupDoctorCheckKey, str]] = ..., status: _Optional[_Union[DiagnosticCheckStatus, str]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class DiagnosticPathConvention(_message.Message):
    __slots__ = ("key", "scope", "format", "relative_path", "absolute_path")
    KEY_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTE_PATH_FIELD_NUMBER: _ClassVar[int]
    key: DiagnosticPathKey
    scope: DiagnosticPathScope
    format: DiagnosticFileFormat
    relative_path: str
    absolute_path: str
    def __init__(self, key: _Optional[_Union[DiagnosticPathKey, str]] = ..., scope: _Optional[_Union[DiagnosticPathScope, str]] = ..., format: _Optional[_Union[DiagnosticFileFormat, str]] = ..., relative_path: _Optional[str] = ..., absolute_path: _Optional[str] = ...) -> None: ...

class DiagnosticsCounts(_message.Message):
    __slots__ = ("test_suites", "cycles", "installed_plugin_errors", "state_cycles", "state_executions_total", "state_record_count", "telemetry_cursor_entries")
    TEST_SUITES_FIELD_NUMBER: _ClassVar[int]
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    INSTALLED_PLUGIN_ERRORS_FIELD_NUMBER: _ClassVar[int]
    STATE_CYCLES_FIELD_NUMBER: _ClassVar[int]
    STATE_EXECUTIONS_TOTAL_FIELD_NUMBER: _ClassVar[int]
    STATE_RECORD_COUNT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_CURSOR_ENTRIES_FIELD_NUMBER: _ClassVar[int]
    test_suites: int
    cycles: int
    installed_plugin_errors: int
    state_cycles: int
    state_executions_total: int
    state_record_count: int
    telemetry_cursor_entries: int
    def __init__(self, test_suites: _Optional[int] = ..., cycles: _Optional[int] = ..., installed_plugin_errors: _Optional[int] = ..., state_cycles: _Optional[int] = ..., state_executions_total: _Optional[int] = ..., state_record_count: _Optional[int] = ..., telemetry_cursor_entries: _Optional[int] = ...) -> None: ...

class InstalledPluginErrorRecord(_message.Message):
    __slots__ = ("plugin_id", "contribution_id", "phase", "level", "message", "code", "source", "cause_chain", "stacktrace_attachment", "trace_id", "scope", "session_id", "test_cycle_id", "test_execution_id", "recorded_at")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    CONTRIBUTION_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    CAUSE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    STACKTRACE_ATTACHMENT_FIELD_NUMBER: _ClassVar[int]
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    TEST_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RECORDED_AT_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    contribution_id: str
    phase: InstalledPluginPhase
    level: _common_pb2.Severity
    message: str
    code: str
    source: str
    cause_chain: _containers.RepeatedCompositeFieldContainer[ErrorCause]
    stacktrace_attachment: str
    trace_id: str
    scope: _common_pb2.EvidenceScope
    session_id: str
    test_cycle_id: str
    test_execution_id: str
    recorded_at: _timestamp_pb2.Timestamp
    def __init__(self, plugin_id: _Optional[str] = ..., contribution_id: _Optional[str] = ..., phase: _Optional[_Union[InstalledPluginPhase, str]] = ..., level: _Optional[_Union[_common_pb2.Severity, str]] = ..., message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[ErrorCause, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., session_id: _Optional[str] = ..., test_cycle_id: _Optional[str] = ..., test_execution_id: _Optional[str] = ..., recorded_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ErrorCause(_message.Message):
    __slots__ = ("message", "code", "source")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

class DiagnosticsBundleFrame(_message.Message):
    __slots__ = ("header", "chunk", "complete")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    header: DiagnosticsBundleHeader
    chunk: DiagnosticsBundleChunk
    complete: DiagnosticsBundleCompletion
    def __init__(self, header: _Optional[_Union[DiagnosticsBundleHeader, _Mapping]] = ..., chunk: _Optional[_Union[DiagnosticsBundleChunk, _Mapping]] = ..., complete: _Optional[_Union[DiagnosticsBundleCompletion, _Mapping]] = ...) -> None: ...

class DiagnosticsBundleHeader(_message.Message):
    __slots__ = ("filename", "content_type", "generated_at", "total_size_bytes", "entries")
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    TOTAL_SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    filename: str
    content_type: str
    generated_at: _timestamp_pb2.Timestamp
    total_size_bytes: int
    entries: _containers.RepeatedCompositeFieldContainer[DiagnosticsBundleEntry]
    def __init__(self, filename: _Optional[str] = ..., content_type: _Optional[str] = ..., generated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., total_size_bytes: _Optional[int] = ..., entries: _Optional[_Iterable[_Union[DiagnosticsBundleEntry, _Mapping]]] = ...) -> None: ...

class DiagnosticsBundleEntry(_message.Message):
    __slots__ = ("path", "size_bytes", "present", "note")
    PATH_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    PRESENT_FIELD_NUMBER: _ClassVar[int]
    NOTE_FIELD_NUMBER: _ClassVar[int]
    path: str
    size_bytes: int
    present: bool
    note: str
    def __init__(self, path: _Optional[str] = ..., size_bytes: _Optional[int] = ..., present: _Optional[bool] = ..., note: _Optional[str] = ...) -> None: ...

class DiagnosticsBundleChunk(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class DiagnosticsBundleCompletion(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class LogFrame(_message.Message):
    __slots__ = ("time", "level", "target", "message", "fields", "dropped")
    class FieldsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    TIME_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    DROPPED_FIELD_NUMBER: _ClassVar[int]
    time: _timestamp_pb2.Timestamp
    level: LogLevel
    target: str
    message: str
    fields: _containers.ScalarMap[str, str]
    dropped: int
    def __init__(self, time: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., level: _Optional[_Union[LogLevel, str]] = ..., target: _Optional[str] = ..., message: _Optional[str] = ..., fields: _Optional[_Mapping[str, str]] = ..., dropped: _Optional[int] = ...) -> None: ...
