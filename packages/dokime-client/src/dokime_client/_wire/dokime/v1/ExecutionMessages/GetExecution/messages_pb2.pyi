from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id",)
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    def __init__(self, execution_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("report", "summary", "cycle_id", "scope", "admitted_suite", "admitted_template_id", "admitted_template", "admitted_plugin", "invalidation", "started_at", "finished_at", "result", "input_parameter_count", "output_parameter_count", "step_count", "observed_cursor", "controls")
    REPORT_FIELD_NUMBER: _ClassVar[int]
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
    report: Row3
    summary: Row4
    cycle_id: str
    scope: Row5
    admitted_suite: Row6
    admitted_template_id: str
    admitted_template: Row6
    admitted_plugin: Row7
    invalidation: Row8
    started_at: Row9
    finished_at: Row9
    result: Row10
    input_parameter_count: int
    output_parameter_count: int
    step_count: int
    observed_cursor: Row11
    controls: Row12
    def __init__(self, report: _Optional[_Union[Row3, _Mapping]] = ..., summary: _Optional[_Union[Row4, _Mapping]] = ..., cycle_id: _Optional[str] = ..., scope: _Optional[_Union[Row5, _Mapping]] = ..., admitted_suite: _Optional[_Union[Row6, _Mapping]] = ..., admitted_template_id: _Optional[str] = ..., admitted_template: _Optional[_Union[Row6, _Mapping]] = ..., admitted_plugin: _Optional[_Union[Row7, _Mapping]] = ..., invalidation: _Optional[_Union[Row8, _Mapping]] = ..., started_at: _Optional[_Union[Row9, _Mapping]] = ..., finished_at: _Optional[_Union[Row9, _Mapping]] = ..., result: _Optional[_Union[Row10, _Mapping]] = ..., input_parameter_count: _Optional[int] = ..., output_parameter_count: _Optional[int] = ..., step_count: _Optional[int] = ..., observed_cursor: _Optional[_Union[Row11, _Mapping]] = ..., controls: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("state", "failure")
    STATE_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    state: Row13
    failure: str
    def __init__(self, state: _Optional[_Union[Row13, _Mapping]] = ..., failure: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("execution_id", "case_id", "status", "ordinal", "archived", "sealed")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    status: Row14
    ordinal: int
    archived: bool
    sealed: bool
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ..., status: _Optional[_Union[Row14, _Mapping]] = ..., ordinal: _Optional[int] = ..., archived: _Optional[bool] = ..., sealed: _Optional[bool] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row15
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row16]
    created_at: Row9
    updated_at: Row9
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row16, _Mapping]]] = ..., created_at: _Optional[_Union[Row9, _Mapping]] = ..., updated_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("source", "version", "implementation")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    IMPLEMENTATION_FIELD_NUMBER: _ClassVar[int]
    source: str
    version: str
    implementation: Row17
    def __init__(self, source: _Optional[str] = ..., version: _Optional[str] = ..., implementation: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("previous", "current")
    PREVIOUS_FIELD_NUMBER: _ClassVar[int]
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    previous: Row7
    current: Row7
    def __init__(self, previous: _Optional[_Union[Row7, _Mapping]] = ..., current: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row18
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row18, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row12(_message.Message):
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
    mode: Row19
    def __init__(self, request_id: _Optional[str] = ..., paused: _Optional[bool] = ..., stop_requested: _Optional[bool] = ..., can_pause: _Optional[bool] = ..., can_resume: _Optional[bool] = ..., can_stop: _Optional[bool] = ..., mode: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Pending", "Ready", "Failed")
    PENDING_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    Pending: Row20
    Ready: Row21
    Failed: Row22
    def __init__(self, Pending: _Optional[_Union[Row20, _Mapping]] = ..., Ready: _Optional[_Union[Row21, _Mapping]] = ..., Failed: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("Unspecified", "Invalidated", "Pending", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row23
    Invalidated: Row24
    Pending: Row25
    Running: Row26
    Passed: Row27
    Failed: Row28
    Blocked: Row29
    Cancelled: Row30
    Interrupted: Row31
    Skipped: Row32
    def __init__(self, Unspecified: _Optional[_Union[Row23, _Mapping]] = ..., Invalidated: _Optional[_Union[Row24, _Mapping]] = ..., Pending: _Optional[_Union[Row25, _Mapping]] = ..., Running: _Optional[_Union[Row26, _Mapping]] = ..., Passed: _Optional[_Union[Row27, _Mapping]] = ..., Failed: _Optional[_Union[Row28, _Mapping]] = ..., Blocked: _Optional[_Union[Row29, _Mapping]] = ..., Cancelled: _Optional[_Union[Row30, _Mapping]] = ..., Interrupted: _Optional[_Union[Row31, _Mapping]] = ..., Skipped: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row33
    def __init__(self, kind: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row34
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("ContentDigest", "Generation")
    CONTENTDIGEST_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ContentDigest: Row35
    Generation: Row36
    def __init__(self, ContentDigest: _Optional[_Union[Row35, _Mapping]] = ..., Generation: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row37
    Pending: Row38
    Passed: Row39
    Failed: Row40
    Blocked: Row41
    Cancelled: Row42
    def __init__(self, Unspecified: _Optional[_Union[Row37, _Mapping]] = ..., Pending: _Optional[_Union[Row38, _Mapping]] = ..., Passed: _Optional[_Union[Row39, _Mapping]] = ..., Failed: _Optional[_Union[Row40, _Mapping]] = ..., Blocked: _Optional[_Union[Row41, _Mapping]] = ..., Cancelled: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Automatic", "Interactive")
    AUTOMATIC_FIELD_NUMBER: _ClassVar[int]
    INTERACTIVE_FIELD_NUMBER: _ClassVar[int]
    Automatic: Row43
    Interactive: Row44
    def __init__(self, Automatic: _Optional[_Union[Row43, _Mapping]] = ..., Interactive: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row45
    Initialization: Row46
    StepAttemptId: Row47
    Cleanup: Row48
    def __init__(self, Default: _Optional[_Union[Row45, _Mapping]] = ..., Initialization: _Optional[_Union[Row46, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row47, _Mapping]] = ..., Cleanup: _Optional[_Union[Row48, _Mapping]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row49]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row49, _Mapping]]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

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
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row50
    def __init__(self, kind: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row50(_message.Message):
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
    Null: Row51
    Bool: Row52
    Integer: Row53
    Unsigned: Row54
    Number: Row55
    Text: Row56
    Datetime: Row57
    Array: Row58
    Object: Row59
    def __init__(self, Null: _Optional[_Union[Row51, _Mapping]] = ..., Bool: _Optional[_Union[Row52, _Mapping]] = ..., Integer: _Optional[_Union[Row53, _Mapping]] = ..., Unsigned: _Optional[_Union[Row54, _Mapping]] = ..., Number: _Optional[_Union[Row55, _Mapping]] = ..., Text: _Optional[_Union[Row56, _Mapping]] = ..., Datetime: _Optional[_Union[Row57, _Mapping]] = ..., Array: _Optional[_Union[Row58, _Mapping]] = ..., Object: _Optional[_Union[Row59, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row58(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row60
    def __init__(self, field_0: _Optional[_Union[Row60, _Mapping]] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row61
    def __init__(self, field_0: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row61(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row62]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row62, _Mapping]]] = ...) -> None: ...

class Row62(_message.Message):
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
