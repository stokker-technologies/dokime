from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("epoch", "mutation_id", "owner", "message")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    MUTATION_ID_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    mutation_id: str
    owner: Row3
    message: str
    def __init__(self, epoch: _Optional[str] = ..., mutation_id: _Optional[str] = ..., owner: _Optional[_Union[Row3, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("epoch", "mutation_id", "owner", "note", "checkpoint_targets", "publication", "unconfirmed_targets")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    MUTATION_ID_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    NOTE_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_TARGETS_FIELD_NUMBER: _ClassVar[int]
    PUBLICATION_FIELD_NUMBER: _ClassVar[int]
    UNCONFIRMED_TARGETS_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    mutation_id: str
    owner: Row3
    note: Row4
    checkpoint_targets: _containers.RepeatedScalarFieldContainer[str]
    publication: Row5
    unconfirmed_targets: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, epoch: _Optional[str] = ..., mutation_id: _Optional[str] = ..., owner: _Optional[_Union[Row3, _Mapping]] = ..., note: _Optional[_Union[Row4, _Mapping]] = ..., checkpoint_targets: _Optional[_Iterable[str]] = ..., publication: _Optional[_Union[Row5, _Mapping]] = ..., unconfirmed_targets: _Optional[_Iterable[str]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("owner",)
    OWNER_FIELD_NUMBER: _ClassVar[int]
    owner: Row6
    def __init__(self, owner: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("note_id", "scope", "author", "message", "created_at")
    NOTE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AUTHOR_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    note_id: str
    scope: Row7
    author: str
    message: str
    created_at: Row8
    def __init__(self, note_id: _Optional[str] = ..., scope: _Optional[_Union[Row7, _Mapping]] = ..., author: _Optional[str] = ..., message: _Optional[str] = ..., created_at: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "NoExistingDatasets", "Confirmed", "Unconfirmed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NOEXISTINGDATASETS_FIELD_NUMBER: _ClassVar[int]
    CONFIRMED_FIELD_NUMBER: _ClassVar[int]
    UNCONFIRMED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row9
    NoExistingDatasets: Row10
    Confirmed: Row11
    Unconfirmed: Row12
    def __init__(self, Unspecified: _Optional[_Union[Row9, _Mapping]] = ..., NoExistingDatasets: _Optional[_Union[Row10, _Mapping]] = ..., Confirmed: _Optional[_Union[Row11, _Mapping]] = ..., Unconfirmed: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("CycleId", "PlannedCase", "Execution")
    CYCLEID_FIELD_NUMBER: _ClassVar[int]
    PLANNEDCASE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CycleId: Row13
    PlannedCase: Row14
    Execution: Row15
    def __init__(self, CycleId: _Optional[_Union[Row13, _Mapping]] = ..., PlannedCase: _Optional[_Union[Row14, _Mapping]] = ..., Execution: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

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
    origin: Row16
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row17
    def __init__(self, field_0: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row18
    def __init__(self, field_0: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row19
    def __init__(self, kind: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row20
    Initialization: Row21
    StepAttemptId: Row22
    Cleanup: Row23
    def __init__(self, Default: _Optional[_Union[Row20, _Mapping]] = ..., Initialization: _Optional[_Union[Row21, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row22, _Mapping]] = ..., Cleanup: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row23(_message.Message):
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
