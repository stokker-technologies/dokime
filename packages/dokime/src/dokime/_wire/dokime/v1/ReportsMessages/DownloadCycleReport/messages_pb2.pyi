from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "identifier")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    identifier: str
    def __init__(self, cycle_id: _Optional[str] = ..., identifier: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row4
    def __init__(self, payload: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Report", "Chunk", "Complete")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    Report: Row5
    Chunk: Row6
    Complete: Row7
    def __init__(self, Report: _Optional[_Union[Row5, _Mapping]] = ..., Chunk: _Optional[_Union[Row6, _Mapping]] = ..., Complete: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("test_cycle_id", "test_execution_id", "identifier", "metadata", "generated_at", "origin", "warnings")
    TEST_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    test_cycle_id: str
    test_execution_id: str
    identifier: str
    metadata: Row11
    generated_at: Row12
    origin: Row13
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, test_cycle_id: _Optional[str] = ..., test_execution_id: _Optional[str] = ..., identifier: _Optional[str] = ..., metadata: _Optional[_Union[Row11, _Mapping]] = ..., generated_at: _Optional[_Union[Row12, _Mapping]] = ..., origin: _Optional[_Union[Row13, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

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
    attributes: _containers.RepeatedCompositeFieldContainer[Row14]
    created_at: Row12
    updated_at: Row12
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row14, _Mapping]]] = ..., created_at: _Optional[_Union[Row12, _Mapping]] = ..., updated_at: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Sealed", "Current")
    SEALED_FIELD_NUMBER: _ClassVar[int]
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    Sealed: Row15
    Current: Row16
    def __init__(self, Sealed: _Optional[_Union[Row15, _Mapping]] = ..., Current: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row17
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row18]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row19
    def __init__(self, kind: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
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
    Null: Row20
    Bool: Row21
    Integer: Row22
    Unsigned: Row23
    Number: Row24
    Text: Row25
    Datetime: Row26
    Array: Row27
    Object: Row28
    def __init__(self, Null: _Optional[_Union[Row20, _Mapping]] = ..., Bool: _Optional[_Union[Row21, _Mapping]] = ..., Integer: _Optional[_Union[Row22, _Mapping]] = ..., Unsigned: _Optional[_Union[Row23, _Mapping]] = ..., Number: _Optional[_Union[Row24, _Mapping]] = ..., Text: _Optional[_Union[Row25, _Mapping]] = ..., Datetime: _Optional[_Union[Row26, _Mapping]] = ..., Array: _Optional[_Union[Row27, _Mapping]] = ..., Object: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row12
    def __init__(self, field_0: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row31]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row31, _Mapping]]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
