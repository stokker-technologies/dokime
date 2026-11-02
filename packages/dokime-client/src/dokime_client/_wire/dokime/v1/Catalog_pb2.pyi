from dokime_client._wire.dokime.v1.CatalogMessages.GetRecordSchema import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.CatalogMessages.GetSuite import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.CatalogMessages.GetTelemetrySchema import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.CatalogMessages.ListRecordSchemas import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.CatalogMessages.ListSuites import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.CatalogMessages.ListTelemetrySchemas import messages_pb2 as _messages_pb2_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CatalogGetRecordSchemaCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogGetRecordSchemaRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: CatalogGetRecordSchemaCancellation
    finish: CatalogGetRecordSchemaCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogGetRecordSchemaCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogGetRecordSchemaCancellation, _Mapping]] = ...) -> None: ...

class CatalogGetRecordSchemaRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class CatalogGetSuiteCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogGetSuiteRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: CatalogGetSuiteCancellation
    finish: CatalogGetSuiteCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogGetSuiteCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogGetSuiteCancellation, _Mapping]] = ...) -> None: ...

class CatalogGetSuiteRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class CatalogGetTelemetrySchemaCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogGetTelemetrySchemaRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: CatalogGetTelemetrySchemaCancellation
    finish: CatalogGetTelemetrySchemaCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogGetTelemetrySchemaCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogGetTelemetrySchemaCancellation, _Mapping]] = ...) -> None: ...

class CatalogGetTelemetrySchemaRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class CatalogListRecordSchemasCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogListRecordSchemasRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: CatalogListRecordSchemasCancellation
    finish: CatalogListRecordSchemasCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogListRecordSchemasCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogListRecordSchemasCancellation, _Mapping]] = ...) -> None: ...

class CatalogListRecordSchemasRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class CatalogListSuitesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogListSuitesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: CatalogListSuitesCancellation
    finish: CatalogListSuitesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogListSuitesCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogListSuitesCancellation, _Mapping]] = ...) -> None: ...

class CatalogListSuitesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class CatalogListTelemetrySchemasCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CatalogListTelemetrySchemasRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: CatalogListTelemetrySchemasCancellation
    finish: CatalogListTelemetrySchemasCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[CatalogListTelemetrySchemasCancellation, _Mapping]] = ..., finish: _Optional[_Union[CatalogListTelemetrySchemasCancellation, _Mapping]] = ...) -> None: ...

class CatalogListTelemetrySchemasRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
