from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import execution_pb2 as _execution_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class StepsExecuteRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    execution: str
    step: str
    test_case: _execution_pb2.TestCase
    origin: _execution_pb2.Origin
    def __init__(self, execution: _Optional[str] = ..., step: _Optional[str] = ..., test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

@final
class StepsExecuteResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...
