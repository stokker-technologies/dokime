import datetime

from dokime_sdk._wire.host.v1 import backend_pb2 as _backend_pb2
from dokime_sdk._wire.host.v1.backend import overview_pb2 as _overview_pb2
from dokime_sdk._wire.host.v1 import cases_pb2 as _cases_pb2
from dokime_sdk._wire.host.v1.content import artifacts_pb2 as _artifacts_pb2
from dokime_sdk._wire.host.v1.content import execution_files_pb2 as _execution_files_pb2
from dokime_sdk._wire.host.v1 import cycles_pb2 as _cycles_pb2
from dokime_sdk._wire.host.v1 import documents_pb2 as _documents_pb2
from dokime_sdk._wire.host.v1.evidence import health_events_pb2 as _health_events_pb2
from dokime_sdk._wire.host.v1.evidence import notes_pb2 as _notes_pb2
from dokime_sdk._wire.host.v1.evidence import records_pb2 as _records_pb2
from dokime_sdk._wire.host.v1.evidence import traceability_pb2 as _traceability_pb2
from dokime_sdk._wire.host.v1 import executions_pb2 as _executions_pb2
from dokime_sdk._wire.host.v1.executions import schemas_pb2 as _schemas_pb2
from dokime_sdk._wire.host.v1 import files_pb2 as _files_pb2
from dokime_sdk._wire.host.v1.plots import snapshots_pb2 as _snapshots_pb2
from dokime_sdk._wire.host.v1 import suites_pb2 as _suites_pb2
from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
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

class ReportCapture(_message.Message):
    __slots__ = ("report", "cycle", "cases", "executions", "document", "appearance", "notes", "health", "records", "artifacts", "files", "suites", "overview")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    CASES_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_FIELD_NUMBER: _ClassVar[int]
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    SUITES_FIELD_NUMBER: _ClassVar[int]
    OVERVIEW_FIELD_NUMBER: _ClassVar[int]
    report: ReportMetadata
    cycle: _cycles_pb2.CyclesGetResponse
    cases: _containers.RepeatedCompositeFieldContainer[ReportCase]
    executions: _containers.RepeatedCompositeFieldContainer[ReportExecution]
    document: _documents_pb2.DocumentsGetResponse
    appearance: _backend_pb2.BackendAppearanceResponse
    notes: _containers.RepeatedCompositeFieldContainer[_notes_pb2.NotesListResponse]
    health: _containers.RepeatedCompositeFieldContainer[_health_events_pb2.HealthEventsListResponse]
    records: _containers.RepeatedCompositeFieldContainer[_records_pb2.RecordsListResponse]
    artifacts: _containers.RepeatedCompositeFieldContainer[_artifacts_pb2.ArtifactsGetResponse]
    files: _containers.RepeatedCompositeFieldContainer[ReportFile]
    suites: _containers.RepeatedCompositeFieldContainer[_suites_pb2.SuitesGetResponse]
    overview: _overview_pb2.OverviewGetResponse
    def __init__(self, report: _Optional[_Union[ReportMetadata, _Mapping]] = ..., cycle: _Optional[_Union[_cycles_pb2.CyclesGetResponse, _Mapping]] = ..., cases: _Optional[_Iterable[_Union[ReportCase, _Mapping]]] = ..., executions: _Optional[_Iterable[_Union[ReportExecution, _Mapping]]] = ..., document: _Optional[_Union[_documents_pb2.DocumentsGetResponse, _Mapping]] = ..., appearance: _Optional[_Union[_backend_pb2.BackendAppearanceResponse, _Mapping]] = ..., notes: _Optional[_Iterable[_Union[_notes_pb2.NotesListResponse, _Mapping]]] = ..., health: _Optional[_Iterable[_Union[_health_events_pb2.HealthEventsListResponse, _Mapping]]] = ..., records: _Optional[_Iterable[_Union[_records_pb2.RecordsListResponse, _Mapping]]] = ..., artifacts: _Optional[_Iterable[_Union[_artifacts_pb2.ArtifactsGetResponse, _Mapping]]] = ..., files: _Optional[_Iterable[_Union[ReportFile, _Mapping]]] = ..., suites: _Optional[_Iterable[_Union[_suites_pb2.SuitesGetResponse, _Mapping]]] = ..., overview: _Optional[_Union[_overview_pb2.OverviewGetResponse, _Mapping]] = ...) -> None: ...

class ReportCase(_message.Message):
    __slots__ = ("detail", "parameters")
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    detail: _cases_pb2.CasesGetResponse
    parameters: _cases_pb2.CasesParametersResponse
    def __init__(self, detail: _Optional[_Union[_cases_pb2.CasesGetResponse, _Mapping]] = ..., parameters: _Optional[_Union[_cases_pb2.CasesParametersResponse, _Mapping]] = ...) -> None: ...

class ReportExecution(_message.Message):
    __slots__ = ("detail", "inputs", "outputs", "steps", "document", "plots", "traceability", "files", "schemas")
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    INPUTS_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_FIELD_NUMBER: _ClassVar[int]
    PLOTS_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    detail: _executions_pb2.ExecutionsGetResponse
    inputs: _executions_pb2.ExecutionsParametersResponse
    outputs: _executions_pb2.ExecutionsParametersResponse
    steps: _executions_pb2.ExecutionsStepsResponse
    document: _documents_pb2.DocumentsGetResponse
    plots: _snapshots_pb2.SnapshotsListResponse
    traceability: _containers.RepeatedCompositeFieldContainer[_traceability_pb2.TraceabilityGetResponse]
    files: _containers.RepeatedCompositeFieldContainer[_execution_files_pb2.ExecutionFilesGetResponse]
    schemas: _containers.RepeatedCompositeFieldContainer[_schemas_pb2.SchemasGetResponse]
    def __init__(self, detail: _Optional[_Union[_executions_pb2.ExecutionsGetResponse, _Mapping]] = ..., inputs: _Optional[_Union[_executions_pb2.ExecutionsParametersResponse, _Mapping]] = ..., outputs: _Optional[_Union[_executions_pb2.ExecutionsParametersResponse, _Mapping]] = ..., steps: _Optional[_Union[_executions_pb2.ExecutionsStepsResponse, _Mapping]] = ..., document: _Optional[_Union[_documents_pb2.DocumentsGetResponse, _Mapping]] = ..., plots: _Optional[_Union[_snapshots_pb2.SnapshotsListResponse, _Mapping]] = ..., traceability: _Optional[_Iterable[_Union[_traceability_pb2.TraceabilityGetResponse, _Mapping]]] = ..., files: _Optional[_Iterable[_Union[_execution_files_pb2.ExecutionFilesGetResponse, _Mapping]]] = ..., schemas: _Optional[_Iterable[_Union[_schemas_pb2.SchemasGetResponse, _Mapping]]] = ...) -> None: ...

class ReportFile(_message.Message):
    __slots__ = ("file", "reads")
    FILE_FIELD_NUMBER: _ClassVar[int]
    READS_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    reads: _containers.RepeatedCompositeFieldContainer[_files_pb2.FilesReadResponse]
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., reads: _Optional[_Iterable[_Union[_files_pb2.FilesReadResponse, _Mapping]]] = ...) -> None: ...
