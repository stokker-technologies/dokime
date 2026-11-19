from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.host.v1 import documents_pb2 as _documents_pb2
from dokime_sdk._wire.model.v1 import plots_pb2 as _plots_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class SettingsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: _documents_pb2.DocumentScope
    document_id: str
    plot_id: str
    at: _position_pb2.ReadCut
    def __init__(self, scope: _Optional[_Union[_documents_pb2.DocumentScope, _Mapping]] = ..., document_id: _Optional[str] = ..., plot_id: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SettingsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: PlotSettingsValue
    revision: bytes
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[PlotSettingsValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class SettingsSetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    scope: _documents_pb2.DocumentScope
    document_id: str
    plot_id: str
    value: PlotSettingsValue
    expected_revision: bytes
    request_id: str
    def __init__(self, scope: _Optional[_Union[_documents_pb2.DocumentScope, _Mapping]] = ..., document_id: _Optional[str] = ..., plot_id: _Optional[str] = ..., value: _Optional[_Union[PlotSettingsValue, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class SettingsSetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: PlotSettingsValue
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[PlotSettingsValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class PlotSettingsValue(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    DISPLAY_POINTS_FIELD_NUMBER: _ClassVar[int]
    DOWNSAMPLE_FIELD_NUMBER: _ClassVar[int]
    display_points: int
    downsample: _plots_pb2.Downsample
    def __init__(self, display_points: _Optional[int] = ..., downsample: _Optional[_Union[_plots_pb2.Downsample, str]] = ...) -> None: ...
