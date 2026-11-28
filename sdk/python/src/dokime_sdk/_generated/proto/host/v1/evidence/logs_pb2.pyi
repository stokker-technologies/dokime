from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._generated.proto.model.v1 import evidence_pb2 as _evidence_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class LogsAppendRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    execution: str
    severity: _evidence_pb2.Severity
    message: str
    at: _timestamp_pb2.Timestamp
    request_id: str
    def __init__(self, execution: _Optional[str] = ..., severity: _Optional[_Union[_evidence_pb2.Severity, str]] = ..., message: _Optional[str] = ..., at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class LogsAppendResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...
