from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("selection", "limit", "after_id", "cursor")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    selection: Row3
    limit: int
    after_id: str
    cursor: Row4
    def __init__(self, selection: _Optional[_Union[Row3, _Mapping]] = ..., limit: _Optional[int] = ..., after_id: _Optional[str] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("items", "cursor", "next_after_id")
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ID_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedCompositeFieldContainer[Row5]
    cursor: Row4
    next_after_id: str
    def __init__(self, items: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ..., next_after_id: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("owner", "include_context")
    OWNER_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_CONTEXT_FIELD_NUMBER: _ClassVar[int]
    owner: Row6
    include_context: bool
    def __init__(self, owner: _Optional[_Union[Row6, _Mapping]] = ..., include_context: _Optional[bool] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("health_item_id", "scope", "severity", "message", "detail", "observed_at")
    HEALTH_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    health_item_id: str
    scope: Row7
    severity: Row8
    message: str
    detail: str
    observed_at: Row9
    def __init__(self, health_item_id: _Optional[str] = ..., scope: _Optional[_Union[Row7, _Mapping]] = ..., severity: _Optional[_Union[Row8, _Mapping]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ..., observed_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("owner",)
    OWNER_FIELD_NUMBER: _ClassVar[int]
    owner: Row10
    def __init__(self, owner: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
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
    origin: Row11
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row12
    Trace: Row13
    Debug: Row14
    Info: Row15
    Warn: Row16
    Error: Row17
    def __init__(self, Unspecified: _Optional[_Union[Row12, _Mapping]] = ..., Trace: _Optional[_Union[Row13, _Mapping]] = ..., Debug: _Optional[_Union[Row14, _Mapping]] = ..., Info: _Optional[_Union[Row15, _Mapping]] = ..., Warn: _Optional[_Union[Row16, _Mapping]] = ..., Error: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("CycleId", "PlannedCase", "Execution")
    CYCLEID_FIELD_NUMBER: _ClassVar[int]
    PLANNEDCASE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CycleId: Row18
    PlannedCase: Row19
    Execution: Row20
    def __init__(self, CycleId: _Optional[_Union[Row18, _Mapping]] = ..., PlannedCase: _Optional[_Union[Row19, _Mapping]] = ..., Execution: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row21
    def __init__(self, kind: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row22
    def __init__(self, field_0: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row23
    def __init__(self, field_0: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row24
    Initialization: Row25
    StepAttemptId: Row26
    Cleanup: Row27
    def __init__(self, Default: _Optional[_Union[Row24, _Mapping]] = ..., Initialization: _Optional[_Union[Row25, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row26, _Mapping]] = ..., Cleanup: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
