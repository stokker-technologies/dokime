from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FileRuleAction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FILE_RULE_ACTION_UNSPECIFIED: _ClassVar[FileRuleAction]
    FILE_RULE_ACTION_INCLUDE: _ClassVar[FileRuleAction]
    FILE_RULE_ACTION_EXCLUDE: _ClassVar[FileRuleAction]

class CaptureGapReason(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CAPTURE_GAP_REASON_UNSPECIFIED: _ClassVar[CaptureGapReason]
    CAPTURE_GAP_REASON_TRUNCATED: _ClassVar[CaptureGapReason]
    CAPTURE_GAP_REASON_REPLACED: _ClassVar[CaptureGapReason]
    CAPTURE_GAP_REASON_VANISHED: _ClassVar[CaptureGapReason]
    CAPTURE_GAP_REASON_RESUME_UNSUPPORTED: _ClassVar[CaptureGapReason]
FILE_RULE_ACTION_UNSPECIFIED: FileRuleAction
FILE_RULE_ACTION_INCLUDE: FileRuleAction
FILE_RULE_ACTION_EXCLUDE: FileRuleAction
CAPTURE_GAP_REASON_UNSPECIFIED: CaptureGapReason
CAPTURE_GAP_REASON_TRUNCATED: CaptureGapReason
CAPTURE_GAP_REASON_REPLACED: CaptureGapReason
CAPTURE_GAP_REASON_VANISHED: CaptureGapReason
CAPTURE_GAP_REASON_RESUME_UNSUPPORTED: CaptureGapReason

class FileSource(_message.Message):
    __slots__ = ("file", "tree", "newest")
    FILE_FIELD_NUMBER: _ClassVar[int]
    TREE_FIELD_NUMBER: _ClassVar[int]
    NEWEST_FIELD_NUMBER: _ClassVar[int]
    file: FileLocation
    tree: FileSelection
    newest: FileSelection
    def __init__(self, file: _Optional[_Union[FileLocation, _Mapping]] = ..., tree: _Optional[_Union[FileSelection, _Mapping]] = ..., newest: _Optional[_Union[FileSelection, _Mapping]] = ...) -> None: ...

class FileLocation(_message.Message):
    __slots__ = ("local", "ftp")
    LOCAL_FIELD_NUMBER: _ClassVar[int]
    FTP_FIELD_NUMBER: _ClassVar[int]
    local: str
    ftp: FtpLocation
    def __init__(self, local: _Optional[str] = ..., ftp: _Optional[_Union[FtpLocation, _Mapping]] = ...) -> None: ...

class FtpLocation(_message.Message):
    __slots__ = ("url", "secret_id")
    URL_FIELD_NUMBER: _ClassVar[int]
    SECRET_ID_FIELD_NUMBER: _ClassVar[int]
    url: str
    secret_id: str
    def __init__(self, url: _Optional[str] = ..., secret_id: _Optional[str] = ...) -> None: ...

class FileSelection(_message.Message):
    __slots__ = ("root", "rules")
    ROOT_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    root: FileLocation
    rules: _containers.RepeatedCompositeFieldContainer[FileRule]
    def __init__(self, root: _Optional[_Union[FileLocation, _Mapping]] = ..., rules: _Optional[_Iterable[_Union[FileRule, _Mapping]]] = ...) -> None: ...

class FileRule(_message.Message):
    __slots__ = ("glob", "action")
    GLOB_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    glob: str
    action: FileRuleAction
    def __init__(self, glob: _Optional[str] = ..., action: _Optional[_Union[FileRuleAction, str]] = ...) -> None: ...

class CapturedArtifact(_message.Message):
    __slots__ = ("artifact", "size_bytes")
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    size_bytes: int
    def __init__(self, artifact: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class CaptureGap(_message.Message):
    __slots__ = ("path", "offset", "reason")
    PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    path: str
    offset: int
    reason: CaptureGapReason
    def __init__(self, path: _Optional[str] = ..., offset: _Optional[int] = ..., reason: _Optional[_Union[CaptureGapReason, str]] = ...) -> None: ...
