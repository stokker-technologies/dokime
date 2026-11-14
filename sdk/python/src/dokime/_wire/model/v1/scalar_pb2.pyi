import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
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
    VALUE_TYPE_UNSIGNED_INTEGER: _ClassVar[ValueType]
    VALUE_TYPE_NUMBER: _ClassVar[ValueType]
    VALUE_TYPE_TEXT: _ClassVar[ValueType]
    VALUE_TYPE_BYTES: _ClassVar[ValueType]
    VALUE_TYPE_LIST: _ClassVar[ValueType]
    VALUE_TYPE_DATETIME: _ClassVar[ValueType]
VALUE_TYPE_UNSPECIFIED: ValueType
VALUE_TYPE_BOOLEAN: ValueType
VALUE_TYPE_INTEGER: ValueType
VALUE_TYPE_UNSIGNED_INTEGER: ValueType
VALUE_TYPE_NUMBER: ValueType
VALUE_TYPE_TEXT: ValueType
VALUE_TYPE_BYTES: ValueType
VALUE_TYPE_LIST: ValueType
VALUE_TYPE_DATETIME: ValueType

class Value(_message.Message):
    __slots__ = ("null", "boolean", "integer", "unsigned_integer", "number", "text", "data", "list", "datetime")
    NULL_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    LIST_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    null: Null
    boolean: bool
    integer: int
    unsigned_integer: int
    number: float
    text: str
    data: bytes
    list: ValueList
    datetime: _timestamp_pb2.Timestamp
    def __init__(self, null: _Optional[_Union[Null, _Mapping]] = ..., boolean: _Optional[bool] = ..., integer: _Optional[int] = ..., unsigned_integer: _Optional[int] = ..., number: _Optional[float] = ..., text: _Optional[str] = ..., data: _Optional[bytes] = ..., list: _Optional[_Union[ValueList, _Mapping]] = ..., datetime: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class Null(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ValueList(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[Value]
    def __init__(self, values: _Optional[_Iterable[_Union[Value, _Mapping]]] = ...) -> None: ...

class MapValue(_message.Message):
    __slots__ = ("values",)
    class ValuesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Value, _Mapping]] = ...) -> None: ...
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.MessageMap[str, Value]
    def __init__(self, values: _Optional[_Mapping[str, Value]] = ...) -> None: ...
