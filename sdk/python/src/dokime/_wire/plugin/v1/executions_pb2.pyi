from dokime._wire.model.v1 import execution_pb2 as _execution_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ExecutionsInitializeRequest(_message.Message):
    __slots__ = ("execution", "test_case", "origin")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    execution: str
    test_case: _execution_pb2.TestCase
    origin: _execution_pb2.Origin
    def __init__(self, execution: _Optional[str] = ..., test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

class ExecutionsInitializeResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ExecutionsCleanupRequest(_message.Message):
    __slots__ = ("execution", "termination", "test_case", "origin")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    TERMINATION_FIELD_NUMBER: _ClassVar[int]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    execution: str
    termination: _execution_pb2.Termination
    test_case: _execution_pb2.TestCase
    origin: _execution_pb2.Origin
    def __init__(self, execution: _Optional[str] = ..., termination: _Optional[_Union[_execution_pb2.Termination, str]] = ..., test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ..., origin: _Optional[_Union[_execution_pb2.Origin, _Mapping]] = ...) -> None: ...

class ExecutionsCleanupResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
