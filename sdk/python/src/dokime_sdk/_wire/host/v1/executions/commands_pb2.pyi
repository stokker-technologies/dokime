from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

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

@final
class CommandsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    parent: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class CommandsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COMMAND_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    command: ExecutionCommandReceipt
    resolved_at: _position_pb2.Position
    def __init__(self, command: _Optional[_Union[ExecutionCommandReceipt, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class CommandsCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    parent: str
    request_id: str
    command_id: str
    action: ExecutionCommandAction
    step_id: str
    def __init__(self, parent: _Optional[str] = ..., request_id: _Optional[str] = ..., command_id: _Optional[str] = ..., action: _Optional[_Union[ExecutionCommandAction, str]] = ..., step_id: _Optional[str] = ...) -> None: ...

@final
class CommandsCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COMMAND_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    command: ExecutionCommandReceipt
    receipt: _receipt_pb2.Receipt
    def __init__(self, command: _Optional[_Union[ExecutionCommandReceipt, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class ExecutionCommandReceipt(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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
