from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ValueType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    VALUE_TYPE_UNSPECIFIED: _ClassVar[ValueType]
    VALUE_TYPE_BOOLEAN: _ClassVar[ValueType]
    VALUE_TYPE_INTEGER: _ClassVar[ValueType]
    VALUE_TYPE_NUMBER: _ClassVar[ValueType]
    VALUE_TYPE_TEXT: _ClassVar[ValueType]
    VALUE_TYPE_FILE: _ClassVar[ValueType]
    VALUE_TYPE_OBJECT: _ClassVar[ValueType]
    VALUE_TYPE_ARRAY: _ClassVar[ValueType]
    VALUE_TYPE_DATETIME: _ClassVar[ValueType]

class ValidationMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    VALIDATION_MODE_UNSPECIFIED: _ClassVar[ValidationMode]
    VALIDATION_MODE_STRICT: _ClassVar[ValidationMode]
    VALIDATION_MODE_PROTOTYPE: _ClassVar[ValidationMode]

class UnitFamily(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    UNIT_FAMILY_UNSPECIFIED: _ClassVar[UnitFamily]
    UNIT_FAMILY_COUNT: _ClassVar[UnitFamily]
    UNIT_FAMILY_DISTANCE: _ClassVar[UnitFamily]
    UNIT_FAMILY_DURATION: _ClassVar[UnitFamily]
    UNIT_FAMILY_FREQUENCY: _ClassVar[UnitFamily]
    UNIT_FAMILY_POWER: _ClassVar[UnitFamily]
    UNIT_FAMILY_TEMPERATURE: _ClassVar[UnitFamily]
    UNIT_FAMILY_VOLTAGE: _ClassVar[UnitFamily]
    UNIT_FAMILY_RATIO: _ClassVar[UnitFamily]
    UNIT_FAMILY_CUSTOM: _ClassVar[UnitFamily]

class RecordIcon(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RECORD_ICON_UNSPECIFIED: _ClassVar[RecordIcon]
    RECORD_ICON_ACTIVITY: _ClassVar[RecordIcon]
    RECORD_ICON_ALERT_TRIANGLE: _ClassVar[RecordIcon]
    RECORD_ICON_ARCHIVE: _ClassVar[RecordIcon]
    RECORD_ICON_BELL: _ClassVar[RecordIcon]
    RECORD_ICON_CHECK_CIRCLE: _ClassVar[RecordIcon]
    RECORD_ICON_CIRCLE_DASHED: _ClassVar[RecordIcon]
    RECORD_ICON_CLIPBOARD_CHECK: _ClassVar[RecordIcon]
    RECORD_ICON_CPU: _ClassVar[RecordIcon]
    RECORD_ICON_FILE_TEXT: _ClassVar[RecordIcon]
    RECORD_ICON_FLAG: _ClassVar[RecordIcon]
    RECORD_ICON_GAUGE: _ClassVar[RecordIcon]
    RECORD_ICON_HEART_PULSE: _ClassVar[RecordIcon]
    RECORD_ICON_HELP_CIRCLE: _ClassVar[RecordIcon]
    RECORD_ICON_MESSAGE_SQUARE_TEXT: _ClassVar[RecordIcon]
    RECORD_ICON_PACKAGE: _ClassVar[RecordIcon]
    RECORD_ICON_PLUG: _ClassVar[RecordIcon]
    RECORD_ICON_RADIO_TOWER: _ClassVar[RecordIcon]
    RECORD_ICON_ROTATE_CCW: _ClassVar[RecordIcon]
    RECORD_ICON_SHIELD_ALERT: _ClassVar[RecordIcon]
    RECORD_ICON_TARGET: _ClassVar[RecordIcon]
    RECORD_ICON_TIMER_RESET: _ClassVar[RecordIcon]
    RECORD_ICON_WRENCH: _ClassVar[RecordIcon]
    RECORD_ICON_ZAP: _ClassVar[RecordIcon]
    RECORD_ICON_CLOCK: _ClassVar[RecordIcon]

class RecordTone(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RECORD_TONE_UNSPECIFIED: _ClassVar[RecordTone]
    RECORD_TONE_INFO: _ClassVar[RecordTone]
    RECORD_TONE_SUCCESS: _ClassVar[RecordTone]
    RECORD_TONE_WARNING: _ClassVar[RecordTone]
    RECORD_TONE_DANGER: _ClassVar[RecordTone]
    RECORD_TONE_ACCENT: _ClassVar[RecordTone]
    RECORD_TONE_MUTED: _ClassVar[RecordTone]
    RECORD_TONE_NEUTRAL: _ClassVar[RecordTone]

class TelemetryRole(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TELEMETRY_ROLE_UNSPECIFIED: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_VALUE: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_COUNTER: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_STATE: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_DISTANCE: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_LATENCY: _ClassVar[TelemetryRole]
    TELEMETRY_ROLE_THROUGHPUT: _ClassVar[TelemetryRole]

class ElapsedUnit(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ELAPSED_UNIT_UNSPECIFIED: _ClassVar[ElapsedUnit]
    ELAPSED_UNIT_SECONDS: _ClassVar[ElapsedUnit]
    ELAPSED_UNIT_MILLISECONDS: _ClassVar[ElapsedUnit]
    ELAPSED_UNIT_MINUTES: _ClassVar[ElapsedUnit]
VALUE_TYPE_UNSPECIFIED: ValueType
VALUE_TYPE_BOOLEAN: ValueType
VALUE_TYPE_INTEGER: ValueType
VALUE_TYPE_NUMBER: ValueType
VALUE_TYPE_TEXT: ValueType
VALUE_TYPE_FILE: ValueType
VALUE_TYPE_OBJECT: ValueType
VALUE_TYPE_ARRAY: ValueType
VALUE_TYPE_DATETIME: ValueType
VALIDATION_MODE_UNSPECIFIED: ValidationMode
VALIDATION_MODE_STRICT: ValidationMode
VALIDATION_MODE_PROTOTYPE: ValidationMode
UNIT_FAMILY_UNSPECIFIED: UnitFamily
UNIT_FAMILY_COUNT: UnitFamily
UNIT_FAMILY_DISTANCE: UnitFamily
UNIT_FAMILY_DURATION: UnitFamily
UNIT_FAMILY_FREQUENCY: UnitFamily
UNIT_FAMILY_POWER: UnitFamily
UNIT_FAMILY_TEMPERATURE: UnitFamily
UNIT_FAMILY_VOLTAGE: UnitFamily
UNIT_FAMILY_RATIO: UnitFamily
UNIT_FAMILY_CUSTOM: UnitFamily
RECORD_ICON_UNSPECIFIED: RecordIcon
RECORD_ICON_ACTIVITY: RecordIcon
RECORD_ICON_ALERT_TRIANGLE: RecordIcon
RECORD_ICON_ARCHIVE: RecordIcon
RECORD_ICON_BELL: RecordIcon
RECORD_ICON_CHECK_CIRCLE: RecordIcon
RECORD_ICON_CIRCLE_DASHED: RecordIcon
RECORD_ICON_CLIPBOARD_CHECK: RecordIcon
RECORD_ICON_CPU: RecordIcon
RECORD_ICON_FILE_TEXT: RecordIcon
RECORD_ICON_FLAG: RecordIcon
RECORD_ICON_GAUGE: RecordIcon
RECORD_ICON_HEART_PULSE: RecordIcon
RECORD_ICON_HELP_CIRCLE: RecordIcon
RECORD_ICON_MESSAGE_SQUARE_TEXT: RecordIcon
RECORD_ICON_PACKAGE: RecordIcon
RECORD_ICON_PLUG: RecordIcon
RECORD_ICON_RADIO_TOWER: RecordIcon
RECORD_ICON_ROTATE_CCW: RecordIcon
RECORD_ICON_SHIELD_ALERT: RecordIcon
RECORD_ICON_TARGET: RecordIcon
RECORD_ICON_TIMER_RESET: RecordIcon
RECORD_ICON_WRENCH: RecordIcon
RECORD_ICON_ZAP: RecordIcon
RECORD_ICON_CLOCK: RecordIcon
RECORD_TONE_UNSPECIFIED: RecordTone
RECORD_TONE_INFO: RecordTone
RECORD_TONE_SUCCESS: RecordTone
RECORD_TONE_WARNING: RecordTone
RECORD_TONE_DANGER: RecordTone
RECORD_TONE_ACCENT: RecordTone
RECORD_TONE_MUTED: RecordTone
RECORD_TONE_NEUTRAL: RecordTone
TELEMETRY_ROLE_UNSPECIFIED: TelemetryRole
TELEMETRY_ROLE_VALUE: TelemetryRole
TELEMETRY_ROLE_COUNTER: TelemetryRole
TELEMETRY_ROLE_STATE: TelemetryRole
TELEMETRY_ROLE_DISTANCE: TelemetryRole
TELEMETRY_ROLE_LATENCY: TelemetryRole
TELEMETRY_ROLE_THROUGHPUT: TelemetryRole
ELAPSED_UNIT_UNSPECIFIED: ElapsedUnit
ELAPSED_UNIT_SECONDS: ElapsedUnit
ELAPSED_UNIT_MILLISECONDS: ElapsedUnit
ELAPSED_UNIT_MINUTES: ElapsedUnit

class MeasurementUnit(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    family: UnitFamily
    symbol: str
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., family: _Optional[_Union[UnitFamily, str]] = ..., symbol: _Optional[str] = ...) -> None: ...

class RecordField(_message.Message):
    __slots__ = ("metadata", "value_type", "required", "nullable", "unit", "choices", "default_value", "format")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    value_type: ValueType
    required: bool
    nullable: bool
    unit: MeasurementUnit
    choices: _containers.RepeatedCompositeFieldContainer[_value_pb2.Value]
    default_value: _value_pb2.Value
    format: ValueFormat
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value_type: _Optional[_Union[ValueType, str]] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., unit: _Optional[_Union[MeasurementUnit, _Mapping]] = ..., choices: _Optional[_Iterable[_Union[_value_pb2.Value, _Mapping]]] = ..., default_value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., format: _Optional[_Union[ValueFormat, _Mapping]] = ...) -> None: ...

class RecordSchema(_message.Message):
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
    metadata: _common_pb2.Metadata
    template_identifier: str
    owner_suite_id: str
    default_severity: _common_pb2.Severity
    fields: _containers.RepeatedCompositeFieldContainer[RecordField]
    validation_mode: ValidationMode
    allow_extra_fields: bool
    display: RecordDisplay
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., default_severity: _Optional[_Union[_common_pb2.Severity, str]] = ..., fields: _Optional[_Iterable[_Union[RecordField, _Mapping]]] = ..., validation_mode: _Optional[_Union[ValidationMode, str]] = ..., allow_extra_fields: _Optional[bool] = ..., display: _Optional[_Union[RecordDisplay, _Mapping]] = ...) -> None: ...

class RecordDisplay(_message.Message):
    __slots__ = ("icon", "tone")
    ICON_FIELD_NUMBER: _ClassVar[int]
    TONE_FIELD_NUMBER: _ClassVar[int]
    icon: RecordIcon
    tone: RecordTone
    def __init__(self, icon: _Optional[_Union[RecordIcon, str]] = ..., tone: _Optional[_Union[RecordTone, str]] = ...) -> None: ...

class TelemetryColumn(_message.Message):
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
    metadata: _common_pb2.Metadata
    value_type: ValueType
    role: TelemetryRole
    unit: MeasurementUnit
    display_unit: str
    required: bool
    nullable: bool
    default_value: _value_pb2.Value
    format: ValueFormat
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value_type: _Optional[_Union[ValueType, str]] = ..., role: _Optional[_Union[TelemetryRole, str]] = ..., unit: _Optional[_Union[MeasurementUnit, _Mapping]] = ..., display_unit: _Optional[str] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., default_value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., format: _Optional[_Union[ValueFormat, _Mapping]] = ...) -> None: ...

class TelemetrySchema(_message.Message):
    __slots__ = ("stream", "schema_version", "metadata", "template_identifier", "owner_suite_id", "columns", "default_x", "plot_window_ms", "allow_extra_fields", "scalar", "validation_mode", "foreign")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_X_FIELD_NUMBER: _ClassVar[int]
    PLOT_WINDOW_MS_FIELD_NUMBER: _ClassVar[int]
    ALLOW_EXTRA_FIELDS_FIELD_NUMBER: _ClassVar[int]
    SCALAR_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_MODE_FIELD_NUMBER: _ClassVar[int]
    FOREIGN_FIELD_NUMBER: _ClassVar[int]
    stream: str
    schema_version: int
    metadata: _common_pb2.Metadata
    template_identifier: str
    owner_suite_id: str
    columns: _containers.RepeatedCompositeFieldContainer[TelemetryColumn]
    default_x: str
    plot_window_ms: int
    allow_extra_fields: bool
    scalar: bool
    validation_mode: ValidationMode
    foreign: bool
    def __init__(self, stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., columns: _Optional[_Iterable[_Union[TelemetryColumn, _Mapping]]] = ..., default_x: _Optional[str] = ..., plot_window_ms: _Optional[int] = ..., allow_extra_fields: _Optional[bool] = ..., scalar: _Optional[bool] = ..., validation_mode: _Optional[_Union[ValidationMode, str]] = ..., foreign: _Optional[bool] = ...) -> None: ...

class ValueFormat(_message.Message):
    __slots__ = ("normal", "fixed", "scientific", "engineering", "si_prefix", "absolute_time", "elapsed_since_run", "elapsed_since_cycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SI_PREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTE_TIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_SINCE_RUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_SINCE_CYCLE_FIELD_NUMBER: _ClassVar[int]
    normal: NormalFormat
    fixed: FixedFormat
    scientific: ScientificFormat
    engineering: EngineeringFormat
    si_prefix: SiPrefixFormat
    absolute_time: AbsoluteTimeFormat
    elapsed_since_run: ElapsedFormat
    elapsed_since_cycle: ElapsedFormat
    def __init__(self, normal: _Optional[_Union[NormalFormat, _Mapping]] = ..., fixed: _Optional[_Union[FixedFormat, _Mapping]] = ..., scientific: _Optional[_Union[ScientificFormat, _Mapping]] = ..., engineering: _Optional[_Union[EngineeringFormat, _Mapping]] = ..., si_prefix: _Optional[_Union[SiPrefixFormat, _Mapping]] = ..., absolute_time: _Optional[_Union[AbsoluteTimeFormat, _Mapping]] = ..., elapsed_since_run: _Optional[_Union[ElapsedFormat, _Mapping]] = ..., elapsed_since_cycle: _Optional[_Union[ElapsedFormat, _Mapping]] = ...) -> None: ...

class NormalFormat(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class FixedFormat(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class ScientificFormat(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class EngineeringFormat(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class SiPrefixFormat(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class AbsoluteTimeFormat(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class ElapsedFormat(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: ElapsedUnit
    def __init__(self, unit: _Optional[_Union[ElapsedUnit, str]] = ...) -> None: ...
