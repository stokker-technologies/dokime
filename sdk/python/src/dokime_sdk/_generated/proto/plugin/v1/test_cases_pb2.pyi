from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import execution_pb2 as _execution_pb2
from dokime_sdk._generated.proto.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class TestCasesMaterializeRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    @final
    class ParametersEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
        __slots__: tuple[str, ...]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    template: str
    parameters: _containers.MessageMap[str, _scalar_pb2.Value]
    def __init__(self, template: _Optional[str] = ..., parameters: _Optional[_Mapping[str, _scalar_pb2.Value]] = ...) -> None: ...

@final
class TestCasesMaterializeResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    test_case: _execution_pb2.TestCase
    def __init__(self, test_case: _Optional[_Union[_execution_pb2.TestCase, _Mapping]] = ...) -> None: ...
