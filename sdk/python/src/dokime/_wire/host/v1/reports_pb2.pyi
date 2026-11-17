import datetime

from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ReportOrigin(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    REPORT_ORIGIN_UNSPECIFIED: _ClassVar[ReportOrigin]
    REPORT_ORIGIN_SEALED: _ClassVar[ReportOrigin]
    REPORT_ORIGIN_CURRENT: _ClassVar[ReportOrigin]
REPORT_ORIGIN_UNSPECIFIED: ReportOrigin
REPORT_ORIGIN_SEALED: ReportOrigin
REPORT_ORIGIN_CURRENT: ReportOrigin

class ReportsGetRequest(_message.Message):
    __slots__ = ("subject", "name", "at")
    SUBJECT_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    subject: ReportSubject
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, subject: _Optional[_Union[ReportSubject, _Mapping]] = ..., name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class ReportsGetResponse(_message.Message):
    __slots__ = ("report", "file", "resolved_at")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    report: ReportMetadata
    file: _file_pb2.FileReference
    resolved_at: _position_pb2.ReadCut
    def __init__(self, report: _Optional[_Union[ReportMetadata, _Mapping]] = ..., file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class ReportsCreateRequest(_message.Message):
    __slots__ = ("subject", "document_id", "request_id")
    SUBJECT_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    subject: ReportSubject
    document_id: str
    request_id: str
    def __init__(self, subject: _Optional[_Union[ReportSubject, _Mapping]] = ..., document_id: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class ReportsCreateResponse(_message.Message):
    __slots__ = ("report", "file", "resolved_at", "receipt")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    report: ReportMetadata
    file: _file_pb2.FileReference
    resolved_at: _position_pb2.ReadCut
    receipt: _receipt_pb2.Receipt
    def __init__(self, report: _Optional[_Union[ReportMetadata, _Mapping]] = ..., file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ReportsPublishRequest(_message.Message):
    __slots__ = ("name", "subject", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SUBJECT_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    subject: ReportSubject
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., subject: _Optional[_Union[ReportSubject, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class ReportsPublishResponse(_message.Message):
    __slots__ = ("report", "seal", "receipt")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    SEAL_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    report: ReportMetadata
    seal: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, report: _Optional[_Union[ReportMetadata, _Mapping]] = ..., seal: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class ReportMetadata(_message.Message):
    __slots__ = ("test_cycle_id", "test_execution_id", "identifier", "metadata", "generated_at", "origin", "warnings")
    TEST_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    test_cycle_id: str
    test_execution_id: str
    identifier: str
    metadata: _metadata_pb2.Metadata
    generated_at: _timestamp_pb2.Timestamp
    origin: ReportOrigin
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, test_cycle_id: _Optional[str] = ..., test_execution_id: _Optional[str] = ..., identifier: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., generated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., origin: _Optional[_Union[ReportOrigin, str]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class ReportSubject(_message.Message):
    __slots__ = ("execution", "cycle")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    execution: str
    cycle: str
    def __init__(self, execution: _Optional[str] = ..., cycle: _Optional[str] = ...) -> None: ...
