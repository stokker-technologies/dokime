import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from dokime.client._generated.dokime.v1 import planning_pb2 as _planning_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CycleOrderKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CYCLE_ORDER_KEY_UNSPECIFIED: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_UPDATED_AT: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_CREATED_AT: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_LABEL: _ClassVar[CycleOrderKey]

class ExecutionMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_MODE_AUTOMATIC: _ClassVar[ExecutionMode]
    EXECUTION_MODE_INTERACTIVE: _ClassVar[ExecutionMode]

class ExecutionCommandAction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_COMMAND_ACTION_UNSPECIFIED: _ClassVar[ExecutionCommandAction]
    EXECUTION_COMMAND_ACTION_NEXT: _ClassVar[ExecutionCommandAction]
    EXECUTION_COMMAND_ACTION_RUN_STEP: _ClassVar[ExecutionCommandAction]
    EXECUTION_COMMAND_ACTION_FINISH: _ClassVar[ExecutionCommandAction]

class ExecutionCommandPhase(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_COMMAND_PHASE_UNSPECIFIED: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_ACCEPTED: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_DISPATCHING: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_RUNNING: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_COMPLETED: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_REFUSED: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_FAILED: _ClassVar[ExecutionCommandPhase]
    EXECUTION_COMMAND_PHASE_INTERRUPTED: _ClassVar[ExecutionCommandPhase]

class RunRequestPhase(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RUN_REQUEST_PHASE_UNSPECIFIED: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_QUEUED: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_PREPARING: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_RUNNING: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_FINISHING: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_COMPLETED: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_FAILED: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_STOPPED: _ClassVar[RunRequestPhase]
    RUN_REQUEST_PHASE_INTERRUPTED: _ClassVar[RunRequestPhase]

class CycleStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CYCLE_STATUS_UNSPECIFIED: _ClassVar[CycleStatus]
    CYCLE_STATUS_PREVIEWED: _ClassVar[CycleStatus]
    CYCLE_STATUS_READY: _ClassVar[CycleStatus]
    CYCLE_STATUS_RUNNING: _ClassVar[CycleStatus]
    CYCLE_STATUS_PAUSED: _ClassVar[CycleStatus]
    CYCLE_STATUS_STOPPING: _ClassVar[CycleStatus]
    CYCLE_STATUS_COMPLETED: _ClassVar[CycleStatus]
    CYCLE_STATUS_FAILED: _ClassVar[CycleStatus]
    CYCLE_STATUS_BLOCKED: _ClassVar[CycleStatus]
    CYCLE_STATUS_STOPPED: _ClassVar[CycleStatus]
    CYCLE_STATUS_CLEARED: _ClassVar[CycleStatus]

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

class ExecutionStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_STATUS_UNSPECIFIED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_INVALIDATED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_PENDING: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_RUNNING: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_PASSED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_FAILED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_BLOCKED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_CANCELLED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_INTERRUPTED: _ClassVar[ExecutionStatus]
    EXECUTION_STATUS_SKIPPED: _ClassVar[ExecutionStatus]

class ResultStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RESULT_STATUS_UNSPECIFIED: _ClassVar[ResultStatus]
    RESULT_STATUS_PENDING: _ClassVar[ResultStatus]
    RESULT_STATUS_PASSED: _ClassVar[ResultStatus]
    RESULT_STATUS_FAILED: _ClassVar[ResultStatus]
    RESULT_STATUS_BLOCKED: _ClassVar[ResultStatus]
    RESULT_STATUS_CANCELLED: _ClassVar[ResultStatus]

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

class StepPageKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STEP_PAGE_KIND_UNSPECIFIED: _ClassVar[StepPageKind]
    STEP_PAGE_KIND_STEPS: _ClassVar[StepPageKind]
    STEP_PAGE_KIND_MEASUREMENTS: _ClassVar[StepPageKind]
    STEP_PAGE_KIND_CRITERIA: _ClassVar[StepPageKind]

class StepStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STEP_STATUS_UNSPECIFIED: _ClassVar[StepStatus]
    STEP_STATUS_PENDING: _ClassVar[StepStatus]
    STEP_STATUS_RUNNING: _ClassVar[StepStatus]
    STEP_STATUS_PASSED: _ClassVar[StepStatus]
    STEP_STATUS_FAILED: _ClassVar[StepStatus]
    STEP_STATUS_SKIPPED: _ClassVar[StepStatus]
    STEP_STATUS_CANCELLED: _ClassVar[StepStatus]

class CriterionStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CRITERION_STATUS_UNSPECIFIED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_PENDING: _ClassVar[CriterionStatus]
    CRITERION_STATUS_PASSED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_FAILED: _ClassVar[CriterionStatus]
    CRITERION_STATUS_INDETERMINATE: _ClassVar[CriterionStatus]
    CRITERION_STATUS_SKIPPED: _ClassVar[CriterionStatus]
CYCLE_ORDER_KEY_UNSPECIFIED: CycleOrderKey
CYCLE_ORDER_KEY_UPDATED_AT: CycleOrderKey
CYCLE_ORDER_KEY_CREATED_AT: CycleOrderKey
CYCLE_ORDER_KEY_LABEL: CycleOrderKey
EXECUTION_MODE_AUTOMATIC: ExecutionMode
EXECUTION_MODE_INTERACTIVE: ExecutionMode
EXECUTION_COMMAND_ACTION_UNSPECIFIED: ExecutionCommandAction
EXECUTION_COMMAND_ACTION_NEXT: ExecutionCommandAction
EXECUTION_COMMAND_ACTION_RUN_STEP: ExecutionCommandAction
EXECUTION_COMMAND_ACTION_FINISH: ExecutionCommandAction
EXECUTION_COMMAND_PHASE_UNSPECIFIED: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_ACCEPTED: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_DISPATCHING: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_RUNNING: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_COMPLETED: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_REFUSED: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_FAILED: ExecutionCommandPhase
EXECUTION_COMMAND_PHASE_INTERRUPTED: ExecutionCommandPhase
RUN_REQUEST_PHASE_UNSPECIFIED: RunRequestPhase
RUN_REQUEST_PHASE_QUEUED: RunRequestPhase
RUN_REQUEST_PHASE_PREPARING: RunRequestPhase
RUN_REQUEST_PHASE_RUNNING: RunRequestPhase
RUN_REQUEST_PHASE_FINISHING: RunRequestPhase
RUN_REQUEST_PHASE_COMPLETED: RunRequestPhase
RUN_REQUEST_PHASE_FAILED: RunRequestPhase
RUN_REQUEST_PHASE_STOPPED: RunRequestPhase
RUN_REQUEST_PHASE_INTERRUPTED: RunRequestPhase
CYCLE_STATUS_UNSPECIFIED: CycleStatus
CYCLE_STATUS_PREVIEWED: CycleStatus
CYCLE_STATUS_READY: CycleStatus
CYCLE_STATUS_RUNNING: CycleStatus
CYCLE_STATUS_PAUSED: CycleStatus
CYCLE_STATUS_STOPPING: CycleStatus
CYCLE_STATUS_COMPLETED: CycleStatus
CYCLE_STATUS_FAILED: CycleStatus
CYCLE_STATUS_BLOCKED: CycleStatus
CYCLE_STATUS_STOPPED: CycleStatus
CYCLE_STATUS_CLEARED: CycleStatus
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
EXECUTION_STATUS_UNSPECIFIED: ExecutionStatus
EXECUTION_STATUS_INVALIDATED: ExecutionStatus
EXECUTION_STATUS_PENDING: ExecutionStatus
EXECUTION_STATUS_RUNNING: ExecutionStatus
EXECUTION_STATUS_PASSED: ExecutionStatus
EXECUTION_STATUS_FAILED: ExecutionStatus
EXECUTION_STATUS_BLOCKED: ExecutionStatus
EXECUTION_STATUS_CANCELLED: ExecutionStatus
EXECUTION_STATUS_INTERRUPTED: ExecutionStatus
EXECUTION_STATUS_SKIPPED: ExecutionStatus
RESULT_STATUS_UNSPECIFIED: ResultStatus
RESULT_STATUS_PENDING: ResultStatus
RESULT_STATUS_PASSED: ResultStatus
RESULT_STATUS_FAILED: ResultStatus
RESULT_STATUS_BLOCKED: ResultStatus
RESULT_STATUS_CANCELLED: ResultStatus
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
STEP_PAGE_KIND_UNSPECIFIED: StepPageKind
STEP_PAGE_KIND_STEPS: StepPageKind
STEP_PAGE_KIND_MEASUREMENTS: StepPageKind
STEP_PAGE_KIND_CRITERIA: StepPageKind
STEP_STATUS_UNSPECIFIED: StepStatus
STEP_STATUS_PENDING: StepStatus
STEP_STATUS_RUNNING: StepStatus
STEP_STATUS_PASSED: StepStatus
STEP_STATUS_FAILED: StepStatus
STEP_STATUS_SKIPPED: StepStatus
STEP_STATUS_CANCELLED: StepStatus
CRITERION_STATUS_UNSPECIFIED: CriterionStatus
CRITERION_STATUS_PENDING: CriterionStatus
CRITERION_STATUS_PASSED: CriterionStatus
CRITERION_STATUS_FAILED: CriterionStatus
CRITERION_STATUS_INDETERMINATE: CriterionStatus
CRITERION_STATUS_SKIPPED: CriterionStatus

class MutationContext(_message.Message):
    __slots__ = ("correlation_id",)
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    def __init__(self, correlation_id: _Optional[str] = ...) -> None: ...

class CopyCycleRequest(_message.Message):
    __slots__ = ("source_cycle_id", "context")
    SOURCE_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    source_cycle_id: str
    context: MutationContext
    def __init__(self, source_cycle_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class CreateCycleRequest(_message.Message):
    __slots__ = ("plan", "metadata", "continue_on_failure", "context")
    PLAN_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    plan: _planning_pb2.SelectionPlan
    metadata: _common_pb2.Metadata
    continue_on_failure: bool
    context: MutationContext
    def __init__(self, plan: _Optional[_Union[_planning_pb2.SelectionPlan, _Mapping]] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., continue_on_failure: _Optional[bool] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class CreateCycleReply(_message.Message):
    __slots__ = ("cycle_id", "observed_cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    observed_cursor: _common_pb2.Cursor
    def __init__(self, cycle_id: _Optional[str] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class AppendPlannedCasesRequest(_message.Message):
    __slots__ = ("cycle_id", "plan", "context")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    PLAN_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    plan: _planning_pb2.SelectionPlan
    context: MutationContext
    def __init__(self, cycle_id: _Optional[str] = ..., plan: _Optional[_Union[_planning_pb2.SelectionPlan, _Mapping]] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class GetCycleRequest(_message.Message):
    __slots__ = ("cycle_id",)
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    def __init__(self, cycle_id: _Optional[str] = ...) -> None: ...

class CycleFilter(_message.Message):
    __slots__ = ("text", "statuses", "suite_ids", "created_after", "created_before")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    SUITE_IDS_FIELD_NUMBER: _ClassVar[int]
    CREATED_AFTER_FIELD_NUMBER: _ClassVar[int]
    CREATED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedScalarFieldContainer[CycleStatus]
    suite_ids: _containers.RepeatedScalarFieldContainer[str]
    created_after: _timestamp_pb2.Timestamp
    created_before: _timestamp_pb2.Timestamp
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[CycleStatus, str]]] = ..., suite_ids: _Optional[_Iterable[str]] = ..., created_after: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., created_before: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class CycleOrder(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: CycleOrderKey
    descending: bool
    def __init__(self, key: _Optional[_Union[CycleOrderKey, str]] = ..., descending: _Optional[bool] = ...) -> None: ...

class ListCyclesRequest(_message.Message):
    __slots__ = ("limit", "after_cycle_id", "cursor", "filter", "order")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_cycle_id: str
    cursor: _common_pb2.Cursor
    filter: CycleFilter
    order: CycleOrder
    def __init__(self, limit: _Optional[int] = ..., after_cycle_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., filter: _Optional[_Union[CycleFilter, _Mapping]] = ..., order: _Optional[_Union[CycleOrder, _Mapping]] = ...) -> None: ...

class ListCyclesReply(_message.Message):
    __slots__ = ("cycles", "next_after_cycle_id", "cursor")
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycles: _containers.RepeatedCompositeFieldContainer[CycleSummary]
    next_after_cycle_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, cycles: _Optional[_Iterable[_Union[CycleSummary, _Mapping]]] = ..., next_after_cycle_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class CycleControlRequest(_message.Message):
    __slots__ = ("cycle_id", "context")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    context: MutationContext
    def __init__(self, cycle_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class ExecutionControlRequest(_message.Message):
    __slots__ = ("execution_id", "context")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    context: MutationContext
    def __init__(self, execution_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class CaseControlRequest(_message.Message):
    __slots__ = ("cycle_id", "case_id", "context")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    context: MutationContext
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class UpdateCyclePolicyRequest(_message.Message):
    __slots__ = ("cycle_id", "continue_on_failure", "context")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    continue_on_failure: bool
    context: MutationContext
    def __init__(self, cycle_id: _Optional[str] = ..., continue_on_failure: _Optional[bool] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ...) -> None: ...

class RunCycleRequest(_message.Message):
    __slots__ = ("cycle_id", "context", "idempotency_key")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    IDEMPOTENCY_KEY_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    context: MutationContext
    idempotency_key: str
    def __init__(self, cycle_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ..., idempotency_key: _Optional[str] = ...) -> None: ...

class RunCaseRequest(_message.Message):
    __slots__ = ("cycle_id", "case_id", "context", "idempotency_key")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    IDEMPOTENCY_KEY_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    context: MutationContext
    idempotency_key: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ..., idempotency_key: _Optional[str] = ...) -> None: ...

class RunExecutionRequest(_message.Message):
    __slots__ = ("execution_id", "context", "idempotency_key", "mode")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    IDEMPOTENCY_KEY_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    context: MutationContext
    idempotency_key: str
    mode: ExecutionMode
    def __init__(self, execution_id: _Optional[str] = ..., context: _Optional[_Union[MutationContext, _Mapping]] = ..., idempotency_key: _Optional[str] = ..., mode: _Optional[_Union[ExecutionMode, str]] = ...) -> None: ...

class GetRunRequestRequest(_message.Message):
    __slots__ = ("request_id",)
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    def __init__(self, request_id: _Optional[str] = ...) -> None: ...

class SubmitExecutionCommandRequest(_message.Message):
    __slots__ = ("request_id", "command_id", "execution_id", "action", "step_id")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    command_id: str
    execution_id: str
    action: ExecutionCommandAction
    step_id: str
    def __init__(self, request_id: _Optional[str] = ..., command_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., action: _Optional[_Union[ExecutionCommandAction, str]] = ..., step_id: _Optional[str] = ...) -> None: ...

class GetExecutionCommandRequest(_message.Message):
    __slots__ = ("request_id", "command_id")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    command_id: str
    def __init__(self, request_id: _Optional[str] = ..., command_id: _Optional[str] = ...) -> None: ...

class ExecutionCommandReceipt(_message.Message):
    __slots__ = ("request_id", "command_id", "execution_id", "action", "step_id", "phase", "attempt_id", "accepted_at", "updated_at", "detail")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    command_id: str
    execution_id: str
    action: ExecutionCommandAction
    step_id: str
    phase: ExecutionCommandPhase
    attempt_id: str
    accepted_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    detail: str
    def __init__(self, request_id: _Optional[str] = ..., command_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., action: _Optional[_Union[ExecutionCommandAction, str]] = ..., step_id: _Optional[str] = ..., phase: _Optional[_Union[ExecutionCommandPhase, str]] = ..., attempt_id: _Optional[str] = ..., accepted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., detail: _Optional[str] = ...) -> None: ...

class RunRequestReceipt(_message.Message):
    __slots__ = ("request_id", "node_id", "cycle_id", "phase", "paused", "stop_requested", "planned_count", "prepared_count", "finished_count", "unstarted_count", "failed_count", "claimed_count", "last_claim", "accepted_at", "provider_started_at", "finished_at", "submitted_by", "interruption", "mode")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    STOP_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    PLANNED_COUNT_FIELD_NUMBER: _ClassVar[int]
    PREPARED_COUNT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_COUNT_FIELD_NUMBER: _ClassVar[int]
    UNSTARTED_COUNT_FIELD_NUMBER: _ClassVar[int]
    FAILED_COUNT_FIELD_NUMBER: _ClassVar[int]
    CLAIMED_COUNT_FIELD_NUMBER: _ClassVar[int]
    LAST_CLAIM_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    PROVIDER_STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    SUBMITTED_BY_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTION_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    node_id: str
    cycle_id: str
    phase: RunRequestPhase
    paused: bool
    stop_requested: bool
    planned_count: int
    prepared_count: int
    finished_count: int
    unstarted_count: int
    failed_count: int
    claimed_count: int
    last_claim: RunAttemptClaim
    accepted_at: _timestamp_pb2.Timestamp
    provider_started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    submitted_by: str
    interruption: str
    mode: ExecutionMode
    def __init__(self, request_id: _Optional[str] = ..., node_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., phase: _Optional[_Union[RunRequestPhase, str]] = ..., paused: _Optional[bool] = ..., stop_requested: _Optional[bool] = ..., planned_count: _Optional[int] = ..., prepared_count: _Optional[int] = ..., finished_count: _Optional[int] = ..., unstarted_count: _Optional[int] = ..., failed_count: _Optional[int] = ..., claimed_count: _Optional[int] = ..., last_claim: _Optional[_Union[RunAttemptClaim, _Mapping]] = ..., accepted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., provider_started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., submitted_by: _Optional[str] = ..., interruption: _Optional[str] = ..., mode: _Optional[_Union[ExecutionMode, str]] = ...) -> None: ...

class RunAttemptClaim(_message.Message):
    __slots__ = ("ordinal", "case_id", "execution_id")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    case_id: str
    execution_id: str
    def __init__(self, ordinal: _Optional[int] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class ControlReply(_message.Message):
    __slots__ = ("observed_cursor",)
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    observed_cursor: _common_pb2.Cursor
    def __init__(self, observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class RunReply(_message.Message):
    __slots__ = ("accepted", "rejected")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    accepted: Admission
    rejected: Rejection
    def __init__(self, accepted: _Optional[_Union[Admission, _Mapping]] = ..., rejected: _Optional[_Union[Rejection, _Mapping]] = ...) -> None: ...

class Admission(_message.Message):
    __slots__ = ("command_id", "job_id", "cursor", "accepted_at")
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    command_id: str
    job_id: str
    cursor: _common_pb2.Cursor
    accepted_at: _timestamp_pb2.Timestamp
    def __init__(self, command_id: _Optional[str] = ..., job_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., accepted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class Rejection(_message.Message):
    __slots__ = ("code", "message", "issues")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    issues: _containers.RepeatedCompositeFieldContainer[ValidationIssue]
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ..., issues: _Optional[_Iterable[_Union[ValidationIssue, _Mapping]]] = ...) -> None: ...

class ValidationIssue(_message.Message):
    __slots__ = ("path", "message", "expected")
    PATH_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_FIELD_NUMBER: _ClassVar[int]
    path: str
    message: str
    expected: str
    def __init__(self, path: _Optional[str] = ..., message: _Optional[str] = ..., expected: _Optional[str] = ...) -> None: ...

class CycleSummary(_message.Message):
    __slots__ = ("cycle_id", "metadata", "status", "case_count", "execution_count", "continue_on_failure", "observed_cursor", "created_at", "started_at", "updated_at", "result", "suite")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    CASE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: _common_pb2.Metadata
    status: CycleStatus
    case_count: int
    execution_count: int
    continue_on_failure: bool
    observed_cursor: _common_pb2.Cursor
    created_at: _timestamp_pb2.Timestamp
    started_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    result: ResultSummary
    suite: _common_pb2.Metadata
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[CycleStatus, str]] = ..., case_count: _Optional[int] = ..., execution_count: _Optional[int] = ..., continue_on_failure: _Optional[bool] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., result: _Optional[_Union[ResultSummary, _Mapping]] = ..., suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ...) -> None: ...

class ListCasesRequest(_message.Message):
    __slots__ = ("cycle_id", "limit", "after_case_id", "cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_CASE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    limit: int
    after_case_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, cycle_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_case_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListCasesReply(_message.Message):
    __slots__ = ("cases", "next_after_case_id", "cursor")
    CASES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_CASE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cases: _containers.RepeatedCompositeFieldContainer[CaseSummary]
    next_after_case_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, cases: _Optional[_Iterable[_Union[CaseSummary, _Mapping]]] = ..., next_after_case_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListExecutionsRequest(_message.Message):
    __slots__ = ("cycle_id", "limit", "after_execution_id", "cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    limit: int
    after_execution_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, cycle_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_execution_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListExecutionsReply(_message.Message):
    __slots__ = ("executions", "next_after_execution_id", "cursor")
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    executions: _containers.RepeatedCompositeFieldContainer[ExecutionSummary]
    next_after_execution_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, executions: _Optional[_Iterable[_Union[ExecutionSummary, _Mapping]]] = ..., next_after_execution_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class CaseSummary(_message.Message):
    __slots__ = ("case_id", "template_id", "metadata", "ordinal", "status", "latest_execution_id", "latest_execution_status", "attempt_count", "rerun_scheduled", "actions", "suite_id")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_STATUS_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_COUNT_FIELD_NUMBER: _ClassVar[int]
    RERUN_SCHEDULED_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    template_id: str
    metadata: _common_pb2.Metadata
    ordinal: int
    status: CaseStatus
    latest_execution_id: str
    latest_execution_status: ExecutionStatus
    attempt_count: int
    rerun_scheduled: bool
    actions: CaseActions
    suite_id: str
    def __init__(self, case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., ordinal: _Optional[int] = ..., status: _Optional[_Union[CaseStatus, str]] = ..., latest_execution_id: _Optional[str] = ..., latest_execution_status: _Optional[_Union[ExecutionStatus, str]] = ..., attempt_count: _Optional[int] = ..., rerun_scheduled: _Optional[bool] = ..., actions: _Optional[_Union[CaseActions, _Mapping]] = ..., suite_id: _Optional[str] = ...) -> None: ...

class CaseActions(_message.Message):
    __slots__ = ("run", "schedule_rerun", "delete", "unavailable_reasons")
    RUN_FIELD_NUMBER: _ClassVar[int]
    SCHEDULE_RERUN_FIELD_NUMBER: _ClassVar[int]
    DELETE_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_REASONS_FIELD_NUMBER: _ClassVar[int]
    run: bool
    schedule_rerun: bool
    delete: bool
    unavailable_reasons: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, run: _Optional[bool] = ..., schedule_rerun: _Optional[bool] = ..., delete: _Optional[bool] = ..., unavailable_reasons: _Optional[_Iterable[str]] = ...) -> None: ...

class ExecutionSummary(_message.Message):
    __slots__ = ("execution_id", "case_id", "status", "ordinal")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    status: ExecutionStatus
    ordinal: int
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ..., status: _Optional[_Union[ExecutionStatus, str]] = ..., ordinal: _Optional[int] = ...) -> None: ...

class GetCaseRequest(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class CaseDetail(_message.Message):
    __slots__ = ("summary", "suite", "parameter_count", "generation_index", "observed_cursor")
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    GENERATION_INDEX_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    summary: CaseSummary
    suite: _common_pb2.Metadata
    parameter_count: int
    generation_index: int
    observed_cursor: _common_pb2.Cursor
    def __init__(self, summary: _Optional[_Union[CaseSummary, _Mapping]] = ..., suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., parameter_count: _Optional[int] = ..., generation_index: _Optional[int] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class GetExecutionRequest(_message.Message):
    __slots__ = ("execution_id",)
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    def __init__(self, execution_id: _Optional[str] = ...) -> None: ...

class ExecutionDetail(_message.Message):
    __slots__ = ("summary", "cycle_id", "scope", "admitted_suite", "admitted_template_id", "admitted_template", "admitted_plugin", "invalidation", "started_at", "finished_at", "result", "input_parameter_count", "output_parameter_count", "step_count", "observed_cursor", "controls")
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_SUITE_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ADMITTED_PLUGIN_FIELD_NUMBER: _ClassVar[int]
    INVALIDATION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    INPUT_PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    STEP_COUNT_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    CONTROLS_FIELD_NUMBER: _ClassVar[int]
    summary: ExecutionSummary
    cycle_id: str
    scope: _common_pb2.EvidenceScope
    admitted_suite: _common_pb2.Metadata
    admitted_template_id: str
    admitted_template: _common_pb2.Metadata
    admitted_plugin: ExecutionPluginIdentity
    invalidation: ExecutionInvalidation
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    result: ResultSummary
    input_parameter_count: int
    output_parameter_count: int
    step_count: int
    observed_cursor: _common_pb2.Cursor
    controls: ExecutionControls
    def __init__(self, summary: _Optional[_Union[ExecutionSummary, _Mapping]] = ..., cycle_id: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., admitted_suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., admitted_template_id: _Optional[str] = ..., admitted_template: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., admitted_plugin: _Optional[_Union[ExecutionPluginIdentity, _Mapping]] = ..., invalidation: _Optional[_Union[ExecutionInvalidation, _Mapping]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., result: _Optional[_Union[ResultSummary, _Mapping]] = ..., input_parameter_count: _Optional[int] = ..., output_parameter_count: _Optional[int] = ..., step_count: _Optional[int] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., controls: _Optional[_Union[ExecutionControls, _Mapping]] = ...) -> None: ...

class ExecutionControls(_message.Message):
    __slots__ = ("request_id", "paused", "stop_requested", "can_pause", "can_resume", "can_stop", "mode")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    STOP_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    CAN_PAUSE_FIELD_NUMBER: _ClassVar[int]
    CAN_RESUME_FIELD_NUMBER: _ClassVar[int]
    CAN_STOP_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    paused: bool
    stop_requested: bool
    can_pause: bool
    can_resume: bool
    can_stop: bool
    mode: ExecutionMode
    def __init__(self, request_id: _Optional[str] = ..., paused: _Optional[bool] = ..., stop_requested: _Optional[bool] = ..., can_pause: _Optional[bool] = ..., can_resume: _Optional[bool] = ..., can_stop: _Optional[bool] = ..., mode: _Optional[_Union[ExecutionMode, str]] = ...) -> None: ...

class ExecutionPluginIdentity(_message.Message):
    __slots__ = ("source", "version", "content_digest", "generation")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    CONTENT_DIGEST_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    source: str
    version: str
    content_digest: str
    generation: str
    def __init__(self, source: _Optional[str] = ..., version: _Optional[str] = ..., content_digest: _Optional[str] = ..., generation: _Optional[str] = ...) -> None: ...

class ExecutionInvalidation(_message.Message):
    __slots__ = ("previous", "current")
    PREVIOUS_FIELD_NUMBER: _ClassVar[int]
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    previous: ExecutionPluginIdentity
    current: ExecutionPluginIdentity
    def __init__(self, previous: _Optional[_Union[ExecutionPluginIdentity, _Mapping]] = ..., current: _Optional[_Union[ExecutionPluginIdentity, _Mapping]] = ...) -> None: ...

class ResultSummary(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: ResultStatus
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[ResultStatus, str]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class PlannedCaseParameters(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class ParameterCursor(_message.Message):
    __slots__ = ("observed", "execution_inputs_id", "execution_outputs_id", "planned_case")
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_INPUTS_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_OUTPUTS_ID_FIELD_NUMBER: _ClassVar[int]
    PLANNED_CASE_FIELD_NUMBER: _ClassVar[int]
    observed: _common_pb2.Cursor
    execution_inputs_id: str
    execution_outputs_id: str
    planned_case: PlannedCaseParameters
    def __init__(self, observed: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., execution_inputs_id: _Optional[str] = ..., execution_outputs_id: _Optional[str] = ..., planned_case: _Optional[_Union[PlannedCaseParameters, _Mapping]] = ...) -> None: ...

class ListCaseParametersRequest(_message.Message):
    __slots__ = ("cycle_id", "case_id", "limit", "after_name", "cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    limit: int
    after_name: str
    cursor: ParameterCursor
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_name: _Optional[str] = ..., cursor: _Optional[_Union[ParameterCursor, _Mapping]] = ...) -> None: ...

class CaseParameter(_message.Message):
    __slots__ = ("name", "value")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: _value_pb2.Value
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ...) -> None: ...

class ListCaseParametersReply(_message.Message):
    __slots__ = ("parameters", "next_after_name", "cursor")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[CaseParameter]
    next_after_name: str
    cursor: ParameterCursor
    def __init__(self, parameters: _Optional[_Iterable[_Union[CaseParameter, _Mapping]]] = ..., next_after_name: _Optional[str] = ..., cursor: _Optional[_Union[ParameterCursor, _Mapping]] = ...) -> None: ...

class ListExecutionParametersRequest(_message.Message):
    __slots__ = ("execution_id", "set", "limit", "after_name", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    SET_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    set: ExecutionParameterSet
    limit: int
    after_name: str
    cursor: ParameterCursor
    def __init__(self, execution_id: _Optional[str] = ..., set: _Optional[_Union[ExecutionParameterSet, str]] = ..., limit: _Optional[int] = ..., after_name: _Optional[str] = ..., cursor: _Optional[_Union[ParameterCursor, _Mapping]] = ...) -> None: ...

class ListMaterializedParametersReply(_message.Message):
    __slots__ = ("parameters", "next_after_name", "cursor")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[MaterializedParameter]
    next_after_name: str
    cursor: ParameterCursor
    def __init__(self, parameters: _Optional[_Iterable[_Union[MaterializedParameter, _Mapping]]] = ..., next_after_name: _Optional[str] = ..., cursor: _Optional[_Union[ParameterCursor, _Mapping]] = ...) -> None: ...

class MaterializedParameter(_message.Message):
    __slots__ = ("name", "metadata", "group_id", "source", "value", "unit")
    NAME_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    name: str
    metadata: _common_pb2.Metadata
    group_id: str
    source: ParameterSource
    value: _value_pb2.Value
    unit: _schema_pb2.MeasurementUnit
    def __init__(self, name: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., group_id: _Optional[str] = ..., source: _Optional[_Union[ParameterSource, str]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., unit: _Optional[_Union[_schema_pb2.MeasurementUnit, _Mapping]] = ...) -> None: ...

class StepPageCursor(_message.Message):
    __slots__ = ("revision", "execution_id", "step_id", "kind", "next_ordinal")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    NEXT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    revision: _common_pb2.Cursor
    execution_id: str
    step_id: str
    kind: StepPageKind
    next_ordinal: int
    def __init__(self, revision: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., kind: _Optional[_Union[StepPageKind, str]] = ..., next_ordinal: _Optional[int] = ...) -> None: ...

class ListExecutionStepsRequest(_message.Message):
    __slots__ = ("execution_id", "limit", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    limit: int
    cursor: StepPageCursor
    def __init__(self, execution_id: _Optional[str] = ..., limit: _Optional[int] = ..., cursor: _Optional[_Union[StepPageCursor, _Mapping]] = ...) -> None: ...

class ListStepEvidenceRequest(_message.Message):
    __slots__ = ("execution_id", "step_id", "limit", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    step_id: str
    limit: int
    cursor: StepPageCursor
    def __init__(self, execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., limit: _Optional[int] = ..., cursor: _Optional[_Union[StepPageCursor, _Mapping]] = ...) -> None: ...

class ListExecutionStepsReply(_message.Message):
    __slots__ = ("steps", "next_cursor", "observed_cursor")
    STEPS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    steps: _containers.RepeatedCompositeFieldContainer[ExecutionStep]
    next_cursor: StepPageCursor
    observed_cursor: _common_pb2.Cursor
    def __init__(self, steps: _Optional[_Iterable[_Union[ExecutionStep, _Mapping]]] = ..., next_cursor: _Optional[_Union[StepPageCursor, _Mapping]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListStepCriteriaReply(_message.Message):
    __slots__ = ("criteria", "next_cursor", "observed_cursor")
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    criteria: _containers.RepeatedCompositeFieldContainer[StepCriterion]
    next_cursor: StepPageCursor
    observed_cursor: _common_pb2.Cursor
    def __init__(self, criteria: _Optional[_Iterable[_Union[StepCriterion, _Mapping]]] = ..., next_cursor: _Optional[_Union[StepPageCursor, _Mapping]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ExecutionStep(_message.Message):
    __slots__ = ("step_id", "metadata", "status", "started_at", "finished_at", "measurement_count", "criterion_count", "ordinal", "detail")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENT_COUNT_FIELD_NUMBER: _ClassVar[int]
    CRITERION_COUNT_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    metadata: _common_pb2.Metadata
    status: StepStatus
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    measurement_count: int
    criterion_count: int
    ordinal: int
    detail: _common_pb2.Diagnostic
    def __init__(self, step_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[StepStatus, str]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., measurement_count: _Optional[int] = ..., criterion_count: _Optional[int] = ..., ordinal: _Optional[int] = ..., detail: _Optional[_Union[_common_pb2.Diagnostic, _Mapping]] = ...) -> None: ...

class StepMeasurement(_message.Message):
    __slots__ = ("metadata", "value", "unit", "ordinal")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    value: _value_pb2.Value
    unit: _schema_pb2.MeasurementUnit
    ordinal: int
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., unit: _Optional[_Union[_schema_pb2.MeasurementUnit, _Mapping]] = ..., ordinal: _Optional[int] = ...) -> None: ...

class StepCriterion(_message.Message):
    __slots__ = ("criterion_id", "metadata", "status", "observed", "message", "ordinal")
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    criterion_id: str
    metadata: _common_pb2.Metadata
    status: CriterionStatus
    observed: _value_pb2.Value
    message: str
    ordinal: int
    def __init__(self, criterion_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[CriterionStatus, str]] = ..., observed: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., message: _Optional[str] = ..., ordinal: _Optional[int] = ...) -> None: ...
