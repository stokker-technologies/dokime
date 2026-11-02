from dokime_client._wire.dokime.v1.QueryMessages.Aggregate import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.QueryMessages.AggregateSamples import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.QueryMessages.ListCycles import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.QueryMessages.ListExecutions import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.QueryMessages.ListRecords import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.QueryMessages.ListSamples import messages_pb2 as _messages_pb2_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class QueryAggregateCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryAggregateRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: QueryAggregateCancellation
    finish: QueryAggregateCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryAggregateCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryAggregateCancellation, _Mapping]] = ...) -> None: ...

class QueryAggregateRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class QueryAggregateSamplesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryAggregateSamplesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: QueryAggregateSamplesCancellation
    finish: QueryAggregateSamplesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryAggregateSamplesCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryAggregateSamplesCancellation, _Mapping]] = ...) -> None: ...

class QueryAggregateSamplesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class QueryListCyclesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryListCyclesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: QueryListCyclesCancellation
    finish: QueryListCyclesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryListCyclesCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryListCyclesCancellation, _Mapping]] = ...) -> None: ...

class QueryListCyclesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class QueryListExecutionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryListExecutionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: QueryListExecutionsCancellation
    finish: QueryListExecutionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryListExecutionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryListExecutionsCancellation, _Mapping]] = ...) -> None: ...

class QueryListExecutionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class QueryListRecordsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryListRecordsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: QueryListRecordsCancellation
    finish: QueryListRecordsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryListRecordsCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryListRecordsCancellation, _Mapping]] = ...) -> None: ...

class QueryListRecordsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class QueryListSamplesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QueryListSamplesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: QueryListSamplesCancellation
    finish: QueryListSamplesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[QueryListSamplesCancellation, _Mapping]] = ..., finish: _Optional[_Union[QueryListSamplesCancellation, _Mapping]] = ...) -> None: ...

class QueryListSamplesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
