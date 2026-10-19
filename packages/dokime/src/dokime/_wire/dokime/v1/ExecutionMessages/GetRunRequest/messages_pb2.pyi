from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("request_id",)
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    def __init__(self, request_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
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
    phase: Row3
    paused: bool
    stop_requested: bool
    planned_count: int
    prepared_count: int
    finished_count: int
    unstarted_count: int
    failed_count: int
    claimed_count: int
    last_claim: Row4
    accepted_at: Row5
    provider_started_at: Row5
    finished_at: Row5
    submitted_by: str
    interruption: str
    mode: Row6
    def __init__(self, request_id: _Optional[str] = ..., node_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., phase: _Optional[_Union[Row3, _Mapping]] = ..., paused: _Optional[bool] = ..., stop_requested: _Optional[bool] = ..., planned_count: _Optional[int] = ..., prepared_count: _Optional[int] = ..., finished_count: _Optional[int] = ..., unstarted_count: _Optional[int] = ..., failed_count: _Optional[int] = ..., claimed_count: _Optional[int] = ..., last_claim: _Optional[_Union[Row4, _Mapping]] = ..., accepted_at: _Optional[_Union[Row5, _Mapping]] = ..., provider_started_at: _Optional[_Union[Row5, _Mapping]] = ..., finished_at: _Optional[_Union[Row5, _Mapping]] = ..., submitted_by: _Optional[str] = ..., interruption: _Optional[str] = ..., mode: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Queued", "Preparing", "Running", "Finishing", "Completed", "Failed", "Stopped", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    QUEUED_FIELD_NUMBER: _ClassVar[int]
    PREPARING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    FINISHING_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    STOPPED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row7
    Queued: Row8
    Preparing: Row9
    Running: Row10
    Finishing: Row11
    Completed: Row12
    Failed: Row13
    Stopped: Row14
    Interrupted: Row15
    def __init__(self, Unspecified: _Optional[_Union[Row7, _Mapping]] = ..., Queued: _Optional[_Union[Row8, _Mapping]] = ..., Preparing: _Optional[_Union[Row9, _Mapping]] = ..., Running: _Optional[_Union[Row10, _Mapping]] = ..., Finishing: _Optional[_Union[Row11, _Mapping]] = ..., Completed: _Optional[_Union[Row12, _Mapping]] = ..., Failed: _Optional[_Union[Row13, _Mapping]] = ..., Stopped: _Optional[_Union[Row14, _Mapping]] = ..., Interrupted: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("ordinal", "case_id", "execution_id")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    case_id: str
    execution_id: str
    def __init__(self, ordinal: _Optional[int] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Automatic", "Interactive")
    AUTOMATIC_FIELD_NUMBER: _ClassVar[int]
    INTERACTIVE_FIELD_NUMBER: _ClassVar[int]
    Automatic: Row16
    Interactive: Row17
    def __init__(self, Automatic: _Optional[_Union[Row16, _Mapping]] = ..., Interactive: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
