from dokime._wire.host.v1 import executions_pb2 as _executions_pb2
from dokime._wire.model.v1 import execution_pb2 as _execution_pb2
from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CaseStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CASE_STATUS_UNSPECIFIED: _ClassVar[CaseStatus]
    CASE_STATUS_SKIPPED: _ClassVar[CaseStatus]
    CASE_STATUS_INVALIDATED: _ClassVar[CaseStatus]
    CASE_STATUS_PLANNED: _ClassVar[CaseStatus]
    CASE_STATUS_QUEUED: _ClassVar[CaseStatus]
    CASE_STATUS_RUNNING: _ClassVar[CaseStatus]
    CASE_STATUS_PASSED: _ClassVar[CaseStatus]
    CASE_STATUS_FAILED: _ClassVar[CaseStatus]
    CASE_STATUS_BLOCKED: _ClassVar[CaseStatus]
    CASE_STATUS_CANCELLED: _ClassVar[CaseStatus]
    CASE_STATUS_INTERRUPTED: _ClassVar[CaseStatus]
CASE_STATUS_UNSPECIFIED: CaseStatus
CASE_STATUS_SKIPPED: CaseStatus
CASE_STATUS_INVALIDATED: CaseStatus
CASE_STATUS_PLANNED: CaseStatus
CASE_STATUS_QUEUED: CaseStatus
CASE_STATUS_RUNNING: CaseStatus
CASE_STATUS_PASSED: CaseStatus
CASE_STATUS_FAILED: CaseStatus
CASE_STATUS_BLOCKED: CaseStatus
CASE_STATUS_CANCELLED: CaseStatus
CASE_STATUS_INTERRUPTED: CaseStatus

class CasesCreateRequest(_message.Message):
    __slots__ = ("plan", "validate_only", "request_id")
    PLAN_FIELD_NUMBER: _ClassVar[int]
    VALIDATE_ONLY_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    plan: SelectionPlan
    validate_only: bool
    request_id: str
    def __init__(self, plan: _Optional[_Union[SelectionPlan, _Mapping]] = ..., validate_only: _Optional[bool] = ..., request_id: _Optional[str] = ...) -> None: ...

class CasesCreateResponse(_message.Message):
    __slots__ = ("cases", "receipt")
    CASES_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    cases: _containers.RepeatedCompositeFieldContainer[PlannedCase]
    receipt: _receipt_pb2.Receipt
    def __init__(self, cases: _Optional[_Iterable[_Union[PlannedCase, _Mapping]]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class CasesGetRequest(_message.Message):
    __slots__ = ("name", "parent", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    parent: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesGetResponse(_message.Message):
    __slots__ = ("value", "resolved_at")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: CaseDetail
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[CaseDetail, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesListRequest(_message.Message):
    __slots__ = ("parent", "at")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    parent: str
    at: _position_pb2.ReadCut
    def __init__(self, parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesListResponse(_message.Message):
    __slots__ = ("cases", "resolved_at")
    CASES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    cases: _containers.RepeatedCompositeFieldContainer[CaseSummary]
    resolved_at: _position_pb2.ReadCut
    def __init__(self, cases: _Optional[_Iterable[_Union[CaseSummary, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesWatchRequest(_message.Message):
    __slots__ = ("parent", "at")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    parent: str
    at: _position_pb2.ReadCut
    def __init__(self, parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesWatchResponse(_message.Message):
    __slots__ = ("cases", "resolved_at", "opening", "resync_required")
    CASES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    cases: _containers.RepeatedCompositeFieldContainer[CaseSummary]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, cases: _Optional[_Iterable[_Union[CaseSummary, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class CasesDeleteRequest(_message.Message):
    __slots__ = ("name", "parent", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    parent: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., parent: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class CasesDeleteResponse(_message.Message):
    __slots__ = ("receipt",)
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class CasesParametersRequest(_message.Message):
    __slots__ = ("name", "parent", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    parent: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CasesParametersResponse(_message.Message):
    __slots__ = ("parameters", "resolved_at")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[CaseParameter]
    resolved_at: _position_pb2.ReadCut
    def __init__(self, parameters: _Optional[_Iterable[_Union[CaseParameter, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CaseActions(_message.Message):
    __slots__ = ("delete", "unavailable_reasons")
    DELETE_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_REASONS_FIELD_NUMBER: _ClassVar[int]
    delete: bool
    unavailable_reasons: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, delete: _Optional[bool] = ..., unavailable_reasons: _Optional[_Iterable[str]] = ...) -> None: ...

class CaseDetail(_message.Message):
    __slots__ = ("summary", "suite", "parameter_count", "generation_index", "observed_cursor")
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    GENERATION_INDEX_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    summary: CaseSummary
    suite: _metadata_pb2.Metadata
    parameter_count: int
    generation_index: int
    observed_cursor: _position_pb2.ReadCut
    def __init__(self, summary: _Optional[_Union[CaseSummary, _Mapping]] = ..., suite: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., parameter_count: _Optional[int] = ..., generation_index: _Optional[int] = ..., observed_cursor: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class CaseParameter(_message.Message):
    __slots__ = ("name", "value")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: _scalar_pb2.Value
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...

class CaseSummary(_message.Message):
    __slots__ = ("case_id", "template_id", "metadata", "ordinal", "status", "latest_execution_id", "latest_execution_status", "attempt_count", "actions", "suite_id", "latest_execution_outcome")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_STATUS_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_COUNT_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_OUTCOME_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    template_id: str
    metadata: _metadata_pb2.Metadata
    ordinal: int
    status: CaseStatus
    latest_execution_id: str
    latest_execution_status: _executions_pb2.ExecutionState
    attempt_count: int
    actions: CaseActions
    suite_id: str
    latest_execution_outcome: _executions_pb2.ExecutionOutcome
    def __init__(self, case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., ordinal: _Optional[int] = ..., status: _Optional[_Union[CaseStatus, str]] = ..., latest_execution_id: _Optional[str] = ..., latest_execution_status: _Optional[_Union[_executions_pb2.ExecutionState, str]] = ..., attempt_count: _Optional[int] = ..., actions: _Optional[_Union[CaseActions, _Mapping]] = ..., suite_id: _Optional[str] = ..., latest_execution_outcome: _Optional[_Union[_executions_pb2.ExecutionOutcome, _Mapping]] = ...) -> None: ...

class PlannedCase(_message.Message):
    __slots__ = ("case_id", "suite_id", "template_id", "metadata", "ordinal", "selection_index", "origin")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SELECTION_INDEX_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    suite_id: str
    template_id: str
    metadata: _metadata_pb2.Metadata
    ordinal: int
    selection_index: int
    origin: _execution_pb2.Origin
    def __init__(self, case_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., ordinal: _Optional[int] = ..., selection_index: _Optional[int] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

class RegisteredSelection(_message.Message):
    __slots__ = ("suite_id", "template_id", "parameters", "files")
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    template_id: str
    parameters: _containers.RepeatedCompositeFieldContainer[RegisteredSelectionParametersEntry]
    files: _containers.RepeatedCompositeFieldContainer[RegisteredSelectionFilesEntry]
    def __init__(self, suite_id: _Optional[str] = ..., template_id: _Optional[str] = ..., parameters: _Optional[_Iterable[_Union[RegisteredSelectionParametersEntry, _Mapping]]] = ..., files: _Optional[_Iterable[_Union[RegisteredSelectionFilesEntry, _Mapping]]] = ...) -> None: ...

class RegisteredSelectionFilesEntry(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: SelectedFile
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[SelectedFile, _Mapping]] = ...) -> None: ...

class RegisteredSelectionParametersEntry(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: SelectionValues
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[SelectionValues, _Mapping]] = ...) -> None: ...

class SelectedFile(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: SelectedFileKind
    def __init__(self, kind: _Optional[_Union[SelectedFileKind, _Mapping]] = ...) -> None: ...

class SelectedFileKind(_message.Message):
    __slots__ = ("value", "clear")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    CLEAR_FIELD_NUMBER: _ClassVar[int]
    value: str
    clear: _scalar_pb2.Null
    def __init__(self, value: _Optional[str] = ..., clear: _Optional[_Union[_scalar_pb2.Null, _Mapping]] = ...) -> None: ...

class SelectionPlan(_message.Message):
    __slots__ = ("selections", "max_cases", "case_ids", "cycle_id")
    SELECTIONS_FIELD_NUMBER: _ClassVar[int]
    MAX_CASES_FIELD_NUMBER: _ClassVar[int]
    CASE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    selections: _containers.RepeatedCompositeFieldContainer[RegisteredSelection]
    max_cases: int
    case_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_id: str
    def __init__(self, selections: _Optional[_Iterable[_Union[RegisteredSelection, _Mapping]]] = ..., max_cases: _Optional[int] = ..., case_ids: _Optional[_Iterable[str]] = ..., cycle_id: _Optional[str] = ...) -> None: ...

class SelectionValues(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    def __init__(self, values: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ...) -> None: ...
