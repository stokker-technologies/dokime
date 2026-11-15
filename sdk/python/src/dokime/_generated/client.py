# @generated from protocol descriptors; do not edit.
"""Typed Client Resources."""

from collections.abc import AsyncGenerator, Sequence
from uuid import uuid4

from google.protobuf.duration_pb2 import Duration
from google.protobuf.timestamp_pb2 import Timestamp
from grpc import CallCredentials
from grpc.aio import AioRpcError, Channel
from harmos._client_transport import Metadata as CallMetadata
from harmos._client_transport import call_error, stream, unary
from harmos._connection import Connection as LibraryConnection
from harmos._wire.host.v1.access.certificates_pb2 import CertificatesCreateRequest as AccessCertificatesCreateRequest
from harmos._wire.host.v1.access.certificates_pb2 import CertificatesCreateResponse as AccessCertificatesCreateResponse
from harmos._wire.host.v1.access.listener_pb2 import ListenerGetRequest as AccessListenerGetRequest
from harmos._wire.host.v1.access.listener_pb2 import ListenerGetResponse as AccessListenerGetResponse
from harmos._wire.host.v1.access.listener_pb2 import ListenerSetRequest as AccessListenerSetRequest
from harmos._wire.host.v1.access.listener_pb2 import ListenerSetResponse as AccessListenerSetResponse
from harmos._wire.host.v1.access.listener_pb2 import ListenerValue as AccessListenerValue
from harmos._wire.host.v1.access.pairings_pb2 import PairingsCreateRequest as AccessPairingsCreateRequest
from harmos._wire.host.v1.access.pairings_pb2 import PairingsCreateResponse as AccessPairingsCreateResponse
from harmos._wire.host.v1.access.pairings_pb2 import PairingsDeleteRequest as AccessPairingsDeleteRequest
from harmos._wire.host.v1.access.pairings_pb2 import PairingsDeleteResponse as AccessPairingsDeleteResponse
from harmos._wire.host.v1.access.pairings_pb2 import PairingsListRequest as AccessPairingsListRequest
from harmos._wire.host.v1.access.pairings_pb2 import PairingsListResponse as AccessPairingsListResponse
from harmos._wire.host.v1.invocations_pb2 import (
    InvocationsCancelRequest,
    InvocationsCancelResponse,
    InvocationsCreateRequest,
    InvocationsCreateResponse,
    InvocationsGetRequest,
    InvocationsGetResponse,
    InvocationsWatchRequest,
    InvocationsWatchResponse,
)
from harmos._wire.host.v1.operations_pb2 import OperationsGetRequest, OperationsGetResponse, OperationsListRequest, OperationsListResponse
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListRequest as PluginsSettingsListRequest
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListResponse as PluginsSettingsListResponse
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsSetRequest as PluginsSettingsSetRequest
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsSetResponse as PluginsSettingsSetResponse
from harmos._wire.host.v1.plugins_pb2 import (
    PluginEntry,
    PluginPlacement,
    PluginsCreateRequest,
    PluginsCreateResponse,
    PluginsDeleteRequest,
    PluginsDeleteResponse,
    PluginsGetRequest,
    PluginsGetResponse,
    PluginsListRequest,
    PluginsListResponse,
    PluginsReplaceRequest,
    PluginsReplaceResponse,
    PluginsRestartRequest,
    PluginsRestartResponse,
    PluginsSetRequest,
    PluginsSetResponse,
    PluginsWatchRequest,
    PluginsWatchResponse,
    PluginValue,
)
from harmos._wire.host.v1.schemas_pb2 import SchemaDescription, SchemasListRequest, SchemasListResponse
from harmos._wire.host.v1.status.diagnostics_pb2 import DiagnosticsGetRequest as StatusDiagnosticsGetRequest
from harmos._wire.host.v1.status.diagnostics_pb2 import DiagnosticsGetResponse as StatusDiagnosticsGetResponse
from harmos._wire.host.v1.status.logs_pb2 import LogsListRequest as StatusLogsListRequest
from harmos._wire.host.v1.status.logs_pb2 import LogsListResponse as StatusLogsListResponse
from harmos._wire.host.v1.status.logs_pb2 import LogsWatchRequest as StatusLogsWatchRequest
from harmos._wire.host.v1.status.logs_pb2 import LogsWatchResponse as StatusLogsWatchResponse
from harmos._wire.host.v1.status.trace_events_pb2 import TraceEventsListRequest as StatusTraceEventsListRequest
from harmos._wire.host.v1.status.trace_events_pb2 import TraceEventsListResponse as StatusTraceEventsListResponse
from harmos._wire.host.v1.status.trace_events_pb2 import TraceEventsWatchRequest as StatusTraceEventsWatchRequest
from harmos._wire.host.v1.status.trace_events_pb2 import TraceEventsWatchResponse as StatusTraceEventsWatchResponse
from harmos._wire.host.v1.status_pb2 import StatusGetRequest, StatusGetResponse, StatusWatchRequest, StatusWatchResponse
from harmos._wire.host.v1.streams_pb2 import StreamIdentity, StreamsListRequest, StreamsListResponse
from harmos._wire.host.v1.time_pb2 import TimeGetRequest, TimeGetResponse, TimeSleepRequest, TimeSleepResponse
from harmos._wire.model.v1.file_pb2 import FileMetadata, FileReference, Upload
from harmos._wire.model.v1.invocation_pb2 import InstanceState
from harmos._wire.model.v1.operation_pb2 import DiscoveredOperation
from harmos._wire.model.v1.page_pb2 import PageRequest
from harmos._wire.model.v1.position_pb2 import Position, ReadCut
from harmos._wire.model.v1.receipt_pb2 import Receipt
from harmos._wire.model.v1.setting_pb2 import SettingValues

from dokime._wire.host.v1.aggregates_pb2 import AggregatesGetRequest, AggregatesGetResponse, QueryAggregate, QueryFilter, QuerySource
from dokime._wire.host.v1.analyses_pb2 import AnalysesListRequest, AnalysesListResponse
from dokime._wire.host.v1.backend.overview_pb2 import OverviewGetRequest as BackendOverviewGetRequest
from dokime._wire.host.v1.backend.overview_pb2 import OverviewGetResponse as BackendOverviewGetResponse
from dokime._wire.host.v1.backend.overview_pb2 import OverviewWatchRequest as BackendOverviewWatchRequest
from dokime._wire.host.v1.backend.overview_pb2 import OverviewWatchResponse as BackendOverviewWatchResponse
from dokime._wire.host.v1.backend_pb2 import BackendAppearanceRequest, BackendAppearanceResponse
from dokime._wire.host.v1.cases_pb2 import (
    CasesCreateRequest,
    CasesCreateResponse,
    CasesDeleteRequest,
    CasesDeleteResponse,
    CasesGetRequest,
    CasesGetResponse,
    CasesListRequest,
    CasesListResponse,
    CasesParametersRequest,
    CasesParametersResponse,
    CasesWatchRequest,
    CasesWatchResponse,
    SelectionPlan,
)
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateRequest as ContentArtifactsCreateRequest
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse as ContentArtifactsCreateResponse
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsGetRequest as ContentArtifactsGetRequest
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsGetResponse as ContentArtifactsGetResponse
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsListRequest as ContentArtifactsListRequest
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsListResponse as ContentArtifactsListResponse
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsWatchRequest as ContentArtifactsWatchRequest
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsWatchResponse as ContentArtifactsWatchResponse
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesGetRequest as ContentExecutionFilesGetRequest
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesGetResponse as ContentExecutionFilesGetResponse
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesListRequest as ContentExecutionFilesListRequest
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesListResponse as ContentExecutionFilesListResponse
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesWatchRequest as ContentExecutionFilesWatchRequest
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesWatchResponse as ContentExecutionFilesWatchResponse
from dokime._wire.host.v1.cycles_pb2 import (
    CycleFilter,
    CycleOrder,
    CyclesCreateRequest,
    CyclesCreateResponse,
    CycleSelection,
    CyclesGetRequest,
    CyclesGetResponse,
    CyclesListRequest,
    CyclesListResponse,
    CyclesSealRequest,
    CyclesSealResponse,
    CyclesSetRequest,
    CyclesSetResponse,
    CyclesWatchRequest,
    CyclesWatchResponse,
    CycleWritable,
)
from dokime._wire.host.v1.documents_pb2 import (
    DocumentScope,
    DocumentsGetRequest,
    DocumentsGetResponse,
    DocumentsWatchRequest,
    DocumentsWatchResponse,
)
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsListRequest as EvidenceHealthEventsListRequest
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsListResponse as EvidenceHealthEventsListResponse
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsWatchRequest as EvidenceHealthEventsWatchRequest
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsWatchResponse as EvidenceHealthEventsWatchResponse
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationOwner as EvidenceAnnotationOwner
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationSelection as EvidenceAnnotationSelection
from dokime._wire.host.v1.evidence.notes_pb2 import NotesCreateRequest as EvidenceNotesCreateRequest
from dokime._wire.host.v1.evidence.notes_pb2 import NotesCreateResponse as EvidenceNotesCreateResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesGetRequest as EvidenceNotesGetRequest
from dokime._wire.host.v1.evidence.notes_pb2 import NotesGetResponse as EvidenceNotesGetResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesListRequest as EvidenceNotesListRequest
from dokime._wire.host.v1.evidence.notes_pb2 import NotesListResponse as EvidenceNotesListResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesSetRequest as EvidenceNotesSetRequest
from dokime._wire.host.v1.evidence.notes_pb2 import NotesSetResponse as EvidenceNotesSetResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesWatchRequest as EvidenceNotesWatchRequest
from dokime._wire.host.v1.evidence.notes_pb2 import NotesWatchResponse as EvidenceNotesWatchResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NoteWritable as EvidenceNoteWritable
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecordSelection as EvidenceCapturedRecordSelection
from dokime._wire.host.v1.evidence.records_pb2 import RecordScope as EvidenceRecordScope
from dokime._wire.host.v1.evidence.records_pb2 import RecordsListRequest as EvidenceRecordsListRequest
from dokime._wire.host.v1.evidence.records_pb2 import RecordsListResponse as EvidenceRecordsListResponse
from dokime._wire.host.v1.evidence.records_pb2 import RecordsWatchRequest as EvidenceRecordsWatchRequest
from dokime._wire.host.v1.evidence.records_pb2 import RecordsWatchResponse as EvidenceRecordsWatchResponse
from dokime._wire.host.v1.evidence.samples_pb2 import BucketRequest as EvidenceBucketRequest
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesListRequest as EvidenceSamplesListRequest
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesListResponse as EvidenceSamplesListResponse
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesWatchRequest as EvidenceSamplesWatchRequest
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesWatchResponse as EvidenceSamplesWatchResponse
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityGetRequest as EvidenceTraceabilityGetRequest
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityGetResponse as EvidenceTraceabilityGetResponse
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchRequest as EvidenceTraceabilityWatchRequest
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchResponse as EvidenceTraceabilityWatchResponse
from dokime._wire.host.v1.evidence_pb2 import (
    EvidenceArchiveRequest,
    EvidenceArchiveResponse,
    EvidenceStatisticsRequest,
    EvidenceStatisticsResponse,
    EvidenceSubject,
    Output,
)
from dokime._wire.host.v1.executions.commands_pb2 import CommandsCreateRequest as ExecutionsCommandsCreateRequest
from dokime._wire.host.v1.executions.commands_pb2 import CommandsCreateResponse as ExecutionsCommandsCreateResponse
from dokime._wire.host.v1.executions.commands_pb2 import CommandsGetRequest as ExecutionsCommandsGetRequest
from dokime._wire.host.v1.executions.commands_pb2 import CommandsGetResponse as ExecutionsCommandsGetResponse
from dokime._wire.host.v1.executions.commands_pb2 import ExecutionCommandAction as ExecutionsExecutionCommandAction
from dokime._wire.host.v1.executions.progress_pb2 import ProgressGetRequest as ExecutionsProgressGetRequest
from dokime._wire.host.v1.executions.progress_pb2 import ProgressGetResponse as ExecutionsProgressGetResponse
from dokime._wire.host.v1.executions.progress_pb2 import ProgressWatchRequest as ExecutionsProgressWatchRequest
from dokime._wire.host.v1.executions.progress_pb2 import ProgressWatchResponse as ExecutionsProgressWatchResponse
from dokime._wire.host.v1.executions.schemas_pb2 import SchemasGetRequest as ExecutionsSchemasGetRequest
from dokime._wire.host.v1.executions.schemas_pb2 import SchemasGetResponse as ExecutionsSchemasGetResponse
from dokime._wire.host.v1.executions_pb2 import (
    ExecutionBinding,
    ExecutionFilter,
    ExecutionMode,
    ExecutionOrder,
    ExecutionParameterSet,
    ExecutionsCancelRequest,
    ExecutionsCancelResponse,
    ExecutionsCreateRequest,
    ExecutionsCreateResponse,
    ExecutionsDeleteRequest,
    ExecutionsDeleteResponse,
    ExecutionSelection,
    ExecutionsGetRequest,
    ExecutionsGetResponse,
    ExecutionsListRequest,
    ExecutionsListResponse,
    ExecutionsParametersRequest,
    ExecutionsParametersResponse,
    ExecutionsSetRequest,
    ExecutionsSetResponse,
    ExecutionsStartRequest,
    ExecutionsStartResponse,
    ExecutionsStepsRequest,
    ExecutionsStepsResponse,
    ExecutionsStopRequest,
    ExecutionsStopResponse,
    ExecutionsWatchRequest,
    ExecutionsWatchResponse,
    ExecutionWritable,
)
from dokime._wire.host.v1.files.uploads_pb2 import UploadsAppendRequest as FilesUploadsAppendRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsAppendResponse as FilesUploadsAppendResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsCreateRequest as FilesUploadsCreateRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsCreateResponse as FilesUploadsCreateResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsDeleteRequest as FilesUploadsDeleteRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsDeleteResponse as FilesUploadsDeleteResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsGetRequest as FilesUploadsGetRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsGetResponse as FilesUploadsGetResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsPublishRequest as FilesUploadsPublishRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsPublishResponse as FilesUploadsPublishResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsResumeRequest as FilesUploadsResumeRequest
from dokime._wire.host.v1.files.uploads_pb2 import UploadsResumeResponse as FilesUploadsResumeResponse
from dokime._wire.host.v1.files_pb2 import (
    FilesDeleteRequest,
    FilesDeleteResponse,
    FilesGetRequest,
    FilesGetResponse,
    FilesReadRequest,
    FilesReadResponse,
    FilesWatchRequest,
    FilesWatchResponse,
)
from dokime._wire.host.v1.media.captures_pb2 import CaptureKind as MediaCaptureKind
from dokime._wire.host.v1.media.captures_pb2 import CapturesCreateRequest as MediaCapturesCreateRequest
from dokime._wire.host.v1.media.captures_pb2 import CapturesCreateResponse as MediaCapturesCreateResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesGetRequest as MediaCapturesGetRequest
from dokime._wire.host.v1.media.captures_pb2 import CapturesGetResponse as MediaCapturesGetResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesStopRequest as MediaCapturesStopRequest
from dokime._wire.host.v1.media.captures_pb2 import CapturesStopResponse as MediaCapturesStopResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesWatchRequest as MediaCapturesWatchRequest
from dokime._wire.host.v1.media.captures_pb2 import CapturesWatchResponse as MediaCapturesWatchResponse
from dokime._wire.host.v1.media.captures_pb2 import RecordingAudio as MediaRecordingAudio
from dokime._wire.host.v1.plots.settings_pb2 import PlotSettingsValue as PlotsPlotSettingsValue
from dokime._wire.host.v1.plots.settings_pb2 import SettingsGetRequest as PlotsSettingsGetRequest
from dokime._wire.host.v1.plots.settings_pb2 import SettingsGetResponse as PlotsSettingsGetResponse
from dokime._wire.host.v1.plots.settings_pb2 import SettingsSetRequest as PlotsSettingsSetRequest
from dokime._wire.host.v1.plots.settings_pb2 import SettingsSetResponse as PlotsSettingsSetResponse
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotRequest as PlotsPlotRequest
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsListRequest as PlotsSnapshotsListRequest
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsListResponse as PlotsSnapshotsListResponse
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsWatchRequest as PlotsSnapshotsWatchRequest
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsWatchResponse as PlotsSnapshotsWatchResponse
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretBindingValue as PluginsSecretBindingValue
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsDeleteRequest as PluginsSecretsDeleteRequest
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsDeleteResponse as PluginsSecretsDeleteResponse
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsListRequest as PluginsSecretsListRequest
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsListResponse as PluginsSecretsListResponse
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsSetRequest as PluginsSecretsSetRequest
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsSetResponse as PluginsSecretsSetResponse
from dokime._wire.host.v1.prompt_definitions_pb2 import (
    PromptDefinitionsGetRequest,
    PromptDefinitionsGetResponse,
    PromptDefinitionsListRequest,
    PromptDefinitionsListResponse,
)
from dokime._wire.host.v1.prompts.answers_pb2 import AnswersCreateRequest as PromptsAnswersCreateRequest
from dokime._wire.host.v1.prompts.answers_pb2 import AnswersCreateResponse as PromptsAnswersCreateResponse
from dokime._wire.host.v1.prompts_pb2 import (
    PromptsGetRequest,
    PromptsGetResponse,
    PromptsListRequest,
    PromptsListResponse,
    PromptsWatchRequest,
    PromptsWatchResponse,
)
from dokime._wire.host.v1.record_schemas_pb2 import (
    RecordSchemasGetRequest,
    RecordSchemasGetResponse,
    RecordSchemasListRequest,
    RecordSchemasListResponse,
)
from dokime._wire.host.v1.reports_pb2 import (
    ReportsCreateRequest,
    ReportsCreateResponse,
    ReportsGetRequest,
    ReportsGetResponse,
    ReportsPublishRequest,
    ReportsPublishResponse,
    ReportSubject,
)
from dokime._wire.host.v1.secrets_pb2 import (
    SecretImportChoice,
    SecretsDeleteRequest,
    SecretsDeleteResponse,
    SecretsExportRequest,
    SecretsExportResponse,
    SecretsGetRequest,
    SecretsGetResponse,
    SecretsImportRequest,
    SecretsImportResponse,
    SecretsListRequest,
    SecretsListResponse,
    SecretsSetRequest,
    SecretsSetResponse,
    SecretsWatchRequest,
    SecretsWatchResponse,
    SecretWritable,
)
from dokime._wire.host.v1.settings_pb2 import (
    ApplicationSettings,
    SettingsGetRequest,
    SettingsGetResponse,
    SettingsSchemaRequest,
    SettingsSchemaResponse,
    SettingsSetRequest,
    SettingsSetResponse,
)
from dokime._wire.host.v1.storage.sources_pb2 import SourcesGetRequest as StorageSourcesGetRequest
from dokime._wire.host.v1.storage.sources_pb2 import SourcesGetResponse as StorageSourcesGetResponse
from dokime._wire.host.v1.storage.sources_pb2 import SourcesSetRequest as StorageSourcesSetRequest
from dokime._wire.host.v1.storage.sources_pb2 import SourcesSetResponse as StorageSourcesSetResponse
from dokime._wire.host.v1.storage.sources_pb2 import StorageSources as StorageStorageSources
from dokime._wire.host.v1.stream_schemas_pb2 import (
    StreamSchemasGetRequest,
    StreamSchemasGetResponse,
    StreamSchemasListRequest,
    StreamSchemasListResponse,
)
from dokime._wire.host.v1.suites_pb2 import SuitesGetRequest, SuitesGetResponse, SuitesListRequest, SuitesListResponse
from dokime._wire.model.v1.evidence_pb2 import ArtifactScope, EvidenceScope, Window
from dokime._wire.model.v1.execution_pb2 import Origin
from dokime._wire.model.v1.metadata_pb2 import Metadata
from dokime._wire.model.v1.scalar_pb2 import MapValue


class AccessCertificatesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self, *, call_metadata: CallMetadata | None = None, timeout: float | None = None, credentials: CallCredentials | None = None
    ) -> str:
        request = AccessCertificatesCreateRequest()
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Certificates/Create",
            request,
            AccessCertificatesCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.certificates.create",
        )
        return response.certificate_pem


class AccessListenerResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self, *, call_metadata: CallMetadata | None = None, timeout: float | None = None, credentials: CallCredentials | None = None
    ) -> AccessListenerGetResponse:
        request = AccessListenerGetRequest()
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Listener/Get",
            request,
            AccessListenerGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.listener.get",
        )
        return response

    async def set(
        self,
        value: AccessListenerValue,
        expected_revision: bytes,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AccessListenerSetResponse:
        request = AccessListenerSetRequest(value=value, expected_revision=expected_revision)
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Listener/Set",
            request,
            AccessListenerSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.listener.set",
        )
        return response


class AccessPairingsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        label: str,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AccessPairingsCreateResponse:
        request = AccessPairingsCreateRequest(label=label)
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Pairings/Create",
            request,
            AccessPairingsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.pairings.create",
        )
        return response

    async def list(
        self,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AccessPairingsListResponse:
        request = AccessPairingsListRequest(at=at)
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Pairings/List",
            request,
            AccessPairingsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.pairings.list",
        )
        return response

    async def delete(
        self,
        name: str,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AccessPairingsDeleteResponse:
        request = AccessPairingsDeleteRequest(name=name)
        response = await unary(
            self._channel,
            "/harmos.host.v1.access.Pairings/Delete",
            request,
            AccessPairingsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="access.pairings.delete",
        )
        return response


class BackendOverviewResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> BackendOverviewGetResponse:
        request = BackendOverviewGetRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.backend.Overview/Get",
            request,
            BackendOverviewGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="backend.overview.get",
        )
        return response

    async def watch(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[BackendOverviewWatchResponse]:
        request = BackendOverviewWatchRequest(at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.backend.Overview/Watch",
            request,
            BackendOverviewWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("backend.overview.watch", error) from error
        finally:
            call.cancel()


class ContentArtifactsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ContentArtifactsGetResponse:
        request = ContentArtifactsGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.content.Artifacts/Get",
            request,
            ContentArtifactsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="content.artifacts.get",
        )
        return response

    async def create(
        self,
        file: FileReference,
        scope: ArtifactScope,
        filename: str,
        content_type: str,
        *,
        request_id: str | None = None,
        metadata: Metadata | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ContentArtifactsCreateResponse:
        request = ContentArtifactsCreateRequest(
            file=file,
            scope=scope,
            filename=filename,
            content_type=content_type,
            request_id=request_id if request_id is not None else str(uuid4()),
            metadata=metadata,
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.content.Artifacts/Create",
            request,
            ContentArtifactsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="content.artifacts.create",
        )
        return response

    async def list(
        self,
        scope: ArtifactScope,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ContentArtifactsListResponse:
        request = ContentArtifactsListRequest(scope=scope, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.content.Artifacts/List",
            request,
            ContentArtifactsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="content.artifacts.list",
        )
        return response

    async def watch(
        self,
        scope: ArtifactScope,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[ContentArtifactsWatchResponse]:
        request = ContentArtifactsWatchRequest(scope=scope, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.content.Artifacts/Watch",
            request,
            ContentArtifactsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("content.artifacts.watch", error) from error
        finally:
            call.cancel()


class ContentExecutionFilesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        parent: str,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ContentExecutionFilesGetResponse:
        request = ContentExecutionFilesGetRequest(parent=parent, name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.content.ExecutionFiles/Get",
            request,
            ContentExecutionFilesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="content.execution_files.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ContentExecutionFilesListResponse:
        request = ContentExecutionFilesListRequest(parent=parent, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.content.ExecutionFiles/List",
            request,
            ContentExecutionFilesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="content.execution_files.list",
        )
        return response

    async def watch(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[ContentExecutionFilesWatchResponse]:
        request = ContentExecutionFilesWatchRequest(parent=parent, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.content.ExecutionFiles/Watch",
            request,
            ContentExecutionFilesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("content.execution_files.watch", error) from error
        finally:
            call.cancel()


class EvidenceHealthEventsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        selection: EvidenceAnnotationSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceHealthEventsListResponse:
        request = EvidenceHealthEventsListRequest(selection=selection, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.HealthEvents/List",
            request,
            EvidenceHealthEventsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.health_events.list",
        )
        return response

    async def watch(
        self,
        selection: EvidenceAnnotationSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[EvidenceHealthEventsWatchResponse]:
        request = EvidenceHealthEventsWatchRequest(selection=selection, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.evidence.HealthEvents/Watch",
            request,
            EvidenceHealthEventsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("evidence.health_events.watch", error) from error
        finally:
            call.cancel()


class EvidenceNotesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceNotesGetResponse:
        request = EvidenceNotesGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Notes/Get",
            request,
            EvidenceNotesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.notes.get",
        )
        return response

    async def set(
        self,
        name: str,
        value: EvidenceNoteWritable,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceNotesSetResponse:
        request = EvidenceNotesSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Notes/Set",
            request,
            EvidenceNotesSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.notes.set",
        )
        return response

    async def list(
        self,
        selection: EvidenceAnnotationSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceNotesListResponse:
        request = EvidenceNotesListRequest(selection=selection, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Notes/List",
            request,
            EvidenceNotesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.notes.list",
        )
        return response

    async def watch(
        self,
        selection: EvidenceAnnotationSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[EvidenceNotesWatchResponse]:
        request = EvidenceNotesWatchRequest(selection=selection, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.evidence.Notes/Watch",
            request,
            EvidenceNotesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("evidence.notes.watch", error) from error
        finally:
            call.cancel()

    async def create(
        self,
        owner: EvidenceAnnotationOwner,
        value: EvidenceNoteWritable,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceNotesCreateResponse:
        request = EvidenceNotesCreateRequest(owner=owner, value=value, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Notes/Create",
            request,
            EvidenceNotesCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.notes.create",
        )
        return response


class EvidenceRecordsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        scope: EvidenceRecordScope,
        record_type: str,
        versions: Sequence[int],
        between: Window,
        step: str,
        selection: EvidenceCapturedRecordSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceRecordsListResponse:
        request = EvidenceRecordsListRequest(
            scope=scope, record_type=record_type, versions=versions, between=between, step=step, selection=selection, at=at, page=page
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Records/List",
            request,
            EvidenceRecordsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.records.list",
        )
        return response

    async def watch(
        self,
        scope: EvidenceRecordScope,
        record_type: str,
        versions: Sequence[int],
        between: Window,
        step: str,
        selection: EvidenceCapturedRecordSelection,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[EvidenceRecordsWatchResponse]:
        request = EvidenceRecordsWatchRequest(
            scope=scope, record_type=record_type, versions=versions, between=between, step=step, selection=selection, at=at, page=page
        )
        call = stream(
            self._channel,
            "/dokime.host.v1.evidence.Records/Watch",
            request,
            EvidenceRecordsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("evidence.records.watch", error) from error
        finally:
            call.cancel()


class EvidenceSamplesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        scope: EvidenceScope,
        streams: Sequence[str],
        versions: Sequence[int],
        fields: Sequence[str],
        sources: Sequence[str],
        signals: Sequence[str],
        between: Window,
        bucket: EvidenceBucketRequest,
        latest: bool,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceSamplesListResponse:
        request = EvidenceSamplesListRequest(
            scope=scope,
            streams=streams,
            versions=versions,
            fields=fields,
            sources=sources,
            signals=signals,
            between=between,
            bucket=bucket,
            latest=latest,
            at=at,
            page=page,
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Samples/List",
            request,
            EvidenceSamplesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.samples.list",
        )
        return response

    async def watch(
        self,
        scope: EvidenceScope,
        streams: Sequence[str],
        versions: Sequence[int],
        fields: Sequence[str],
        sources: Sequence[str],
        signals: Sequence[str],
        between: Window,
        bucket: EvidenceBucketRequest,
        latest: bool,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[EvidenceSamplesWatchResponse]:
        request = EvidenceSamplesWatchRequest(
            scope=scope,
            streams=streams,
            versions=versions,
            fields=fields,
            sources=sources,
            signals=signals,
            between=between,
            bucket=bucket,
            latest=latest,
            at=at,
            page=page,
        )
        call = stream(
            self._channel,
            "/dokime.host.v1.evidence.Samples/Watch",
            request,
            EvidenceSamplesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("evidence.samples.watch", error) from error
        finally:
            call.cancel()


class EvidenceTraceabilityResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        scope: EvidenceScope,
        *,
        page: PageRequest | None = None,
        at: ReadCut | None = None,
        entry: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceTraceabilityGetResponse:
        request = EvidenceTraceabilityGetRequest(scope=scope, page=page, at=at, entry=entry)
        response = await unary(
            self._channel,
            "/dokime.host.v1.evidence.Traceability/Get",
            request,
            EvidenceTraceabilityGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.traceability.get",
        )
        return response

    async def watch(
        self,
        scope: EvidenceScope,
        *,
        page: PageRequest | None = None,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[EvidenceTraceabilityWatchResponse]:
        request = EvidenceTraceabilityWatchRequest(scope=scope, page=page, at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.evidence.Traceability/Watch",
            request,
            EvidenceTraceabilityWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("evidence.traceability.watch", error) from error
        finally:
            call.cancel()


class ExecutionsCommandsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        parent: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsCommandsGetResponse:
        request = ExecutionsCommandsGetRequest(name=name, parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.executions.Commands/Get",
            request,
            ExecutionsCommandsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.commands.get",
        )
        return response

    async def create(
        self,
        parent: str,
        command_id: str,
        action: ExecutionsExecutionCommandAction,
        *,
        request_id: str | None = None,
        step_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsCommandsCreateResponse:
        request = ExecutionsCommandsCreateRequest(
            parent=parent,
            request_id=request_id if request_id is not None else str(uuid4()),
            command_id=command_id,
            action=action,
            step_id=step_id,
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.executions.Commands/Create",
            request,
            ExecutionsCommandsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.commands.create",
        )
        return response


class ExecutionsProgressResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsProgressGetResponse:
        request = ExecutionsProgressGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.executions.Progress/Get",
            request,
            ExecutionsProgressGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.progress.get",
        )
        return response

    async def watch(
        self,
        name: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[ExecutionsProgressWatchResponse]:
        request = ExecutionsProgressWatchRequest(name=name, at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.executions.Progress/Watch",
            request,
            ExecutionsProgressWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("executions.progress.watch", error) from error
        finally:
            call.cancel()


class ExecutionsSchemasResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: Position | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsSchemasGetResponse:
        request = ExecutionsSchemasGetRequest(name=name, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.executions.Schemas/Get",
            request,
            ExecutionsSchemasGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.schemas.get",
        )
        return response


class FilesUploadsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        metadata: FileMetadata,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsCreateResponse:
        request = FilesUploadsCreateRequest(metadata=metadata, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Create",
            request,
            FilesUploadsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.create",
        )
        return response

    async def get(
        self,
        upload: Upload,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsGetResponse:
        request = FilesUploadsGetRequest(upload=upload, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Get",
            request,
            FilesUploadsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.get",
        )
        return response

    async def resume(
        self,
        upload: Upload,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsResumeResponse:
        request = FilesUploadsResumeRequest(upload=upload, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Resume",
            request,
            FilesUploadsResumeResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.resume",
        )
        return response

    async def append(
        self,
        upload: Upload,
        offset: int,
        data: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsAppendResponse:
        request = FilesUploadsAppendRequest(
            upload=upload, offset=offset, data=data, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Append",
            request,
            FilesUploadsAppendResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.append",
        )
        return response

    async def publish(
        self,
        upload: Upload,
        length: int,
        sha256: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsPublishResponse:
        request = FilesUploadsPublishRequest(
            upload=upload, length=length, sha256=sha256, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Publish",
            request,
            FilesUploadsPublishResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.publish",
        )
        return response

    async def delete(
        self,
        upload: Upload,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesUploadsDeleteResponse:
        request = FilesUploadsDeleteRequest(upload=upload, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.files.Uploads/Delete",
            request,
            FilesUploadsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.uploads.delete",
        )
        return response


class MediaCapturesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        scope: ArtifactScope,
        kind: MediaCaptureKind,
        audio: MediaRecordingAudio,
        *,
        input: int | None = None,
        camera: int | None = None,
        display: int | None = None,
        maximum_seconds: int | None = None,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> MediaCapturesCreateResponse:
        request = MediaCapturesCreateRequest(
            scope=scope,
            kind=kind,
            input=input,
            camera=camera,
            display=display,
            audio=audio,
            maximum_seconds=maximum_seconds,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.media.Captures/Create",
            request,
            MediaCapturesCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="media.captures.create",
        )
        return response

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> MediaCapturesGetResponse:
        request = MediaCapturesGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.media.Captures/Get",
            request,
            MediaCapturesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="media.captures.get",
        )
        return response

    async def watch(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[MediaCapturesWatchResponse]:
        request = MediaCapturesWatchRequest(name=name, at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.media.Captures/Watch",
            request,
            MediaCapturesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("media.captures.watch", error) from error
        finally:
            call.cancel()

    async def stop(
        self,
        name: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> MediaCapturesStopResponse:
        request = MediaCapturesStopRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.media.Captures/Stop",
            request,
            MediaCapturesStopResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="media.captures.stop",
        )
        return response


class PlotsSettingsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        scope: DocumentScope,
        document_id: str,
        plot_id: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PlotsSettingsGetResponse:
        request = PlotsSettingsGetRequest(scope=scope, document_id=document_id, plot_id=plot_id, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.plots.Settings/Get",
            request,
            PlotsSettingsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plots.settings.get",
        )
        return response

    async def set(
        self,
        scope: DocumentScope,
        document_id: str,
        plot_id: str,
        value: PlotsPlotSettingsValue,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PlotsSettingsSetResponse:
        request = PlotsSettingsSetRequest(
            scope=scope,
            document_id=document_id,
            plot_id=plot_id,
            value=value,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.plots.Settings/Set",
            request,
            PlotsSettingsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plots.settings.set",
        )
        return response


class PlotsSnapshotsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        requests: Sequence[PlotsPlotRequest],
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PlotsSnapshotsListResponse:
        request = PlotsSnapshotsListRequest(requests=requests, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.plots.Snapshots/List",
            request,
            PlotsSnapshotsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plots.snapshots.list",
        )
        return response

    async def watch(
        self,
        requests: Sequence[PlotsPlotRequest],
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[PlotsSnapshotsWatchResponse]:
        request = PlotsSnapshotsWatchRequest(requests=requests, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.plots.Snapshots/Watch",
            request,
            PlotsSnapshotsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("plots.snapshots.watch", error) from error
        finally:
            call.cancel()


class PluginsSecretsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        name: str | None = None,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsSecretsListResponse:
        request = PluginsSecretsListRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.plugins.Secrets/List",
            request,
            PluginsSecretsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.secrets.list",
        )
        return response

    async def set(
        self,
        name: str,
        bindings: Sequence[PluginsSecretBindingValue],
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsSecretsSetResponse:
        request = PluginsSecretsSetRequest(
            name=name,
            bindings=bindings,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.plugins.Secrets/Set",
            request,
            PluginsSecretsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.secrets.set",
        )
        return response

    async def delete(
        self,
        name: str,
        identifiers: Sequence[str],
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> Receipt:
        request = PluginsSecretsDeleteRequest(
            name=name,
            identifiers=identifiers,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.plugins.Secrets/Delete",
            request,
            PluginsSecretsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.secrets.delete",
        )
        if not response.HasField("receipt"):
            raise ValueError("Read Secrets.Delete: Missing Payload")
        return response.receipt


class PluginsSettingsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsSettingsListResponse:
        request = PluginsSettingsListRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/harmos.host.v1.plugins.Settings/List",
            request,
            PluginsSettingsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.settings.list",
        )
        return response

    async def set(
        self,
        name: str,
        value: SettingValues,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsSettingsSetResponse:
        request = PluginsSettingsSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/harmos.host.v1.plugins.Settings/Set",
            request,
            PluginsSettingsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.settings.set",
        )
        return response


class PromptsAnswersResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        parent: str,
        action: str,
        values: MapValue,
        accepted: bool,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PromptsAnswersCreateResponse:
        request = PromptsAnswersCreateRequest(
            parent=parent,
            action=action,
            values=values,
            accepted=accepted,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.prompts.Answers/Create",
            request,
            PromptsAnswersCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="prompts.answers.create",
        )
        return response


class StatusDiagnosticsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        offset: int,
        max_bytes: int,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StatusDiagnosticsGetResponse:
        request = StatusDiagnosticsGetRequest(name=name, offset=offset, max_bytes=max_bytes)
        response = await unary(
            self._channel,
            "/harmos.host.v1.status.Diagnostics/Get",
            request,
            StatusDiagnosticsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="status.diagnostics.get",
        )
        return response


class StatusLogsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        at: Position | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StatusLogsListResponse:
        request = StatusLogsListRequest(at=at, page=page)
        response = await unary(
            self._channel,
            "/harmos.host.v1.status.Logs/List",
            request,
            StatusLogsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="status.logs.list",
        )
        return response

    async def watch(
        self,
        max_bytes: int,
        max_events: int,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[StatusLogsWatchResponse]:
        request = StatusLogsWatchRequest(at=at, max_bytes=max_bytes, max_events=max_events)
        call = stream(
            self._channel,
            "/harmos.host.v1.status.Logs/Watch",
            request,
            StatusLogsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("status.logs.watch", error) from error
        finally:
            call.cancel()


class StatusTraceEventsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        at: Position | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StatusTraceEventsListResponse:
        request = StatusTraceEventsListRequest(at=at, page=page)
        response = await unary(
            self._channel,
            "/harmos.host.v1.status.TraceEvents/List",
            request,
            StatusTraceEventsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="status.trace_events.list",
        )
        return response

    async def watch(
        self,
        max_bytes: int,
        max_events: int,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[StatusTraceEventsWatchResponse]:
        request = StatusTraceEventsWatchRequest(at=at, max_bytes=max_bytes, max_events=max_events)
        call = stream(
            self._channel,
            "/harmos.host.v1.status.TraceEvents/Watch",
            request,
            StatusTraceEventsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("status.trace_events.watch", error) from error
        finally:
            call.cancel()


class StorageSourcesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StorageSourcesGetResponse:
        request = StorageSourcesGetRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.storage.Sources/Get",
            request,
            StorageSourcesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="storage.sources.get",
        )
        return response

    async def set(
        self,
        value: StorageStorageSources,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StorageSourcesSetResponse:
        request = StorageSourcesSetRequest(
            value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.storage.Sources/Set",
            request,
            StorageSourcesSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="storage.sources.set",
        )
        return response


class AccessResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_certificates = AccessCertificatesResource(channel)
        self._composed_listener = AccessListenerResource(channel)
        self._composed_pairings = AccessPairingsResource(channel)

    @property
    def certificates(self) -> AccessCertificatesResource:
        return self._composed_certificates

    @property
    def listener(self) -> AccessListenerResource:
        return self._composed_listener

    @property
    def pairings(self) -> AccessPairingsResource:
        return self._composed_pairings


class AggregatesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        source: QuerySource,
        filters: Sequence[QueryFilter],
        group_by: Sequence[str],
        aggregates: Sequence[QueryAggregate],
        limit: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AggregatesGetResponse:
        request = AggregatesGetRequest(source=source, filters=filters, group_by=group_by, aggregates=aggregates, limit=limit, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Aggregates/Get",
            request,
            AggregatesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="aggregates.get",
        )
        return response


class AnalysesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AnalysesListResponse:
        request = AnalysesListRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Analyses/List",
            request,
            AnalysesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="analyses.list",
        )
        return response


class BackendResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_overview = BackendOverviewResource(channel)

    @property
    def overview(self) -> BackendOverviewResource:
        return self._composed_overview

    async def appearance(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> BackendAppearanceResponse:
        request = BackendAppearanceRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Backend/Appearance",
            request,
            BackendAppearanceResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="backend.appearance",
        )
        return response


class CasesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        plan: SelectionPlan,
        validate_only: bool,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CasesCreateResponse:
        request = CasesCreateRequest(
            plan=plan, validate_only=validate_only, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cases/Create",
            request,
            CasesCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cases.create",
        )
        return response

    async def get(
        self,
        name: str,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CasesGetResponse:
        request = CasesGetRequest(name=name, parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cases/Get",
            request,
            CasesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cases.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CasesListResponse:
        request = CasesListRequest(parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cases/List",
            request,
            CasesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cases.list",
        )
        return response

    async def watch(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[CasesWatchResponse]:
        request = CasesWatchRequest(parent=parent, at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.Cases/Watch",
            request,
            CasesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("cases.watch", error) from error
        finally:
            call.cancel()

    async def delete(
        self,
        name: str,
        parent: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> Receipt:
        request = CasesDeleteRequest(name=name, parent=parent, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cases/Delete",
            request,
            CasesDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cases.delete",
        )
        if not response.HasField("receipt"):
            raise ValueError("Read Cases.Delete: Missing Payload")
        return response.receipt

    async def parameters(
        self,
        name: str,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CasesParametersResponse:
        request = CasesParametersRequest(name=name, parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cases/Parameters",
            request,
            CasesParametersResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cases.parameters",
        )
        return response


class ContentResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_artifacts = ContentArtifactsResource(channel)
        self._composed_execution_files = ContentExecutionFilesResource(channel)

    @property
    def artifacts(self) -> ContentArtifactsResource:
        return self._composed_artifacts

    @property
    def execution_files(self) -> ContentExecutionFilesResource:
        return self._composed_execution_files


class CyclesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CyclesGetResponse:
        request = CyclesGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cycles/Get",
            request,
            CyclesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cycles.get",
        )
        return response

    async def set(
        self,
        name: str,
        value: CycleWritable,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CyclesSetResponse:
        request = CyclesSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cycles/Set",
            request,
            CyclesSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cycles.set",
        )
        return response

    async def list(
        self,
        selection: CycleSelection,
        filter: CycleFilter,
        order: CycleOrder,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CyclesListResponse:
        request = CyclesListRequest(selection=selection, filter=filter, order=order, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cycles/List",
            request,
            CyclesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cycles.list",
        )
        return response

    async def watch(
        self,
        selection: CycleSelection,
        filter: CycleFilter,
        order: CycleOrder,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[CyclesWatchResponse]:
        request = CyclesWatchRequest(selection=selection, filter=filter, order=order, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.Cycles/Watch",
            request,
            CyclesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("cycles.watch", error) from error
        finally:
            call.cancel()

    async def create(
        self,
        origins: Sequence[Origin],
        metadata: Metadata,
        continue_automatically: bool,
        source_cycle: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CyclesCreateResponse:
        request = CyclesCreateRequest(
            origins=origins,
            metadata=metadata,
            continue_automatically=continue_automatically,
            source_cycle=source_cycle,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cycles/Create",
            request,
            CyclesCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cycles.create",
        )
        return response

    async def seal(
        self,
        name: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> CyclesSealResponse:
        request = CyclesSealRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Cycles/Seal",
            request,
            CyclesSealResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="cycles.seal",
        )
        return response


class DocumentsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        scope: DocumentScope,
        document_id: str,
        structure_only: bool,
        card_ids: Sequence[str],
        page_token: str,
        max_bytes: int,
        max_frames: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> DocumentsGetResponse:
        request = DocumentsGetRequest(
            scope=scope,
            document_id=document_id,
            structure_only=structure_only,
            card_ids=card_ids,
            page_token=page_token,
            max_bytes=max_bytes,
            max_frames=max_frames,
            at=at,
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Documents/Get",
            request,
            DocumentsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="documents.get",
        )
        return response

    async def watch(
        self,
        scope: DocumentScope,
        document_id: str,
        structure_only: bool,
        card_ids: Sequence[str],
        page_token: str,
        max_bytes: int,
        max_frames: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[DocumentsWatchResponse]:
        request = DocumentsWatchRequest(
            scope=scope,
            document_id=document_id,
            structure_only=structure_only,
            card_ids=card_ids,
            page_token=page_token,
            max_bytes=max_bytes,
            max_frames=max_frames,
            at=at,
        )
        call = stream(
            self._channel,
            "/dokime.host.v1.Documents/Watch",
            request,
            DocumentsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("documents.watch", error) from error
        finally:
            call.cancel()


class EvidenceResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_health_events = EvidenceHealthEventsResource(channel)
        self._composed_notes = EvidenceNotesResource(channel)
        self._composed_records = EvidenceRecordsResource(channel)
        self._composed_samples = EvidenceSamplesResource(channel)
        self._composed_traceability = EvidenceTraceabilityResource(channel)

    @property
    def health_events(self) -> EvidenceHealthEventsResource:
        return self._composed_health_events

    @property
    def notes(self) -> EvidenceNotesResource:
        return self._composed_notes

    @property
    def records(self) -> EvidenceRecordsResource:
        return self._composed_records

    @property
    def samples(self) -> EvidenceSamplesResource:
        return self._composed_samples

    @property
    def traceability(self) -> EvidenceTraceabilityResource:
        return self._composed_traceability

    async def statistics(
        self,
        scope: EvidenceScope,
        stream: str,
        versions: Sequence[int],
        between: Window,
        outputs: Sequence[Output],
        field: str,
        step: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceStatisticsResponse:
        request = EvidenceStatisticsRequest(
            scope=scope, stream=stream, versions=versions, between=between, outputs=outputs, field=field, step=step, at=at
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Evidence/Statistics",
            request,
            EvidenceStatisticsResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.statistics",
        )
        return response

    async def archive(
        self,
        subject: EvidenceSubject,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> EvidenceArchiveResponse:
        request = EvidenceArchiveRequest(subject=subject, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Evidence/Archive",
            request,
            EvidenceArchiveResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="evidence.archive",
        )
        return response


class ExecutionsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_commands = ExecutionsCommandsResource(channel)
        self._composed_progress = ExecutionsProgressResource(channel)
        self._composed_schemas = ExecutionsSchemasResource(channel)

    @property
    def commands(self) -> ExecutionsCommandsResource:
        return self._composed_commands

    @property
    def progress(self) -> ExecutionsProgressResource:
        return self._composed_progress

    @property
    def schemas(self) -> ExecutionsSchemasResource:
        return self._composed_schemas

    async def get(
        self,
        name: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsGetResponse:
        request = ExecutionsGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Get",
            request,
            ExecutionsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.get",
        )
        return response

    async def set(
        self,
        name: str,
        value: ExecutionWritable,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsSetResponse:
        request = ExecutionsSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Set",
            request,
            ExecutionsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.set",
        )
        return response

    async def list(
        self,
        parent: str,
        selection: ExecutionSelection,
        filter: ExecutionFilter,
        order: ExecutionOrder,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsListResponse:
        request = ExecutionsListRequest(parent=parent, selection=selection, filter=filter, order=order, at=at, page=page)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/List",
            request,
            ExecutionsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.list",
        )
        return response

    async def watch(
        self,
        parent: str,
        selection: ExecutionSelection,
        filter: ExecutionFilter,
        order: ExecutionOrder,
        *,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[ExecutionsWatchResponse]:
        request = ExecutionsWatchRequest(parent=parent, selection=selection, filter=filter, order=order, at=at, page=page)
        call = stream(
            self._channel,
            "/dokime.host.v1.Executions/Watch",
            request,
            ExecutionsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("executions.watch", error) from error
        finally:
            call.cancel()

    async def create(
        self,
        parent: str,
        executions: Sequence[ExecutionBinding],
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsCreateResponse:
        request = ExecutionsCreateRequest(
            parent=parent, executions=executions, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Create",
            request,
            ExecutionsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.create",
        )
        return response

    async def start(
        self,
        name: str,
        mode: ExecutionMode,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsStartResponse:
        request = ExecutionsStartRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()), mode=mode)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Start",
            request,
            ExecutionsStartResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.start",
        )
        return response

    async def stop(
        self,
        name: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsStopResponse:
        request = ExecutionsStopRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Stop",
            request,
            ExecutionsStopResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.stop",
        )
        return response

    async def cancel(
        self,
        name: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsCancelResponse:
        request = ExecutionsCancelRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Cancel",
            request,
            ExecutionsCancelResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.cancel",
        )
        return response

    async def delete(
        self,
        name: str,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsDeleteResponse:
        request = ExecutionsDeleteRequest(name=name, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Delete",
            request,
            ExecutionsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.delete",
        )
        return response

    async def parameters(
        self,
        name: str,
        set: ExecutionParameterSet,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsParametersResponse:
        request = ExecutionsParametersRequest(name=name, set=set, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Parameters",
            request,
            ExecutionsParametersResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.parameters",
        )
        return response

    async def steps(
        self,
        name: str,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ExecutionsStepsResponse:
        request = ExecutionsStepsRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Executions/Steps",
            request,
            ExecutionsStepsResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="executions.steps",
        )
        return response


class FilesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_uploads = FilesUploadsResource(channel)

    @property
    def uploads(self) -> FilesUploadsResource:
        return self._composed_uploads

    async def get(
        self,
        file: FileReference,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesGetResponse:
        request = FilesGetRequest(file=file, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Files/Get",
            request,
            FilesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.get",
        )
        return response

    async def read(
        self,
        file: FileReference,
        offset: int,
        max_bytes: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> FilesReadResponse:
        request = FilesReadRequest(file=file, offset=offset, max_bytes=max_bytes, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Files/Read",
            request,
            FilesReadResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.read",
        )
        return response

    async def watch(
        self,
        offset: int,
        *,
        file: FileReference | None = None,
        upload: Upload | None = None,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[FilesWatchResponse]:
        request = FilesWatchRequest(file=file, upload=upload, offset=offset, at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.Files/Watch",
            request,
            FilesWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("files.watch", error) from error
        finally:
            call.cancel()

    async def delete(
        self,
        file: FileReference,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> Receipt:
        request = FilesDeleteRequest(file=file, request_id=request_id if request_id is not None else str(uuid4()))
        response = await unary(
            self._channel,
            "/dokime.host.v1.Files/Delete",
            request,
            FilesDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="files.delete",
        )
        if not response.HasField("receipt"):
            raise ValueError("Read Files.Delete: Missing Payload")
        return response.receipt


class InvocationsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def create(
        self,
        operation: str,
        fingerprint: bytes,
        payload: bytes,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> bytes:
        request = InvocationsCreateRequest(operation=operation, fingerprint=fingerprint, payload=payload)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Invocations/Create",
            request,
            InvocationsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="invocations.create",
        )
        return response.payload

    async def get(
        self,
        instance: int,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> InstanceState:
        request = InvocationsGetRequest(instance=instance)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Invocations/Get",
            request,
            InvocationsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="invocations.get",
        )
        if not response.HasField("invocation"):
            raise ValueError("Read Invocations.Get: Missing Payload")
        return response.invocation

    async def cancel(
        self,
        instance: int,
        reason: str,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> InstanceState:
        request = InvocationsCancelRequest(instance=instance, reason=reason)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Invocations/Cancel",
            request,
            InvocationsCancelResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="invocations.cancel",
        )
        if not response.HasField("invocation"):
            raise ValueError("Read Invocations.Cancel: Missing Payload")
        return response.invocation

    async def watch(
        self,
        instance: int,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[InstanceState]:
        request = InvocationsWatchRequest(instance=instance)
        call = stream(
            self._channel,
            "/harmos.host.v1.Invocations/Watch",
            request,
            InvocationsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                if not response.HasField("invocation"):
                    raise ValueError("Read Invocations.Watch: Missing Payload")
                yield response.invocation
        except AioRpcError as error:
            raise call_error("invocations.watch", error) from error
        finally:
            call.cancel()


class MediaResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_captures = MediaCapturesResource(channel)

    @property
    def captures(self) -> MediaCapturesResource:
        return self._composed_captures


class OperationsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self, *, call_metadata: CallMetadata | None = None, timeout: float | None = None, credentials: CallCredentials | None = None
    ) -> list[DiscoveredOperation]:
        request = OperationsListRequest()
        response = await unary(
            self._channel,
            "/harmos.host.v1.Operations/List",
            request,
            OperationsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="operations.list",
        )
        return list(response.operations)

    async def get(
        self,
        id: str,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> DiscoveredOperation:
        request = OperationsGetRequest(id=id)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Operations/Get",
            request,
            OperationsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="operations.get",
        )
        if not response.HasField("operation"):
            raise ValueError("Read Operations.Get: Missing Payload")
        return response.operation


class PlotsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_settings = PlotsSettingsResource(channel)
        self._composed_snapshots = PlotsSnapshotsResource(channel)

    @property
    def settings(self) -> PlotsSettingsResource:
        return self._composed_settings

    @property
    def snapshots(self) -> PlotsSnapshotsResource:
        return self._composed_snapshots


class PluginsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_secrets = PluginsSecretsResource(channel)
        self._composed_settings = PluginsSettingsResource(channel)

    @property
    def secrets(self) -> PluginsSecretsResource:
        return self._composed_secrets

    @property
    def settings(self) -> PluginsSettingsResource:
        return self._composed_settings

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsGetResponse:
        request = PluginsGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Get",
            request,
            PluginsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.get",
        )
        return response

    async def list(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsListResponse:
        request = PluginsListRequest(at=at)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/List",
            request,
            PluginsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.list",
        )
        return response

    async def watch(
        self,
        *,
        name: str | None = None,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[PluginsWatchResponse]:
        request = PluginsWatchRequest(name=name, at=at)
        call = stream(
            self._channel,
            "/harmos.host.v1.Plugins/Watch",
            request,
            PluginsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("plugins.watch", error) from error
        finally:
            call.cancel()

    async def create(
        self,
        name: str,
        placement: PluginPlacement,
        value: PluginValue,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsCreateResponse:
        request = PluginsCreateRequest(
            request_id=request_id if request_id is not None else str(uuid4()), name=name, placement=placement, value=value
        )
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Create",
            request,
            PluginsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.create",
        )
        return response

    async def set(
        self,
        name: str,
        value: PluginValue,
        expected_revision: bytes,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsSetResponse:
        request = PluginsSetRequest(name=name, value=value, expected_revision=expected_revision)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Set",
            request,
            PluginsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.set",
        )
        return response

    async def delete(
        self,
        name: str,
        expected_revision: bytes,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> Receipt:
        request = PluginsDeleteRequest(name=name, expected_revision=expected_revision)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Delete",
            request,
            PluginsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.delete",
        )
        if not response.HasField("receipt"):
            raise ValueError("Read Plugins.Delete: Missing Payload")
        return response.receipt

    async def restart(
        self,
        name: str,
        expected_revision: bytes,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginEntry:
        request = PluginsRestartRequest(name=name, expected_revision=expected_revision)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Restart",
            request,
            PluginsRestartResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.restart",
        )
        if not response.HasField("entry"):
            raise ValueError("Read Plugins.Restart: Missing Payload")
        return response.entry

    async def replace(
        self,
        name: str,
        placement: PluginPlacement,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PluginsReplaceResponse:
        request = PluginsReplaceRequest(
            name=name,
            placement=placement,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/harmos.host.v1.Plugins/Replace",
            request,
            PluginsReplaceResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="plugins.replace",
        )
        return response


class PromptDefinitionsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        parent: str,
        schema_version: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PromptDefinitionsGetResponse:
        request = PromptDefinitionsGetRequest(name=name, parent=parent, schema_version=schema_version, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.PromptDefinitions/Get",
            request,
            PromptDefinitionsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="prompt_definitions.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PromptDefinitionsListResponse:
        request = PromptDefinitionsListRequest(parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.PromptDefinitions/List",
            request,
            PromptDefinitionsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="prompt_definitions.list",
        )
        return response


class PromptsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_answers = PromptsAnswersResource(channel)

    @property
    def answers(self) -> PromptsAnswersResource:
        return self._composed_answers

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PromptsGetResponse:
        request = PromptsGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Prompts/Get",
            request,
            PromptsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="prompts.get",
        )
        return response

    async def list(
        self,
        parent: str,
        pending_only: bool,
        *,
        cycle_id: str | None = None,
        execution_id: str | None = None,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> PromptsListResponse:
        request = PromptsListRequest(
            parent=parent, cycle_id=cycle_id, execution_id=execution_id, pending_only=pending_only, at=at, page=page
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Prompts/List",
            request,
            PromptsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="prompts.list",
        )
        return response

    async def watch(
        self,
        parent: str,
        pending_only: bool,
        *,
        cycle_id: str | None = None,
        execution_id: str | None = None,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[PromptsWatchResponse]:
        request = PromptsWatchRequest(
            parent=parent, cycle_id=cycle_id, execution_id=execution_id, pending_only=pending_only, at=at, page=page
        )
        call = stream(
            self._channel,
            "/dokime.host.v1.Prompts/Watch",
            request,
            PromptsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("prompts.watch", error) from error
        finally:
            call.cancel()


class RecordSchemasResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        parent: str,
        schema_version: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> RecordSchemasGetResponse:
        request = RecordSchemasGetRequest(name=name, parent=parent, schema_version=schema_version, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.RecordSchemas/Get",
            request,
            RecordSchemasGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="record_schemas.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> RecordSchemasListResponse:
        request = RecordSchemasListRequest(parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.RecordSchemas/List",
            request,
            RecordSchemasListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="record_schemas.list",
        )
        return response


class ReportsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        subject: ReportSubject,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ReportsGetResponse:
        request = ReportsGetRequest(subject=subject, name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Reports/Get",
            request,
            ReportsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="reports.get",
        )
        return response

    async def create(
        self,
        subject: ReportSubject,
        document_id: str,
        *,
        at: ReadCut | None = None,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ReportsCreateResponse:
        request = ReportsCreateRequest(
            subject=subject, document_id=document_id, at=at, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Reports/Create",
            request,
            ReportsCreateResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="reports.create",
        )
        return response

    async def publish(
        self,
        name: str,
        subject: ReportSubject,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> ReportsPublishResponse:
        request = ReportsPublishRequest(
            name=name,
            subject=subject,
            expected_revision=expected_revision,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Reports/Publish",
            request,
            ReportsPublishResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="reports.publish",
        )
        return response


class SchemasResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        schema: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> list[SchemaDescription]:
        request = SchemasListRequest(schema=schema)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Schemas/List",
            request,
            SchemasListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="schemas.list",
        )
        return list(response.schemas)


class SecretsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsGetResponse:
        request = SecretsGetRequest(name=name, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/Get",
            request,
            SecretsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.get",
        )
        return response

    async def set(
        self,
        name: str,
        value: SecretWritable,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsSetResponse:
        request = SecretsSetRequest(
            name=name, value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/Set",
            request,
            SecretsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.set",
        )
        return response

    async def list(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsListResponse:
        request = SecretsListRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/List",
            request,
            SecretsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.list",
        )
        return response

    async def watch(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[SecretsWatchResponse]:
        request = SecretsWatchRequest(at=at)
        call = stream(
            self._channel,
            "/dokime.host.v1.Secrets/Watch",
            request,
            SecretsWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("secrets.watch", error) from error
        finally:
            call.cancel()

    async def delete(
        self,
        names: Sequence[str],
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsDeleteResponse:
        request = SecretsDeleteRequest(
            names=names, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/Delete",
            request,
            SecretsDeleteResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.delete",
        )
        return response

    async def export(
        self,
        names: Sequence[str],
        generation: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsExportResponse:
        request = SecretsExportRequest(names=names, generation=generation, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/Export",
            request,
            SecretsExportResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.export",
        )
        return response

    async def import_(
        self,
        data: bytes,
        choices: Sequence[SecretImportChoice],
        generation: int,
        validate_only: bool,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SecretsImportResponse:
        request = SecretsImportRequest(
            data=data,
            choices=choices,
            generation=generation,
            validate_only=validate_only,
            request_id=request_id if request_id is not None else str(uuid4()),
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Secrets/Import",
            request,
            SecretsImportResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="secrets.import",
        )
        return response


class SettingsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SettingsGetResponse:
        request = SettingsGetRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Settings/Get",
            request,
            SettingsGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="settings.get",
        )
        return response

    async def set(
        self,
        value: ApplicationSettings,
        expected_revision: bytes,
        *,
        request_id: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SettingsSetResponse:
        request = SettingsSetRequest(
            value=value, expected_revision=expected_revision, request_id=request_id if request_id is not None else str(uuid4())
        )
        response = await unary(
            self._channel,
            "/dokime.host.v1.Settings/Set",
            request,
            SettingsSetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="settings.set",
        )
        return response

    async def schema(
        self,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SettingsSchemaResponse:
        request = SettingsSchemaRequest(at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Settings/Schema",
            request,
            SettingsSchemaResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="settings.schema",
        )
        return response


class StatusResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_diagnostics = StatusDiagnosticsResource(channel)
        self._composed_logs = StatusLogsResource(channel)
        self._composed_trace_events = StatusTraceEventsResource(channel)

    @property
    def diagnostics(self) -> StatusDiagnosticsResource:
        return self._composed_diagnostics

    @property
    def logs(self) -> StatusLogsResource:
        return self._composed_logs

    @property
    def trace_events(self) -> StatusTraceEventsResource:
        return self._composed_trace_events

    async def get(
        self, *, call_metadata: CallMetadata | None = None, timeout: float | None = None, credentials: CallCredentials | None = None
    ) -> StatusGetResponse:
        request = StatusGetRequest()
        response = await unary(
            self._channel,
            "/harmos.host.v1.Status/Get",
            request,
            StatusGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="status.get",
        )
        return response

    async def watch(
        self,
        *,
        at: Position | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> AsyncGenerator[StatusWatchResponse]:
        request = StatusWatchRequest(at=at)
        call = stream(
            self._channel,
            "/harmos.host.v1.Status/Watch",
            request,
            StatusWatchResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
        )
        try:
            async for response in call:
                yield response
        except AioRpcError as error:
            raise call_error("status.watch", error) from error
        finally:
            call.cancel()


class StorageResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel
        self._composed_sources = StorageSourcesResource(channel)

    @property
    def sources(self) -> StorageSourcesResource:
        return self._composed_sources


class StreamSchemasResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        parent: str,
        schema_version: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StreamSchemasGetResponse:
        request = StreamSchemasGetRequest(name=name, parent=parent, schema_version=schema_version, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.StreamSchemas/Get",
            request,
            StreamSchemasGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="stream_schemas.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> StreamSchemasListResponse:
        request = StreamSchemasListRequest(parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.StreamSchemas/List",
            request,
            StreamSchemasListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="stream_schemas.list",
        )
        return response


class StreamsResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def list(
        self,
        *,
        schema: str | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> list[StreamIdentity]:
        request = StreamsListRequest(schema=schema)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Streams/List",
            request,
            StreamsListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="streams.list",
        )
        return list(response.streams)


class SuitesResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self,
        name: str,
        parent: str,
        schema_version: int,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SuitesGetResponse:
        request = SuitesGetRequest(name=name, parent=parent, schema_version=schema_version, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Suites/Get",
            request,
            SuitesGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="suites.get",
        )
        return response

    async def list(
        self,
        parent: str,
        *,
        at: ReadCut | None = None,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> SuitesListResponse:
        request = SuitesListRequest(parent=parent, at=at)
        response = await unary(
            self._channel,
            "/dokime.host.v1.Suites/List",
            request,
            SuitesListResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="suites.list",
        )
        return response


class TimeResource:
    def __init__(self, channel: Channel) -> None:
        self._channel = channel

    async def get(
        self, *, call_metadata: CallMetadata | None = None, timeout: float | None = None, credentials: CallCredentials | None = None
    ) -> Timestamp:
        request = TimeGetRequest()
        response = await unary(
            self._channel,
            "/harmos.host.v1.Time/Get",
            request,
            TimeGetResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="time.get",
        )
        if not response.HasField("now"):
            raise ValueError("Read Time.Get: Missing Payload")
        return response.now

    async def sleep(
        self,
        duration: Duration,
        *,
        call_metadata: CallMetadata | None = None,
        timeout: float | None = None,
        credentials: CallCredentials | None = None,
    ) -> TimeSleepResponse:
        request = TimeSleepRequest(duration=duration)
        response = await unary(
            self._channel,
            "/harmos.host.v1.Time/Sleep",
            request,
            TimeSleepResponse,
            metadata=call_metadata,
            timeout=timeout,
            credentials=credentials,
            action="time.sleep",
        )
        return response


class ClientRoot(LibraryConnection):
    def __init__(self, channel: Channel) -> None:
        super().__init__(channel)
        self._channel = channel
        self._composed_access = AccessResource(channel)
        self._composed_aggregates = AggregatesResource(channel)
        self._composed_analyses = AnalysesResource(channel)
        self._composed_backend = BackendResource(channel)
        self._composed_cases = CasesResource(channel)
        self._composed_content = ContentResource(channel)
        self._composed_cycles = CyclesResource(channel)
        self._composed_documents = DocumentsResource(channel)
        self._composed_evidence = EvidenceResource(channel)
        self._composed_executions = ExecutionsResource(channel)
        self._composed_files = FilesResource(channel)
        self._composed_invocations = InvocationsResource(channel)
        self._composed_media = MediaResource(channel)
        self._composed_operations = OperationsResource(channel)
        self._composed_plots = PlotsResource(channel)
        self._composed_plugins = PluginsResource(channel)
        self._composed_prompt_definitions = PromptDefinitionsResource(channel)
        self._composed_prompts = PromptsResource(channel)
        self._composed_record_schemas = RecordSchemasResource(channel)
        self._composed_reports = ReportsResource(channel)
        self._composed_schemas = SchemasResource(channel)
        self._composed_secrets = SecretsResource(channel)
        self._composed_settings = SettingsResource(channel)
        self._composed_status = StatusResource(channel)
        self._composed_storage = StorageResource(channel)
        self._composed_stream_schemas = StreamSchemasResource(channel)
        self._composed_streams = StreamsResource(channel)
        self._composed_suites = SuitesResource(channel)
        self._composed_time = TimeResource(channel)

    @property
    def access(self) -> AccessResource:
        return self._composed_access

    @property
    def aggregates(self) -> AggregatesResource:
        return self._composed_aggregates

    @property
    def analyses(self) -> AnalysesResource:
        return self._composed_analyses

    @property
    def backend(self) -> BackendResource:
        return self._composed_backend

    @property
    def cases(self) -> CasesResource:
        return self._composed_cases

    @property
    def content(self) -> ContentResource:
        return self._composed_content

    @property
    def cycles(self) -> CyclesResource:
        return self._composed_cycles

    @property
    def documents(self) -> DocumentsResource:
        return self._composed_documents

    @property
    def evidence(self) -> EvidenceResource:
        return self._composed_evidence

    @property
    def executions(self) -> ExecutionsResource:
        return self._composed_executions

    @property
    def files(self) -> FilesResource:
        return self._composed_files

    @property
    def invocations(self) -> InvocationsResource:
        return self._composed_invocations

    @property
    def media(self) -> MediaResource:
        return self._composed_media

    @property
    def operations(self) -> OperationsResource:
        return self._composed_operations

    @property
    def plots(self) -> PlotsResource:
        return self._composed_plots

    @property
    def plugins(self) -> PluginsResource:
        return self._composed_plugins

    @property
    def prompt_definitions(self) -> PromptDefinitionsResource:
        return self._composed_prompt_definitions

    @property
    def prompts(self) -> PromptsResource:
        return self._composed_prompts

    @property
    def record_schemas(self) -> RecordSchemasResource:
        return self._composed_record_schemas

    @property
    def reports(self) -> ReportsResource:
        return self._composed_reports

    @property
    def schemas(self) -> SchemasResource:
        return self._composed_schemas

    @property
    def secrets(self) -> SecretsResource:
        return self._composed_secrets

    @property
    def settings(self) -> SettingsResource:
        return self._composed_settings

    @property
    def status(self) -> StatusResource:
        return self._composed_status

    @property
    def storage(self) -> StorageResource:
        return self._composed_storage

    @property
    def stream_schemas(self) -> StreamSchemasResource:
        return self._composed_stream_schemas

    @property
    def streams(self) -> StreamsResource:
        return self._composed_streams

    @property
    def suites(self) -> SuitesResource:
        return self._composed_suites

    @property
    def time(self) -> TimeResource:
        return self._composed_time
