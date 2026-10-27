from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("checkpoint", "sections")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    sections: _containers.RepeatedCompositeFieldContainer[Row4]
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., sections: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("checkpoint", "outputs", "artifacts", "record_schemas", "telemetry_tables")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_TABLES_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    outputs: _containers.RepeatedCompositeFieldContainer[Row5]
    artifacts: _containers.RepeatedCompositeFieldContainer[Row6]
    record_schemas: _containers.RepeatedCompositeFieldContainer[Row7]
    telemetry_tables: _containers.RepeatedCompositeFieldContainer[Row8]
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., outputs: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., artifacts: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., record_schemas: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., telemetry_tables: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row9
    version: bytes
    evidence_through: Row9
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row9, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Outputs", "Artifacts", "RecordSchemas", "TelemetryTables")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    RECORDSCHEMAS_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYTABLES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Outputs: Row11
    Artifacts: Row12
    RecordSchemas: Row13
    TelemetryTables: Row14
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Outputs: _Optional[_Union[Row11, _Mapping]] = ..., Artifacts: _Optional[_Union[Row12, _Mapping]] = ..., RecordSchemas: _Optional[_Union[Row13, _Mapping]] = ..., TelemetryTables: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "size_bytes", "created_at")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: Row15
    state: Row16
    size_bytes: int
    created_at: Row17
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[Row15, _Mapping]] = ..., state: _Optional[_Union[Row16, _Mapping]] = ..., size_bytes: _Optional[int] = ..., created_at: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("artifact_id", "upload_id", "scope", "filename", "content_type", "size_bytes", "sha256", "state", "created_at")
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    UPLOAD_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    upload_id: str
    scope: Row18
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: Row19
    created_at: Row17
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[Row18, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[Row19, _Mapping]] = ..., created_at: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("record_type", "schema_version", "metadata", "template_identifier", "owner_suite_id", "default_severity", "fields", "validation_mode", "allow_extra_fields", "display")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_SEVERITY_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_MODE_FIELD_NUMBER: _ClassVar[int]
    ALLOW_EXTRA_FIELDS_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    schema_version: int
    metadata: Row20
    template_identifier: str
    owner_suite_id: str
    default_severity: Row21
    fields: _containers.RepeatedCompositeFieldContainer[Row22]
    validation_mode: Row23
    allow_extra_fields: bool
    display: Row24
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[Row20, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., default_severity: _Optional[_Union[Row21, _Mapping]] = ..., fields: _Optional[_Iterable[_Union[Row22, _Mapping]]] = ..., validation_mode: _Optional[_Union[Row23, _Mapping]] = ..., allow_extra_fields: _Optional[bool] = ..., display: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("table", "schema")
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    table: str
    schema: Row25
    def __init__(self, table: _Optional[str] = ..., schema: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row26
    sequence: int
    def __init__(self, history: _Optional[_Union[Row26, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Unspecified", "Text", "Binary", "Media", "Unknown")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    UNKNOWN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row27
    Text: Row28
    Binary: Row29
    Media: Row30
    Unknown: Row31
    def __init__(self, Unspecified: _Optional[_Union[Row27, _Mapping]] = ..., Text: _Optional[_Union[Row28, _Mapping]] = ..., Binary: _Optional[_Union[Row29, _Mapping]] = ..., Media: _Optional[_Union[Row30, _Mapping]] = ..., Unknown: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Streaming", "Finalized", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    FINALIZED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row32
    Streaming: Row33
    Finalized: Row34
    Interrupted: Row35
    def __init__(self, Unspecified: _Optional[_Union[Row32, _Mapping]] = ..., Streaming: _Optional[_Union[Row33, _Mapping]] = ..., Finalized: _Optional[_Union[Row34, _Mapping]] = ..., Interrupted: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Unspecified", "Complete", "Cancelled", "Rejected")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row36
    Complete: Row37
    Cancelled: Row38
    Rejected: Row39
    def __init__(self, Unspecified: _Optional[_Union[Row36, _Mapping]] = ..., Complete: _Optional[_Union[Row37, _Mapping]] = ..., Cancelled: _Optional[_Union[Row38, _Mapping]] = ..., Rejected: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row40]
    created_at: Row17
    updated_at: Row17
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row40, _Mapping]]] = ..., created_at: _Optional[_Union[Row17, _Mapping]] = ..., updated_at: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row41
    Trace: Row42
    Debug: Row43
    Info: Row44
    Warn: Row45
    Error: Row46
    def __init__(self, Unspecified: _Optional[_Union[Row41, _Mapping]] = ..., Trace: _Optional[_Union[Row42, _Mapping]] = ..., Debug: _Optional[_Union[Row43, _Mapping]] = ..., Info: _Optional[_Union[Row44, _Mapping]] = ..., Warn: _Optional[_Union[Row45, _Mapping]] = ..., Error: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("metadata", "value_type", "required", "nullable", "unit", "choices", "default_value", "format")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row20
    value_type: Row47
    required: bool
    nullable: bool
    unit: Row48
    choices: _containers.RepeatedCompositeFieldContainer[Row49]
    default_value: Row49
    format: Row50
    def __init__(self, metadata: _Optional[_Union[Row20, _Mapping]] = ..., value_type: _Optional[_Union[Row47, _Mapping]] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., unit: _Optional[_Union[Row48, _Mapping]] = ..., choices: _Optional[_Iterable[_Union[Row49, _Mapping]]] = ..., default_value: _Optional[_Union[Row49, _Mapping]] = ..., format: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("Unspecified", "Strict", "Prototype")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STRICT_FIELD_NUMBER: _ClassVar[int]
    PROTOTYPE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row51
    Strict: Row52
    Prototype: Row53
    def __init__(self, Unspecified: _Optional[_Union[Row51, _Mapping]] = ..., Strict: _Optional[_Union[Row52, _Mapping]] = ..., Prototype: _Optional[_Union[Row53, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("icon", "tone")
    ICON_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    icon: Row54
    tone: Row55
    def __init__(self, icon: _Optional[_Union[Row54, _Mapping]] = ..., tone: _Optional[_Union[Row55, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("stream", "schema_version", "storage_schema", "metadata", "template_identifier", "owner_suite_id", "columns", "default_x", "allow_extra_fields", "scalar", "validation_mode", "foreign")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    STORAGE_SCHEMA_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_X_FIELD_NUMBER: _ClassVar[int]
    ALLOW_EXTRA_FIELDS_FIELD_NUMBER: _ClassVar[int]
    SCALAR_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_MODE_FIELD_NUMBER: _ClassVar[int]
    FOREIGN_FIELD_NUMBER: _ClassVar[int]
    stream: str
    schema_version: int
    storage_schema: Row56
    metadata: Row20
    template_identifier: str
    owner_suite_id: str
    columns: _containers.RepeatedCompositeFieldContainer[Row57]
    default_x: str
    allow_extra_fields: bool
    scalar: bool
    validation_mode: Row23
    foreign: bool
    def __init__(self, stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., storage_schema: _Optional[_Union[Row56, _Mapping]] = ..., metadata: _Optional[_Union[Row20, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., columns: _Optional[_Iterable[_Union[Row57, _Mapping]]] = ..., default_x: _Optional[str] = ..., allow_extra_fields: _Optional[bool] = ..., scalar: _Optional[bool] = ..., validation_mode: _Optional[_Union[Row23, _Mapping]] = ..., foreign: _Optional[bool] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row49
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Boolean", "Integer", "Number", "Text", "File", "Object", "Array", "Datetime")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row58
    Boolean: Row59
    Integer: Row60
    Number: Row61
    Text: Row62
    File: Row63
    Object: Row64
    Array: Row65
    Datetime: Row66
    def __init__(self, Unspecified: _Optional[_Union[Row58, _Mapping]] = ..., Boolean: _Optional[_Union[Row59, _Mapping]] = ..., Integer: _Optional[_Union[Row60, _Mapping]] = ..., Number: _Optional[_Union[Row61, _Mapping]] = ..., Text: _Optional[_Union[Row62, _Mapping]] = ..., File: _Optional[_Union[Row63, _Mapping]] = ..., Object: _Optional[_Union[Row64, _Mapping]] = ..., Array: _Optional[_Union[Row65, _Mapping]] = ..., Datetime: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row20
    family: Row67
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row20, _Mapping]] = ..., family: _Optional[_Union[Row67, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row68]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row68, _Mapping]]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row69
    def __init__(self, kind: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Activity", "AlertTriangle", "Archive", "Bell", "CheckCircle", "CircleDashed", "ClipboardCheck", "Cpu", "FileText", "Flag", "Gauge", "HeartPulse", "HelpCircle", "MessageSquareText", "Package", "Plug", "RadioTower", "RotateCcw", "ShieldAlert", "Target", "TimerReset", "Wrench", "Zap", "Clock")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ACTIVITY_FIELD_NUMBER: _ClassVar[int]
    ALERTTRIANGLE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    BELL_FIELD_NUMBER: _ClassVar[int]
    CHECKCIRCLE_FIELD_NUMBER: _ClassVar[int]
    CIRCLEDASHED_FIELD_NUMBER: _ClassVar[int]
    CLIPBOARDCHECK_FIELD_NUMBER: _ClassVar[int]
    CPU_FIELD_NUMBER: _ClassVar[int]
    FILETEXT_FIELD_NUMBER: _ClassVar[int]
    FLAG_FIELD_NUMBER: _ClassVar[int]
    GAUGE_FIELD_NUMBER: _ClassVar[int]
    HEARTPULSE_FIELD_NUMBER: _ClassVar[int]
    HELPCIRCLE_FIELD_NUMBER: _ClassVar[int]
    MESSAGESQUARETEXT_FIELD_NUMBER: _ClassVar[int]
    PACKAGE_FIELD_NUMBER: _ClassVar[int]
    PLUG_FIELD_NUMBER: _ClassVar[int]
    RADIOTOWER_FIELD_NUMBER: _ClassVar[int]
    ROTATECCW_FIELD_NUMBER: _ClassVar[int]
    SHIELDALERT_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TIMERRESET_FIELD_NUMBER: _ClassVar[int]
    WRENCH_FIELD_NUMBER: _ClassVar[int]
    ZAP_FIELD_NUMBER: _ClassVar[int]
    CLOCK_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row70
    Activity: Row71
    AlertTriangle: Row72
    Archive: Row73
    Bell: Row74
    CheckCircle: Row75
    CircleDashed: Row76
    ClipboardCheck: Row77
    Cpu: Row78
    FileText: Row79
    Flag: Row80
    Gauge: Row81
    HeartPulse: Row82
    HelpCircle: Row83
    MessageSquareText: Row84
    Package: Row85
    Plug: Row86
    RadioTower: Row87
    RotateCcw: Row88
    ShieldAlert: Row89
    Target: Row90
    TimerReset: Row91
    Wrench: Row92
    Zap: Row93
    Clock: Row94
    def __init__(self, Unspecified: _Optional[_Union[Row70, _Mapping]] = ..., Activity: _Optional[_Union[Row71, _Mapping]] = ..., AlertTriangle: _Optional[_Union[Row72, _Mapping]] = ..., Archive: _Optional[_Union[Row73, _Mapping]] = ..., Bell: _Optional[_Union[Row74, _Mapping]] = ..., CheckCircle: _Optional[_Union[Row75, _Mapping]] = ..., CircleDashed: _Optional[_Union[Row76, _Mapping]] = ..., ClipboardCheck: _Optional[_Union[Row77, _Mapping]] = ..., Cpu: _Optional[_Union[Row78, _Mapping]] = ..., FileText: _Optional[_Union[Row79, _Mapping]] = ..., Flag: _Optional[_Union[Row80, _Mapping]] = ..., Gauge: _Optional[_Union[Row81, _Mapping]] = ..., HeartPulse: _Optional[_Union[Row82, _Mapping]] = ..., HelpCircle: _Optional[_Union[Row83, _Mapping]] = ..., MessageSquareText: _Optional[_Union[Row84, _Mapping]] = ..., Package: _Optional[_Union[Row85, _Mapping]] = ..., Plug: _Optional[_Union[Row86, _Mapping]] = ..., RadioTower: _Optional[_Union[Row87, _Mapping]] = ..., RotateCcw: _Optional[_Union[Row88, _Mapping]] = ..., ShieldAlert: _Optional[_Union[Row89, _Mapping]] = ..., Target: _Optional[_Union[Row90, _Mapping]] = ..., TimerReset: _Optional[_Union[Row91, _Mapping]] = ..., Wrench: _Optional[_Union[Row92, _Mapping]] = ..., Zap: _Optional[_Union[Row93, _Mapping]] = ..., Clock: _Optional[_Union[Row94, _Mapping]] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("Unspecified", "Info", "Success", "Warning", "Danger", "Accent", "Muted", "Neutral")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    ACCENT_FIELD_NUMBER: _ClassVar[int]
    MUTED_FIELD_NUMBER: _ClassVar[int]
    NEUTRAL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row95
    Info: Row96
    Success: Row97
    Warning: Row98
    Danger: Row99
    Accent: Row100
    Muted: Row101
    Neutral: Row102
    def __init__(self, Unspecified: _Optional[_Union[Row95, _Mapping]] = ..., Info: _Optional[_Union[Row96, _Mapping]] = ..., Success: _Optional[_Union[Row97, _Mapping]] = ..., Warning: _Optional[_Union[Row98, _Mapping]] = ..., Danger: _Optional[_Union[Row99, _Mapping]] = ..., Accent: _Optional[_Union[Row100, _Mapping]] = ..., Muted: _Optional[_Union[Row101, _Mapping]] = ..., Neutral: _Optional[_Union[Row102, _Mapping]] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("root", "definitions")
    ROOT_FIELD_NUMBER: _ClassVar[int]
    DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    root: int
    definitions: _containers.RepeatedCompositeFieldContainer[Row103]
    def __init__(self, root: _Optional[int] = ..., definitions: _Optional[_Iterable[_Union[Row103, _Mapping]]] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ("metadata", "value_type", "role", "unit", "display_unit", "required", "nullable", "default_value", "format")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    ROLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_UNIT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row20
    value_type: Row47
    role: Row104
    unit: Row48
    display_unit: str
    required: bool
    nullable: bool
    default_value: Row49
    format: Row50
    def __init__(self, metadata: _Optional[_Union[Row20, _Mapping]] = ..., value_type: _Optional[_Union[Row47, _Mapping]] = ..., role: _Optional[_Union[Row104, _Mapping]] = ..., unit: _Optional[_Union[Row48, _Mapping]] = ..., display_unit: _Optional[str] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., default_value: _Optional[_Union[Row49, _Mapping]] = ..., format: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row62(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row67(_message.Message):
    __slots__ = ("Unspecified", "Count", "Distance", "Duration", "Frequency", "Power", "Temperature", "Voltage", "Ratio", "Custom")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    FREQUENCY_FIELD_NUMBER: _ClassVar[int]
    POWER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_FIELD_NUMBER: _ClassVar[int]
    VOLTAGE_FIELD_NUMBER: _ClassVar[int]
    RATIO_FIELD_NUMBER: _ClassVar[int]
    CUSTOM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row105
    Count: Row106
    Distance: Row107
    Duration: Row108
    Frequency: Row109
    Power: Row110
    Temperature: Row111
    Voltage: Row112
    Ratio: Row113
    Custom: Row114
    def __init__(self, Unspecified: _Optional[_Union[Row105, _Mapping]] = ..., Count: _Optional[_Union[Row106, _Mapping]] = ..., Distance: _Optional[_Union[Row107, _Mapping]] = ..., Duration: _Optional[_Union[Row108, _Mapping]] = ..., Frequency: _Optional[_Union[Row109, _Mapping]] = ..., Power: _Optional[_Union[Row110, _Mapping]] = ..., Temperature: _Optional[_Union[Row111, _Mapping]] = ..., Voltage: _Optional[_Union[Row112, _Mapping]] = ..., Ratio: _Optional[_Union[Row113, _Mapping]] = ..., Custom: _Optional[_Union[Row114, _Mapping]] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row115
    def __init__(self, kind: _Optional[_Union[Row115, _Mapping]] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row116
    Fixed: Row117
    Scientific: Row118
    Engineering: Row119
    SiPrefix: Row120
    AbsoluteTime: Row121
    ElapsedSinceRun: Row122
    ElapsedSinceCycle: Row123
    def __init__(self, Normal: _Optional[_Union[Row116, _Mapping]] = ..., Fixed: _Optional[_Union[Row117, _Mapping]] = ..., Scientific: _Optional[_Union[Row118, _Mapping]] = ..., Engineering: _Optional[_Union[Row119, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row120, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row121, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row122, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row123, _Mapping]] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row71(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row72(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row73(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row74(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row75(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row76(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row77(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row78(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row79(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row80(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row81(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row82(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row83(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row84(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row85(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row86(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row87(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row88(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row89(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row90(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row91(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row92(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row93(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row94(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row95(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row96(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row97(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row98(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row99(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row100(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row101(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row102(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row103(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row124]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row124, _Mapping]]] = ...) -> None: ...

class Row104(_message.Message):
    __slots__ = ("Unspecified", "Value", "Counter", "State", "Distance", "Latency", "Throughput")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    COUNTER_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    LATENCY_FIELD_NUMBER: _ClassVar[int]
    THROUGHPUT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row125
    Value: Row126
    Counter: Row127
    State: Row128
    Distance: Row129
    Latency: Row130
    Throughput: Row131
    def __init__(self, Unspecified: _Optional[_Union[Row125, _Mapping]] = ..., Value: _Optional[_Union[Row126, _Mapping]] = ..., Counter: _Optional[_Union[Row127, _Mapping]] = ..., State: _Optional[_Union[Row128, _Mapping]] = ..., Distance: _Optional[_Union[Row129, _Mapping]] = ..., Latency: _Optional[_Union[Row130, _Mapping]] = ..., Throughput: _Optional[_Union[Row131, _Mapping]] = ...) -> None: ...

class Row105(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row106(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row107(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row108(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row109(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row110(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row111(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row112(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row113(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row114(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row115(_message.Message):
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
    Null: Row132
    Bool: Row133
    Integer: Row134
    Unsigned: Row135
    Number: Row136
    Text: Row137
    Datetime: Row138
    Array: Row139
    Object: Row140
    def __init__(self, Null: _Optional[_Union[Row132, _Mapping]] = ..., Bool: _Optional[_Union[Row133, _Mapping]] = ..., Integer: _Optional[_Union[Row134, _Mapping]] = ..., Unsigned: _Optional[_Union[Row135, _Mapping]] = ..., Number: _Optional[_Union[Row136, _Mapping]] = ..., Text: _Optional[_Union[Row137, _Mapping]] = ..., Datetime: _Optional[_Union[Row138, _Mapping]] = ..., Array: _Optional[_Union[Row139, _Mapping]] = ..., Object: _Optional[_Union[Row140, _Mapping]] = ...) -> None: ...

class Row116(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row141
    def __init__(self, field_0: _Optional[_Union[Row141, _Mapping]] = ...) -> None: ...

class Row117(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row142
    def __init__(self, field_0: _Optional[_Union[Row142, _Mapping]] = ...) -> None: ...

class Row118(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row143
    def __init__(self, field_0: _Optional[_Union[Row143, _Mapping]] = ...) -> None: ...

class Row119(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row144
    def __init__(self, field_0: _Optional[_Union[Row144, _Mapping]] = ...) -> None: ...

class Row120(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row145
    def __init__(self, field_0: _Optional[_Union[Row145, _Mapping]] = ...) -> None: ...

class Row121(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row146
    def __init__(self, field_0: _Optional[_Union[Row146, _Mapping]] = ...) -> None: ...

class Row122(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row147
    def __init__(self, field_0: _Optional[_Union[Row147, _Mapping]] = ...) -> None: ...

class Row123(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row147
    def __init__(self, field_0: _Optional[_Union[Row147, _Mapping]] = ...) -> None: ...

class Row124(_message.Message):
    __slots__ = ("name", "tag", "kind", "cardinality", "definition")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TAG_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    CARDINALITY_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    name: str
    tag: int
    kind: int
    cardinality: int
    definition: int
    def __init__(self, name: _Optional[str] = ..., tag: _Optional[int] = ..., kind: _Optional[int] = ..., cardinality: _Optional[int] = ..., definition: _Optional[int] = ...) -> None: ...

class Row125(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row126(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row127(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row128(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row129(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row130(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row131(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row132(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row133(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row134(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row135(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row136(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row137(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row138(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row17
    def __init__(self, field_0: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row139(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row148
    def __init__(self, field_0: _Optional[_Union[Row148, _Mapping]] = ...) -> None: ...

class Row140(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row149
    def __init__(self, field_0: _Optional[_Union[Row149, _Mapping]] = ...) -> None: ...

class Row141(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row142(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row143(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row144(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row145(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row146(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row147(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row150
    def __init__(self, unit: _Optional[_Union[Row150, _Mapping]] = ...) -> None: ...

class Row148(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row149(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row151]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row151, _Mapping]]] = ...) -> None: ...

class Row150(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row152
    Seconds: Row153
    Milliseconds: Row154
    Minutes: Row155
    def __init__(self, Unspecified: _Optional[_Union[Row152, _Mapping]] = ..., Seconds: _Optional[_Union[Row153, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row154, _Mapping]] = ..., Minutes: _Optional[_Union[Row155, _Mapping]] = ...) -> None: ...

class Row151(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row152(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row153(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row154(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row155(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
