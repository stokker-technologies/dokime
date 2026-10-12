from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RegisteredSelection(_message.Message):
    __slots__ = ("suite_id", "template_id", "parameters", "files")
    class ParametersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: SelectionValues
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[SelectionValues, _Mapping]] = ...) -> None: ...
    class FilesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: SelectedFile
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[SelectedFile, _Mapping]] = ...) -> None: ...
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    template_id: str
    parameters: _containers.MessageMap[str, SelectionValues]
    files: _containers.MessageMap[str, SelectedFile]
    def __init__(self, suite_id: _Optional[str] = ..., template_id: _Optional[str] = ..., parameters: _Optional[_Mapping[str, SelectionValues]] = ..., files: _Optional[_Mapping[str, SelectedFile]] = ...) -> None: ...

class SelectionValues(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_value_pb2.Value]
    def __init__(self, values: _Optional[_Iterable[_Union[_value_pb2.Value, _Mapping]]] = ...) -> None: ...

class SelectedFile(_message.Message):
    __slots__ = ("value", "clear")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    CLEAR_FIELD_NUMBER: _ClassVar[int]
    value: str
    clear: _value_pb2.NullValue
    def __init__(self, value: _Optional[str] = ..., clear: _Optional[_Union[_value_pb2.NullValue, str]] = ...) -> None: ...

class SelectionPlan(_message.Message):
    __slots__ = ("selections", "max_cases", "case_ids")
    SELECTIONS_FIELD_NUMBER: _ClassVar[int]
    MAX_CASES_FIELD_NUMBER: _ClassVar[int]
    CASE_IDS_FIELD_NUMBER: _ClassVar[int]
    selections: _containers.RepeatedCompositeFieldContainer[RegisteredSelection]
    max_cases: int
    case_ids: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, selections: _Optional[_Iterable[_Union[RegisteredSelection, _Mapping]]] = ..., max_cases: _Optional[int] = ..., case_ids: _Optional[_Iterable[str]] = ...) -> None: ...

class RegisterSelectionRequest(_message.Message):
    __slots__ = ("plan", "idempotency_key")
    PLAN_FIELD_NUMBER: _ClassVar[int]
    IDEMPOTENCY_KEY_FIELD_NUMBER: _ClassVar[int]
    plan: SelectionPlan
    idempotency_key: str
    def __init__(self, plan: _Optional[_Union[SelectionPlan, _Mapping]] = ..., idempotency_key: _Optional[str] = ...) -> None: ...

class PreviewSelectionRequest(_message.Message):
    __slots__ = ("plan",)
    PLAN_FIELD_NUMBER: _ClassVar[int]
    plan: SelectionPlan
    def __init__(self, plan: _Optional[_Union[SelectionPlan, _Mapping]] = ...) -> None: ...

class PreviewSelectionReply(_message.Message):
    __slots__ = ("cases", "observed_cursor")
    CASES_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    cases: _containers.RepeatedCompositeFieldContainer[PlannedCase]
    observed_cursor: _common_pb2.Cursor
    def __init__(self, cases: _Optional[_Iterable[_Union[PlannedCase, _Mapping]]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class PlannedCase(_message.Message):
    __slots__ = ("case_id", "suite_id", "template_id", "metadata", "parameters", "ordinal", "selection_index")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SELECTION_INDEX_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    suite_id: str
    template_id: str
    metadata: _common_pb2.Metadata
    parameters: _value_pb2.ValueObject
    ordinal: int
    selection_index: int
    def __init__(self, case_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., parameters: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., ordinal: _Optional[int] = ..., selection_index: _Optional[int] = ...) -> None: ...
