from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NullValue(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    NULL_VALUE_NULL: _ClassVar[NullValue]
NULL_VALUE_NULL: NullValue

class Value(_message.Message):
    __slots__ = ("null_value", "bool_value", "integer_value", "number_value", "text_value", "array_value", "object_value", "datetime_value", "unsigned_value")
    NULL_VALUE_FIELD_NUMBER: _ClassVar[int]
    BOOL_VALUE_FIELD_NUMBER: _ClassVar[int]
    INTEGER_VALUE_FIELD_NUMBER: _ClassVar[int]
    NUMBER_VALUE_FIELD_NUMBER: _ClassVar[int]
    TEXT_VALUE_FIELD_NUMBER: _ClassVar[int]
    ARRAY_VALUE_FIELD_NUMBER: _ClassVar[int]
    OBJECT_VALUE_FIELD_NUMBER: _ClassVar[int]
    DATETIME_VALUE_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_VALUE_FIELD_NUMBER: _ClassVar[int]
    null_value: NullValue
    bool_value: bool
    integer_value: int
    number_value: float
    text_value: str
    array_value: ValueList
    object_value: ValueObject
    datetime_value: str
    unsigned_value: int
    def __init__(self, null_value: _Optional[_Union[NullValue, str]] = ..., bool_value: _Optional[bool] = ..., integer_value: _Optional[int] = ..., number_value: _Optional[float] = ..., text_value: _Optional[str] = ..., array_value: _Optional[_Union[ValueList, _Mapping]] = ..., object_value: _Optional[_Union[ValueObject, _Mapping]] = ..., datetime_value: _Optional[str] = ..., unsigned_value: _Optional[int] = ...) -> None: ...

class ValueList(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[Value]
    def __init__(self, values: _Optional[_Iterable[_Union[Value, _Mapping]]] = ...) -> None: ...

class ValueObject(_message.Message):
    __slots__ = ("fields",)
    class FieldsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Value, _Mapping]] = ...) -> None: ...
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.MessageMap[str, Value]
    def __init__(self, fields: _Optional[_Mapping[str, Value]] = ...) -> None: ...
