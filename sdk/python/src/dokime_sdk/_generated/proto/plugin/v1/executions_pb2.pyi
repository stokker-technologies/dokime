from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import execution_pb2 as _execution_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class ExecutionsInitializeRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    execution: str
    test_case: _execution_pb2.TestCase
    origin: _execution_pb2.Origin
    def __init__(self, execution: _Optional[str] = ..., test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

@final
class ExecutionsInitializeResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...

@final
class ExecutionsCleanupRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    TERMINATION_FIELD_NUMBER: _ClassVar[int]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    execution: str
    termination: _execution_pb2.Termination
    test_case: _execution_pb2.TestCase
    origin: _execution_pb2.Origin
    def __init__(self, execution: _Optional[str] = ..., termination: _Optional[_Union[_execution_pb2.Termination, str]] = ..., test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

@final
class ExecutionsCleanupResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    def __init__(self) -> None: ...
