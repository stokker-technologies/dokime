from dokime_client._wire.dokime.v1.PlanningMessages.MaterializeTemplate import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.PlanningMessages.PreviewSelection import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.PlanningMessages.RegisterSelection import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PlanningMaterializeTemplateCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlanningMaterializeTemplateRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: PlanningMaterializeTemplateCancellation
    finish: PlanningMaterializeTemplateCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlanningMaterializeTemplateCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlanningMaterializeTemplateCancellation, _Mapping]] = ...) -> None: ...

class PlanningMaterializeTemplateRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class PlanningPreviewSelectionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlanningPreviewSelectionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: PlanningPreviewSelectionCancellation
    finish: PlanningPreviewSelectionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlanningPreviewSelectionCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlanningPreviewSelectionCancellation, _Mapping]] = ...) -> None: ...

class PlanningPreviewSelectionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class PlanningRegisterSelectionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlanningRegisterSelectionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: PlanningRegisterSelectionCancellation
    finish: PlanningRegisterSelectionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlanningRegisterSelectionCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlanningRegisterSelectionCancellation, _Mapping]] = ...) -> None: ...

class PlanningRegisterSelectionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...
