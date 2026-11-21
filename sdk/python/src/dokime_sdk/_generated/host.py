# @generated from protocol descriptors; do not edit.
from collections.abc import AsyncGenerator
from enum import Enum
from typing import TYPE_CHECKING, cast
from uuid import uuid4

from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos._contracts import Operation
from harmos._wire.host.v1.invocations_pb2 import (
    InvocationsCancelRequest,
    InvocationsCancelResponse,
    InvocationsGetRequest,
    InvocationsGetResponse,
    InvocationsWatchRequest,
    InvocationsWatchResponse,
)
from harmos._wire.host.v1.operations_pb2 import OperationsGetRequest, OperationsGetResponse, OperationsListRequest, OperationsListResponse
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListRequest, SettingsListResponse
from harmos._wire.host.v1.plugins.state_pb2 import StateGetRequest, StateGetResponse, StatePutRequest, StatePutResponse
from harmos._wire.host.v1.time_pb2 import TimeGetRequest, TimeGetResponse, TimeSleepRequest, TimeSleepResponse
from harmos._wire.model.v1.file_pb2 import FileMetadata, FileReference, Upload
from harmos._wire.model.v1.invocation_pb2 import InstanceState
from harmos._wire.model.v1.operation_pb2 import VISIBILITY_PUBLIC, DiscoveredOperation
from harmos._wire.model.v1.page_pb2 import PageRequest
from harmos._wire.model.v1.policy_pb2 import OperationPolicy
from harmos._wire.model.v1.position_pb2 import Position, ReadCut
from harmos._wire.model.v1.receipt_pb2 import Receipt

import dokime_sdk._wire.model.v1.metadata_pb2
from dokime_sdk._wire.host.v1.backend_pb2 import BackendEnvironmentRequest, BackendEnvironmentResponse
from dokime_sdk._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateRequest, ArtifactsCreateResponse
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentsAppendPartsRequest, DocumentsAppendPartsResponse
from dokime_sdk._wire.host.v1.effects_pb2 import (
    EffectsCreateRequest,
    EffectsCreateResponse,
    EffectsGetRequest,
    EffectsGetResponse,
    EffectsSetRequest,
    EffectsSetResponse,
    EffectWritable,
)
from dokime_sdk._wire.host.v1.evidence.logs_pb2 import LogsAppendRequest, LogsAppendResponse
from dokime_sdk._wire.host.v1.evidence.records_pb2 import (
    CapturedRecordSelection,
    RecordsAppendRequest,
    RecordsAppendResponse,
    RecordScope,
    RecordsListRequest,
    RecordsListResponse,
)
from dokime_sdk._wire.host.v1.evidence.traceability_pb2 import (
    TraceabilityGetRequest,
    TraceabilityGetResponse,
    TraceabilitySetRequest,
    TraceabilitySetResponse,
)
from dokime_sdk._wire.host.v1.evidence_pb2 import EvidenceStatisticsRequest, EvidenceStatisticsResponse, Output
from dokime_sdk._wire.host.v1.executions.progress_pb2 import (
    ProgressGetRequest,
    ProgressGetResponse,
    ProgressSetRequest,
    ProgressSetResponse,
)
from dokime_sdk._wire.host.v1.files.captures_pb2 import (
    CaptureMode,
    CapturesStartRequest,
    CapturesStartResponse,
    CapturesStopRequest,
    CapturesStopResponse,
)
from dokime_sdk._wire.host.v1.files.uploads_pb2 import (
    UploadsAppendRequest,
    UploadsAppendResponse,
    UploadsCreateRequest,
    UploadsCreateResponse,
    UploadsDeleteRequest,
    UploadsDeleteResponse,
    UploadsGetRequest,
    UploadsGetResponse,
    UploadsPublishRequest,
    UploadsPublishResponse,
    UploadsResumeRequest,
    UploadsResumeResponse,
)
from dokime_sdk._wire.host.v1.files_pb2 import (
    FilesCaptureRequest,
    FilesCaptureResponse,
    FilesDeleteRequest,
    FilesDeleteResponse,
    FilesGetRequest,
    FilesGetResponse,
    FilesReadRequest,
    FilesReadResponse,
    FilesWatchRequest,
    FilesWatchResponse,
)
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptsCreateRequest, PromptsCreateResponse
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsGetRequest, SecretsGetResponse
from dokime_sdk._wire.model.v1.documents_pb2 import Card, Section
from dokime_sdk._wire.model.v1.evidence_pb2 import ArtifactScope, EvidenceScope, Severity, Window
from dokime_sdk._wire.model.v1.execution_pb2 import ExecutionProgressValue
from dokime_sdk._wire.model.v1.files_pb2 import FileSource
from dokime_sdk._wire.model.v1.prompts_pb2 import Prompt
from dokime_sdk._wire.model.v1.scalar_pb2 import Value
from dokime_sdk._wire.model.v1.traceability_pb2 import TraceabilityEntry

if TYPE_CHECKING:
    from harmos._invocation import Context


class HostOperation(Enum):
    BACKEND_ENVIRONMENT = "dokime.host.v1.Backend/Environment"
    CONTENT_ARTIFACTS_CREATE = "dokime.host.v1.content.Artifacts/Create"
    DOCUMENTS_APPEND_PARTS = "dokime.host.v1.Documents/AppendParts"
    EFFECTS_CREATE = "dokime.host.v1.Effects/Create"
    EFFECTS_GET = "dokime.host.v1.Effects/Get"
    EFFECTS_SET = "dokime.host.v1.Effects/Set"
    EVIDENCE_LOGS_APPEND = "dokime.host.v1.evidence.Logs/Append"
    EVIDENCE_RECORDS_LIST = "dokime.host.v1.evidence.Records/List"
    EVIDENCE_RECORDS_APPEND = "dokime.host.v1.evidence.Records/Append"
    EVIDENCE_TRACEABILITY_GET = "dokime.host.v1.evidence.Traceability/Get"
    EVIDENCE_TRACEABILITY_SET = "dokime.host.v1.evidence.Traceability/Set"
    EVIDENCE_STATISTICS = "dokime.host.v1.Evidence/Statistics"
    EXECUTIONS_PROGRESS_GET = "dokime.host.v1.executions.Progress/Get"
    EXECUTIONS_PROGRESS_SET = "dokime.host.v1.executions.Progress/Set"
    FILES_CAPTURES_START = "dokime.host.v1.files.Captures/Start"
    FILES_CAPTURES_STOP = "dokime.host.v1.files.Captures/Stop"
    FILES_UPLOADS_CREATE = "dokime.host.v1.files.Uploads/Create"
    FILES_UPLOADS_GET = "dokime.host.v1.files.Uploads/Get"
    FILES_UPLOADS_RESUME = "dokime.host.v1.files.Uploads/Resume"
    FILES_UPLOADS_APPEND = "dokime.host.v1.files.Uploads/Append"
    FILES_UPLOADS_PUBLISH = "dokime.host.v1.files.Uploads/Publish"
    FILES_UPLOADS_DELETE = "dokime.host.v1.files.Uploads/Delete"
    FILES_GET = "dokime.host.v1.Files/Get"
    FILES_READ = "dokime.host.v1.Files/Read"
    FILES_WATCH = "dokime.host.v1.Files/Watch"
    FILES_DELETE = "dokime.host.v1.Files/Delete"
    FILES_CAPTURE = "dokime.host.v1.Files/Capture"
    PROMPTS_CREATE = "dokime.host.v1.Prompts/Create"
    SECRETS_GET = "dokime.host.v1.Secrets/Get"
    INVOCATIONS_GET = "harmos.plugin.v1.Invocations/Get"
    INVOCATIONS_CANCEL = "harmos.plugin.v1.Invocations/Cancel"
    INVOCATIONS_WATCH = "harmos.plugin.v1.Invocations/Watch"
    OPERATIONS_LIST = "harmos.plugin.v1.Operations/List"
    OPERATIONS_GET = "harmos.plugin.v1.Operations/Get"
    PLUGINS_SETTINGS_LIST = "harmos.plugin.v1.plugins.Settings/List"
    PLUGINS_STATE_GET = "harmos.plugin.v1.plugins.State/Get"
    PLUGINS_STATE_PUT = "harmos.plugin.v1.plugins.State/Put"
    TIME_GET = "harmos.plugin.v1.Time/Get"
    TIME_SLEEP = "harmos.plugin.v1.Time/Sleep"


_BACKEND_ENVIRONMENT = Operation(
    HostOperation.BACKEND_ENVIRONMENT.value,
    BackendEnvironmentRequest,
    BackendEnvironmentResponse,
    visibility=VISIBILITY_PUBLIC,
    watch=False,
)
_CONTENT_ARTIFACTS_CREATE = Operation(
    HostOperation.CONTENT_ARTIFACTS_CREATE.value, ArtifactsCreateRequest, ArtifactsCreateResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_DOCUMENTS_APPEND_PARTS = Operation(
    HostOperation.DOCUMENTS_APPEND_PARTS.value,
    DocumentsAppendPartsRequest,
    DocumentsAppendPartsResponse,
    visibility=VISIBILITY_PUBLIC,
    watch=False,
)
_EFFECTS_CREATE = Operation(
    HostOperation.EFFECTS_CREATE.value, EffectsCreateRequest, EffectsCreateResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_EFFECTS_GET = Operation(HostOperation.EFFECTS_GET.value, EffectsGetRequest, EffectsGetResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_EFFECTS_SET = Operation(HostOperation.EFFECTS_SET.value, EffectsSetRequest, EffectsSetResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_EVIDENCE_LOGS_APPEND = Operation(
    HostOperation.EVIDENCE_LOGS_APPEND.value, LogsAppendRequest, LogsAppendResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_EVIDENCE_RECORDS_LIST = Operation(
    HostOperation.EVIDENCE_RECORDS_LIST.value, RecordsListRequest, RecordsListResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_EVIDENCE_RECORDS_APPEND = Operation(
    HostOperation.EVIDENCE_RECORDS_APPEND.value, RecordsAppendRequest, RecordsAppendResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_EVIDENCE_TRACEABILITY_GET = Operation(
    HostOperation.EVIDENCE_TRACEABILITY_GET.value,
    TraceabilityGetRequest,
    TraceabilityGetResponse,
    visibility=VISIBILITY_PUBLIC,
    watch=False,
)
_EVIDENCE_TRACEABILITY_SET = Operation(
    HostOperation.EVIDENCE_TRACEABILITY_SET.value,
    TraceabilitySetRequest,
    TraceabilitySetResponse,
    visibility=VISIBILITY_PUBLIC,
    watch=False,
)
_EVIDENCE_STATISTICS = Operation(
    HostOperation.EVIDENCE_STATISTICS.value,
    EvidenceStatisticsRequest,
    EvidenceStatisticsResponse,
    visibility=VISIBILITY_PUBLIC,
    watch=False,
)
_EXECUTIONS_PROGRESS_GET = Operation(
    HostOperation.EXECUTIONS_PROGRESS_GET.value, ProgressGetRequest, ProgressGetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_EXECUTIONS_PROGRESS_SET = Operation(
    HostOperation.EXECUTIONS_PROGRESS_SET.value, ProgressSetRequest, ProgressSetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_CAPTURES_START = Operation(
    HostOperation.FILES_CAPTURES_START.value, CapturesStartRequest, CapturesStartResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_CAPTURES_STOP = Operation(
    HostOperation.FILES_CAPTURES_STOP.value, CapturesStopRequest, CapturesStopResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_CREATE = Operation(
    HostOperation.FILES_UPLOADS_CREATE.value, UploadsCreateRequest, UploadsCreateResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_GET = Operation(
    HostOperation.FILES_UPLOADS_GET.value, UploadsGetRequest, UploadsGetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_RESUME = Operation(
    HostOperation.FILES_UPLOADS_RESUME.value, UploadsResumeRequest, UploadsResumeResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_APPEND = Operation(
    HostOperation.FILES_UPLOADS_APPEND.value, UploadsAppendRequest, UploadsAppendResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_PUBLISH = Operation(
    HostOperation.FILES_UPLOADS_PUBLISH.value, UploadsPublishRequest, UploadsPublishResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_UPLOADS_DELETE = Operation(
    HostOperation.FILES_UPLOADS_DELETE.value, UploadsDeleteRequest, UploadsDeleteResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_GET = Operation(HostOperation.FILES_GET.value, FilesGetRequest, FilesGetResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_FILES_READ = Operation(HostOperation.FILES_READ.value, FilesReadRequest, FilesReadResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_FILES_WATCH = Operation(HostOperation.FILES_WATCH.value, FilesWatchRequest, FilesWatchResponse, visibility=VISIBILITY_PUBLIC, watch=True)
_FILES_DELETE = Operation(
    HostOperation.FILES_DELETE.value, FilesDeleteRequest, FilesDeleteResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_FILES_CAPTURE = Operation(
    HostOperation.FILES_CAPTURE.value, FilesCaptureRequest, FilesCaptureResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_PROMPTS_CREATE = Operation(
    HostOperation.PROMPTS_CREATE.value, PromptsCreateRequest, PromptsCreateResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_SECRETS_GET = Operation(HostOperation.SECRETS_GET.value, SecretsGetRequest, SecretsGetResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_INVOCATIONS_GET = Operation(
    HostOperation.INVOCATIONS_GET.value, InvocationsGetRequest, InvocationsGetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_INVOCATIONS_CANCEL = Operation(
    HostOperation.INVOCATIONS_CANCEL.value, InvocationsCancelRequest, InvocationsCancelResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_INVOCATIONS_WATCH = Operation(
    HostOperation.INVOCATIONS_WATCH.value, InvocationsWatchRequest, InvocationsWatchResponse, visibility=VISIBILITY_PUBLIC, watch=True
)
_OPERATIONS_LIST = Operation(
    HostOperation.OPERATIONS_LIST.value, OperationsListRequest, OperationsListResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_OPERATIONS_GET = Operation(
    HostOperation.OPERATIONS_GET.value, OperationsGetRequest, OperationsGetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_PLUGINS_SETTINGS_LIST = Operation(
    HostOperation.PLUGINS_SETTINGS_LIST.value, SettingsListRequest, SettingsListResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_PLUGINS_STATE_GET = Operation(
    HostOperation.PLUGINS_STATE_GET.value, StateGetRequest, StateGetResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_PLUGINS_STATE_PUT = Operation(
    HostOperation.PLUGINS_STATE_PUT.value, StatePutRequest, StatePutResponse, visibility=VISIBILITY_PUBLIC, watch=False
)
_TIME_GET = Operation(HostOperation.TIME_GET.value, TimeGetRequest, TimeGetResponse, visibility=VISIBILITY_PUBLIC, watch=False)
_TIME_SLEEP = Operation(HostOperation.TIME_SLEEP.value, TimeSleepRequest, TimeSleepResponse, visibility=VISIBILITY_PUBLIC, watch=False)


def host_requirement(operation: HostOperation) -> Operation[Message, Message]:
    match operation:
        case HostOperation.BACKEND_ENVIRONMENT:
            return cast(Operation[Message, Message], _BACKEND_ENVIRONMENT)
        case HostOperation.CONTENT_ARTIFACTS_CREATE:
            return cast(Operation[Message, Message], _CONTENT_ARTIFACTS_CREATE)
        case HostOperation.DOCUMENTS_APPEND_PARTS:
            return cast(Operation[Message, Message], _DOCUMENTS_APPEND_PARTS)
        case HostOperation.EFFECTS_CREATE:
            return cast(Operation[Message, Message], _EFFECTS_CREATE)
        case HostOperation.EFFECTS_GET:
            return cast(Operation[Message, Message], _EFFECTS_GET)
        case HostOperation.EFFECTS_SET:
            return cast(Operation[Message, Message], _EFFECTS_SET)
        case HostOperation.EVIDENCE_LOGS_APPEND:
            return cast(Operation[Message, Message], _EVIDENCE_LOGS_APPEND)
        case HostOperation.EVIDENCE_RECORDS_LIST:
            return cast(Operation[Message, Message], _EVIDENCE_RECORDS_LIST)
        case HostOperation.EVIDENCE_RECORDS_APPEND:
            return cast(Operation[Message, Message], _EVIDENCE_RECORDS_APPEND)
        case HostOperation.EVIDENCE_TRACEABILITY_GET:
            return cast(Operation[Message, Message], _EVIDENCE_TRACEABILITY_GET)
        case HostOperation.EVIDENCE_TRACEABILITY_SET:
            return cast(Operation[Message, Message], _EVIDENCE_TRACEABILITY_SET)
        case HostOperation.EVIDENCE_STATISTICS:
            return cast(Operation[Message, Message], _EVIDENCE_STATISTICS)
        case HostOperation.EXECUTIONS_PROGRESS_GET:
            return cast(Operation[Message, Message], _EXECUTIONS_PROGRESS_GET)
        case HostOperation.EXECUTIONS_PROGRESS_SET:
            return cast(Operation[Message, Message], _EXECUTIONS_PROGRESS_SET)
        case HostOperation.FILES_CAPTURES_START:
            return cast(Operation[Message, Message], _FILES_CAPTURES_START)
        case HostOperation.FILES_CAPTURES_STOP:
            return cast(Operation[Message, Message], _FILES_CAPTURES_STOP)
        case HostOperation.FILES_UPLOADS_CREATE:
            return cast(Operation[Message, Message], _FILES_UPLOADS_CREATE)
        case HostOperation.FILES_UPLOADS_GET:
            return cast(Operation[Message, Message], _FILES_UPLOADS_GET)
        case HostOperation.FILES_UPLOADS_RESUME:
            return cast(Operation[Message, Message], _FILES_UPLOADS_RESUME)
        case HostOperation.FILES_UPLOADS_APPEND:
            return cast(Operation[Message, Message], _FILES_UPLOADS_APPEND)
        case HostOperation.FILES_UPLOADS_PUBLISH:
            return cast(Operation[Message, Message], _FILES_UPLOADS_PUBLISH)
        case HostOperation.FILES_UPLOADS_DELETE:
            return cast(Operation[Message, Message], _FILES_UPLOADS_DELETE)
        case HostOperation.FILES_GET:
            return cast(Operation[Message, Message], _FILES_GET)
        case HostOperation.FILES_READ:
            return cast(Operation[Message, Message], _FILES_READ)
        case HostOperation.FILES_WATCH:
            return cast(Operation[Message, Message], _FILES_WATCH)
        case HostOperation.FILES_DELETE:
            return cast(Operation[Message, Message], _FILES_DELETE)
        case HostOperation.FILES_CAPTURE:
            return cast(Operation[Message, Message], _FILES_CAPTURE)
        case HostOperation.PROMPTS_CREATE:
            return cast(Operation[Message, Message], _PROMPTS_CREATE)
        case HostOperation.SECRETS_GET:
            return cast(Operation[Message, Message], _SECRETS_GET)
        case HostOperation.INVOCATIONS_GET:
            return cast(Operation[Message, Message], _INVOCATIONS_GET)
        case HostOperation.INVOCATIONS_CANCEL:
            return cast(Operation[Message, Message], _INVOCATIONS_CANCEL)
        case HostOperation.INVOCATIONS_WATCH:
            return cast(Operation[Message, Message], _INVOCATIONS_WATCH)
        case HostOperation.OPERATIONS_LIST:
            return cast(Operation[Message, Message], _OPERATIONS_LIST)
        case HostOperation.OPERATIONS_GET:
            return cast(Operation[Message, Message], _OPERATIONS_GET)
        case HostOperation.PLUGINS_SETTINGS_LIST:
            return cast(Operation[Message, Message], _PLUGINS_SETTINGS_LIST)
        case HostOperation.PLUGINS_STATE_GET:
            return cast(Operation[Message, Message], _PLUGINS_STATE_GET)
        case HostOperation.PLUGINS_STATE_PUT:
            return cast(Operation[Message, Message], _PLUGINS_STATE_PUT)
        case HostOperation.TIME_GET:
            return cast(Operation[Message, Message], _TIME_GET)
        case HostOperation.TIME_SLEEP:
            return cast(Operation[Message, Message], _TIME_SLEEP)
    raise ValueError("Declare Host Requirement: Unknown Operation")


class HostRoot:
    def __init__(self, context: "Context") -> None:
        self.backend = HostBackendResource(context)
        self.content = HostContentResource(context)
        self.documents = HostDocumentsResource(context)
        self.effects = HostEffectsResource(context)
        self.evidence = HostEvidenceResource(context)
        self.executions = HostExecutionsResource(context)
        self.files = HostFilesResource(context)
        self.invocations = HostInvocationsResource(context)
        self.operations = HostOperationsResource(context)
        self.plugins = HostPluginsResource(context)
        self.prompts = HostPromptsResource(context)
        self.secrets = HostSecretsResource(context)
        self.time = HostTimeResource(context)


class HostBackendResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def environment(self, *, at: Position | None = None, policy: OperationPolicy | None = None) -> BackendEnvironmentResponse:
        request = BackendEnvironmentRequest(at=at)
        response = await self._context.call(_BACKEND_ENVIRONMENT, request, policy=policy)
        return response


class HostContentResource:
    def __init__(self, context: "Context") -> None:
        self._context = context
        self.artifacts = HostContentArtifactsResource(context)


class HostContentArtifactsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def create(
        self,
        file: FileReference,
        filename: str,
        content_type: str,
        *,
        scope: ArtifactScope | None = None,
        request_id: str | None = None,
        metadata: dokime_sdk._wire.model.v1.metadata_pb2.Metadata | None = None,
        policy: OperationPolicy | None = None,
    ) -> ArtifactsCreateResponse:
        request = ArtifactsCreateRequest(
            file=file,
            scope=scope,
            filename=filename,
            content_type=content_type,
            request_id=request_id if request_id is not None else str(uuid4()),
            metadata=metadata,
        )
        response = await self._context.call(_CONTENT_ARTIFACTS_CREATE, request, policy=policy)
        return response


class HostDocumentsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def append_parts(
        self,
        execution: str,
        document: str,
        section: str,
        *,
        subsection: Section | None = None,
        card: Card | None = None,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> Receipt:
        request = DocumentsAppendPartsRequest(
            execution=execution,
            document=document,
            section=section,
            subsection=subsection,
            card=card,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await self._context.call(_DOCUMENTS_APPEND_PARTS, request, policy=policy)
        if not response.HasField("receipt"):
            raise ValueError("Read DOCUMENTS_APPEND_PARTS: Missing Payload")
        return response.receipt


class HostEffectsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def create(
        self,
        resource: str,
        inverse_operation: str,
        inverse: bytes,
        description: str,
        *,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> EffectsCreateResponse:
        request = EffectsCreateRequest(
            resource=resource,
            inverse_operation=inverse_operation,
            inverse=inverse,
            description=description,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await self._context.call(_EFFECTS_CREATE, request, policy=policy)
        return response

    async def get(self, name: str, *, at: Position | None = None, policy: OperationPolicy | None = None) -> EffectsGetResponse:
        request = EffectsGetRequest(name=name, at=at)
        response = await self._context.call(_EFFECTS_GET, request, policy=policy)
        return response

    async def set(
        self,
        name: str,
        value: EffectWritable,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> EffectsSetResponse:
        request = EffectsSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await self._context.call(_EFFECTS_SET, request, policy=policy)
        return response


class HostEvidenceResource:
    def __init__(self, context: "Context") -> None:
        self._context = context
        self.logs = HostEvidenceLogsResource(context)
        self.records = HostEvidenceRecordsResource(context)
        self.traceability = HostEvidenceTraceabilityResource(context)

    async def statistics(
        self,
        stream: str,
        versions: list[int],
        between: Window,
        outputs: list[Output],
        field: str,
        step: str,
        *,
        scope: EvidenceScope | None = None,
        at: ReadCut | None = None,
        policy: OperationPolicy | None = None,
    ) -> EvidenceStatisticsResponse:
        request = EvidenceStatisticsRequest(
            scope=scope, stream=stream, versions=versions, between=between, outputs=outputs, field=field, step=step, at=at
        )
        response = await self._context.call(_EVIDENCE_STATISTICS, request, policy=policy)
        return response


class HostEvidenceLogsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def append(
        self,
        execution: str,
        severity: Severity,
        message: str,
        at: Timestamp,
        *,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> Receipt:
        request = LogsAppendRequest(
            execution=execution,
            severity=severity,
            message=message,
            at=at,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await self._context.call(_EVIDENCE_LOGS_APPEND, request, policy=policy)
        if not response.HasField("receipt"):
            raise ValueError("Read EVIDENCE_LOGS_APPEND: Missing Payload")
        return response.receipt


class HostEvidenceRecordsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def list(
        self,
        record_type: str,
        versions: list[int],
        between: Window,
        step: str,
        selection: CapturedRecordSelection,
        *,
        scope: RecordScope | None = None,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        policy: OperationPolicy | None = None,
    ) -> RecordsListResponse:
        request = RecordsListRequest(
            scope=scope, record_type=record_type, versions=versions, between=between, step=step, selection=selection, at=at, page=page
        )
        response = await self._context.call(_EVIDENCE_RECORDS_LIST, request, policy=policy)
        return response

    async def append(
        self,
        execution: str,
        kind: str,
        payload: bytes,
        source_at: Timestamp,
        *,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> RecordsAppendResponse:
        request = RecordsAppendRequest(
            execution=execution,
            kind=kind,
            payload=payload,
            source_at=source_at,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await self._context.call(_EVIDENCE_RECORDS_APPEND, request, policy=policy)
        return response


class HostEvidenceTraceabilityResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(
        self,
        *,
        scope: EvidenceScope | None = None,
        page: PageRequest | None = None,
        at: ReadCut | None = None,
        entry: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> TraceabilityGetResponse:
        request = TraceabilityGetRequest(scope=scope, page=page, at=at, entry=entry)
        response = await self._context.call(_EVIDENCE_TRACEABILITY_GET, request, policy=policy)
        return response

    async def set(
        self,
        value: TraceabilityEntry,
        expected_revision: bytes,
        entry: str,
        *,
        name: str | None = None,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> TraceabilitySetResponse:
        request = TraceabilitySetRequest(
            name=name,
            value=value,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
            entry=entry,
        )
        response = await self._context.call(_EVIDENCE_TRACEABILITY_SET, request, policy=policy)
        return response


class HostExecutionsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context
        self.progress = HostExecutionsProgressResource(context)


class HostExecutionsProgressResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(
        self, *, name: str | None = None, at: Position | None = None, policy: OperationPolicy | None = None
    ) -> ProgressGetResponse:
        request = ProgressGetRequest(name=name, at=at)
        response = await self._context.call(_EXECUTIONS_PROGRESS_GET, request, policy=policy)
        return response

    async def set(
        self,
        value: ExecutionProgressValue,
        expected_revision: bytes,
        *,
        name: str | None = None,
        request_id: str | None = None,
        policy: OperationPolicy | None = None,
    ) -> ProgressSetResponse:
        request = ProgressSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await self._context.call(_EXECUTIONS_PROGRESS_SET, request, policy=policy)
        return response


class HostFilesResource:
    def __init__(self, context: "Context") -> None:
        self._context = context
        self.captures = HostFilesCapturesResource(context)
        self.uploads = HostFilesUploadsResource(context)

    async def get(self, file: FileReference, *, at: ReadCut | None = None, policy: OperationPolicy | None = None) -> FilesGetResponse:
        request = FilesGetRequest(file=file, at=at)
        response = await self._context.call(_FILES_GET, request, policy=policy)
        return response

    async def read(
        self, file: FileReference, offset: int, max_bytes: int, *, at: ReadCut | None = None, policy: OperationPolicy | None = None
    ) -> FilesReadResponse:
        request = FilesReadRequest(file=file, offset=offset, max_bytes=max_bytes, at=at)
        response = await self._context.call(_FILES_READ, request, policy=policy)
        return response

    async def watch(
        self,
        offset: int,
        *,
        file: FileReference | None = None,
        upload: Upload | None = None,
        at: ReadCut | None = None,
        max_item_bytes: int,
    ) -> AsyncGenerator[FilesWatchResponse]:
        request = FilesWatchRequest(file=file, upload=upload, offset=offset, at=at)
        call = self._context.watch(_FILES_WATCH, request, max_item_bytes=max_item_bytes)
        try:
            async for response in call:
                yield response
        finally:
            await call.aclose()

    async def delete(self, file: FileReference, *, request_id: str | None = None, policy: OperationPolicy | None = None) -> Receipt:
        request = FilesDeleteRequest(file=file, request_id=request_id if request_id is not None else str(uuid4()))
        response = await self._context.call(_FILES_DELETE, request, policy=policy)
        if not response.HasField("receipt"):
            raise ValueError("Read FILES_DELETE: Missing Payload")
        return response.receipt

    async def capture(self, source: FileSource, output: str, *, policy: OperationPolicy | None = None) -> FilesCaptureResponse:
        request = FilesCaptureRequest(source=source, output=output)
        response = await self._context.call(_FILES_CAPTURE, request, policy=policy)
        return response


class HostFilesCapturesResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def start(self, source: FileSource, output: str, mode: CaptureMode, *, policy: OperationPolicy | None = None) -> int:
        request = CapturesStartRequest(source=source, output=output, mode=mode)
        response = await self._context.call(_FILES_CAPTURES_START, request, policy=policy)
        return response.capture_id

    async def stop(self, capture_id: int, *, policy: OperationPolicy | None = None) -> CapturesStopResponse:
        request = CapturesStopRequest(capture_id=capture_id)
        response = await self._context.call(_FILES_CAPTURES_STOP, request, policy=policy)
        return response


class HostFilesUploadsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def create(
        self, metadata: FileMetadata, *, request_id: str | None = None, policy: OperationPolicy | None = None
    ) -> UploadsCreateResponse:
        request = UploadsCreateRequest(metadata=metadata, request_id=request_id if request_id is not None else str(uuid4()))
        response = await self._context.call(_FILES_UPLOADS_CREATE, request, policy=policy)
        return response

    async def get(self, upload: Upload, *, at: ReadCut | None = None, policy: OperationPolicy | None = None) -> UploadsGetResponse:
        request = UploadsGetRequest(upload=upload, at=at)
        response = await self._context.call(_FILES_UPLOADS_GET, request, policy=policy)
        return response

    async def resume(
        self, upload: Upload, *, request_id: str | None = None, policy: OperationPolicy | None = None
    ) -> UploadsResumeResponse:
        request = UploadsResumeRequest(upload=upload, request_id=request_id if request_id is not None else str(uuid4()))
        response = await self._context.call(_FILES_UPLOADS_RESUME, request, policy=policy)
        return response

    async def append(
        self, upload: Upload, offset: int, data: bytes, *, request_id: str | None = None, policy: OperationPolicy | None = None
    ) -> UploadsAppendResponse:
        request = UploadsAppendRequest(
            upload=upload, offset=offset, data=data, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await self._context.call(_FILES_UPLOADS_APPEND, request, policy=policy)
        return response

    async def publish(
        self, upload: Upload, length: int, sha256: bytes, *, request_id: str | None = None, policy: OperationPolicy | None = None
    ) -> UploadsPublishResponse:
        request = UploadsPublishRequest(
            upload=upload, length=length, sha256=sha256, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await self._context.call(_FILES_UPLOADS_PUBLISH, request, policy=policy)
        return response

    async def delete(
        self, upload: Upload, *, request_id: str | None = None, policy: OperationPolicy | None = None
    ) -> UploadsDeleteResponse:
        request = UploadsDeleteRequest(upload=upload, request_id=request_id if request_id is not None else str(uuid4()))
        response = await self._context.call(_FILES_UPLOADS_DELETE, request, policy=policy)
        return response


class HostInvocationsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(self, instance: int, *, policy: OperationPolicy | None = None) -> InstanceState:
        request = InvocationsGetRequest(instance=instance)
        response = await self._context.call(_INVOCATIONS_GET, request, policy=policy)
        if not response.HasField("invocation"):
            raise ValueError("Read INVOCATIONS_GET: Missing Payload")
        return response.invocation

    async def cancel(self, instance: int, reason: str, *, policy: OperationPolicy | None = None) -> InstanceState:
        request = InvocationsCancelRequest(instance=instance, reason=reason)
        response = await self._context.call(_INVOCATIONS_CANCEL, request, policy=policy)
        if not response.HasField("invocation"):
            raise ValueError("Read INVOCATIONS_CANCEL: Missing Payload")
        return response.invocation

    async def watch(self, instance: int, *, max_item_bytes: int) -> AsyncGenerator[InstanceState]:
        request = InvocationsWatchRequest(instance=instance)
        call = self._context.watch(_INVOCATIONS_WATCH, request, max_item_bytes=max_item_bytes)
        try:
            async for response in call:
                if not response.HasField("invocation"):
                    raise ValueError("Read INVOCATIONS_WATCH: Missing Payload")
                yield response.invocation
        finally:
            await call.aclose()


class HostOperationsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def list(self, *, policy: OperationPolicy | None = None) -> list[DiscoveredOperation]:
        request = OperationsListRequest()
        response = await self._context.call(_OPERATIONS_LIST, request, policy=policy)
        return list(response.operations)

    async def get(self, id: str, *, policy: OperationPolicy | None = None) -> DiscoveredOperation:
        request = OperationsGetRequest(id=id)
        response = await self._context.call(_OPERATIONS_GET, request, policy=policy)
        if not response.HasField("operation"):
            raise ValueError("Read OPERATIONS_GET: Missing Payload")
        return response.operation


class HostPluginsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context
        self.settings = HostPluginsSettingsResource(context)
        self.state = HostPluginsStateResource(context)


class HostPluginsSettingsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def list(
        self, *, name: str | None = None, at: ReadCut | None = None, policy: OperationPolicy | None = None
    ) -> SettingsListResponse:
        request = SettingsListRequest(name=name, at=at)
        response = await self._context.call(_PLUGINS_SETTINGS_LIST, request, policy=policy)
        return response


class HostPluginsStateResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(self, *, policy: OperationPolicy | None = None) -> StateGetResponse:
        request = StateGetRequest()
        response = await self._context.call(_PLUGINS_STATE_GET, request, policy=policy)
        return response

    async def put(self, expected_revision: int, value: bytes, *, policy: OperationPolicy | None = None) -> int:
        request = StatePutRequest(expected_revision=expected_revision, value=value)
        response = await self._context.call(_PLUGINS_STATE_PUT, request, policy=policy)
        return response.revision


class HostPromptsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def create(
        self,
        prompt: str,
        prompt_type: str,
        values: dict[str, Value],
        *,
        definition: Prompt | None = None,
        policy: OperationPolicy | None = None,
    ) -> PromptsCreateResponse:
        request = PromptsCreateRequest(prompt=prompt, prompt_type=prompt_type, values=values, definition=definition)
        response = await self._context.call(_PROMPTS_CREATE, request, policy=policy)
        return response


class HostSecretsResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(self, name: str, *, at: ReadCut | None = None, policy: OperationPolicy | None = None) -> SecretsGetResponse:
        request = SecretsGetRequest(name=name, at=at)
        response = await self._context.call(_SECRETS_GET, request, policy=policy)
        return response


class HostTimeResource:
    def __init__(self, context: "Context") -> None:
        self._context = context

    async def get(self, *, policy: OperationPolicy | None = None) -> Timestamp:
        request = TimeGetRequest()
        response = await self._context.call(_TIME_GET, request, policy=policy)
        if not response.HasField("now"):
            raise ValueError("Read TIME_GET: Missing Payload")
        return response.now

    async def sleep(self, duration: Duration, *, policy: OperationPolicy | None = None) -> TimeSleepResponse:
        request = TimeSleepRequest(duration=duration)
        response = await self._context.call(_TIME_SLEEP, request, policy=policy)
        return response
