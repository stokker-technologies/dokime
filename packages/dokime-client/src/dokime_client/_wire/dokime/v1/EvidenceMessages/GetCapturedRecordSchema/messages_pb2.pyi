from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("record_type", "schema_version", "epoch")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    schema_version: int
    epoch: str
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
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
    metadata: Row3
    template_identifier: str
    owner_suite_id: str
    default_severity: Row4
    fields: _containers.RepeatedCompositeFieldContainer[Row5]
    validation_mode: Row6
    allow_extra_fields: bool
    display: Row7
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[Row3, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., default_severity: _Optional[_Union[Row4, _Mapping]] = ..., fields: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., validation_mode: _Optional[_Union[Row6, _Mapping]] = ..., allow_extra_fields: _Optional[bool] = ..., display: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row8]
    created_at: Row9
    updated_at: Row9
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., created_at: _Optional[_Union[Row9, _Mapping]] = ..., updated_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Trace: Row11
    Debug: Row12
    Info: Row13
    Warn: Row14
    Error: Row15
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Trace: _Optional[_Union[Row11, _Mapping]] = ..., Debug: _Optional[_Union[Row12, _Mapping]] = ..., Info: _Optional[_Union[Row13, _Mapping]] = ..., Warn: _Optional[_Union[Row14, _Mapping]] = ..., Error: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("metadata", "value_type", "required", "nullable", "unit", "choices", "default_value", "format")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row3
    value_type: Row16
    required: bool
    nullable: bool
    unit: Row17
    choices: _containers.RepeatedCompositeFieldContainer[Row18]
    default_value: Row18
    format: Row19
    def __init__(self, metadata: _Optional[_Union[Row3, _Mapping]] = ..., value_type: _Optional[_Union[Row16, _Mapping]] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., unit: _Optional[_Union[Row17, _Mapping]] = ..., choices: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ..., default_value: _Optional[_Union[Row18, _Mapping]] = ..., format: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Strict", "Prototype")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STRICT_FIELD_NUMBER: _ClassVar[int]
    PROTOTYPE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row20
    Strict: Row21
    Prototype: Row22
    def __init__(self, Unspecified: _Optional[_Union[Row20, _Mapping]] = ..., Strict: _Optional[_Union[Row21, _Mapping]] = ..., Prototype: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("icon", "tone")
    ICON_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    icon: Row23
    tone: Row24
    def __init__(self, icon: _Optional[_Union[Row23, _Mapping]] = ..., tone: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row18
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
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
    Unspecified: Row25
    Boolean: Row26
    Integer: Row27
    Number: Row28
    Text: Row29
    File: Row30
    Object: Row31
    Array: Row32
    Datetime: Row33
    def __init__(self, Unspecified: _Optional[_Union[Row25, _Mapping]] = ..., Boolean: _Optional[_Union[Row26, _Mapping]] = ..., Integer: _Optional[_Union[Row27, _Mapping]] = ..., Number: _Optional[_Union[Row28, _Mapping]] = ..., Text: _Optional[_Union[Row29, _Mapping]] = ..., File: _Optional[_Union[Row30, _Mapping]] = ..., Object: _Optional[_Union[Row31, _Mapping]] = ..., Array: _Optional[_Union[Row32, _Mapping]] = ..., Datetime: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row3
    family: Row34
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row3, _Mapping]] = ..., family: _Optional[_Union[Row34, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row35]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row35, _Mapping]]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row36
    def __init__(self, kind: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
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
    Unspecified: Row37
    Activity: Row38
    AlertTriangle: Row39
    Archive: Row40
    Bell: Row41
    CheckCircle: Row42
    CircleDashed: Row43
    ClipboardCheck: Row44
    Cpu: Row45
    FileText: Row46
    Flag: Row47
    Gauge: Row48
    HeartPulse: Row49
    HelpCircle: Row50
    MessageSquareText: Row51
    Package: Row52
    Plug: Row53
    RadioTower: Row54
    RotateCcw: Row55
    ShieldAlert: Row56
    Target: Row57
    TimerReset: Row58
    Wrench: Row59
    Zap: Row60
    Clock: Row61
    def __init__(self, Unspecified: _Optional[_Union[Row37, _Mapping]] = ..., Activity: _Optional[_Union[Row38, _Mapping]] = ..., AlertTriangle: _Optional[_Union[Row39, _Mapping]] = ..., Archive: _Optional[_Union[Row40, _Mapping]] = ..., Bell: _Optional[_Union[Row41, _Mapping]] = ..., CheckCircle: _Optional[_Union[Row42, _Mapping]] = ..., CircleDashed: _Optional[_Union[Row43, _Mapping]] = ..., ClipboardCheck: _Optional[_Union[Row44, _Mapping]] = ..., Cpu: _Optional[_Union[Row45, _Mapping]] = ..., FileText: _Optional[_Union[Row46, _Mapping]] = ..., Flag: _Optional[_Union[Row47, _Mapping]] = ..., Gauge: _Optional[_Union[Row48, _Mapping]] = ..., HeartPulse: _Optional[_Union[Row49, _Mapping]] = ..., HelpCircle: _Optional[_Union[Row50, _Mapping]] = ..., MessageSquareText: _Optional[_Union[Row51, _Mapping]] = ..., Package: _Optional[_Union[Row52, _Mapping]] = ..., Plug: _Optional[_Union[Row53, _Mapping]] = ..., RadioTower: _Optional[_Union[Row54, _Mapping]] = ..., RotateCcw: _Optional[_Union[Row55, _Mapping]] = ..., ShieldAlert: _Optional[_Union[Row56, _Mapping]] = ..., Target: _Optional[_Union[Row57, _Mapping]] = ..., TimerReset: _Optional[_Union[Row58, _Mapping]] = ..., Wrench: _Optional[_Union[Row59, _Mapping]] = ..., Zap: _Optional[_Union[Row60, _Mapping]] = ..., Clock: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("Unspecified", "Info", "Success", "Warning", "Danger", "Accent", "Muted", "Neutral")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    ACCENT_FIELD_NUMBER: _ClassVar[int]
    MUTED_FIELD_NUMBER: _ClassVar[int]
    NEUTRAL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row62
    Info: Row63
    Success: Row64
    Warning: Row65
    Danger: Row66
    Accent: Row67
    Muted: Row68
    Neutral: Row69
    def __init__(self, Unspecified: _Optional[_Union[Row62, _Mapping]] = ..., Info: _Optional[_Union[Row63, _Mapping]] = ..., Success: _Optional[_Union[Row64, _Mapping]] = ..., Warning: _Optional[_Union[Row65, _Mapping]] = ..., Danger: _Optional[_Union[Row66, _Mapping]] = ..., Accent: _Optional[_Union[Row67, _Mapping]] = ..., Muted: _Optional[_Union[Row68, _Mapping]] = ..., Neutral: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    Unspecified: Row70
    Count: Row71
    Distance: Row72
    Duration: Row73
    Frequency: Row74
    Power: Row75
    Temperature: Row76
    Voltage: Row77
    Ratio: Row78
    Custom: Row79
    def __init__(self, Unspecified: _Optional[_Union[Row70, _Mapping]] = ..., Count: _Optional[_Union[Row71, _Mapping]] = ..., Distance: _Optional[_Union[Row72, _Mapping]] = ..., Duration: _Optional[_Union[Row73, _Mapping]] = ..., Frequency: _Optional[_Union[Row74, _Mapping]] = ..., Power: _Optional[_Union[Row75, _Mapping]] = ..., Temperature: _Optional[_Union[Row76, _Mapping]] = ..., Voltage: _Optional[_Union[Row77, _Mapping]] = ..., Ratio: _Optional[_Union[Row78, _Mapping]] = ..., Custom: _Optional[_Union[Row79, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row80
    def __init__(self, kind: _Optional[_Union[Row80, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row81
    Fixed: Row82
    Scientific: Row83
    Engineering: Row84
    SiPrefix: Row85
    AbsoluteTime: Row86
    ElapsedSinceRun: Row87
    ElapsedSinceCycle: Row88
    def __init__(self, Normal: _Optional[_Union[Row81, _Mapping]] = ..., Fixed: _Optional[_Union[Row82, _Mapping]] = ..., Scientific: _Optional[_Union[Row83, _Mapping]] = ..., Engineering: _Optional[_Union[Row84, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row85, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row86, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row87, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row88, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row68(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row69(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    Null: Row89
    Bool: Row90
    Integer: Row91
    Unsigned: Row92
    Number: Row93
    Text: Row94
    Datetime: Row95
    Array: Row96
    Object: Row97
    def __init__(self, Null: _Optional[_Union[Row89, _Mapping]] = ..., Bool: _Optional[_Union[Row90, _Mapping]] = ..., Integer: _Optional[_Union[Row91, _Mapping]] = ..., Unsigned: _Optional[_Union[Row92, _Mapping]] = ..., Number: _Optional[_Union[Row93, _Mapping]] = ..., Text: _Optional[_Union[Row94, _Mapping]] = ..., Datetime: _Optional[_Union[Row95, _Mapping]] = ..., Array: _Optional[_Union[Row96, _Mapping]] = ..., Object: _Optional[_Union[Row97, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row98
    def __init__(self, field_0: _Optional[_Union[Row98, _Mapping]] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row99
    def __init__(self, field_0: _Optional[_Union[Row99, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row100
    def __init__(self, field_0: _Optional[_Union[Row100, _Mapping]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row101
    def __init__(self, field_0: _Optional[_Union[Row101, _Mapping]] = ...) -> None: ...

class Row85(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row102
    def __init__(self, field_0: _Optional[_Union[Row102, _Mapping]] = ...) -> None: ...

class Row86(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row103
    def __init__(self, field_0: _Optional[_Union[Row103, _Mapping]] = ...) -> None: ...

class Row87(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row104
    def __init__(self, field_0: _Optional[_Union[Row104, _Mapping]] = ...) -> None: ...

class Row88(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row104
    def __init__(self, field_0: _Optional[_Union[Row104, _Mapping]] = ...) -> None: ...

class Row89(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row90(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row91(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row92(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row93(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row94(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row95(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row96(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row105
    def __init__(self, field_0: _Optional[_Union[Row105, _Mapping]] = ...) -> None: ...

class Row97(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row106
    def __init__(self, field_0: _Optional[_Union[Row106, _Mapping]] = ...) -> None: ...

class Row98(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row99(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row100(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row101(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row102(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row103(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row104(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row107
    def __init__(self, unit: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row105(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row106(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row108]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row108, _Mapping]]] = ...) -> None: ...

class Row107(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row109
    Seconds: Row110
    Milliseconds: Row111
    Minutes: Row112
    def __init__(self, Unspecified: _Optional[_Union[Row109, _Mapping]] = ..., Seconds: _Optional[_Union[Row110, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row111, _Mapping]] = ..., Minutes: _Optional[_Union[Row112, _Mapping]] = ...) -> None: ...

class Row108(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
