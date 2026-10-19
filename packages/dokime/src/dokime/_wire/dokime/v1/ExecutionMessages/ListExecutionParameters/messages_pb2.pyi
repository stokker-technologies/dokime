from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "set", "limit", "after_name", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    SET_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    set: Row3
    limit: int
    after_name: str
    cursor: Row4
    def __init__(self, execution_id: _Optional[str] = ..., set: _Optional[_Union[Row3, _Mapping]] = ..., limit: _Optional[int] = ..., after_name: _Optional[str] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("parameters", "next_after_name", "cursor")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[Row5]
    next_after_name: str
    cursor: Row4
    def __init__(self, parameters: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., next_after_name: _Optional[str] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Inputs", "Outputs")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INPUTS_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row6
    Inputs: Row7
    Outputs: Row8
    def __init__(self, Unspecified: _Optional[_Union[Row6, _Mapping]] = ..., Inputs: _Optional[_Union[Row7, _Mapping]] = ..., Outputs: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("observed", "selection")
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    observed: Row9
    selection: Row10
    def __init__(self, observed: _Optional[_Union[Row9, _Mapping]] = ..., selection: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("name", "metadata", "group_id", "source", "value", "unit")
    NAME_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    name: str
    metadata: Row11
    group_id: str
    source: Row12
    value: Row13
    unit: Row14
    def __init__(self, name: _Optional[str] = ..., metadata: _Optional[_Union[Row11, _Mapping]] = ..., group_id: _Optional[str] = ..., source: _Optional[_Union[Row12, _Mapping]] = ..., value: _Optional[_Union[Row13, _Mapping]] = ..., unit: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("ExecutionInputsId", "ExecutionOutputsId", "PlannedCase")
    EXECUTIONINPUTSID_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONOUTPUTSID_FIELD_NUMBER: _ClassVar[int]
    PLANNEDCASE_FIELD_NUMBER: _ClassVar[int]
    ExecutionInputsId: Row15
    ExecutionOutputsId: Row16
    PlannedCase: Row17
    def __init__(self, ExecutionInputsId: _Optional[_Union[Row15, _Mapping]] = ..., ExecutionOutputsId: _Optional[_Union[Row16, _Mapping]] = ..., PlannedCase: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row18]
    created_at: Row19
    updated_at: Row19
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ..., created_at: _Optional[_Union[Row19, _Mapping]] = ..., updated_at: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Unspecified", "Definition", "Traceability", "Environment", "Resource", "Operator", "Runtime", "Telemetry", "Record", "Output")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    ENVIRONMENT_FIELD_NUMBER: _ClassVar[int]
    RESOURCE_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row20
    Definition: Row21
    Traceability: Row22
    Environment: Row23
    Resource: Row24
    Operator: Row25
    Runtime: Row26
    Telemetry: Row27
    Record: Row28
    Output: Row29
    def __init__(self, Unspecified: _Optional[_Union[Row20, _Mapping]] = ..., Definition: _Optional[_Union[Row21, _Mapping]] = ..., Traceability: _Optional[_Union[Row22, _Mapping]] = ..., Environment: _Optional[_Union[Row23, _Mapping]] = ..., Resource: _Optional[_Union[Row24, _Mapping]] = ..., Operator: _Optional[_Union[Row25, _Mapping]] = ..., Runtime: _Optional[_Union[Row26, _Mapping]] = ..., Telemetry: _Optional[_Union[Row27, _Mapping]] = ..., Record: _Optional[_Union[Row28, _Mapping]] = ..., Output: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row30]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row30, _Mapping]]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row11
    family: Row31
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row11, _Mapping]] = ..., family: _Optional[_Union[Row31, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row13
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row33
    def __init__(self, kind: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
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
    Unspecified: Row34
    Count: Row35
    Distance: Row36
    Duration: Row37
    Frequency: Row38
    Power: Row39
    Temperature: Row40
    Voltage: Row41
    Ratio: Row42
    Custom: Row43
    def __init__(self, Unspecified: _Optional[_Union[Row34, _Mapping]] = ..., Count: _Optional[_Union[Row35, _Mapping]] = ..., Distance: _Optional[_Union[Row36, _Mapping]] = ..., Duration: _Optional[_Union[Row37, _Mapping]] = ..., Frequency: _Optional[_Union[Row38, _Mapping]] = ..., Power: _Optional[_Union[Row39, _Mapping]] = ..., Temperature: _Optional[_Union[Row40, _Mapping]] = ..., Voltage: _Optional[_Union[Row41, _Mapping]] = ..., Ratio: _Optional[_Union[Row42, _Mapping]] = ..., Custom: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row33(_message.Message):
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
    Null: Row44
    Bool: Row45
    Integer: Row46
    Unsigned: Row47
    Number: Row48
    Text: Row49
    Datetime: Row50
    Array: Row51
    Object: Row52
    def __init__(self, Null: _Optional[_Union[Row44, _Mapping]] = ..., Bool: _Optional[_Union[Row45, _Mapping]] = ..., Integer: _Optional[_Union[Row46, _Mapping]] = ..., Unsigned: _Optional[_Union[Row47, _Mapping]] = ..., Number: _Optional[_Union[Row48, _Mapping]] = ..., Text: _Optional[_Union[Row49, _Mapping]] = ..., Datetime: _Optional[_Union[Row50, _Mapping]] = ..., Array: _Optional[_Union[Row51, _Mapping]] = ..., Object: _Optional[_Union[Row52, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row19
    def __init__(self, field_0: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row53
    def __init__(self, field_0: _Optional[_Union[Row53, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row54
    def __init__(self, field_0: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row55]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row55, _Mapping]]] = ...) -> None: ...

class Row55(_message.Message):
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
