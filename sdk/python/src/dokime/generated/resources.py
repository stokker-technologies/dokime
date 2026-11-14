"""Typed Resources Generated From the Frozen Descriptor Table."""

from collections.abc import AsyncIterator, Awaitable
from typing import Protocol, cast

import harmos.v1.access.certificates_pb2 as wire_35
import harmos.v1.access.listener_pb2 as wire_36
import harmos.v1.access.pairings_pb2 as wire_37
import harmos.v1.catalog_entries_pb2 as wire_38
import harmos.v1.invocations_pb2 as wire_39
import harmos.v1.plugins_pb2 as wire_40
import harmos.v1.status.diagnostics_pb2 as wire_41
import harmos.v1.status.logs_pb2 as wire_42
import harmos.v1.status.trace_events_pb2 as wire_43
import harmos.v1.status_pb2 as wire_44
import harmos.v1.time_pb2 as wire_45
from google.protobuf import descriptor_pool
from google.protobuf.descriptor import MethodDescriptor
from google.protobuf.message import Message

import dokime.client.v1.aggregates_pb2 as wire_0
import dokime.client.v1.analyses_pb2 as wire_1
import dokime.client.v1.backend.overview_pb2 as wire_2
import dokime.client.v1.backend_pb2 as wire_3
import dokime.client.v1.cases_pb2 as wire_4
import dokime.client.v1.content.artifacts_pb2 as wire_5
import dokime.client.v1.content.execution_files_pb2 as wire_6
import dokime.client.v1.cycles_pb2 as wire_7
import dokime.client.v1.documents_pb2 as wire_8
import dokime.client.v1.evidence.health_events_pb2 as wire_9
import dokime.client.v1.evidence.notes_pb2 as wire_10
import dokime.client.v1.evidence.records_pb2 as wire_11
import dokime.client.v1.evidence.samples_pb2 as wire_12
import dokime.client.v1.evidence.traceability_pb2 as wire_13
import dokime.client.v1.evidence_pb2 as wire_14
import dokime.client.v1.executions.commands_pb2 as wire_15
import dokime.client.v1.executions.progress_pb2 as wire_16
import dokime.client.v1.executions.schemas_pb2 as wire_17
import dokime.client.v1.executions_pb2 as wire_18
import dokime.client.v1.files.uploads_pb2 as wire_19
import dokime.client.v1.files_pb2 as wire_20
import dokime.client.v1.media.captures_pb2 as wire_21
import dokime.client.v1.plots.settings_pb2 as wire_22
import dokime.client.v1.plots.snapshots_pb2 as wire_23
import dokime.client.v1.plugins.settings_pb2 as wire_24
import dokime.client.v1.prompt_definitions_pb2 as wire_25
import dokime.client.v1.prompts.answers_pb2 as wire_26
import dokime.client.v1.prompts_pb2 as wire_27
import dokime.client.v1.record_schemas_pb2 as wire_28
import dokime.client.v1.reports_pb2 as wire_29
import dokime.client.v1.secrets_pb2 as wire_30
import dokime.client.v1.settings_pb2 as wire_31
import dokime.client.v1.storage.sources_pb2 as wire_32
import dokime.client.v1.stream_schemas_pb2 as wire_33
import dokime.client.v1.suites_pb2 as wire_34


class Caller(Protocol):
    def call(
        self, request: Message, *, method: MethodDescriptor, timeout: float | None = None
    ) -> Awaitable[Message] | AsyncIterator[Message]: ...


class AccessCertificatesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(
        self, request: wire_35.CertificatesCreateRequest, *, timeout: float | None = None
    ) -> wire_35.CertificatesCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Certificates").methods_by_name["Create"]
        return await cast(Awaitable[wire_35.CertificatesCreateResponse], self._client.call(request, method=method, timeout=timeout))


class AccessListenerResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_36.ListenerGetRequest, *, timeout: float | None = None) -> wire_36.ListenerGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Listener").methods_by_name["Get"]
        return await cast(Awaitable[wire_36.ListenerGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_36.ListenerSetRequest, *, timeout: float | None = None) -> wire_36.ListenerSetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Listener").methods_by_name["Set"]
        return await cast(Awaitable[wire_36.ListenerSetResponse], self._client.call(request, method=method, timeout=timeout))


class AccessPairingsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_37.PairingsCreateRequest, *, timeout: float | None = None) -> wire_37.PairingsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Pairings").methods_by_name["Create"]
        return await cast(Awaitable[wire_37.PairingsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def delete(self, request: wire_37.PairingsDeleteRequest, *, timeout: float | None = None) -> wire_37.PairingsDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Pairings").methods_by_name["Delete"]
        return await cast(Awaitable[wire_37.PairingsDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_37.PairingsListRequest, *, timeout: float | None = None) -> wire_37.PairingsListResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.access.Pairings").methods_by_name["List"]
        return await cast(Awaitable[wire_37.PairingsListResponse], self._client.call(request, method=method, timeout=timeout))


class BackendOverviewResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_2.OverviewGetRequest, *, timeout: float | None = None) -> wire_2.OverviewGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.backend.Overview").methods_by_name["Get"]
        return await cast(Awaitable[wire_2.OverviewGetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_2.OverviewWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_2.OverviewWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.backend.Overview").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_2.OverviewWatchResponse], self._client.call(request, method=method, timeout=timeout))


class ContentArtifactsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_5.ArtifactsCreateRequest, *, timeout: float | None = None) -> wire_5.ArtifactsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.Artifacts").methods_by_name["Create"]
        return await cast(Awaitable[wire_5.ArtifactsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_5.ArtifactsGetRequest, *, timeout: float | None = None) -> wire_5.ArtifactsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.Artifacts").methods_by_name["Get"]
        return await cast(Awaitable[wire_5.ArtifactsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_5.ArtifactsListRequest, *, timeout: float | None = None) -> wire_5.ArtifactsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.Artifacts").methods_by_name["List"]
        return await cast(Awaitable[wire_5.ArtifactsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_5.ArtifactsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_5.ArtifactsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.Artifacts").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_5.ArtifactsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class ContentExecutionFilesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_6.ExecutionFilesGetRequest, *, timeout: float | None = None) -> wire_6.ExecutionFilesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.ExecutionFiles").methods_by_name["Get"]
        return await cast(Awaitable[wire_6.ExecutionFilesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_6.ExecutionFilesListRequest, *, timeout: float | None = None) -> wire_6.ExecutionFilesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.ExecutionFiles").methods_by_name["List"]
        return await cast(Awaitable[wire_6.ExecutionFilesListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_6.ExecutionFilesWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_6.ExecutionFilesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.content.ExecutionFiles").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_6.ExecutionFilesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceHealthEventsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_9.HealthEventsListRequest, *, timeout: float | None = None) -> wire_9.HealthEventsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.HealthEvents").methods_by_name["List"]
        return await cast(Awaitable[wire_9.HealthEventsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_9.HealthEventsWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_9.HealthEventsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.HealthEvents").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_9.HealthEventsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceNotesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_10.NotesCreateRequest, *, timeout: float | None = None) -> wire_10.NotesCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Notes").methods_by_name["Create"]
        return await cast(Awaitable[wire_10.NotesCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_10.NotesGetRequest, *, timeout: float | None = None) -> wire_10.NotesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Notes").methods_by_name["Get"]
        return await cast(Awaitable[wire_10.NotesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_10.NotesListRequest, *, timeout: float | None = None) -> wire_10.NotesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Notes").methods_by_name["List"]
        return await cast(Awaitable[wire_10.NotesListResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_10.NotesSetRequest, *, timeout: float | None = None) -> wire_10.NotesSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Notes").methods_by_name["Set"]
        return await cast(Awaitable[wire_10.NotesSetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_10.NotesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_10.NotesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Notes").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_10.NotesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceRecordsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_11.RecordsListRequest, *, timeout: float | None = None) -> wire_11.RecordsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Records").methods_by_name["List"]
        return await cast(Awaitable[wire_11.RecordsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_11.RecordsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_11.RecordsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Records").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_11.RecordsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceSamplesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_12.SamplesListRequest, *, timeout: float | None = None) -> wire_12.SamplesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Samples").methods_by_name["List"]
        return await cast(Awaitable[wire_12.SamplesListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_12.SamplesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_12.SamplesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Samples").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_12.SamplesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceTraceabilityResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_13.TraceabilityGetRequest, *, timeout: float | None = None) -> wire_13.TraceabilityGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Traceability").methods_by_name["Get"]
        return await cast(Awaitable[wire_13.TraceabilityGetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_13.TraceabilityWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_13.TraceabilityWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.evidence.Traceability").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_13.TraceabilityWatchResponse], self._client.call(request, method=method, timeout=timeout))


class ExecutionsCommandsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_15.CommandsCreateRequest, *, timeout: float | None = None) -> wire_15.CommandsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.executions.Commands").methods_by_name["Create"]
        return await cast(Awaitable[wire_15.CommandsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_15.CommandsGetRequest, *, timeout: float | None = None) -> wire_15.CommandsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.executions.Commands").methods_by_name["Get"]
        return await cast(Awaitable[wire_15.CommandsGetResponse], self._client.call(request, method=method, timeout=timeout))


class ExecutionsProgressResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_16.ProgressGetRequest, *, timeout: float | None = None) -> wire_16.ProgressGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.executions.Progress").methods_by_name["Get"]
        return await cast(Awaitable[wire_16.ProgressGetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_16.ProgressWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_16.ProgressWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.executions.Progress").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_16.ProgressWatchResponse], self._client.call(request, method=method, timeout=timeout))


class ExecutionsSchemasResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_17.SchemasGetRequest, *, timeout: float | None = None) -> wire_17.SchemasGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.executions.Schemas").methods_by_name["Get"]
        return await cast(Awaitable[wire_17.SchemasGetResponse], self._client.call(request, method=method, timeout=timeout))


class FilesUploadsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def append(self, request: wire_19.UploadsAppendRequest, *, timeout: float | None = None) -> wire_19.UploadsAppendResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Append"]
        return await cast(Awaitable[wire_19.UploadsAppendResponse], self._client.call(request, method=method, timeout=timeout))

    async def create(self, request: wire_19.UploadsCreateRequest, *, timeout: float | None = None) -> wire_19.UploadsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Create"]
        return await cast(Awaitable[wire_19.UploadsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def delete(self, request: wire_19.UploadsDeleteRequest, *, timeout: float | None = None) -> wire_19.UploadsDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Delete"]
        return await cast(Awaitable[wire_19.UploadsDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_19.UploadsGetRequest, *, timeout: float | None = None) -> wire_19.UploadsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Get"]
        return await cast(Awaitable[wire_19.UploadsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def publish(self, request: wire_19.UploadsPublishRequest, *, timeout: float | None = None) -> wire_19.UploadsPublishResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Publish"]
        return await cast(Awaitable[wire_19.UploadsPublishResponse], self._client.call(request, method=method, timeout=timeout))

    async def resume(self, request: wire_19.UploadsResumeRequest, *, timeout: float | None = None) -> wire_19.UploadsResumeResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.files.Uploads").methods_by_name["Resume"]
        return await cast(Awaitable[wire_19.UploadsResumeResponse], self._client.call(request, method=method, timeout=timeout))


class MediaCapturesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_21.CapturesCreateRequest, *, timeout: float | None = None) -> wire_21.CapturesCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.media.Captures").methods_by_name["Create"]
        return await cast(Awaitable[wire_21.CapturesCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_21.CapturesGetRequest, *, timeout: float | None = None) -> wire_21.CapturesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.media.Captures").methods_by_name["Get"]
        return await cast(Awaitable[wire_21.CapturesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def stop(self, request: wire_21.CapturesStopRequest, *, timeout: float | None = None) -> wire_21.CapturesStopResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.media.Captures").methods_by_name["Stop"]
        return await cast(Awaitable[wire_21.CapturesStopResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_21.CapturesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_21.CapturesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.media.Captures").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_21.CapturesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class PlotsSettingsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_22.SettingsGetRequest, *, timeout: float | None = None) -> wire_22.SettingsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plots.Settings").methods_by_name["Get"]
        return await cast(Awaitable[wire_22.SettingsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_22.SettingsSetRequest, *, timeout: float | None = None) -> wire_22.SettingsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plots.Settings").methods_by_name["Set"]
        return await cast(Awaitable[wire_22.SettingsSetResponse], self._client.call(request, method=method, timeout=timeout))


class PlotsSnapshotsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_23.SnapshotsListRequest, *, timeout: float | None = None) -> wire_23.SnapshotsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plots.Snapshots").methods_by_name["List"]
        return await cast(Awaitable[wire_23.SnapshotsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_23.SnapshotsWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_23.SnapshotsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plots.Snapshots").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_23.SnapshotsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class PluginsSettingsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_24.SettingsGetRequest, *, timeout: float | None = None) -> wire_24.SettingsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plugins.Settings").methods_by_name["Get"]
        return await cast(Awaitable[wire_24.SettingsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_24.SettingsSetRequest, *, timeout: float | None = None) -> wire_24.SettingsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.plugins.Settings").methods_by_name["Set"]
        return await cast(Awaitable[wire_24.SettingsSetResponse], self._client.call(request, method=method, timeout=timeout))


class PromptsAnswersResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_26.AnswersCreateRequest, *, timeout: float | None = None) -> wire_26.AnswersCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.prompts.Answers").methods_by_name["Create"]
        return await cast(Awaitable[wire_26.AnswersCreateResponse], self._client.call(request, method=method, timeout=timeout))


class StatusDiagnosticsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_41.DiagnosticsGetRequest, *, timeout: float | None = None) -> wire_41.DiagnosticsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.status.Diagnostics").methods_by_name["Get"]
        return await cast(Awaitable[wire_41.DiagnosticsGetResponse], self._client.call(request, method=method, timeout=timeout))


class StatusLogsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_42.LogsListRequest, *, timeout: float | None = None) -> wire_42.LogsListResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.status.Logs").methods_by_name["List"]
        return await cast(Awaitable[wire_42.LogsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_42.LogsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_42.LogsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.status.Logs").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_42.LogsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class StatusTraceEventsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_43.TraceEventsListRequest, *, timeout: float | None = None) -> wire_43.TraceEventsListResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.status.TraceEvents").methods_by_name["List"]
        return await cast(Awaitable[wire_43.TraceEventsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_43.TraceEventsWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_43.TraceEventsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.status.TraceEvents").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_43.TraceEventsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class StorageSourcesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_32.SourcesGetRequest, *, timeout: float | None = None) -> wire_32.SourcesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.storage.Sources").methods_by_name["Get"]
        return await cast(Awaitable[wire_32.SourcesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_32.SourcesSetRequest, *, timeout: float | None = None) -> wire_32.SourcesSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.storage.Sources").methods_by_name["Set"]
        return await cast(Awaitable[wire_32.SourcesSetResponse], self._client.call(request, method=method, timeout=timeout))


class AccessResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.certificates = AccessCertificatesResource(client)
        self.listener = AccessListenerResource(client)
        self.pairings = AccessPairingsResource(client)


class AggregatesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_0.AggregatesGetRequest, *, timeout: float | None = None) -> wire_0.AggregatesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Aggregates").methods_by_name["Get"]
        return await cast(Awaitable[wire_0.AggregatesGetResponse], self._client.call(request, method=method, timeout=timeout))


class AnalysesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_1.AnalysesListRequest, *, timeout: float | None = None) -> wire_1.AnalysesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Analyses").methods_by_name["List"]
        return await cast(Awaitable[wire_1.AnalysesListResponse], self._client.call(request, method=method, timeout=timeout))


class BackendResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.overview = BackendOverviewResource(client)

    async def appearance(
        self, request: wire_3.BackendAppearanceRequest, *, timeout: float | None = None
    ) -> wire_3.BackendAppearanceResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Backend").methods_by_name["Appearance"]
        return await cast(Awaitable[wire_3.BackendAppearanceResponse], self._client.call(request, method=method, timeout=timeout))


class CasesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_4.CasesCreateRequest, *, timeout: float | None = None) -> wire_4.CasesCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["Create"]
        return await cast(Awaitable[wire_4.CasesCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def delete(self, request: wire_4.CasesDeleteRequest, *, timeout: float | None = None) -> wire_4.CasesDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["Delete"]
        return await cast(Awaitable[wire_4.CasesDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_4.CasesGetRequest, *, timeout: float | None = None) -> wire_4.CasesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["Get"]
        return await cast(Awaitable[wire_4.CasesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_4.CasesListRequest, *, timeout: float | None = None) -> wire_4.CasesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["List"]
        return await cast(Awaitable[wire_4.CasesListResponse], self._client.call(request, method=method, timeout=timeout))

    async def parameters(self, request: wire_4.CasesParametersRequest, *, timeout: float | None = None) -> wire_4.CasesParametersResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["Parameters"]
        return await cast(Awaitable[wire_4.CasesParametersResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_4.CasesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_4.CasesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cases").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_4.CasesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class CatalogEntriesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def list(self, request: wire_38.CatalogEntriesListRequest, *, timeout: float | None = None) -> wire_38.CatalogEntriesListResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.CatalogEntries").methods_by_name["List"]
        return await cast(Awaitable[wire_38.CatalogEntriesListResponse], self._client.call(request, method=method, timeout=timeout))


class ContentResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.artifacts = ContentArtifactsResource(client)
        self.execution_files = ContentExecutionFilesResource(client)


class CyclesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_7.CyclesCreateRequest, *, timeout: float | None = None) -> wire_7.CyclesCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["Create"]
        return await cast(Awaitable[wire_7.CyclesCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_7.CyclesGetRequest, *, timeout: float | None = None) -> wire_7.CyclesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["Get"]
        return await cast(Awaitable[wire_7.CyclesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_7.CyclesListRequest, *, timeout: float | None = None) -> wire_7.CyclesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["List"]
        return await cast(Awaitable[wire_7.CyclesListResponse], self._client.call(request, method=method, timeout=timeout))

    async def seal(self, request: wire_7.CyclesSealRequest, *, timeout: float | None = None) -> wire_7.CyclesSealResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["Seal"]
        return await cast(Awaitable[wire_7.CyclesSealResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_7.CyclesSetRequest, *, timeout: float | None = None) -> wire_7.CyclesSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["Set"]
        return await cast(Awaitable[wire_7.CyclesSetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_7.CyclesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_7.CyclesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Cycles").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_7.CyclesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class DocumentsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_8.DocumentsGetRequest, *, timeout: float | None = None) -> wire_8.DocumentsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Documents").methods_by_name["Get"]
        return await cast(Awaitable[wire_8.DocumentsGetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_8.DocumentsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_8.DocumentsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Documents").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_8.DocumentsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class EvidenceResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.health_events = EvidenceHealthEventsResource(client)
        self.notes = EvidenceNotesResource(client)
        self.records = EvidenceRecordsResource(client)
        self.samples = EvidenceSamplesResource(client)
        self.traceability = EvidenceTraceabilityResource(client)

    async def archive(self, request: wire_14.EvidenceArchiveRequest, *, timeout: float | None = None) -> wire_14.EvidenceArchiveResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Evidence").methods_by_name["Archive"]
        return await cast(Awaitable[wire_14.EvidenceArchiveResponse], self._client.call(request, method=method, timeout=timeout))

    async def statistics(
        self, request: wire_14.EvidenceStatisticsRequest, *, timeout: float | None = None
    ) -> wire_14.EvidenceStatisticsResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Evidence").methods_by_name["Statistics"]
        return await cast(Awaitable[wire_14.EvidenceStatisticsResponse], self._client.call(request, method=method, timeout=timeout))


class ExecutionsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.commands = ExecutionsCommandsResource(client)
        self.progress = ExecutionsProgressResource(client)
        self.schemas = ExecutionsSchemasResource(client)

    async def cancel(self, request: wire_18.ExecutionsCancelRequest, *, timeout: float | None = None) -> wire_18.ExecutionsCancelResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Cancel"]
        return await cast(Awaitable[wire_18.ExecutionsCancelResponse], self._client.call(request, method=method, timeout=timeout))

    async def create(self, request: wire_18.ExecutionsCreateRequest, *, timeout: float | None = None) -> wire_18.ExecutionsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Create"]
        return await cast(Awaitable[wire_18.ExecutionsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def delete(self, request: wire_18.ExecutionsDeleteRequest, *, timeout: float | None = None) -> wire_18.ExecutionsDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Delete"]
        return await cast(Awaitable[wire_18.ExecutionsDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_18.ExecutionsGetRequest, *, timeout: float | None = None) -> wire_18.ExecutionsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Get"]
        return await cast(Awaitable[wire_18.ExecutionsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_18.ExecutionsListRequest, *, timeout: float | None = None) -> wire_18.ExecutionsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["List"]
        return await cast(Awaitable[wire_18.ExecutionsListResponse], self._client.call(request, method=method, timeout=timeout))

    async def parameters(
        self, request: wire_18.ExecutionsParametersRequest, *, timeout: float | None = None
    ) -> wire_18.ExecutionsParametersResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Parameters"]
        return await cast(Awaitable[wire_18.ExecutionsParametersResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_18.ExecutionsSetRequest, *, timeout: float | None = None) -> wire_18.ExecutionsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Set"]
        return await cast(Awaitable[wire_18.ExecutionsSetResponse], self._client.call(request, method=method, timeout=timeout))

    async def start(self, request: wire_18.ExecutionsStartRequest, *, timeout: float | None = None) -> wire_18.ExecutionsStartResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Start"]
        return await cast(Awaitable[wire_18.ExecutionsStartResponse], self._client.call(request, method=method, timeout=timeout))

    async def steps(self, request: wire_18.ExecutionsStepsRequest, *, timeout: float | None = None) -> wire_18.ExecutionsStepsResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Steps"]
        return await cast(Awaitable[wire_18.ExecutionsStepsResponse], self._client.call(request, method=method, timeout=timeout))

    async def stop(self, request: wire_18.ExecutionsStopRequest, *, timeout: float | None = None) -> wire_18.ExecutionsStopResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Stop"]
        return await cast(Awaitable[wire_18.ExecutionsStopResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_18.ExecutionsWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_18.ExecutionsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Executions").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_18.ExecutionsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class FilesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.uploads = FilesUploadsResource(client)

    async def delete(self, request: wire_20.FilesDeleteRequest, *, timeout: float | None = None) -> wire_20.FilesDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Files").methods_by_name["Delete"]
        return await cast(Awaitable[wire_20.FilesDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_20.FilesGetRequest, *, timeout: float | None = None) -> wire_20.FilesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Files").methods_by_name["Get"]
        return await cast(Awaitable[wire_20.FilesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def read(self, request: wire_20.FilesReadRequest, *, timeout: float | None = None) -> wire_20.FilesReadResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Files").methods_by_name["Read"]
        return await cast(Awaitable[wire_20.FilesReadResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_20.FilesWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_20.FilesWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Files").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_20.FilesWatchResponse], self._client.call(request, method=method, timeout=timeout))


class InvocationsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def cancel(self, request: wire_39.InvocationsCancelRequest, *, timeout: float | None = None) -> wire_39.InvocationsCancelResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Invocations").methods_by_name["Cancel"]
        return await cast(Awaitable[wire_39.InvocationsCancelResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_39.InvocationsGetRequest, *, timeout: float | None = None) -> wire_39.InvocationsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Invocations").methods_by_name["Get"]
        return await cast(Awaitable[wire_39.InvocationsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def invoke(self, request: wire_39.InvocationsInvokeRequest, *, timeout: float | None = None) -> wire_39.InvocationsInvokeResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Invocations").methods_by_name["Invoke"]
        return await cast(Awaitable[wire_39.InvocationsInvokeResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(
        self, request: wire_39.InvocationsWatchRequest, *, timeout: float | None = None
    ) -> AsyncIterator[wire_39.InvocationsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Invocations").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_39.InvocationsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class MediaResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.captures = MediaCapturesResource(client)


class PlotsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.settings = PlotsSettingsResource(client)
        self.snapshots = PlotsSnapshotsResource(client)


class PluginsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.settings = PluginsSettingsResource(client)

    async def create(self, request: wire_40.PluginsCreateRequest, *, timeout: float | None = None) -> wire_40.PluginsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Create"]
        return await cast(Awaitable[wire_40.PluginsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def delete(self, request: wire_40.PluginsDeleteRequest, *, timeout: float | None = None) -> wire_40.PluginsDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Delete"]
        return await cast(Awaitable[wire_40.PluginsDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_40.PluginsGetRequest, *, timeout: float | None = None) -> wire_40.PluginsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Get"]
        return await cast(Awaitable[wire_40.PluginsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_40.PluginsListRequest, *, timeout: float | None = None) -> wire_40.PluginsListResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["List"]
        return await cast(Awaitable[wire_40.PluginsListResponse], self._client.call(request, method=method, timeout=timeout))

    async def restart(self, request: wire_40.PluginsRestartRequest, *, timeout: float | None = None) -> wire_40.PluginsRestartResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Restart"]
        return await cast(Awaitable[wire_40.PluginsRestartResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_40.PluginsSetRequest, *, timeout: float | None = None) -> wire_40.PluginsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Set"]
        return await cast(Awaitable[wire_40.PluginsSetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_40.PluginsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_40.PluginsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Plugins").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_40.PluginsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class PromptDefinitionsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(
        self, request: wire_25.PromptDefinitionsGetRequest, *, timeout: float | None = None
    ) -> wire_25.PromptDefinitionsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.PromptDefinitions").methods_by_name["Get"]
        return await cast(Awaitable[wire_25.PromptDefinitionsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(
        self, request: wire_25.PromptDefinitionsListRequest, *, timeout: float | None = None
    ) -> wire_25.PromptDefinitionsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.PromptDefinitions").methods_by_name["List"]
        return await cast(Awaitable[wire_25.PromptDefinitionsListResponse], self._client.call(request, method=method, timeout=timeout))


class PromptsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.answers = PromptsAnswersResource(client)

    async def get(self, request: wire_27.PromptsGetRequest, *, timeout: float | None = None) -> wire_27.PromptsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Prompts").methods_by_name["Get"]
        return await cast(Awaitable[wire_27.PromptsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_27.PromptsListRequest, *, timeout: float | None = None) -> wire_27.PromptsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Prompts").methods_by_name["List"]
        return await cast(Awaitable[wire_27.PromptsListResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_27.PromptsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_27.PromptsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Prompts").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_27.PromptsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class RecordSchemasResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_28.RecordSchemasGetRequest, *, timeout: float | None = None) -> wire_28.RecordSchemasGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.RecordSchemas").methods_by_name["Get"]
        return await cast(Awaitable[wire_28.RecordSchemasGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_28.RecordSchemasListRequest, *, timeout: float | None = None) -> wire_28.RecordSchemasListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.RecordSchemas").methods_by_name["List"]
        return await cast(Awaitable[wire_28.RecordSchemasListResponse], self._client.call(request, method=method, timeout=timeout))


class ReportsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def create(self, request: wire_29.ReportsCreateRequest, *, timeout: float | None = None) -> wire_29.ReportsCreateResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Reports").methods_by_name["Create"]
        return await cast(Awaitable[wire_29.ReportsCreateResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_29.ReportsGetRequest, *, timeout: float | None = None) -> wire_29.ReportsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Reports").methods_by_name["Get"]
        return await cast(Awaitable[wire_29.ReportsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def publish(self, request: wire_29.ReportsPublishRequest, *, timeout: float | None = None) -> wire_29.ReportsPublishResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Reports").methods_by_name["Publish"]
        return await cast(Awaitable[wire_29.ReportsPublishResponse], self._client.call(request, method=method, timeout=timeout))


class SecretsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def delete(self, request: wire_30.SecretsDeleteRequest, *, timeout: float | None = None) -> wire_30.SecretsDeleteResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Delete"]
        return await cast(Awaitable[wire_30.SecretsDeleteResponse], self._client.call(request, method=method, timeout=timeout))

    async def export(self, request: wire_30.SecretsExportRequest, *, timeout: float | None = None) -> wire_30.SecretsExportResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Export"]
        return await cast(Awaitable[wire_30.SecretsExportResponse], self._client.call(request, method=method, timeout=timeout))

    async def get(self, request: wire_30.SecretsGetRequest, *, timeout: float | None = None) -> wire_30.SecretsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Get"]
        return await cast(Awaitable[wire_30.SecretsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def import_(self, request: wire_30.SecretsImportRequest, *, timeout: float | None = None) -> wire_30.SecretsImportResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Import"]
        return await cast(Awaitable[wire_30.SecretsImportResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_30.SecretsListRequest, *, timeout: float | None = None) -> wire_30.SecretsListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["List"]
        return await cast(Awaitable[wire_30.SecretsListResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_30.SecretsSetRequest, *, timeout: float | None = None) -> wire_30.SecretsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Set"]
        return await cast(Awaitable[wire_30.SecretsSetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_30.SecretsWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_30.SecretsWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Secrets").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_30.SecretsWatchResponse], self._client.call(request, method=method, timeout=timeout))


class SettingsResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_31.SettingsGetRequest, *, timeout: float | None = None) -> wire_31.SettingsGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Settings").methods_by_name["Get"]
        return await cast(Awaitable[wire_31.SettingsGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def schema(self, request: wire_31.SettingsSchemaRequest, *, timeout: float | None = None) -> wire_31.SettingsSchemaResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Settings").methods_by_name["Schema"]
        return await cast(Awaitable[wire_31.SettingsSchemaResponse], self._client.call(request, method=method, timeout=timeout))

    async def set(self, request: wire_31.SettingsSetRequest, *, timeout: float | None = None) -> wire_31.SettingsSetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Settings").methods_by_name["Set"]
        return await cast(Awaitable[wire_31.SettingsSetResponse], self._client.call(request, method=method, timeout=timeout))


class StatusResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.diagnostics = StatusDiagnosticsResource(client)
        self.logs = StatusLogsResource(client)
        self.trace_events = StatusTraceEventsResource(client)

    async def get(self, request: wire_44.StatusGetRequest, *, timeout: float | None = None) -> wire_44.StatusGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Status").methods_by_name["Get"]
        return await cast(Awaitable[wire_44.StatusGetResponse], self._client.call(request, method=method, timeout=timeout))

    def watch(self, request: wire_44.StatusWatchRequest, *, timeout: float | None = None) -> AsyncIterator[wire_44.StatusWatchResponse]:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Status").methods_by_name["Watch"]
        return cast(AsyncIterator[wire_44.StatusWatchResponse], self._client.call(request, method=method, timeout=timeout))


class StorageResource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.sources = StorageSourcesResource(client)


class StreamSchemasResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_33.StreamSchemasGetRequest, *, timeout: float | None = None) -> wire_33.StreamSchemasGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.StreamSchemas").methods_by_name["Get"]
        return await cast(Awaitable[wire_33.StreamSchemasGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_33.StreamSchemasListRequest, *, timeout: float | None = None) -> wire_33.StreamSchemasListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.StreamSchemas").methods_by_name["List"]
        return await cast(Awaitable[wire_33.StreamSchemasListResponse], self._client.call(request, method=method, timeout=timeout))


class SuitesResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_34.SuitesGetRequest, *, timeout: float | None = None) -> wire_34.SuitesGetResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Suites").methods_by_name["Get"]
        return await cast(Awaitable[wire_34.SuitesGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def list(self, request: wire_34.SuitesListRequest, *, timeout: float | None = None) -> wire_34.SuitesListResponse:
        method = descriptor_pool.Default().FindServiceByName("dokime.client.v1.Suites").methods_by_name["List"]
        return await cast(Awaitable[wire_34.SuitesListResponse], self._client.call(request, method=method, timeout=timeout))


class TimeResource:
    def __init__(self, client: Caller) -> None:
        self._client = client

    async def get(self, request: wire_45.TimeGetRequest, *, timeout: float | None = None) -> wire_45.TimeGetResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Time").methods_by_name["Get"]
        return await cast(Awaitable[wire_45.TimeGetResponse], self._client.call(request, method=method, timeout=timeout))

    async def sleep(self, request: wire_45.TimeSleepRequest, *, timeout: float | None = None) -> wire_45.TimeSleepResponse:
        method = descriptor_pool.Default().FindServiceByName("harmos.v1.Time").methods_by_name["Sleep"]
        return await cast(Awaitable[wire_45.TimeSleepResponse], self._client.call(request, method=method, timeout=timeout))


class Resource:
    def __init__(self, client: Caller) -> None:
        self._client = client
        self.access = AccessResource(client)
        self.aggregates = AggregatesResource(client)
        self.analyses = AnalysesResource(client)
        self.backend = BackendResource(client)
        self.cases = CasesResource(client)
        self.catalog_entries = CatalogEntriesResource(client)
        self.content = ContentResource(client)
        self.cycles = CyclesResource(client)
        self.documents = DocumentsResource(client)
        self.evidence = EvidenceResource(client)
        self.executions = ExecutionsResource(client)
        self.files = FilesResource(client)
        self.invocations = InvocationsResource(client)
        self.media = MediaResource(client)
        self.plots = PlotsResource(client)
        self.plugins = PluginsResource(client)
        self.prompt_definitions = PromptDefinitionsResource(client)
        self.prompts = PromptsResource(client)
        self.record_schemas = RecordSchemasResource(client)
        self.reports = ReportsResource(client)
        self.secrets = SecretsResource(client)
        self.settings = SettingsResource(client)
        self.status = StatusResource(client)
        self.storage = StorageResource(client)
        self.stream_schemas = StreamSchemasResource(client)
        self.suites = SuitesResource(client)
        self.time = TimeResource(client)
