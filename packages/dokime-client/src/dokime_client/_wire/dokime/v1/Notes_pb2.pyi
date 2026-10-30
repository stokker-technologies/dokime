from dokime_client._wire.dokime.v1.NotesMessages.CreateNote import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.NotesMessages.EditNote import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.NotesMessages.ListNotes import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NotesCreateNoteCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class NotesCreateNoteRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: NotesCreateNoteCancellation
    finish: NotesCreateNoteCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[NotesCreateNoteCancellation, _Mapping]] = ..., finish: _Optional[_Union[NotesCreateNoteCancellation, _Mapping]] = ...) -> None: ...

class NotesCreateNoteRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class NotesEditNoteCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class NotesEditNoteRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: NotesEditNoteCancellation
    finish: NotesEditNoteCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[NotesEditNoteCancellation, _Mapping]] = ..., finish: _Optional[_Union[NotesEditNoteCancellation, _Mapping]] = ...) -> None: ...

class NotesEditNoteRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class NotesListNotesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class NotesListNotesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: NotesListNotesCancellation
    finish: NotesListNotesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[NotesListNotesCancellation, _Mapping]] = ..., finish: _Optional[_Union[NotesListNotesCancellation, _Mapping]] = ...) -> None: ...

class NotesListNotesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...
