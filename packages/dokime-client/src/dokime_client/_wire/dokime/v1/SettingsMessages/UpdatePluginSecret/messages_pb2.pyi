from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("plugin_id", "secret_id", "secret")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    SECRET_ID_FIELD_NUMBER: _ClassVar[int]
    SECRET_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    secret_id: str
    secret: str
    def __init__(self, plugin_id: _Optional[str] = ..., secret_id: _Optional[str] = ..., secret: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("settings", "values", "secret_bindings", "discovered")
    SETTINGS_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SECRET_BINDINGS_FIELD_NUMBER: _ClassVar[int]
    DISCOVERED_FIELD_NUMBER: _ClassVar[int]
    settings: Row3
    values: Row4
    secret_bindings: Row5
    discovered: Row4
    def __init__(self, settings: _Optional[_Union[Row3, _Mapping]] = ..., values: _Optional[_Union[Row4, _Mapping]] = ..., secret_bindings: _Optional[_Union[Row5, _Mapping]] = ..., discovered: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("contributions",)
    CONTRIBUTIONS_FIELD_NUMBER: _ClassVar[int]
    contributions: _containers.RepeatedCompositeFieldContainer[Row6]
    def __init__(self, contributions: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row7]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("bindings", "missing_required")
    BINDINGS_FIELD_NUMBER: _ClassVar[int]
    MISSING_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    bindings: _containers.RepeatedCompositeFieldContainer[Row8]
    missing_required: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, bindings: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., missing_required: _Optional[_Iterable[str]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row9
    def __init__(self, kind: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row10
    def __init__(self, kind: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row11
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Boolean", "TextArea", "Path", "Number", "MultipleChoice", "SecretReference")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    TEXTAREA_FIELD_NUMBER: _ClassVar[int]
    PATH_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    MULTIPLECHOICE_FIELD_NUMBER: _ClassVar[int]
    SECRETREFERENCE_FIELD_NUMBER: _ClassVar[int]
    Boolean: Row12
    TextArea: Row13
    Path: Row14
    Number: Row15
    MultipleChoice: Row16
    SecretReference: Row17
    def __init__(self, Boolean: _Optional[_Union[Row12, _Mapping]] = ..., TextArea: _Optional[_Union[Row13, _Mapping]] = ..., Path: _Optional[_Union[Row14, _Mapping]] = ..., Number: _Optional[_Union[Row15, _Mapping]] = ..., MultipleChoice: _Optional[_Union[Row16, _Mapping]] = ..., SecretReference: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
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
    Null: Row18
    Bool: Row19
    Integer: Row20
    Unsigned: Row21
    Number: Row22
    Text: Row23
    Datetime: Row24
    Array: Row25
    Object: Row26
    def __init__(self, Null: _Optional[_Union[Row18, _Mapping]] = ..., Bool: _Optional[_Union[Row19, _Mapping]] = ..., Integer: _Optional[_Union[Row20, _Mapping]] = ..., Unsigned: _Optional[_Union[Row21, _Mapping]] = ..., Number: _Optional[_Union[Row22, _Mapping]] = ..., Text: _Optional[_Union[Row23, _Mapping]] = ..., Datetime: _Optional[_Union[Row24, _Mapping]] = ..., Array: _Optional[_Union[Row25, _Mapping]] = ..., Object: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("secret_id", "backend", "configured")
    SECRET_ID_FIELD_NUMBER: _ClassVar[int]
    BACKEND_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_FIELD_NUMBER: _ClassVar[int]
    secret_id: str
    backend: Row27
    configured: bool
    def __init__(self, secret_id: _Optional[str] = ..., backend: _Optional[_Union[Row27, _Mapping]] = ..., configured: _Optional[bool] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row28
    def __init__(self, field_0: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row31
    def __init__(self, field_0: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row35
    def __init__(self, field_0: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row36
    def __init__(self, field_0: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("Unspecified", "OsKeyring", "Memory", "LocalFile")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OSKEYRING_FIELD_NUMBER: _ClassVar[int]
    MEMORY_FIELD_NUMBER: _ClassVar[int]
    LOCALFILE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row37
    OsKeyring: Row38
    Memory: Row39
    LocalFile: Row40
    def __init__(self, Unspecified: _Optional[_Union[Row37, _Mapping]] = ..., OsKeyring: _Optional[_Union[Row38, _Mapping]] = ..., Memory: _Optional[_Union[Row39, _Mapping]] = ..., LocalFile: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("id", "title", "description", "default")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[bool] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., required: _Optional[bool] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., required: _Optional[bool] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("id", "title", "description", "default", "min", "max", "step")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: float
    min: float
    max: float
    step: float
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[float] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("id", "title", "description", "default", "choices", "provider")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    PROVIDER_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: str
    choices: _containers.RepeatedCompositeFieldContainer[Row41]
    provider: str
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[str] = ..., choices: _Optional[_Iterable[_Union[Row41, _Mapping]]] = ..., provider: _Optional[str] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("id", "title", "description", "default", "required")
    ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    id: str
    title: str
    description: str
    default: Row42
    required: bool
    def __init__(self, id: _Optional[str] = ..., title: _Optional[str] = ..., description: _Optional[str] = ..., default: _Optional[_Union[Row42, _Mapping]] = ..., required: _Optional[bool] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row43]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row43, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ("value", "title")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    value: str
    title: str
    def __init__(self, value: _Optional[str] = ..., title: _Optional[str] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row44
    def __init__(self, kind: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row43(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("Environment", "StoredSecret")
    ENVIRONMENT_FIELD_NUMBER: _ClassVar[int]
    STOREDSECRET_FIELD_NUMBER: _ClassVar[int]
    Environment: Row45
    StoredSecret: Row46
    def __init__(self, Environment: _Optional[_Union[Row45, _Mapping]] = ..., StoredSecret: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
