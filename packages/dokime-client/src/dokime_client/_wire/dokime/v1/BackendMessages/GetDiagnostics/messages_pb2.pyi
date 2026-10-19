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
    __slots__ = ("server", "runtime")
    SERVER_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_FIELD_NUMBER: _ClassVar[int]
    server: Row3
    runtime: Row4
    def __init__(self, server: _Optional[_Union[Row3, _Mapping]] = ..., runtime: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("version", "process")
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PROCESS_FIELD_NUMBER: _ClassVar[int]
    version: str
    process: Row5
    def __init__(self, version: _Optional[str] = ..., process: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
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
    captured_at: Row6
    session_id: str
    identity: Row7
    revision: int
    shutting_down: bool
    storage_root: str
    hub_metadata: Row8
    startup_doctor: Row9
    path_conventions: _containers.RepeatedCompositeFieldContainer[Row10]
    counts: Row11
    installed_plugin_errors: _containers.RepeatedCompositeFieldContainer[Row12]
    def __init__(self, captured_at: _Optional[_Union[Row6, _Mapping]] = ..., session_id: _Optional[str] = ..., identity: _Optional[_Union[Row7, _Mapping]] = ..., revision: _Optional[int] = ..., shutting_down: _Optional[bool] = ..., storage_root: _Optional[str] = ..., hub_metadata: _Optional[_Union[Row8, _Mapping]] = ..., startup_doctor: _Optional[_Union[Row9, _Mapping]] = ..., path_conventions: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., counts: _Optional[_Union[Row11, _Mapping]] = ..., installed_plugin_errors: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ...) -> None: ...

class Row5(_message.Message):
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

class Row6(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("node", "actor")
    NODE_FIELD_NUMBER: _ClassVar[int]
    ACTOR_FIELD_NUMBER: _ClassVar[int]
    node: Row13
    actor: Row14
    def __init__(self, node: _Optional[_Union[Row13, _Mapping]] = ..., actor: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("identifier", "label", "description", "tags", "attributes", "created_at", "updated_at")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    tags: _containers.RepeatedScalarFieldContainer[str]
    attributes: _containers.RepeatedCompositeFieldContainer[Row15]
    created_at: Row6
    updated_at: Row6
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row15, _Mapping]]] = ..., created_at: _Optional[_Union[Row6, _Mapping]] = ..., updated_at: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("checked_at", "status", "checks")
    CHECKED_AT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    CHECKS_FIELD_NUMBER: _ClassVar[int]
    checked_at: Row6
    status: Row16
    checks: _containers.RepeatedCompositeFieldContainer[Row17]
    def __init__(self, checked_at: _Optional[_Union[Row6, _Mapping]] = ..., status: _Optional[_Union[Row16, _Mapping]] = ..., checks: _Optional[_Iterable[_Union[Row17, _Mapping]]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("key", "scope", "format", "relative_path", "absolute_path")
    KEY_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTE_PATH_FIELD_NUMBER: _ClassVar[int]
    key: Row18
    scope: Row19
    format: Row20
    relative_path: str
    absolute_path: str
    def __init__(self, key: _Optional[_Union[Row18, _Mapping]] = ..., scope: _Optional[_Union[Row19, _Mapping]] = ..., format: _Optional[_Union[Row20, _Mapping]] = ..., relative_path: _Optional[str] = ..., absolute_path: _Optional[str] = ...) -> None: ...

class Row11(_message.Message):
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

class Row12(_message.Message):
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
    phase: Row21
    level: Row22
    message: str
    code: str
    source: str
    cause_chain: _containers.RepeatedCompositeFieldContainer[Row23]
    stacktrace_attachment: str
    trace_id: str
    scope: Row24
    session_id: str
    test_cycle_id: str
    test_execution_id: str
    recorded_at: Row6
    def __init__(self, plugin_id: _Optional[str] = ..., contribution_id: _Optional[str] = ..., phase: _Optional[_Union[Row21, _Mapping]] = ..., level: _Optional[_Union[Row22, _Mapping]] = ..., message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[Row23, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., scope: _Optional[_Union[Row24, _Mapping]] = ..., session_id: _Optional[str] = ..., test_cycle_id: _Optional[str] = ..., test_execution_id: _Optional[str] = ..., recorded_at: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("node_id", "hostname_label", "created_at", "updated_at")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    HOSTNAME_LABEL_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    hostname_label: str
    created_at: Row6
    updated_at: Row6
    def __init__(self, node_id: _Optional[str] = ..., hostname_label: _Optional[str] = ..., created_at: _Optional[_Union[Row6, _Mapping]] = ..., updated_at: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("actor_id", "display_name")
    ACTOR_ID_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    actor_id: str
    display_name: str
    def __init__(self, actor_id: _Optional[str] = ..., display_name: _Optional[str] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row25
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Ok", "Warning", "Failed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OK_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row26
    Ok: Row27
    Warning: Row28
    Failed: Row29
    def __init__(self, Unspecified: _Optional[_Union[Row26, _Mapping]] = ..., Ok: _Optional[_Union[Row27, _Mapping]] = ..., Warning: _Optional[_Union[Row28, _Mapping]] = ..., Failed: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("key", "status", "message", "detail")
    KEY_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    key: Row30
    status: Row16
    message: str
    detail: str
    def __init__(self, key: _Optional[_Union[Row30, _Mapping]] = ..., status: _Optional[_Union[Row16, _Mapping]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("Unspecified", "HubLog", "StartupDoctor", "DiagnosticsSnapshot", "CollectBundle", "CycleLogPattern", "ExecutionLogPattern")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    HUBLOG_FIELD_NUMBER: _ClassVar[int]
    STARTUPDOCTOR_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICSSNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    COLLECTBUNDLE_FIELD_NUMBER: _ClassVar[int]
    CYCLELOGPATTERN_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONLOGPATTERN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row31
    HubLog: Row32
    StartupDoctor: Row33
    DiagnosticsSnapshot: Row34
    CollectBundle: Row35
    CycleLogPattern: Row36
    ExecutionLogPattern: Row37
    def __init__(self, Unspecified: _Optional[_Union[Row31, _Mapping]] = ..., HubLog: _Optional[_Union[Row32, _Mapping]] = ..., StartupDoctor: _Optional[_Union[Row33, _Mapping]] = ..., DiagnosticsSnapshot: _Optional[_Union[Row34, _Mapping]] = ..., CollectBundle: _Optional[_Union[Row35, _Mapping]] = ..., CycleLogPattern: _Optional[_Union[Row36, _Mapping]] = ..., ExecutionLogPattern: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Unspecified", "Root", "CyclePattern", "ExecutionPattern")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ROOT_FIELD_NUMBER: _ClassVar[int]
    CYCLEPATTERN_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONPATTERN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row38
    Root: Row39
    CyclePattern: Row40
    ExecutionPattern: Row41
    def __init__(self, Unspecified: _Optional[_Union[Row38, _Mapping]] = ..., Root: _Optional[_Union[Row39, _Mapping]] = ..., CyclePattern: _Optional[_Union[Row40, _Mapping]] = ..., ExecutionPattern: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("Unspecified", "Json", "Jsonl", "Zip")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    JSON_FIELD_NUMBER: _ClassVar[int]
    JSONL_FIELD_NUMBER: _ClassVar[int]
    ZIP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row42
    Json: Row43
    Jsonl: Row44
    Zip: Row45
    def __init__(self, Unspecified: _Optional[_Union[Row42, _Mapping]] = ..., Json: _Optional[_Union[Row43, _Mapping]] = ..., Jsonl: _Optional[_Union[Row44, _Mapping]] = ..., Zip: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("Unspecified", "Entrypoint", "Action", "Sidecar", "Provider", "Initialize", "Run", "Finalize", "Interrupt")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ENTRYPOINT_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    SIDECAR_FIELD_NUMBER: _ClassVar[int]
    PROVIDER_FIELD_NUMBER: _ClassVar[int]
    INITIALIZE_FIELD_NUMBER: _ClassVar[int]
    RUN_FIELD_NUMBER: _ClassVar[int]
    FINALIZE_FIELD_NUMBER: _ClassVar[int]
    INTERRUPT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row46
    Entrypoint: Row47
    Action: Row48
    Sidecar: Row49
    Provider: Row50
    Initialize: Row51
    Run: Row52
    Finalize: Row53
    Interrupt: Row54
    def __init__(self, Unspecified: _Optional[_Union[Row46, _Mapping]] = ..., Entrypoint: _Optional[_Union[Row47, _Mapping]] = ..., Action: _Optional[_Union[Row48, _Mapping]] = ..., Sidecar: _Optional[_Union[Row49, _Mapping]] = ..., Provider: _Optional[_Union[Row50, _Mapping]] = ..., Initialize: _Optional[_Union[Row51, _Mapping]] = ..., Run: _Optional[_Union[Row52, _Mapping]] = ..., Finalize: _Optional[_Union[Row53, _Mapping]] = ..., Interrupt: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row55
    Trace: Row56
    Debug: Row57
    Info: Row58
    Warn: Row59
    Error: Row60
    def __init__(self, Unspecified: _Optional[_Union[Row55, _Mapping]] = ..., Trace: _Optional[_Union[Row56, _Mapping]] = ..., Debug: _Optional[_Union[Row57, _Mapping]] = ..., Info: _Optional[_Union[Row58, _Mapping]] = ..., Warn: _Optional[_Union[Row59, _Mapping]] = ..., Error: _Optional[_Union[Row60, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("message", "code", "source")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row61
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row62]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row62, _Mapping]]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ("Unspecified", "StorageRoot", "ShutdownState")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STORAGEROOT_FIELD_NUMBER: _ClassVar[int]
    SHUTDOWNSTATE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row63
    StorageRoot: Row64
    ShutdownState: Row65
    def __init__(self, Unspecified: _Optional[_Union[Row63, _Mapping]] = ..., StorageRoot: _Optional[_Union[Row64, _Mapping]] = ..., ShutdownState: _Optional[_Union[Row65, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row37(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row49(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row50(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row53(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row54(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row55(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row56(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row57(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row58(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row59(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row60(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row61(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row66
    def __init__(self, kind: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row62(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row67
    def __init__(self, kind: _Optional[_Union[Row67, _Mapping]] = ...) -> None: ...

class Row63(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row64(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row65(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row66(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row68
    Initialization: Row69
    StepAttemptId: Row70
    Cleanup: Row71
    def __init__(self, Default: _Optional[_Union[Row68, _Mapping]] = ..., Initialization: _Optional[_Union[Row69, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row70, _Mapping]] = ..., Cleanup: _Optional[_Union[Row71, _Mapping]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("Null", "Bool", "Integer", "Unsigned", "Number", "Text", "Datetime", "Array", "Object")
    NULL_FIELD_NUMBER: _ClassVar[int]
    BOOL_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    Null: Row72
    Bool: Row73
    Integer: Row74
    Unsigned: Row75
    Number: Row76
    Text: Row77
    Datetime: Row78
    Array: Row79
    Object: Row80
    def __init__(self, Null: _Optional[_Union[Row72, _Mapping]] = ..., Bool: _Optional[_Union[Row73, _Mapping]] = ..., Integer: _Optional[_Union[Row74, _Mapping]] = ..., Unsigned: _Optional[_Union[Row75, _Mapping]] = ..., Number: _Optional[_Union[Row76, _Mapping]] = ..., Text: _Optional[_Union[Row77, _Mapping]] = ..., Datetime: _Optional[_Union[Row78, _Mapping]] = ..., Array: _Optional[_Union[Row79, _Mapping]] = ..., Object: _Optional[_Union[Row80, _Mapping]] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row73(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row75(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row78(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row6
    def __init__(self, field_0: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row79(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row81
    def __init__(self, field_0: _Optional[_Union[Row81, _Mapping]] = ...) -> None: ...

class Row80(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row82
    def __init__(self, field_0: _Optional[_Union[Row82, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row83]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row83, _Mapping]]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
