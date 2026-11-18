import datetime

from dokime_sdk._wire.host.v1 import cycles_pb2 as _cycles_pb2
from dokime_sdk._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import execution_pb2 as _execution_pb2
from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LiveExecutionState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LIVE_EXECUTION_STATE_UNSPECIFIED: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_QUEUED: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_INITIALIZING: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_STEPPING: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_EVALUATING: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_CLEANING_UP: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_FINISHED: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_CANCELLED: _ClassVar[LiveExecutionState]
    LIVE_EXECUTION_STATE_INTERRUPTED: _ClassVar[LiveExecutionState]

class CriterionStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CRITERION_STATUS_UNSPECIFIED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_PENDING: _ClassVar[CriterionStatus]
    CRITERION_STATUS_PASSED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_FAILED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_INDETERMINATE: _ClassVar[CriterionStatus]
    CRITERION_STATUS_SKIPPED: _ClassVar[CriterionStatus]

class ExecutionMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_MODE_UNSPECIFIED: _ClassVar[ExecutionMode]
    EXECUTION_MODE_AUTOMATIC: _ClassVar[ExecutionMode]
    EXECUTION_MODE_INTERACTIVE: _ClassVar[ExecutionMode]

class ExecutionOrderKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_ORDER_KEY_UNSPECIFIED: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_STARTED_AT: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_FINISHED_AT: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_TITLE: _ClassVar[ExecutionOrderKey]

class ExecutionParameterSet(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_PARAMETER_SET_UNSPECIFIED: _ClassVar[ExecutionParameterSet]
    EXECUTION_PARAMETER_SET_INPUTS: _ClassVar[ExecutionParameterSet]
    EXECUTION_PARAMETER_SET_OUTPUTS: _ClassVar[ExecutionParameterSet]

class ParameterSource(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PARAMETER_SOURCE_UNSPECIFIED: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_DEFINITION: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_TRACEABILITY: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_ENVIRONMENT: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_RESOURCE: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_OPERATOR: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_RUNTIME: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_TELEMETRY: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_RECORD: _ClassVar[ParameterSource]
    PARAMETER_SOURCE_OUTPUT: _ClassVar[ParameterSource]

class ReportState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    REPORT_STATE_UNSPECIFIED: _ClassVar[ReportState]
    REPORT_STATE_PENDING: _ClassVar[ReportState]
    REPORT_STATE_READY: _ClassVar[ReportState]
    REPORT_STATE_FAILED: _ClassVar[ReportState]

class StepStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STEP_STATUS_UNSPECIFIED: _ClassVar[StepStatus]
    STEP_STATUS_PENDING: _ClassVar[StepStatus]
    STEP_STATUS_RUNNING: _ClassVar[StepStatus]
    STEP_STATUS_PASSED: _ClassVar[StepStatus]
    STEP_STATUS_FAILED: _ClassVar[StepStatus]
    STEP_STATUS_SKIPPED: _ClassVar[StepStatus]
    STEP_STATUS_CANCELLED: _ClassVar[StepStatus]

class ExecutionState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_STATE_UNSPECIFIED: _ClassVar[ExecutionState]
    EXECUTION_STATE_OPEN: _ClassVar[ExecutionState]
    EXECUTION_STATE_CLOSED: _ClassVar[ExecutionState]
    EXECUTION_STATE_SEALED: _ClassVar[ExecutionState]
LIVE_EXECUTION_STATE_UNSPECIFIED: LiveExecutionState
LIVE_EXECUTION_STATE_QUEUED: LiveExecutionState
LIVE_EXECUTION_STATE_INITIALIZING: LiveExecutionState
LIVE_EXECUTION_STATE_STEPPING: LiveExecutionState
LIVE_EXECUTION_STATE_EVALUATING: LiveExecutionState
LIVE_EXECUTION_STATE_CLEANING_UP: LiveExecutionState
LIVE_EXECUTION_STATE_FINISHED: LiveExecutionState
LIVE_EXECUTION_STATE_CANCELLED: LiveExecutionState
LIVE_EXECUTION_STATE_INTERRUPTED: LiveExecutionState
CRITERION_STATUS_UNSPECIFIED: CriterionStatus
CRITERION_STATUS_PENDING: CriterionStatus
CRITERION_STATUS_PASSED: CriterionStatus
CRITERION_STATUS_FAILED: CriterionStatus
CRITERION_STATUS_INDETERMINATE: CriterionStatus
CRITERION_STATUS_SKIPPED: CriterionStatus
EXECUTION_MODE_UNSPECIFIED: ExecutionMode
EXECUTION_MODE_AUTOMATIC: ExecutionMode
EXECUTION_MODE_INTERACTIVE: ExecutionMode
EXECUTION_ORDER_KEY_UNSPECIFIED: ExecutionOrderKey
EXECUTION_ORDER_KEY_STARTED_AT: ExecutionOrderKey
EXECUTION_ORDER_KEY_FINISHED_AT: ExecutionOrderKey
EXECUTION_ORDER_KEY_TITLE: ExecutionOrderKey
EXECUTION_PARAMETER_SET_UNSPECIFIED: ExecutionParameterSet
EXECUTION_PARAMETER_SET_INPUTS: ExecutionParameterSet
EXECUTION_PARAMETER_SET_OUTPUTS: ExecutionParameterSet
PARAMETER_SOURCE_UNSPECIFIED: ParameterSource
PARAMETER_SOURCE_DEFINITION: ParameterSource
PARAMETER_SOURCE_TRACEABILITY: ParameterSource
PARAMETER_SOURCE_ENVIRONMENT: ParameterSource
PARAMETER_SOURCE_RESOURCE: ParameterSource
PARAMETER_SOURCE_OPERATOR: ParameterSource
PARAMETER_SOURCE_RUNTIME: ParameterSource
PARAMETER_SOURCE_TELEMETRY: ParameterSource
PARAMETER_SOURCE_RECORD: ParameterSource
PARAMETER_SOURCE_OUTPUT: ParameterSource
REPORT_STATE_UNSPECIFIED: ReportState
REPORT_STATE_PENDING: ReportState
REPORT_STATE_READY: ReportState
REPORT_STATE_FAILED: ReportState
STEP_STATUS_UNSPECIFIED: StepStatus
STEP_STATUS_PENDING: StepStatus
STEP_STATUS_RUNNING: StepStatus
STEP_STATUS_PASSED: StepStatus
STEP_STATUS_FAILED: StepStatus
STEP_STATUS_SKIPPED: StepStatus
STEP_STATUS_CANCELLED: StepStatus
EXECUTION_STATE_UNSPECIFIED: ExecutionState
EXECUTION_STATE_OPEN: ExecutionState
EXECUTION_STATE_CLOSED: ExecutionState
EXECUTION_STATE_SEALED: ExecutionState

class ExecutionsGetRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class ExecutionsGetResponse(_message.Message):
    __slots__ = ("value", "revision", "execution", "progress", "resolved_at")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: ExecutionWritable
    revision: bytes
    execution: ExecutionDetail
    progress: _execution_pb2.ExecutionProgressValue
    resolved_at: _position_pb2.Position
    def __init__(self, value: _Optional[_Union[ExecutionWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., execution: _Optional[_Union[ExecutionDetail, _Mapping]] = ..., progress: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class ExecutionsSetRequest(_message.Message):
    __slots__ = ("name", "value", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: ExecutionWritable
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[ExecutionWritable, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsSetResponse(_message.Message):
    __slots__ = ("value", "revision", "receipt")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: ExecutionWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[ExecutionWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsListRequest(_message.Message):
    __slots__ = ("parent", "selection", "filter", "order", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    selection: ExecutionSelection
    filter: ExecutionFilter
    order: ExecutionOrder
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., selection: _Optional[_Union[ExecutionSelection, _Mapping]] = ..., filter: _Optional[_Union[ExecutionFilter, _Mapping]] = ..., order: _Optional[_Union[ExecutionOrder, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class ExecutionsListResponse(_message.Message):
    __slots__ = ("executions", "outcome_counts", "resolved_at", "next_page_token")
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_COUNTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    executions: _containers.RepeatedCompositeFieldContainer[ExecutionDetail]
    outcome_counts: ExecutionOutcomeCounts
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, executions: _Optional[_Iterable[_Union[ExecutionDetail, _Mapping]]] = ..., outcome_counts: _Optional[_Union[ExecutionOutcomeCounts, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

class ExecutionsWatchRequest(_message.Message):
    __slots__ = ("parent", "selection", "filter", "order", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    selection: ExecutionSelection
    filter: ExecutionFilter
    order: ExecutionOrder
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., selection: _Optional[_Union[ExecutionSelection, _Mapping]] = ..., filter: _Optional[_Union[ExecutionFilter, _Mapping]] = ..., order: _Optional[_Union[ExecutionOrder, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class ExecutionsWatchResponse(_message.Message):
    __slots__ = ("executions", "outcome_counts", "resolved_at", "opening", "resync_required")
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_COUNTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    executions: _containers.RepeatedCompositeFieldContainer[ExecutionDetail]
    outcome_counts: ExecutionOutcomeCounts
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, executions: _Optional[_Iterable[_Union[ExecutionDetail, _Mapping]]] = ..., outcome_counts: _Optional[_Union[ExecutionOutcomeCounts, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class ExecutionsCreateRequest(_message.Message):
    __slots__ = ("parent", "executions", "request_id")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    parent: str
    executions: _containers.RepeatedCompositeFieldContainer[ExecutionBinding]
    request_id: str
    def __init__(self, parent: _Optional[str] = ..., executions: _Optional[_Iterable[_Union[ExecutionBinding, _Mapping]]] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsCreateResponse(_message.Message):
    __slots__ = ("executions", "receipt")
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    executions: _containers.RepeatedCompositeFieldContainer[ExecutionDetail]
    receipt: _receipt_pb2.Receipt
    def __init__(self, executions: _Optional[_Iterable[_Union[ExecutionDetail, _Mapping]]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsStartRequest(_message.Message):
    __slots__ = ("name", "request_id", "mode")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    mode: ExecutionMode
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ..., mode: _Optional[_Union[ExecutionMode, str]] = ...) -> None: ...

class ExecutionsStartResponse(_message.Message):
    __slots__ = ("execution", "receipt")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    execution: ExecutionDetail
    receipt: _receipt_pb2.Receipt
    def __init__(self, execution: _Optional[_Union[ExecutionDetail, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsStopRequest(_message.Message):
    __slots__ = ("name", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsStopResponse(_message.Message):
    __slots__ = ("execution", "receipt")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    execution: ExecutionDetail
    receipt: _receipt_pb2.Receipt
    def __init__(self, execution: _Optional[_Union[ExecutionDetail, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsCancelRequest(_message.Message):
    __slots__ = ("name", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsCancelResponse(_message.Message):
    __slots__ = ("execution", "receipt")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    execution: ExecutionDetail
    receipt: _receipt_pb2.Receipt
    def __init__(self, execution: _Optional[_Union[ExecutionDetail, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsSealRequest(_message.Message):
    __slots__ = ("name", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsSealResponse(_message.Message):
    __slots__ = ("outcome", "receipt")
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    outcome: ExecutionOutcome
    receipt: _receipt_pb2.Receipt
    def __init__(self, outcome: _Optional[_Union[ExecutionOutcome, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsDeleteRequest(_message.Message):
    __slots__ = ("name", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class ExecutionsDeleteResponse(_message.Message):
    __slots__ = ("execution", "receipt")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    execution: ExecutionDetail
    receipt: _receipt_pb2.Receipt
    def __init__(self, execution: _Optional[_Union[ExecutionDetail, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ExecutionsParametersRequest(_message.Message):
    __slots__ = ("name", "set", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SET_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    set: ExecutionParameterSet
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., set: _Optional[_Union[ExecutionParameterSet, str]] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class ExecutionsParametersResponse(_message.Message):
    __slots__ = ("parameters", "resolved_at")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[MaterializedParameter]
    resolved_at: _position_pb2.Position
    def __init__(self, parameters: _Optional[_Iterable[_Union[MaterializedParameter, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class ExecutionsStepsRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class ExecutionsStepsResponse(_message.Message):
    __slots__ = ("steps", "resolved_at")
    STEPS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    steps: _containers.RepeatedCompositeFieldContainer[ExecutionStep]
    resolved_at: _position_pb2.Position
    def __init__(self, steps: _Optional[_Iterable[_Union[ExecutionStep, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class FrozenSchemas(_message.Message):
    __slots__ = ("digest", "records", "streams")
    DIGEST_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    digest: bytes
    records: _containers.RepeatedCompositeFieldContainer[_record_schemas_pb2.RecordSchema]
    streams: _containers.RepeatedCompositeFieldContainer[_record_schemas_pb2.TelemetrySchema]
    def __init__(self, digest: _Optional[bytes] = ..., records: _Optional[_Iterable[_Union[_record_schemas_pb2.RecordSchema, _Mapping]]] = ..., streams: _Optional[_Iterable[_Union[_record_schemas_pb2.TelemetrySchema, _Mapping]]] = ...) -> None: ...

class Diagnostic(_message.Message):
    __slots__ = ("message", "chain", "location", "span_trace", "backtrace")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CHAIN_FIELD_NUMBER: _ClassVar[int]
    LOCATION_FIELD_NUMBER: _ClassVar[int]
    SPAN_TRACE_FIELD_NUMBER: _ClassVar[int]
    BACKTRACE_FIELD_NUMBER: _ClassVar[int]
    message: str
    chain: _containers.RepeatedScalarFieldContainer[str]
    location: str
    span_trace: _containers.RepeatedScalarFieldContainer[str]
    backtrace: str
    def __init__(self, message: _Optional[str] = ..., chain: _Optional[_Iterable[str]] = ..., location: _Optional[str] = ..., span_trace: _Optional[_Iterable[str]] = ..., backtrace: _Optional[str] = ...) -> None: ...

class ExecutionBinding(_message.Message):
    __slots__ = ("execution_id", "case_id")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class ExecutionControls(_message.Message):
    __slots__ = ("request_id", "paused", "stop_requested", "can_pause", "can_resume", "can_stop", "mode", "can_start", "can_rerun", "can_cancel", "can_inspect", "can_export")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    STOP_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    CAN_PAUSE_FIELD_NUMBER: _ClassVar[int]
    CAN_RESUME_FIELD_NUMBER: _ClassVar[int]
    CAN_STOP_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    CAN_START_FIELD_NUMBER: _ClassVar[int]
    CAN_RERUN_FIELD_NUMBER: _ClassVar[int]
    CAN_CANCEL_FIELD_NUMBER: _ClassVar[int]
    CAN_INSPECT_FIELD_NUMBER: _ClassVar[int]
    CAN_EXPORT_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    paused: bool
    stop_requested: bool
    can_pause: bool
    can_resume: bool
    can_stop: bool
    mode: ExecutionMode
    can_start: bool
    can_rerun: bool
    can_cancel: bool
    can_inspect: bool
    can_export: bool
    def __init__(self, request_id: _Optional[str] = ..., paused: _Optional[bool] = ..., stop_requested: _Optional[bool] = ..., can_pause: _Optional[bool] = ..., can_resume: _Optional[bool] = ..., can_stop: _Optional[bool] = ..., mode: _Optional[_Union[ExecutionMode, str]] = ..., can_start: _Optional[bool] = ..., can_rerun: _Optional[bool] = ..., can_cancel: _Optional[bool] = ..., can_inspect: _Optional[bool] = ..., can_export: _Optional[bool] = ...) -> None: ...

class ExecutionOutcomeCounts(_message.Message):
    __slots__ = ("evaluated_count", "not_evaluated_count", "declared_count", "result_status")
    EVALUATED_COUNT_FIELD_NUMBER: _ClassVar[int]
    NOT_EVALUATED_COUNT_FIELD_NUMBER: _ClassVar[int]
    DECLARED_COUNT_FIELD_NUMBER: _ClassVar[int]
    RESULT_STATUS_FIELD_NUMBER: _ClassVar[int]
    evaluated_count: int
    not_evaluated_count: int
    declared_count: int
    result_status: _cycles_pb2.ResultStatus
    def __init__(self, evaluated_count: _Optional[int] = ..., not_evaluated_count: _Optional[int] = ..., declared_count: _Optional[int] = ..., result_status: _Optional[_Union[_cycles_pb2.ResultStatus, str]] = ...) -> None: ...

class ExecutionDetail(_message.Message):
    __slots__ = ("progress", "resolved_at", "cycle_counts", "seal", "report", "summary", "cycle_id", "scope", "admitted_suite", "admitted_template", "started_at", "finished_at", "input_parameter_count", "output_parameter_count", "step_count", "controls", "origin", "outcome", "schemas", "live_state")
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_COUNTS_FIELD_NUMBER: _ClassVar[int]
    SEAL_FIELD_NUMBER: _ClassVar[int]
    REPORT_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_SUITE_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    INPUT_PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    STEP_COUNT_FIELD_NUMBER: _ClassVar[int]
    CONTROLS_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    LIVE_STATE_FIELD_NUMBER: _ClassVar[int]
    progress: _execution_pb2.ExecutionProgressValue
    resolved_at: _position_pb2.Position
    cycle_counts: ExecutionOutcomeCounts
    seal: bytes
    report: ReportStatus
    summary: ExecutionSummary
    cycle_id: str
    scope: _evidence_pb2.EvidenceScope
    admitted_suite: _metadata_pb2.Metadata
    admitted_template: _metadata_pb2.Metadata
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    input_parameter_count: int
    output_parameter_count: int
    step_count: int
    controls: ExecutionControls
    origin: _execution_pb2.Origin
    outcome: ExecutionOutcome
    schemas: FrozenSchemas
    live_state: LiveExecutionState
    def __init__(self, progress: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., cycle_counts: _Optional[_Union[ExecutionOutcomeCounts, _Mapping]] = ..., seal: _Optional[bytes] = ..., report: _Optional[_Union[ReportStatus, _Mapping]] = ..., summary: _Optional[_Union[ExecutionSummary, _Mapping]] = ..., cycle_id: _Optional[str] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., admitted_suite: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., admitted_template: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., input_parameter_count: _Optional[int] = ..., output_parameter_count: _Optional[int] = ..., step_count: _Optional[int] = ..., controls: _Optional[_Union[ExecutionControls, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ..., outcome: _Optional[_Union[ExecutionOutcome, _Mapping]] = ..., schemas: _Optional[_Union[FrozenSchemas, _Mapping]] = ..., live_state: _Optional[_Union[LiveExecutionState, str]] = ...) -> None: ...

class ExecutionFilter(_message.Message):
    __slots__ = ("text", "statuses", "template_ids", "cycle_ids", "started_after", "started_before", "archive")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AFTER_FIELD_NUMBER: _ClassVar[int]
    STARTED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedScalarFieldContainer[ExecutionState]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    started_after: _timestamp_pb2.Timestamp
    started_before: _timestamp_pb2.Timestamp
    archive: _cycles_pb2.ArchiveScope
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[ExecutionState, str]]] = ..., template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., started_after: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., started_before: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., archive: _Optional[_Union[_cycles_pb2.ArchiveScope, str]] = ...) -> None: ...

class ExecutionOrder(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: ExecutionOrderKey
    descending: bool
    def __init__(self, key: _Optional[_Union[ExecutionOrderKey, str]] = ..., descending: _Optional[bool] = ...) -> None: ...

class ExecutionSelection(_message.Message):
    __slots__ = ("template_ids", "cycle_ids", "statuses", "started_at", "archive")
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    statuses: _containers.RepeatedScalarFieldContainer[ExecutionState]
    started_at: _evidence_pb2.Window
    archive: _cycles_pb2.ArchiveScope
    def __init__(self, template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., statuses: _Optional[_Iterable[_Union[ExecutionState, str]]] = ..., started_at: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., archive: _Optional[_Union[_cycles_pb2.ArchiveScope, str]] = ...) -> None: ...

class ExecutionStep(_message.Message):
    __slots__ = ("step_id", "metadata", "status", "started_at", "finished_at", "measurement_count", "criterion_count", "ordinal", "detail", "criteria")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENT_COUNT_FIELD_NUMBER: _ClassVar[int]
    CRITERION_COUNT_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    metadata: _metadata_pb2.Metadata
    status: StepStatus
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    measurement_count: int
    criterion_count: int
    ordinal: int
    detail: Diagnostic
    criteria: _containers.RepeatedCompositeFieldContainer[StepCriterion]
    def __init__(self, step_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[StepStatus, str]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., measurement_count: _Optional[int] = ..., criterion_count: _Optional[int] = ..., ordinal: _Optional[int] = ..., detail: _Optional[_Union[Diagnostic, _Mapping]] = ..., criteria: _Optional[_Iterable[_Union[StepCriterion, _Mapping]]] = ...) -> None: ...

class ExecutionSummary(_message.Message):
    __slots__ = ("controls", "execution_id", "case_id", "ordinal", "archived", "state", "outcome")
    CONTROLS_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    controls: ExecutionControls
    execution_id: str
    case_id: str
    ordinal: int
    archived: bool
    state: ExecutionState
    outcome: ExecutionOutcome
    def __init__(self, controls: _Optional[_Union[ExecutionControls, _Mapping]] = ..., execution_id: _Optional[str] = ..., case_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., archived: _Optional[bool] = ..., state: _Optional[_Union[ExecutionState, str]] = ..., outcome: _Optional[_Union[ExecutionOutcome, _Mapping]] = ...) -> None: ...

class MaterializedParameter(_message.Message):
    __slots__ = ("name", "metadata", "group_id", "source", "value", "unit")
    NAME_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    name: str
    metadata: _metadata_pb2.Metadata
    group_id: str
    source: ParameterSource
    value: _scalar_pb2.Value
    unit: _record_schemas_pb2.MeasurementUnit
    def __init__(self, name: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., group_id: _Optional[str] = ..., source: _Optional[_Union[ParameterSource, str]] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., unit: _Optional[_Union[_record_schemas_pb2.MeasurementUnit, _Mapping]] = ...) -> None: ...

class ReportStatus(_message.Message):
    __slots__ = ("state", "failure")
    STATE_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    state: ReportState
    failure: str
    def __init__(self, state: _Optional[_Union[ReportState, str]] = ..., failure: _Optional[str] = ...) -> None: ...

class StepCriterion(_message.Message):
    __slots__ = ("criterion_id", "metadata", "status", "observed", "message", "ordinal")
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    criterion_id: str
    metadata: _metadata_pb2.Metadata
    status: CriterionStatus
    observed: _scalar_pb2.Value
    message: str
    ordinal: int
    def __init__(self, criterion_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[CriterionStatus, str]] = ..., observed: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., message: _Optional[str] = ..., ordinal: _Optional[int] = ...) -> None: ...

class ExecutionWritable(_message.Message):
    __slots__ = ("label", "paused", "archived")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    label: str
    paused: bool
    archived: bool
    def __init__(self, label: _Optional[str] = ..., paused: _Optional[bool] = ..., archived: _Optional[bool] = ...) -> None: ...

class ExecutionOutcome(_message.Message):
    __slots__ = ("not_evaluated", "evaluated", "declared")
    NOT_EVALUATED_FIELD_NUMBER: _ClassVar[int]
    EVALUATED_FIELD_NUMBER: _ClassVar[int]
    DECLARED_FIELD_NUMBER: _ClassVar[int]
    not_evaluated: UnevaluatedOutcome
    evaluated: EvaluatedOutcome
    declared: DeclaredOutcome
    def __init__(self, not_evaluated: _Optional[_Union[UnevaluatedOutcome, _Mapping]] = ..., evaluated: _Optional[_Union[EvaluatedOutcome, _Mapping]] = ..., declared: _Optional[_Union[DeclaredOutcome, _Mapping]] = ...) -> None: ...

class UnevaluatedOutcome(_message.Message):
    __slots__ = ("termination", "reason")
    TERMINATION_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    termination: _execution_pb2.Termination
    reason: str
    def __init__(self, termination: _Optional[_Union[_execution_pb2.Termination, str]] = ..., reason: _Optional[str] = ...) -> None: ...

class EvaluatedOutcome(_message.Message):
    __slots__ = ("status", "reason")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    status: _cycles_pb2.ResultStatus
    reason: str
    def __init__(self, status: _Optional[_Union[_cycles_pb2.ResultStatus, str]] = ..., reason: _Optional[str] = ...) -> None: ...

class DeclaredOutcome(_message.Message):
    __slots__ = ("status", "reason", "importer")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    IMPORTER_FIELD_NUMBER: _ClassVar[int]
    status: _cycles_pb2.ResultStatus
    reason: str
    importer: str
    def __init__(self, status: _Optional[_Union[_cycles_pb2.ResultStatus, str]] = ..., reason: _Optional[str] = ..., importer: _Optional[str] = ...) -> None: ...
