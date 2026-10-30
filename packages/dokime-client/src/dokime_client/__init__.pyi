# ***
#
# Harmos
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generated Operation Bindings From The Selected Producer Catalog"""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from google.protobuf.message import Message

import dokime_client._wire.dokime.v1.AppearanceMessages.GetAppearance.messages_pb2 as _wire_0
import dokime_client._wire.dokime.v1.BackendMessages.CollectDiagnostics.messages_pb2 as _wire_2
import dokime_client._wire.dokime.v1.BackendMessages.GetAvailability.messages_pb2 as _wire_4
import dokime_client._wire.dokime.v1.BackendMessages.GetHealth.messages_pb2 as _wire_5
import dokime_client._wire.dokime.v1.BackendMessages.GetIdentity.messages_pb2 as _wire_6
import dokime_client._wire.dokime.v1.BackendMessages.WatchLogs.messages_pb2 as _wire_7
import dokime_client._wire.dokime.v1.BackendMessages.WatchTrace.messages_pb2 as _wire_8
import dokime_client._wire.dokime.v1.CatalogMessages.GetRecordSchema.messages_pb2 as _wire_9
import dokime_client._wire.dokime.v1.CatalogMessages.GetSuite.messages_pb2 as _wire_11
import dokime_client._wire.dokime.v1.CatalogMessages.GetTelemetrySchema.messages_pb2 as _wire_12
import dokime_client._wire.dokime.v1.CatalogMessages.ListRecordSchemas.messages_pb2 as _wire_13
import dokime_client._wire.dokime.v1.CatalogMessages.ListSuites.messages_pb2 as _wire_14
import dokime_client._wire.dokime.v1.CatalogMessages.ListTelemetrySchemas.messages_pb2 as _wire_15
import dokime_client._wire.dokime.v1.ContentMessages.DownloadAppearanceAsset.messages_pb2 as _wire_16
import dokime_client._wire.dokime.v1.ContentMessages.DownloadArtifact.messages_pb2 as _wire_18
import dokime_client._wire.dokime.v1.ContentMessages.DownloadEvidence.messages_pb2 as _wire_19
import dokime_client._wire.dokime.v1.ContentMessages.DownloadExecutionFile.messages_pb2 as _wire_20
import dokime_client._wire.dokime.v1.ContentMessages.FollowExecutionFile.messages_pb2 as _wire_21
import dokime_client._wire.dokime.v1.ContentMessages.ListArtifacts.messages_pb2 as _wire_22
import dokime_client._wire.dokime.v1.ContentMessages.ListExecutionFiles.messages_pb2 as _wire_23
import dokime_client._wire.dokime.v1.ContentMessages.ReadExecutionFile.messages_pb2 as _wire_24
import dokime_client._wire.dokime.v1.ContentMessages.UploadArtifact.messages_pb2 as _wire_25
import dokime_client._wire.dokime.v1.DocumentsMessages.CaptureCurrentDocument.messages_pb2 as _wire_26
import dokime_client._wire.dokime.v1.DocumentsMessages.UpdatePlotSettings.messages_pb2 as _wire_28
import dokime_client._wire.dokime.v1.EvidenceMessages.GetCapturedRecordSchema.messages_pb2 as _wire_29
import dokime_client._wire.dokime.v1.EvidenceMessages.ListRecords.messages_pb2 as _wire_31
import dokime_client._wire.dokime.v1.EvidenceMessages.WatchRecords.messages_pb2 as _wire_32
import dokime_client._wire.dokime.v1.ExecutionMessages.AppendExecutions.messages_pb2 as _wire_33
import dokime_client._wire.dokime.v1.ExecutionMessages.CancelExecution.messages_pb2 as _wire_35
import dokime_client._wire.dokime.v1.ExecutionMessages.CopyCycle.messages_pb2 as _wire_36
import dokime_client._wire.dokime.v1.ExecutionMessages.CreateCycle.messages_pb2 as _wire_37
import dokime_client._wire.dokime.v1.ExecutionMessages.DeleteCase.messages_pb2 as _wire_38
import dokime_client._wire.dokime.v1.ExecutionMessages.GetCase.messages_pb2 as _wire_39
import dokime_client._wire.dokime.v1.ExecutionMessages.GetCycle.messages_pb2 as _wire_40
import dokime_client._wire.dokime.v1.ExecutionMessages.GetExecution.messages_pb2 as _wire_41
import dokime_client._wire.dokime.v1.ExecutionMessages.GetExecutionCommand.messages_pb2 as _wire_42
import dokime_client._wire.dokime.v1.ExecutionMessages.ListCaseParameters.messages_pb2 as _wire_43
import dokime_client._wire.dokime.v1.ExecutionMessages.ListCases.messages_pb2 as _wire_44
import dokime_client._wire.dokime.v1.ExecutionMessages.ListCycles.messages_pb2 as _wire_45
import dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionParameters.messages_pb2 as _wire_46
import dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutions.messages_pb2 as _wire_48
import dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionSteps.messages_pb2 as _wire_47
import dokime_client._wire.dokime.v1.ExecutionMessages.PauseCycle.messages_pb2 as _wire_49
import dokime_client._wire.dokime.v1.ExecutionMessages.PauseExecution.messages_pb2 as _wire_50
import dokime_client._wire.dokime.v1.ExecutionMessages.RenameCycle.messages_pb2 as _wire_51
import dokime_client._wire.dokime.v1.ExecutionMessages.RenameExecution.messages_pb2 as _wire_52
import dokime_client._wire.dokime.v1.ExecutionMessages.ResumeCycle.messages_pb2 as _wire_53
import dokime_client._wire.dokime.v1.ExecutionMessages.ResumeExecution.messages_pb2 as _wire_54
import dokime_client._wire.dokime.v1.ExecutionMessages.SealCycle.messages_pb2 as _wire_55
import dokime_client._wire.dokime.v1.ExecutionMessages.SetCycleArchived.messages_pb2 as _wire_56
import dokime_client._wire.dokime.v1.ExecutionMessages.SetExecutionArchived.messages_pb2 as _wire_57
import dokime_client._wire.dokime.v1.ExecutionMessages.StartExecution.messages_pb2 as _wire_58
import dokime_client._wire.dokime.v1.ExecutionMessages.StopExecution.messages_pb2 as _wire_59
import dokime_client._wire.dokime.v1.ExecutionMessages.SubmitExecutionCommand.messages_pb2 as _wire_60
import dokime_client._wire.dokime.v1.ExecutionMessages.UpdateCyclePolicy.messages_pb2 as _wire_61
import dokime_client._wire.dokime.v1.FilesMessages.FileWatch.messages_pb2 as _wire_62
import dokime_client._wire.dokime.v1.HealthMessages.ListHealth.messages_pb2 as _wire_64
import dokime_client._wire.dokime.v1.HistoryMessages.BrowseCycles.messages_pb2 as _wire_66
import dokime_client._wire.dokime.v1.HistoryMessages.BrowseExecutions.messages_pb2 as _wire_68
import dokime_client._wire.dokime.v1.HistoryMessages.DeleteExecution.messages_pb2 as _wire_69
import dokime_client._wire.dokime.v1.HistoryMessages.DownloadCheckpointArtifact.messages_pb2 as _wire_70
import dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointContext.messages_pb2 as _wire_71
import dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointDocument.messages_pb2 as _wire_72
import dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointEvidence.messages_pb2 as _wire_73
import dokime_client._wire.dokime.v1.HistoryMessages.GetExecutionCheckpoint.messages_pb2 as _wire_74
import dokime_client._wire.dokime.v1.HistoryMessages.ListCheckpointRecords.messages_pb2 as _wire_75
import dokime_client._wire.dokime.v1.HistoryMessages.ListCheckpointTelemetry.messages_pb2 as _wire_76
import dokime_client._wire.dokime.v1.HistoryMessages.ReadCheckpointOutput.messages_pb2 as _wire_77
import dokime_client._wire.dokime.v1.MediaMessages.AudioRecording.messages_pb2 as _wire_78
import dokime_client._wire.dokime.v1.MediaMessages.CameraRecording.messages_pb2 as _wire_80
import dokime_client._wire.dokime.v1.MediaMessages.Photo.messages_pb2 as _wire_81
import dokime_client._wire.dokime.v1.MediaMessages.ScreenRecording.messages_pb2 as _wire_82
import dokime_client._wire.dokime.v1.MediaMessages.Screenshot.messages_pb2 as _wire_83
import dokime_client._wire.dokime.v1.NotesMessages.CreateNote.messages_pb2 as _wire_84
import dokime_client._wire.dokime.v1.NotesMessages.EditNote.messages_pb2 as _wire_86
import dokime_client._wire.dokime.v1.NotesMessages.ListNotes.messages_pb2 as _wire_87
import dokime_client._wire.dokime.v1.ObservationMessages.GetRuntimeShell.messages_pb2 as _wire_88
import dokime_client._wire.dokime.v1.ObservationMessages.GetTraceability.messages_pb2 as _wire_90
import dokime_client._wire.dokime.v1.ObservationMessages.WatchExecutionProgress.messages_pb2 as _wire_91
import dokime_client._wire.dokime.v1.ObservationMessages.WatchRuntime.messages_pb2 as _wire_92
import dokime_client._wire.dokime.v1.PlanningMessages.MaterializeTemplate.messages_pb2 as _wire_93
import dokime_client._wire.dokime.v1.PlanningMessages.PreviewSelection.messages_pb2 as _wire_95
import dokime_client._wire.dokime.v1.PlanningMessages.RegisterSelection.messages_pb2 as _wire_96
import dokime_client._wire.dokime.v1.PlotsMessages.ListAnalyses.messages_pb2 as _wire_97
import dokime_client._wire.dokime.v1.PlotsMessages.QuerySnapshots.messages_pb2 as _wire_99
import dokime_client._wire.dokime.v1.PlotsMessages.Watch.messages_pb2 as _wire_100
import dokime_client._wire.dokime.v1.PluginsMessages.GetInstallation.messages_pb2 as _wire_101
import dokime_client._wire.dokime.v1.PluginsMessages.Install.messages_pb2 as _wire_103
import dokime_client._wire.dokime.v1.PluginsMessages.ListAdmittedPlugins.messages_pb2 as _wire_104
import dokime_client._wire.dokime.v1.PluginsMessages.ListInstallations.messages_pb2 as _wire_105
import dokime_client._wire.dokime.v1.PluginsMessages.Remove.messages_pb2 as _wire_106
import dokime_client._wire.dokime.v1.PluginsMessages.SetEnabled.messages_pb2 as _wire_107
import dokime_client._wire.dokime.v1.PromptsMessages.GetDefinition.messages_pb2 as _wire_108
import dokime_client._wire.dokime.v1.PromptsMessages.GetPrompt.messages_pb2 as _wire_110
import dokime_client._wire.dokime.v1.PromptsMessages.ListDefinitions.messages_pb2 as _wire_111
import dokime_client._wire.dokime.v1.PromptsMessages.ListPending.messages_pb2 as _wire_112
import dokime_client._wire.dokime.v1.PromptsMessages.Respond.messages_pb2 as _wire_113
import dokime_client._wire.dokime.v1.QueryMessages.Aggregate.messages_pb2 as _wire_114
import dokime_client._wire.dokime.v1.RemoteMessages.CreatePairing.messages_pb2 as _wire_116
import dokime_client._wire.dokime.v1.RemoteMessages.ListPairedClients.messages_pb2 as _wire_118
import dokime_client._wire.dokime.v1.RemoteMessages.ListSessions.messages_pb2 as _wire_119
import dokime_client._wire.dokime.v1.RemoteMessages.ReadListener.messages_pb2 as _wire_120
import dokime_client._wire.dokime.v1.RemoteMessages.RegenerateCertificate.messages_pb2 as _wire_121
import dokime_client._wire.dokime.v1.RemoteMessages.RevokePairedClient.messages_pb2 as _wire_122
import dokime_client._wire.dokime.v1.RemoteMessages.UpdateListener.messages_pb2 as _wire_123
import dokime_client._wire.dokime.v1.ReportsMessages.DownloadCycleReport.messages_pb2 as _wire_124
import dokime_client._wire.dokime.v1.ReportsMessages.DownloadExecutionReport.messages_pb2 as _wire_126
import dokime_client._wire.dokime.v1.ReportsMessages.MaterializeReport.messages_pb2 as _wire_127
import dokime_client._wire.dokime.v1.RuntimeMessages.GetHealth.messages_pb2 as _wire_128
import dokime_client._wire.dokime.v1.RuntimeMessages.GetIdentity.messages_pb2 as _wire_130
import dokime_client._wire.dokime.v1.SecretsMessages.Export.messages_pb2 as _wire_131
import dokime_client._wire.dokime.v1.SecretsMessages.Import.messages_pb2 as _wire_133
import dokime_client._wire.dokime.v1.SecretsMessages.List.messages_pb2 as _wire_134
import dokime_client._wire.dokime.v1.SecretsMessages.Lookup.messages_pb2 as _wire_135
import dokime_client._wire.dokime.v1.SecretsMessages.Preview.messages_pb2 as _wire_136
import dokime_client._wire.dokime.v1.SecretsMessages.Put.messages_pb2 as _wire_137
import dokime_client._wire.dokime.v1.SecretsMessages.Remove.messages_pb2 as _wire_138
import dokime_client._wire.dokime.v1.SettingsMessages.GetPluginSettings.messages_pb2 as _wire_139
import dokime_client._wire.dokime.v1.SettingsMessages.GetSettings.messages_pb2 as _wire_141
import dokime_client._wire.dokime.v1.SettingsMessages.ListStorageSources.messages_pb2 as _wire_142
import dokime_client._wire.dokime.v1.SettingsMessages.ReadApplicationSettings.messages_pb2 as _wire_143
import dokime_client._wire.dokime.v1.SettingsMessages.ReorderStorageSources.messages_pb2 as _wire_144
import dokime_client._wire.dokime.v1.SettingsMessages.SetStorageSourceEnabled.messages_pb2 as _wire_145
import dokime_client._wire.dokime.v1.SettingsMessages.UpdateApplicationSettings.messages_pb2 as _wire_146
import dokime_client._wire.dokime.v1.SettingsMessages.UpdatePluginSettings.messages_pb2 as _wire_147
import dokime_client._wire.dokime.v1.TelemetryMessages.ListSamples.messages_pb2 as _wire_148
import dokime_client._wire.dokime.v1.TelemetryMessages.WatchSamples.messages_pb2 as _wire_150
import dokime_client._wire.harmos.host.v1.HostMessages.Catalog.messages_pb2 as _wire_151

# Local Imports
from .operation import Invoker as Invoker
from .operation import Operation as Operation
from .operation import OperationError as OperationError
from .operation import OperationHandle as OperationHandle
from .operation import ResponseStream as ResponseStream

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

DokimeAppearanceGetAppearanceRequest = _wire_0.Row1
DokimeAppearanceGetAppearanceResponse = _wire_0.Row2

DOKIME_APPEARANCE_GET_APPEARANCE: Operation[DokimeAppearanceGetAppearanceRequest, DokimeAppearanceGetAppearanceResponse, Message, Message]

DokimeBackendCollectDiagnosticsRequest = _wire_2.Row1
DokimeBackendCollectDiagnosticsResponse = _wire_2.Row2
DokimeBackendCollectDiagnosticsOutput = _wire_2.Row3

DOKIME_BACKEND_COLLECT_DIAGNOSTICS: Operation[
    DokimeBackendCollectDiagnosticsRequest, DokimeBackendCollectDiagnosticsResponse, Message, DokimeBackendCollectDiagnosticsOutput
]

DokimeBackendGetAvailabilityRequest = _wire_4.Row1
DokimeBackendGetAvailabilityResponse = _wire_4.Row2

DOKIME_BACKEND_GET_AVAILABILITY: Operation[DokimeBackendGetAvailabilityRequest, DokimeBackendGetAvailabilityResponse, Message, Message]

DokimeBackendGetHealthRequest = _wire_5.Row1
DokimeBackendGetHealthResponse = _wire_5.Row2

DOKIME_BACKEND_GET_HEALTH: Operation[DokimeBackendGetHealthRequest, DokimeBackendGetHealthResponse, Message, Message]

DokimeBackendGetIdentityRequest = _wire_6.Row1
DokimeBackendGetIdentityResponse = _wire_6.Row2

DOKIME_BACKEND_GET_IDENTITY: Operation[DokimeBackendGetIdentityRequest, DokimeBackendGetIdentityResponse, Message, Message]

DokimeBackendWatchLogsRequest = _wire_7.Row1
DokimeBackendWatchLogsResponse = _wire_7.Row2
DokimeBackendWatchLogsOutput = _wire_7.Row3

DOKIME_BACKEND_WATCH_LOGS: Operation[DokimeBackendWatchLogsRequest, DokimeBackendWatchLogsResponse, Message, DokimeBackendWatchLogsOutput]

DokimeBackendWatchTraceRequest = _wire_8.Row1
DokimeBackendWatchTraceResponse = _wire_8.Row2
DokimeBackendWatchTraceOutput = _wire_8.Row3

DOKIME_BACKEND_WATCH_TRACE: Operation[
    DokimeBackendWatchTraceRequest, DokimeBackendWatchTraceResponse, Message, DokimeBackendWatchTraceOutput
]

DokimeCatalogGetRecordSchemaRequest = _wire_9.Row1
DokimeCatalogGetRecordSchemaResponse = _wire_9.Row2

DOKIME_CATALOG_GET_RECORD_SCHEMA: Operation[DokimeCatalogGetRecordSchemaRequest, DokimeCatalogGetRecordSchemaResponse, Message, Message]

DokimeCatalogGetSuiteRequest = _wire_11.Row1
DokimeCatalogGetSuiteResponse = _wire_11.Row2

DOKIME_CATALOG_GET_SUITE: Operation[DokimeCatalogGetSuiteRequest, DokimeCatalogGetSuiteResponse, Message, Message]

DokimeCatalogGetTelemetrySchemaRequest = _wire_12.Row1
DokimeCatalogGetTelemetrySchemaResponse = _wire_12.Row2

DOKIME_CATALOG_GET_TELEMETRY_SCHEMA: Operation[
    DokimeCatalogGetTelemetrySchemaRequest, DokimeCatalogGetTelemetrySchemaResponse, Message, Message
]

DokimeCatalogListRecordSchemasRequest = _wire_13.Row1
DokimeCatalogListRecordSchemasResponse = _wire_13.Row2

DOKIME_CATALOG_LIST_RECORD_SCHEMAS: Operation[
    DokimeCatalogListRecordSchemasRequest, DokimeCatalogListRecordSchemasResponse, Message, Message
]

DokimeCatalogListSuitesRequest = _wire_14.Row1
DokimeCatalogListSuitesResponse = _wire_14.Row2

DOKIME_CATALOG_LIST_SUITES: Operation[DokimeCatalogListSuitesRequest, DokimeCatalogListSuitesResponse, Message, Message]

DokimeCatalogListTelemetrySchemasRequest = _wire_15.Row1
DokimeCatalogListTelemetrySchemasResponse = _wire_15.Row2

DOKIME_CATALOG_LIST_TELEMETRY_SCHEMAS: Operation[
    DokimeCatalogListTelemetrySchemasRequest, DokimeCatalogListTelemetrySchemasResponse, Message, Message
]

DokimeContentDownloadAppearanceAssetRequest = _wire_16.Row1
DokimeContentDownloadAppearanceAssetResponse = _wire_16.Row2
DokimeContentDownloadAppearanceAssetOutput = _wire_16.Row3

DOKIME_CONTENT_DOWNLOAD_APPEARANCE_ASSET: Operation[
    DokimeContentDownloadAppearanceAssetRequest,
    DokimeContentDownloadAppearanceAssetResponse,
    Message,
    DokimeContentDownloadAppearanceAssetOutput,
]

DokimeContentDownloadArtifactRequest = _wire_18.Row1
DokimeContentDownloadArtifactResponse = _wire_18.Row2
DokimeContentDownloadArtifactOutput = _wire_18.Row3

DOKIME_CONTENT_DOWNLOAD_ARTIFACT: Operation[
    DokimeContentDownloadArtifactRequest, DokimeContentDownloadArtifactResponse, Message, DokimeContentDownloadArtifactOutput
]

DokimeContentDownloadEvidenceRequest = _wire_19.Row1
DokimeContentDownloadEvidenceResponse = _wire_19.Row2
DokimeContentDownloadEvidenceOutput = _wire_19.Row3

DOKIME_CONTENT_DOWNLOAD_EVIDENCE: Operation[
    DokimeContentDownloadEvidenceRequest, DokimeContentDownloadEvidenceResponse, Message, DokimeContentDownloadEvidenceOutput
]

DokimeContentDownloadExecutionFileRequest = _wire_20.Row1
DokimeContentDownloadExecutionFileResponse = _wire_20.Row2
DokimeContentDownloadExecutionFileOutput = _wire_20.Row3

DOKIME_CONTENT_DOWNLOAD_EXECUTION_FILE: Operation[
    DokimeContentDownloadExecutionFileRequest, DokimeContentDownloadExecutionFileResponse, Message, DokimeContentDownloadExecutionFileOutput
]

DokimeContentFollowExecutionFileRequest = _wire_21.Row1
DokimeContentFollowExecutionFileResponse = _wire_21.Row2
DokimeContentFollowExecutionFileOutput = _wire_21.Row3

DOKIME_CONTENT_FOLLOW_EXECUTION_FILE: Operation[
    DokimeContentFollowExecutionFileRequest, DokimeContentFollowExecutionFileResponse, Message, DokimeContentFollowExecutionFileOutput
]

DokimeContentListArtifactsRequest = _wire_22.Row1
DokimeContentListArtifactsResponse = _wire_22.Row2

DOKIME_CONTENT_LIST_ARTIFACTS: Operation[DokimeContentListArtifactsRequest, DokimeContentListArtifactsResponse, Message, Message]

DokimeContentListExecutionFilesRequest = _wire_23.Row1
DokimeContentListExecutionFilesResponse = _wire_23.Row2

DOKIME_CONTENT_LIST_EXECUTION_FILES: Operation[
    DokimeContentListExecutionFilesRequest, DokimeContentListExecutionFilesResponse, Message, Message
]

DokimeContentReadExecutionFileRequest = _wire_24.Row1
DokimeContentReadExecutionFileResponse = _wire_24.Row2

DOKIME_CONTENT_READ_EXECUTION_FILE: Operation[
    DokimeContentReadExecutionFileRequest, DokimeContentReadExecutionFileResponse, Message, Message
]

DokimeContentUploadArtifactRequest = _wire_25.Row1
DokimeContentUploadArtifactResponse = _wire_25.Row2
DokimeContentUploadArtifactInput = _wire_25.Row3

DOKIME_CONTENT_UPLOAD_ARTIFACT: Operation[
    DokimeContentUploadArtifactRequest, DokimeContentUploadArtifactResponse, DokimeContentUploadArtifactInput, Message
]

DokimeDocumentsCaptureCurrentDocumentRequest = _wire_26.Row1
DokimeDocumentsCaptureCurrentDocumentResponse = _wire_26.Row2
DokimeDocumentsCaptureCurrentDocumentOutput = _wire_26.Row3

DOKIME_DOCUMENTS_CAPTURE_CURRENT_DOCUMENT: Operation[
    DokimeDocumentsCaptureCurrentDocumentRequest,
    DokimeDocumentsCaptureCurrentDocumentResponse,
    Message,
    DokimeDocumentsCaptureCurrentDocumentOutput,
]

DokimeDocumentsUpdatePlotSettingsRequest = _wire_28.Row1
DokimeDocumentsUpdatePlotSettingsResponse = _wire_28.Row2

DOKIME_DOCUMENTS_UPDATE_PLOT_SETTINGS: Operation[
    DokimeDocumentsUpdatePlotSettingsRequest, DokimeDocumentsUpdatePlotSettingsResponse, Message, Message
]

DokimeEvidenceGetCapturedRecordSchemaRequest = _wire_29.Row1
DokimeEvidenceGetCapturedRecordSchemaResponse = _wire_29.Row2

DOKIME_EVIDENCE_GET_CAPTURED_RECORD_SCHEMA: Operation[
    DokimeEvidenceGetCapturedRecordSchemaRequest, DokimeEvidenceGetCapturedRecordSchemaResponse, Message, Message
]

DokimeEvidenceListRecordsRequest = _wire_31.Row1
DokimeEvidenceListRecordsResponse = _wire_31.Row2

DOKIME_EVIDENCE_LIST_RECORDS: Operation[DokimeEvidenceListRecordsRequest, DokimeEvidenceListRecordsResponse, Message, Message]

DokimeEvidenceWatchRecordsRequest = _wire_32.Row1
DokimeEvidenceWatchRecordsResponse = _wire_32.Row2
DokimeEvidenceWatchRecordsOutput = _wire_32.Row3

DOKIME_EVIDENCE_WATCH_RECORDS: Operation[
    DokimeEvidenceWatchRecordsRequest, DokimeEvidenceWatchRecordsResponse, Message, DokimeEvidenceWatchRecordsOutput
]

DokimeExecutionAppendExecutionsRequest = _wire_33.Row1
DokimeExecutionAppendExecutionsResponse = _wire_33.Row2

DOKIME_EXECUTION_APPEND_EXECUTIONS: Operation[
    DokimeExecutionAppendExecutionsRequest, DokimeExecutionAppendExecutionsResponse, Message, Message
]

DokimeExecutionCancelExecutionRequest = _wire_35.Row1
DokimeExecutionCancelExecutionResponse = _wire_35.Row2

DOKIME_EXECUTION_CANCEL_EXECUTION: Operation[
    DokimeExecutionCancelExecutionRequest, DokimeExecutionCancelExecutionResponse, Message, Message
]

DokimeExecutionCopyCycleRequest = _wire_36.Row1
DokimeExecutionCopyCycleResponse = _wire_36.Row2

DOKIME_EXECUTION_COPY_CYCLE: Operation[DokimeExecutionCopyCycleRequest, DokimeExecutionCopyCycleResponse, Message, Message]

DokimeExecutionCreateCycleRequest = _wire_37.Row1
DokimeExecutionCreateCycleResponse = _wire_37.Row2

DOKIME_EXECUTION_CREATE_CYCLE: Operation[DokimeExecutionCreateCycleRequest, DokimeExecutionCreateCycleResponse, Message, Message]

DokimeExecutionDeleteCaseRequest = _wire_38.Row1
DokimeExecutionDeleteCaseResponse = _wire_38.Row2

DOKIME_EXECUTION_DELETE_CASE: Operation[DokimeExecutionDeleteCaseRequest, DokimeExecutionDeleteCaseResponse, Message, Message]

DokimeExecutionGetCaseRequest = _wire_39.Row1
DokimeExecutionGetCaseResponse = _wire_39.Row2

DOKIME_EXECUTION_GET_CASE: Operation[DokimeExecutionGetCaseRequest, DokimeExecutionGetCaseResponse, Message, Message]

DokimeExecutionGetCycleRequest = _wire_40.Row1
DokimeExecutionGetCycleResponse = _wire_40.Row2

DOKIME_EXECUTION_GET_CYCLE: Operation[DokimeExecutionGetCycleRequest, DokimeExecutionGetCycleResponse, Message, Message]

DokimeExecutionGetExecutionRequest = _wire_41.Row1
DokimeExecutionGetExecutionResponse = _wire_41.Row2

DOKIME_EXECUTION_GET_EXECUTION: Operation[DokimeExecutionGetExecutionRequest, DokimeExecutionGetExecutionResponse, Message, Message]

DokimeExecutionGetExecutionCommandRequest = _wire_42.Row1
DokimeExecutionGetExecutionCommandResponse = _wire_42.Row2

DOKIME_EXECUTION_GET_EXECUTION_COMMAND: Operation[
    DokimeExecutionGetExecutionCommandRequest, DokimeExecutionGetExecutionCommandResponse, Message, Message
]

DokimeExecutionListCaseParametersRequest = _wire_43.Row1
DokimeExecutionListCaseParametersResponse = _wire_43.Row2

DOKIME_EXECUTION_LIST_CASE_PARAMETERS: Operation[
    DokimeExecutionListCaseParametersRequest, DokimeExecutionListCaseParametersResponse, Message, Message
]

DokimeExecutionListCasesRequest = _wire_44.Row1
DokimeExecutionListCasesResponse = _wire_44.Row2

DOKIME_EXECUTION_LIST_CASES: Operation[DokimeExecutionListCasesRequest, DokimeExecutionListCasesResponse, Message, Message]

DokimeExecutionListCyclesRequest = _wire_45.Row1
DokimeExecutionListCyclesResponse = _wire_45.Row2

DOKIME_EXECUTION_LIST_CYCLES: Operation[DokimeExecutionListCyclesRequest, DokimeExecutionListCyclesResponse, Message, Message]

DokimeExecutionListExecutionParametersRequest = _wire_46.Row1
DokimeExecutionListExecutionParametersResponse = _wire_46.Row2

DOKIME_EXECUTION_LIST_EXECUTION_PARAMETERS: Operation[
    DokimeExecutionListExecutionParametersRequest, DokimeExecutionListExecutionParametersResponse, Message, Message
]

DokimeExecutionListExecutionStepsRequest = _wire_47.Row1
DokimeExecutionListExecutionStepsResponse = _wire_47.Row2

DOKIME_EXECUTION_LIST_EXECUTION_STEPS: Operation[
    DokimeExecutionListExecutionStepsRequest, DokimeExecutionListExecutionStepsResponse, Message, Message
]

DokimeExecutionListExecutionsRequest = _wire_48.Row1
DokimeExecutionListExecutionsResponse = _wire_48.Row2

DOKIME_EXECUTION_LIST_EXECUTIONS: Operation[DokimeExecutionListExecutionsRequest, DokimeExecutionListExecutionsResponse, Message, Message]

DokimeExecutionPauseCycleRequest = _wire_49.Row1
DokimeExecutionPauseCycleResponse = _wire_49.Row2

DOKIME_EXECUTION_PAUSE_CYCLE: Operation[DokimeExecutionPauseCycleRequest, DokimeExecutionPauseCycleResponse, Message, Message]

DokimeExecutionPauseExecutionRequest = _wire_50.Row1
DokimeExecutionPauseExecutionResponse = _wire_50.Row2

DOKIME_EXECUTION_PAUSE_EXECUTION: Operation[DokimeExecutionPauseExecutionRequest, DokimeExecutionPauseExecutionResponse, Message, Message]

DokimeExecutionRenameCycleRequest = _wire_51.Row1
DokimeExecutionRenameCycleResponse = _wire_51.Row2

DOKIME_EXECUTION_RENAME_CYCLE: Operation[DokimeExecutionRenameCycleRequest, DokimeExecutionRenameCycleResponse, Message, Message]

DokimeExecutionRenameExecutionRequest = _wire_52.Row1
DokimeExecutionRenameExecutionResponse = _wire_52.Row2

DOKIME_EXECUTION_RENAME_EXECUTION: Operation[
    DokimeExecutionRenameExecutionRequest, DokimeExecutionRenameExecutionResponse, Message, Message
]

DokimeExecutionResumeCycleRequest = _wire_53.Row1
DokimeExecutionResumeCycleResponse = _wire_53.Row2

DOKIME_EXECUTION_RESUME_CYCLE: Operation[DokimeExecutionResumeCycleRequest, DokimeExecutionResumeCycleResponse, Message, Message]

DokimeExecutionResumeExecutionRequest = _wire_54.Row1
DokimeExecutionResumeExecutionResponse = _wire_54.Row2

DOKIME_EXECUTION_RESUME_EXECUTION: Operation[
    DokimeExecutionResumeExecutionRequest, DokimeExecutionResumeExecutionResponse, Message, Message
]

DokimeExecutionSealCycleRequest = _wire_55.Row1
DokimeExecutionSealCycleResponse = _wire_55.Row2

DOKIME_EXECUTION_SEAL_CYCLE: Operation[DokimeExecutionSealCycleRequest, DokimeExecutionSealCycleResponse, Message, Message]

DokimeExecutionSetCycleArchivedRequest = _wire_56.Row1
DokimeExecutionSetCycleArchivedResponse = _wire_56.Row2

DOKIME_EXECUTION_SET_CYCLE_ARCHIVED: Operation[
    DokimeExecutionSetCycleArchivedRequest, DokimeExecutionSetCycleArchivedResponse, Message, Message
]

DokimeExecutionSetExecutionArchivedRequest = _wire_57.Row1
DokimeExecutionSetExecutionArchivedResponse = _wire_57.Row2

DOKIME_EXECUTION_SET_EXECUTION_ARCHIVED: Operation[
    DokimeExecutionSetExecutionArchivedRequest, DokimeExecutionSetExecutionArchivedResponse, Message, Message
]

DokimeExecutionStartExecutionRequest = _wire_58.Row1
DokimeExecutionStartExecutionResponse = _wire_58.Row2

DOKIME_EXECUTION_START_EXECUTION: Operation[DokimeExecutionStartExecutionRequest, DokimeExecutionStartExecutionResponse, Message, Message]

DokimeExecutionStopExecutionRequest = _wire_59.Row1
DokimeExecutionStopExecutionResponse = _wire_59.Row2

DOKIME_EXECUTION_STOP_EXECUTION: Operation[DokimeExecutionStopExecutionRequest, DokimeExecutionStopExecutionResponse, Message, Message]

DokimeExecutionSubmitExecutionCommandRequest = _wire_60.Row1
DokimeExecutionSubmitExecutionCommandResponse = _wire_60.Row2

DOKIME_EXECUTION_SUBMIT_EXECUTION_COMMAND: Operation[
    DokimeExecutionSubmitExecutionCommandRequest, DokimeExecutionSubmitExecutionCommandResponse, Message, Message
]

DokimeExecutionUpdateCyclePolicyRequest = _wire_61.Row1
DokimeExecutionUpdateCyclePolicyResponse = _wire_61.Row2

DOKIME_EXECUTION_UPDATE_CYCLE_POLICY: Operation[
    DokimeExecutionUpdateCyclePolicyRequest, DokimeExecutionUpdateCyclePolicyResponse, Message, Message
]

DokimeFilesFileWatchRequest = _wire_62.Row1
DokimeFilesFileWatchResponse = _wire_62.Row2
DokimeFilesFileWatchOutput = _wire_62.Row3

DOKIME_FILES_FILE_WATCH: Operation[DokimeFilesFileWatchRequest, DokimeFilesFileWatchResponse, Message, DokimeFilesFileWatchOutput]

DokimeHealthListHealthRequest = _wire_64.Row1
DokimeHealthListHealthResponse = _wire_64.Row2

DOKIME_HEALTH_LIST_HEALTH: Operation[DokimeHealthListHealthRequest, DokimeHealthListHealthResponse, Message, Message]

DokimeHistoryBrowseCyclesRequest = _wire_66.Row1
DokimeHistoryBrowseCyclesResponse = _wire_66.Row2

DOKIME_HISTORY_BROWSE_CYCLES: Operation[DokimeHistoryBrowseCyclesRequest, DokimeHistoryBrowseCyclesResponse, Message, Message]

DokimeHistoryBrowseExecutionsRequest = _wire_68.Row1
DokimeHistoryBrowseExecutionsResponse = _wire_68.Row2

DOKIME_HISTORY_BROWSE_EXECUTIONS: Operation[DokimeHistoryBrowseExecutionsRequest, DokimeHistoryBrowseExecutionsResponse, Message, Message]

DokimeHistoryDeleteExecutionRequest = _wire_69.Row1
DokimeHistoryDeleteExecutionResponse = _wire_69.Row2

DOKIME_HISTORY_DELETE_EXECUTION: Operation[DokimeHistoryDeleteExecutionRequest, DokimeHistoryDeleteExecutionResponse, Message, Message]

DokimeHistoryDownloadCheckpointArtifactRequest = _wire_70.Row1
DokimeHistoryDownloadCheckpointArtifactResponse = _wire_70.Row2
DokimeHistoryDownloadCheckpointArtifactOutput = _wire_70.Row3

DOKIME_HISTORY_DOWNLOAD_CHECKPOINT_ARTIFACT: Operation[
    DokimeHistoryDownloadCheckpointArtifactRequest,
    DokimeHistoryDownloadCheckpointArtifactResponse,
    Message,
    DokimeHistoryDownloadCheckpointArtifactOutput,
]

DokimeHistoryGetCheckpointContextRequest = _wire_71.Row1
DokimeHistoryGetCheckpointContextResponse = _wire_71.Row2

DOKIME_HISTORY_GET_CHECKPOINT_CONTEXT: Operation[
    DokimeHistoryGetCheckpointContextRequest, DokimeHistoryGetCheckpointContextResponse, Message, Message
]

DokimeHistoryGetCheckpointDocumentRequest = _wire_72.Row1
DokimeHistoryGetCheckpointDocumentResponse = _wire_72.Row2

DOKIME_HISTORY_GET_CHECKPOINT_DOCUMENT: Operation[
    DokimeHistoryGetCheckpointDocumentRequest, DokimeHistoryGetCheckpointDocumentResponse, Message, Message
]

DokimeHistoryGetCheckpointEvidenceRequest = _wire_73.Row1
DokimeHistoryGetCheckpointEvidenceResponse = _wire_73.Row2

DOKIME_HISTORY_GET_CHECKPOINT_EVIDENCE: Operation[
    DokimeHistoryGetCheckpointEvidenceRequest, DokimeHistoryGetCheckpointEvidenceResponse, Message, Message
]

DokimeHistoryGetExecutionCheckpointRequest = _wire_74.Row1
DokimeHistoryGetExecutionCheckpointResponse = _wire_74.Row2

DOKIME_HISTORY_GET_EXECUTION_CHECKPOINT: Operation[
    DokimeHistoryGetExecutionCheckpointRequest, DokimeHistoryGetExecutionCheckpointResponse, Message, Message
]

DokimeHistoryListCheckpointRecordsRequest = _wire_75.Row1
DokimeHistoryListCheckpointRecordsResponse = _wire_75.Row2

DOKIME_HISTORY_LIST_CHECKPOINT_RECORDS: Operation[
    DokimeHistoryListCheckpointRecordsRequest, DokimeHistoryListCheckpointRecordsResponse, Message, Message
]

DokimeHistoryListCheckpointTelemetryRequest = _wire_76.Row1
DokimeHistoryListCheckpointTelemetryResponse = _wire_76.Row2

DOKIME_HISTORY_LIST_CHECKPOINT_TELEMETRY: Operation[
    DokimeHistoryListCheckpointTelemetryRequest, DokimeHistoryListCheckpointTelemetryResponse, Message, Message
]

DokimeHistoryReadCheckpointOutputRequest = _wire_77.Row1
DokimeHistoryReadCheckpointOutputResponse = _wire_77.Row2

DOKIME_HISTORY_READ_CHECKPOINT_OUTPUT: Operation[
    DokimeHistoryReadCheckpointOutputRequest, DokimeHistoryReadCheckpointOutputResponse, Message, Message
]

DokimeMediaAudioRecordingRequest = _wire_78.Row1
DokimeMediaAudioRecordingResponse = _wire_78.Row2
DokimeMediaAudioRecordingOutput = _wire_78.Row3

DOKIME_MEDIA_AUDIO_RECORDING: Operation[
    DokimeMediaAudioRecordingRequest, DokimeMediaAudioRecordingResponse, Message, DokimeMediaAudioRecordingOutput
]

DokimeMediaCameraRecordingRequest = _wire_80.Row1
DokimeMediaCameraRecordingResponse = _wire_80.Row2
DokimeMediaCameraRecordingOutput = _wire_80.Row3

DOKIME_MEDIA_CAMERA_RECORDING: Operation[
    DokimeMediaCameraRecordingRequest, DokimeMediaCameraRecordingResponse, Message, DokimeMediaCameraRecordingOutput
]

DokimeMediaPhotoRequest = _wire_81.Row1
DokimeMediaPhotoResponse = _wire_81.Row2

DOKIME_MEDIA_PHOTO: Operation[DokimeMediaPhotoRequest, DokimeMediaPhotoResponse, Message, Message]

DokimeMediaScreenRecordingRequest = _wire_82.Row1
DokimeMediaScreenRecordingResponse = _wire_82.Row2
DokimeMediaScreenRecordingOutput = _wire_82.Row3

DOKIME_MEDIA_SCREEN_RECORDING: Operation[
    DokimeMediaScreenRecordingRequest, DokimeMediaScreenRecordingResponse, Message, DokimeMediaScreenRecordingOutput
]

DokimeMediaScreenshotRequest = _wire_83.Row1
DokimeMediaScreenshotResponse = _wire_83.Row2

DOKIME_MEDIA_SCREENSHOT: Operation[DokimeMediaScreenshotRequest, DokimeMediaScreenshotResponse, Message, Message]

DokimeNotesCreateNoteRequest = _wire_84.Row1
DokimeNotesCreateNoteResponse = _wire_84.Row2

DOKIME_NOTES_CREATE_NOTE: Operation[DokimeNotesCreateNoteRequest, DokimeNotesCreateNoteResponse, Message, Message]

DokimeNotesEditNoteRequest = _wire_86.Row1
DokimeNotesEditNoteResponse = _wire_86.Row2

DOKIME_NOTES_EDIT_NOTE: Operation[DokimeNotesEditNoteRequest, DokimeNotesEditNoteResponse, Message, Message]

DokimeNotesListNotesRequest = _wire_87.Row1
DokimeNotesListNotesResponse = _wire_87.Row2

DOKIME_NOTES_LIST_NOTES: Operation[DokimeNotesListNotesRequest, DokimeNotesListNotesResponse, Message, Message]

DokimeObservationGetRuntimeShellRequest = _wire_88.Row1
DokimeObservationGetRuntimeShellResponse = _wire_88.Row2

DOKIME_OBSERVATION_GET_RUNTIME_SHELL: Operation[
    DokimeObservationGetRuntimeShellRequest, DokimeObservationGetRuntimeShellResponse, Message, Message
]

DokimeObservationGetTraceabilityRequest = _wire_90.Row1
DokimeObservationGetTraceabilityResponse = _wire_90.Row2

DOKIME_OBSERVATION_GET_TRACEABILITY: Operation[
    DokimeObservationGetTraceabilityRequest, DokimeObservationGetTraceabilityResponse, Message, Message
]

DokimeObservationWatchExecutionProgressRequest = _wire_91.Row1
DokimeObservationWatchExecutionProgressResponse = _wire_91.Row2
DokimeObservationWatchExecutionProgressOutput = _wire_91.Row3

DOKIME_OBSERVATION_WATCH_EXECUTION_PROGRESS: Operation[
    DokimeObservationWatchExecutionProgressRequest,
    DokimeObservationWatchExecutionProgressResponse,
    Message,
    DokimeObservationWatchExecutionProgressOutput,
]

DokimeObservationWatchRuntimeRequest = _wire_92.Row1
DokimeObservationWatchRuntimeResponse = _wire_92.Row2
DokimeObservationWatchRuntimeOutput = _wire_92.Row3

DOKIME_OBSERVATION_WATCH_RUNTIME: Operation[
    DokimeObservationWatchRuntimeRequest, DokimeObservationWatchRuntimeResponse, Message, DokimeObservationWatchRuntimeOutput
]

DokimePlanningMaterializeTemplateRequest = _wire_93.Row1
DokimePlanningMaterializeTemplateResponse = _wire_93.Row2

DOKIME_PLANNING_MATERIALIZE_TEMPLATE: Operation[
    DokimePlanningMaterializeTemplateRequest, DokimePlanningMaterializeTemplateResponse, Message, Message
]

DokimePlanningPreviewSelectionRequest = _wire_95.Row1
DokimePlanningPreviewSelectionResponse = _wire_95.Row2

DOKIME_PLANNING_PREVIEW_SELECTION: Operation[
    DokimePlanningPreviewSelectionRequest, DokimePlanningPreviewSelectionResponse, Message, Message
]

DokimePlanningRegisterSelectionRequest = _wire_96.Row1
DokimePlanningRegisterSelectionResponse = _wire_96.Row2

DOKIME_PLANNING_REGISTER_SELECTION: Operation[
    DokimePlanningRegisterSelectionRequest, DokimePlanningRegisterSelectionResponse, Message, Message
]

DokimePlotsListAnalysesRequest = _wire_97.Row1
DokimePlotsListAnalysesResponse = _wire_97.Row2

DOKIME_PLOTS_LIST_ANALYSES: Operation[DokimePlotsListAnalysesRequest, DokimePlotsListAnalysesResponse, Message, Message]

DokimePlotsQuerySnapshotsRequest = _wire_99.Row1
DokimePlotsQuerySnapshotsResponse = _wire_99.Row2

DOKIME_PLOTS_QUERY_SNAPSHOTS: Operation[DokimePlotsQuerySnapshotsRequest, DokimePlotsQuerySnapshotsResponse, Message, Message]

DokimePlotsWatchRequest = _wire_100.Row1
DokimePlotsWatchResponse = _wire_100.Row2
DokimePlotsWatchOutput = _wire_100.Row3

DOKIME_PLOTS_WATCH: Operation[DokimePlotsWatchRequest, DokimePlotsWatchResponse, Message, DokimePlotsWatchOutput]

DokimePluginsGetInstallationRequest = _wire_101.Row1
DokimePluginsGetInstallationResponse = _wire_101.Row2

DOKIME_PLUGINS_GET_INSTALLATION: Operation[DokimePluginsGetInstallationRequest, DokimePluginsGetInstallationResponse, Message, Message]

DokimePluginsInstallRequest = _wire_103.Row1
DokimePluginsInstallResponse = _wire_103.Row2
DokimePluginsInstallInput = _wire_103.Row3

DOKIME_PLUGINS_INSTALL: Operation[DokimePluginsInstallRequest, DokimePluginsInstallResponse, DokimePluginsInstallInput, Message]

DokimePluginsListAdmittedPluginsRequest = _wire_104.Row1
DokimePluginsListAdmittedPluginsResponse = _wire_104.Row2

DOKIME_PLUGINS_LIST_ADMITTED_PLUGINS: Operation[
    DokimePluginsListAdmittedPluginsRequest, DokimePluginsListAdmittedPluginsResponse, Message, Message
]

DokimePluginsListInstallationsRequest = _wire_105.Row1
DokimePluginsListInstallationsResponse = _wire_105.Row2

DOKIME_PLUGINS_LIST_INSTALLATIONS: Operation[
    DokimePluginsListInstallationsRequest, DokimePluginsListInstallationsResponse, Message, Message
]

DokimePluginsRemoveRequest = _wire_106.Row1
DokimePluginsRemoveResponse = _wire_106.Row2

DOKIME_PLUGINS_REMOVE: Operation[DokimePluginsRemoveRequest, DokimePluginsRemoveResponse, Message, Message]

DokimePluginsSetEnabledRequest = _wire_107.Row1
DokimePluginsSetEnabledResponse = _wire_107.Row2

DOKIME_PLUGINS_SET_ENABLED: Operation[DokimePluginsSetEnabledRequest, DokimePluginsSetEnabledResponse, Message, Message]

DokimePromptsGetDefinitionRequest = _wire_108.Row1
DokimePromptsGetDefinitionResponse = _wire_108.Row2

DOKIME_PROMPTS_GET_DEFINITION: Operation[DokimePromptsGetDefinitionRequest, DokimePromptsGetDefinitionResponse, Message, Message]

DokimePromptsGetPromptRequest = _wire_110.Row1
DokimePromptsGetPromptResponse = _wire_110.Row2

DOKIME_PROMPTS_GET_PROMPT: Operation[DokimePromptsGetPromptRequest, DokimePromptsGetPromptResponse, Message, Message]

DokimePromptsListDefinitionsRequest = _wire_111.Row1
DokimePromptsListDefinitionsResponse = _wire_111.Row2

DOKIME_PROMPTS_LIST_DEFINITIONS: Operation[DokimePromptsListDefinitionsRequest, DokimePromptsListDefinitionsResponse, Message, Message]

DokimePromptsListPendingRequest = _wire_112.Row1
DokimePromptsListPendingResponse = _wire_112.Row2

DOKIME_PROMPTS_LIST_PENDING: Operation[DokimePromptsListPendingRequest, DokimePromptsListPendingResponse, Message, Message]

DokimePromptsRespondRequest = _wire_113.Row1
DokimePromptsRespondResponse = _wire_113.Row2

DOKIME_PROMPTS_RESPOND: Operation[DokimePromptsRespondRequest, DokimePromptsRespondResponse, Message, Message]

DokimeQueryAggregateRequest = _wire_114.Row1
DokimeQueryAggregateResponse = _wire_114.Row2

DOKIME_QUERY_AGGREGATE: Operation[DokimeQueryAggregateRequest, DokimeQueryAggregateResponse, Message, Message]

DokimeRemoteCreatePairingRequest = _wire_116.Row1
DokimeRemoteCreatePairingResponse = _wire_116.Row2

DOKIME_REMOTE_CREATE_PAIRING: Operation[DokimeRemoteCreatePairingRequest, DokimeRemoteCreatePairingResponse, Message, Message]

DokimeRemoteListPairedClientsRequest = _wire_118.Row1
DokimeRemoteListPairedClientsResponse = _wire_118.Row2

DOKIME_REMOTE_LIST_PAIRED_CLIENTS: Operation[DokimeRemoteListPairedClientsRequest, DokimeRemoteListPairedClientsResponse, Message, Message]

DokimeRemoteListSessionsRequest = _wire_119.Row1
DokimeRemoteListSessionsResponse = _wire_119.Row2

DOKIME_REMOTE_LIST_SESSIONS: Operation[DokimeRemoteListSessionsRequest, DokimeRemoteListSessionsResponse, Message, Message]

DokimeRemoteReadListenerRequest = _wire_120.Row1
DokimeRemoteReadListenerResponse = _wire_120.Row2

DOKIME_REMOTE_READ_LISTENER: Operation[DokimeRemoteReadListenerRequest, DokimeRemoteReadListenerResponse, Message, Message]

DokimeRemoteRegenerateCertificateRequest = _wire_121.Row1
DokimeRemoteRegenerateCertificateResponse = _wire_121.Row2

DOKIME_REMOTE_REGENERATE_CERTIFICATE: Operation[
    DokimeRemoteRegenerateCertificateRequest, DokimeRemoteRegenerateCertificateResponse, Message, Message
]

DokimeRemoteRevokePairedClientRequest = _wire_122.Row1
DokimeRemoteRevokePairedClientResponse = _wire_122.Row2

DOKIME_REMOTE_REVOKE_PAIRED_CLIENT: Operation[
    DokimeRemoteRevokePairedClientRequest, DokimeRemoteRevokePairedClientResponse, Message, Message
]

DokimeRemoteUpdateListenerRequest = _wire_123.Row1
DokimeRemoteUpdateListenerResponse = _wire_123.Row2

DOKIME_REMOTE_UPDATE_LISTENER: Operation[DokimeRemoteUpdateListenerRequest, DokimeRemoteUpdateListenerResponse, Message, Message]

DokimeReportsDownloadCycleReportRequest = _wire_124.Row1
DokimeReportsDownloadCycleReportResponse = _wire_124.Row2
DokimeReportsDownloadCycleReportOutput = _wire_124.Row3

DOKIME_REPORTS_DOWNLOAD_CYCLE_REPORT: Operation[
    DokimeReportsDownloadCycleReportRequest, DokimeReportsDownloadCycleReportResponse, Message, DokimeReportsDownloadCycleReportOutput
]

DokimeReportsDownloadExecutionReportRequest = _wire_126.Row1
DokimeReportsDownloadExecutionReportResponse = _wire_126.Row2
DokimeReportsDownloadExecutionReportOutput = _wire_126.Row3

DOKIME_REPORTS_DOWNLOAD_EXECUTION_REPORT: Operation[
    DokimeReportsDownloadExecutionReportRequest,
    DokimeReportsDownloadExecutionReportResponse,
    Message,
    DokimeReportsDownloadExecutionReportOutput,
]

DokimeReportsMaterializeReportRequest = _wire_127.Row1
DokimeReportsMaterializeReportResponse = _wire_127.Row2

DOKIME_REPORTS_MATERIALIZE_REPORT: Operation[
    DokimeReportsMaterializeReportRequest, DokimeReportsMaterializeReportResponse, Message, Message
]

DokimeRuntimeHealthRequest = _wire_128.Row1
DokimeRuntimeHealthResponse = _wire_128.Row2

DOKIME_RUNTIME_HEALTH: Operation[DokimeRuntimeHealthRequest, DokimeRuntimeHealthResponse, Message, Message]

DokimeRuntimeIdentityRequest = _wire_130.Row1
DokimeRuntimeIdentityResponse = _wire_130.Row2

DOKIME_RUNTIME_IDENTITY: Operation[DokimeRuntimeIdentityRequest, DokimeRuntimeIdentityResponse, Message, Message]

DokimeSecretsExportRequest = _wire_131.Row1
DokimeSecretsExportResponse = _wire_131.Row2

DOKIME_SECRETS_EXPORT: Operation[DokimeSecretsExportRequest, DokimeSecretsExportResponse, Message, Message]

DokimeSecretsImportRequest = _wire_133.Row1
DokimeSecretsImportResponse = _wire_133.Row2

DOKIME_SECRETS_IMPORT: Operation[DokimeSecretsImportRequest, DokimeSecretsImportResponse, Message, Message]

DokimeSecretsListRequest = _wire_134.Row1
DokimeSecretsListResponse = _wire_134.Row2

DOKIME_SECRETS_LIST: Operation[DokimeSecretsListRequest, DokimeSecretsListResponse, Message, Message]

DokimeSecretsLookupRequest = _wire_135.Row1
DokimeSecretsLookupResponse = _wire_135.Row2

DOKIME_SECRETS_LOOKUP: Operation[DokimeSecretsLookupRequest, DokimeSecretsLookupResponse, Message, Message]

DokimeSecretsPreviewRequest = _wire_136.Row1
DokimeSecretsPreviewResponse = _wire_136.Row2

DOKIME_SECRETS_PREVIEW: Operation[DokimeSecretsPreviewRequest, DokimeSecretsPreviewResponse, Message, Message]

DokimeSecretsPutRequest = _wire_137.Row1
DokimeSecretsPutResponse = _wire_137.Row2

DOKIME_SECRETS_PUT: Operation[DokimeSecretsPutRequest, DokimeSecretsPutResponse, Message, Message]

DokimeSecretsRemoveRequest = _wire_138.Row1
DokimeSecretsRemoveResponse = _wire_138.Row2

DOKIME_SECRETS_REMOVE: Operation[DokimeSecretsRemoveRequest, DokimeSecretsRemoveResponse, Message, Message]

DokimeSettingsGetPluginSettingsRequest = _wire_139.Row1
DokimeSettingsGetPluginSettingsResponse = _wire_139.Row2

DOKIME_SETTINGS_GET_PLUGIN_SETTINGS: Operation[
    DokimeSettingsGetPluginSettingsRequest, DokimeSettingsGetPluginSettingsResponse, Message, Message
]

DokimeSettingsGetSettingsRequest = _wire_141.Row1
DokimeSettingsGetSettingsResponse = _wire_141.Row2

DOKIME_SETTINGS_GET_SETTINGS: Operation[DokimeSettingsGetSettingsRequest, DokimeSettingsGetSettingsResponse, Message, Message]

DokimeSettingsListStorageSourcesRequest = _wire_142.Row1
DokimeSettingsListStorageSourcesResponse = _wire_142.Row2

DOKIME_SETTINGS_LIST_STORAGE_SOURCES: Operation[
    DokimeSettingsListStorageSourcesRequest, DokimeSettingsListStorageSourcesResponse, Message, Message
]

DokimeSettingsReadApplicationSettingsRequest = _wire_143.Row1
DokimeSettingsReadApplicationSettingsResponse = _wire_143.Row2

DOKIME_SETTINGS_READ_APPLICATION_SETTINGS: Operation[
    DokimeSettingsReadApplicationSettingsRequest, DokimeSettingsReadApplicationSettingsResponse, Message, Message
]

DokimeSettingsReorderStorageSourcesRequest = _wire_144.Row1
DokimeSettingsReorderStorageSourcesResponse = _wire_144.Row2

DOKIME_SETTINGS_REORDER_STORAGE_SOURCES: Operation[
    DokimeSettingsReorderStorageSourcesRequest, DokimeSettingsReorderStorageSourcesResponse, Message, Message
]

DokimeSettingsSetStorageSourceEnabledRequest = _wire_145.Row1
DokimeSettingsSetStorageSourceEnabledResponse = _wire_145.Row2

DOKIME_SETTINGS_SET_STORAGE_SOURCE_ENABLED: Operation[
    DokimeSettingsSetStorageSourceEnabledRequest, DokimeSettingsSetStorageSourceEnabledResponse, Message, Message
]

DokimeSettingsUpdateApplicationSettingsRequest = _wire_146.Row1
DokimeSettingsUpdateApplicationSettingsResponse = _wire_146.Row2

DOKIME_SETTINGS_UPDATE_APPLICATION_SETTINGS: Operation[
    DokimeSettingsUpdateApplicationSettingsRequest, DokimeSettingsUpdateApplicationSettingsResponse, Message, Message
]

DokimeSettingsUpdatePluginSettingsRequest = _wire_147.Row1
DokimeSettingsUpdatePluginSettingsResponse = _wire_147.Row2

DOKIME_SETTINGS_UPDATE_PLUGIN_SETTINGS: Operation[
    DokimeSettingsUpdatePluginSettingsRequest, DokimeSettingsUpdatePluginSettingsResponse, Message, Message
]

DokimeTelemetryListSamplesRequest = _wire_148.Row1
DokimeTelemetryListSamplesResponse = _wire_148.Row2

DOKIME_TELEMETRY_LIST_SAMPLES: Operation[DokimeTelemetryListSamplesRequest, DokimeTelemetryListSamplesResponse, Message, Message]

DokimeTelemetryWatchSamplesRequest = _wire_150.Row1
DokimeTelemetryWatchSamplesResponse = _wire_150.Row2
DokimeTelemetryWatchSamplesOutput = _wire_150.Row3

DOKIME_TELEMETRY_WATCH_SAMPLES: Operation[
    DokimeTelemetryWatchSamplesRequest, DokimeTelemetryWatchSamplesResponse, Message, DokimeTelemetryWatchSamplesOutput
]

HostCatalogRequest = _wire_151.Row1
HostCatalogResponse = _wire_151.Row2

HOST_CATALOG: Operation[HostCatalogRequest, HostCatalogResponse, Message, Message]

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

# Product Client Exports
from .client import DokimeClient as DokimeClient
