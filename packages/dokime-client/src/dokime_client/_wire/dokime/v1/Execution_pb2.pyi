from dokime_client._wire.dokime.v1.ExecutionMessages.AppendExecutions import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.CancelExecution import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.ExecutionMessages.CopyCycle import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.CreateCycle import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.DeleteCase import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.GetCase import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.GetCycle import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.GetExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.GetExecutionCommand import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCaseParameters import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCases import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCycles import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionParameters import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionSteps import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutions import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.PauseCycle import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.PauseExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.RenameCycle import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.RenameExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ResumeCycle import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.ResumeExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.SealCycle import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.SetCycleArchived import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.SetExecutionArchived import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.StartExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.StopExecution import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.SubmitExecutionCommand import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ExecutionMessages.UpdateCyclePolicy import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ExecutionAppendExecutionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionAppendExecutionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: ExecutionAppendExecutionsCancellation
    finish: ExecutionAppendExecutionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionAppendExecutionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionAppendExecutionsCancellation, _Mapping]] = ...) -> None: ...

class ExecutionAppendExecutionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class ExecutionCancelExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionCancelExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: ExecutionCancelExecutionCancellation
    finish: ExecutionCancelExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionCancelExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionCancelExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionCancelExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionCopyCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionCopyCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: ExecutionCopyCycleCancellation
    finish: ExecutionCopyCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionCopyCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionCopyCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionCopyCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionCreateCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionCreateCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: ExecutionCreateCycleCancellation
    finish: ExecutionCreateCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionCreateCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionCreateCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionCreateCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionDeleteCaseCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionDeleteCaseRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: ExecutionDeleteCaseCancellation
    finish: ExecutionDeleteCaseCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionDeleteCaseCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionDeleteCaseCancellation, _Mapping]] = ...) -> None: ...

class ExecutionDeleteCaseRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionGetCaseCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionGetCaseRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: ExecutionGetCaseCancellation
    finish: ExecutionGetCaseCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionGetCaseCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionGetCaseCancellation, _Mapping]] = ...) -> None: ...

class ExecutionGetCaseRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionGetCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionGetCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: ExecutionGetCycleCancellation
    finish: ExecutionGetCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionGetCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionGetCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionGetCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionGetExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionGetExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1.Row1
    cancel: ExecutionGetExecutionCancellation
    finish: ExecutionGetExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionGetExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionGetExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionGetExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionGetExecutionCommandCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionGetExecutionCommandRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionGetExecutionCommandCancellation
    finish: ExecutionGetExecutionCommandCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionGetExecutionCommandCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionGetExecutionCommandCancellation, _Mapping]] = ...) -> None: ...

class ExecutionGetExecutionCommandRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListCaseParametersCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListCaseParametersRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListCaseParametersCancellation
    finish: ExecutionListCaseParametersCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListCaseParametersCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListCaseParametersCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListCaseParametersRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListCasesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListCasesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListCasesCancellation
    finish: ExecutionListCasesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListCasesCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListCasesCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListCasesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListCyclesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListCyclesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListCyclesCancellation
    finish: ExecutionListCyclesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListCyclesCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListCyclesCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListCyclesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionParametersCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListExecutionParametersRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListExecutionParametersCancellation
    finish: ExecutionListExecutionParametersCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListExecutionParametersCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListExecutionParametersCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionParametersRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionStepsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListExecutionStepsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListExecutionStepsCancellation
    finish: ExecutionListExecutionStepsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListExecutionStepsCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListExecutionStepsCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionStepsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionListExecutionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionListExecutionsCancellation
    finish: ExecutionListExecutionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionListExecutionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionListExecutionsCancellation, _Mapping]] = ...) -> None: ...

class ExecutionListExecutionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionPauseCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionPauseCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionPauseCycleCancellation
    finish: ExecutionPauseCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionPauseCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionPauseCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionPauseCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionPauseExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionPauseExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionPauseExecutionCancellation
    finish: ExecutionPauseExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionPauseExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionPauseExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionPauseExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionRenameCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionRenameCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionRenameCycleCancellation
    finish: ExecutionRenameCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionRenameCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionRenameCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionRenameCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionRenameExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionRenameExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionRenameExecutionCancellation
    finish: ExecutionRenameExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionRenameExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionRenameExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionRenameExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionResumeCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionResumeCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionResumeCycleCancellation
    finish: ExecutionResumeCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionResumeCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionResumeCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionResumeCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionResumeExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionResumeExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionResumeExecutionCancellation
    finish: ExecutionResumeExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionResumeExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionResumeExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionResumeExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionSealCycleCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionSealCycleRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionSealCycleCancellation
    finish: ExecutionSealCycleCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionSealCycleCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionSealCycleCancellation, _Mapping]] = ...) -> None: ...

class ExecutionSealCycleRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionSetCycleArchivedCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionSetCycleArchivedRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionSetCycleArchivedCancellation
    finish: ExecutionSetCycleArchivedCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionSetCycleArchivedCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionSetCycleArchivedCancellation, _Mapping]] = ...) -> None: ...

class ExecutionSetCycleArchivedRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionSetExecutionArchivedCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionSetExecutionArchivedRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionSetExecutionArchivedCancellation
    finish: ExecutionSetExecutionArchivedCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionSetExecutionArchivedCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionSetExecutionArchivedCancellation, _Mapping]] = ...) -> None: ...

class ExecutionSetExecutionArchivedRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionStartExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionStartExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionStartExecutionCancellation
    finish: ExecutionStartExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionStartExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionStartExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionStartExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionStopExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionStopExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionStopExecutionCancellation
    finish: ExecutionStopExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionStopExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionStopExecutionCancellation, _Mapping]] = ...) -> None: ...

class ExecutionStopExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionSubmitExecutionCommandCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionSubmitExecutionCommandRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionSubmitExecutionCommandCancellation
    finish: ExecutionSubmitExecutionCommandCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionSubmitExecutionCommandCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionSubmitExecutionCommandCancellation, _Mapping]] = ...) -> None: ...

class ExecutionSubmitExecutionCommandRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ExecutionUpdateCyclePolicyCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionUpdateCyclePolicyRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: ExecutionUpdateCyclePolicyCancellation
    finish: ExecutionUpdateCyclePolicyCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ExecutionUpdateCyclePolicyCancellation, _Mapping]] = ..., finish: _Optional[_Union[ExecutionUpdateCyclePolicyCancellation, _Mapping]] = ...) -> None: ...

class ExecutionUpdateCyclePolicyRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
