# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Capture Sources, Modes and Settled Results."""

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass
from enum import IntEnum
from typing import Self, cast

# Third Party Imports
# Local Imports
from .._generated.proto.host.v1.files.captures_pb2 import CaptureMode as CaptureModeMessage
from .._generated.proto.host.v1.files.captures_pb2 import CapturesStopResponse
from .._generated.proto.host.v1.files.captures_pb2 import LineField as LineFieldMessage
from .._generated.proto.host.v1.files.captures_pb2 import LineMarker as LineMarkerMessage
from .._generated.proto.host.v1.files.captures_pb2 import LineMatcher as LineMatcherMessage
from .._generated.proto.host.v1.files.captures_pb2 import MirrorCapture as MirrorCaptureMessage
from .._generated.proto.host.v1.files.captures_pb2 import StreamCapture as StreamCaptureMessage
from .._generated.proto.host.v1.files_pb2 import FilesCaptureResponse
from .._generated.proto.model.v1.files_pb2 import FileLocation as FileLocationMessage
from .._generated.proto.model.v1.files_pb2 import FileRule as FileRuleMessage
from .._generated.proto.model.v1.files_pb2 import FileRuleAction as FileRuleActionMessage
from .._generated.proto.model.v1.files_pb2 import FileSelection as FileSelectionMessage
from .._generated.proto.model.v1.files_pb2 import FileSource as FileSourceMessage
from .._generated.proto.model.v1.files_pb2 import FtpLocation as FtpLocationMessage

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class FileRuleAction(IntEnum):
    INCLUDE = 1
    EXCLUDE = 2


@dataclass(frozen=True)
class FtpLocation:
    url: str
    secret_id: str | None = None

    def _wire(self) -> FtpLocationMessage:
        return FtpLocationMessage(url=self.url, secret_id=self.secret_id)


@dataclass(frozen=True)
class FileLocation:
    local: str | None = None
    ftp: FtpLocation | None = None

    def __post_init__(self) -> None:
        if (self.local is None) == (self.ftp is None):
            raise ValueError("Declare File Location: Supply Exactly One local or ftp")

    def _wire(self) -> FileLocationMessage:
        return FileLocationMessage(local=self.local, ftp=self.ftp._wire() if self.ftp is not None else None)


@dataclass(frozen=True)
class FileRule:
    glob: str
    action: FileRuleAction

    def _wire(self) -> FileRuleMessage:
        if not isinstance(self.action, FileRuleAction):
            raise TypeError("Declare File Rule: Supply a FileRuleAction Member")
        return FileRuleMessage(glob=self.glob, action=cast(FileRuleActionMessage, self.action.value))


@dataclass(frozen=True)
class FileSelection:
    root: FileLocation
    rules: tuple[FileRule, ...] = ()

    def _wire(self) -> FileSelectionMessage:
        return FileSelectionMessage(root=self.root._wire(), rules=[item._wire() for item in self.rules])


@dataclass(frozen=True)
class FileSource:
    file: FileLocation | None = None
    tree: FileSelection | None = None
    newest: FileSelection | None = None

    def __post_init__(self) -> None:
        if sum(item is not None for item in (self.file, self.tree, self.newest)) != 1:
            raise ValueError("Declare File Source: Supply Exactly One file, tree or newest")

    def _wire(self) -> FileSourceMessage:
        return FileSourceMessage(**{name: item._wire() for name in ("file", "tree", "newest") if (item := getattr(self, name)) is not None})


@dataclass(frozen=True)
class LineField:
    name: str
    type: str

    def _wire(self) -> LineFieldMessage:
        return LineFieldMessage(name=self.name, type=self.type)


@dataclass(frozen=True)
class LineMatcher:
    pattern: str
    stream: str
    fields: tuple[LineField, ...] = ()

    def _wire(self) -> LineMatcherMessage:
        return LineMatcherMessage(pattern=self.pattern, stream=self.stream, fields=[item._wire() for item in self.fields])


@dataclass(frozen=True)
class LineMarker:
    pattern: str
    record_type: str

    def _wire(self) -> LineMarkerMessage:
        return LineMarkerMessage(pattern=self.pattern, record_type=self.record_type)


@dataclass(frozen=True)
class StreamCapture:
    matchers: tuple[LineMatcher, ...] = ()
    markers: tuple[LineMarker, ...] = ()

    def _wire(self) -> StreamCaptureMessage:
        return StreamCaptureMessage(matchers=[item._wire() for item in self.matchers], markers=[item._wire() for item in self.markers])


@dataclass(frozen=True)
class MirrorCapture:
    interval_s: int | None = None

    def __post_init__(self) -> None:
        if self.interval_s is not None and (type(self.interval_s) is not int or self.interval_s <= 0):
            raise ValueError("Declare Mirror Capture: Supply a Positive Integer interval_s")

    def _wire(self) -> MirrorCaptureMessage:
        return MirrorCaptureMessage(interval_s=self.interval_s)


@dataclass(frozen=True)
class CaptureMode:
    stream: StreamCapture | None = None
    mirror: MirrorCapture | None = None

    def __post_init__(self) -> None:
        if (self.stream is None) == (self.mirror is None):
            raise ValueError("Declare Capture Mode: Supply Exactly One stream or mirror")

    def _wire(self) -> CaptureModeMessage:
        return CaptureModeMessage(
            stream=self.stream._wire() if self.stream is not None else None, mirror=self.mirror._wire() if self.mirror is not None else None
        )


@dataclass(frozen=True)
class CapturedArtifact:
    artifact: str
    size_bytes: int


@dataclass(frozen=True)
class CaptureGap:
    path: str
    offset: int
    reason: str


@dataclass(frozen=True)
class CaptureResult:
    artifacts: tuple[CapturedArtifact, ...]
    gaps: tuple[CaptureGap, ...]

    @classmethod
    def _from_wire(cls, value: FilesCaptureResponse | CapturesStopResponse) -> Self:
        reasons = {1: "Truncated", 2: "Replaced", 3: "Vanished", 4: "Resume Unsupported"}
        return cls(
            tuple(CapturedArtifact(item.artifact, item.size_bytes) for item in value.artifacts),
            tuple(CaptureGap(item.path, item.offset, reasons[item.reason]) for item in value.gaps),
        )


@dataclass(frozen=True)
class Environment:
    backend_pid: int
    in_step: bool


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
