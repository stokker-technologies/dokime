// @generated from protocol descriptors; do not edit.
#pragma once
#include "dokime/host/v1/aggregates.grpc.pb.h"
#include "dokime/host/v1/analyses.grpc.pb.h"
#include "dokime/host/v1/backend.grpc.pb.h"
#include "dokime/host/v1/backend/overview.grpc.pb.h"
#include "dokime/host/v1/cases.grpc.pb.h"
#include "dokime/host/v1/content/artifacts.grpc.pb.h"
#include "dokime/host/v1/content/execution_files.grpc.pb.h"
#include "dokime/host/v1/cycles.grpc.pb.h"
#include "dokime/host/v1/documents.grpc.pb.h"
#include "dokime/host/v1/evidence.grpc.pb.h"
#include "dokime/host/v1/evidence/health_events.grpc.pb.h"
#include "dokime/host/v1/evidence/notes.grpc.pb.h"
#include "dokime/host/v1/evidence/records.grpc.pb.h"
#include "dokime/host/v1/evidence/samples.grpc.pb.h"
#include "dokime/host/v1/evidence/traceability.grpc.pb.h"
#include "dokime/host/v1/executions.grpc.pb.h"
#include "dokime/host/v1/executions/commands.grpc.pb.h"
#include "dokime/host/v1/executions/progress.grpc.pb.h"
#include "dokime/host/v1/executions/schemas.grpc.pb.h"
#include "dokime/host/v1/files.grpc.pb.h"
#include "dokime/host/v1/files/uploads.grpc.pb.h"
#include "dokime/host/v1/media/captures.grpc.pb.h"
#include "dokime/host/v1/plots/settings.grpc.pb.h"
#include "dokime/host/v1/plots/snapshots.grpc.pb.h"
#include "dokime/host/v1/plugins/secrets.grpc.pb.h"
#include "dokime/host/v1/prompt_definitions.grpc.pb.h"
#include "dokime/host/v1/prompts.grpc.pb.h"
#include "dokime/host/v1/prompts/answers.grpc.pb.h"
#include "dokime/host/v1/record_schemas.grpc.pb.h"
#include "dokime/host/v1/reports.grpc.pb.h"
#include "dokime/host/v1/secrets.grpc.pb.h"
#include "dokime/host/v1/settings.grpc.pb.h"
#include "dokime/host/v1/storage/sources.grpc.pb.h"
#include "dokime/host/v1/stream_schemas.grpc.pb.h"
#include "dokime/host/v1/suites.grpc.pb.h"
#include "harmos/host/v1/access/certificates.grpc.pb.h"
#include "harmos/host/v1/access/listener.grpc.pb.h"
#include "harmos/host/v1/access/pairings.grpc.pb.h"
#include "harmos/host/v1/invocations.grpc.pb.h"
#include "harmos/host/v1/operations.grpc.pb.h"
#include "harmos/host/v1/plugins.grpc.pb.h"
#include "harmos/host/v1/plugins/settings.grpc.pb.h"
#include "harmos/host/v1/schemas.grpc.pb.h"
#include "harmos/host/v1/status.grpc.pb.h"
#include "harmos/host/v1/status/diagnostics.grpc.pb.h"
#include "harmos/host/v1/status/logs.grpc.pb.h"
#include "harmos/host/v1/status/trace_events.grpc.pb.h"
#include "harmos/host/v1/streams.grpc.pb.h"
#include "harmos/host/v1/time.grpc.pb.h"
namespace dokime::detail {
using AggregatesGetRequest = ::dokime::host::v1::AggregatesGetRequest;
using AggregatesGetResponse = ::dokime::host::v1::AggregatesGetResponse;
using AggregatesGetService = ::dokime::host::v1::Aggregates;
using AnalysesListRequest = ::dokime::host::v1::AnalysesListRequest;
using AnalysesListResponse = ::dokime::host::v1::AnalysesListResponse;
using AnalysesListService = ::dokime::host::v1::Analyses;
using CyclesGetRequest = ::dokime::host::v1::CyclesGetRequest;
using CyclesGetResponse = ::dokime::host::v1::CyclesGetResponse;
using CyclesGetService = ::dokime::host::v1::Cycles;
using CyclesSetRequest = ::dokime::host::v1::CyclesSetRequest;
using CyclesSetResponse = ::dokime::host::v1::CyclesSetResponse;
using CyclesSetService = ::dokime::host::v1::Cycles;
using CyclesListRequest = ::dokime::host::v1::CyclesListRequest;
using CyclesListResponse = ::dokime::host::v1::CyclesListResponse;
using CyclesListService = ::dokime::host::v1::Cycles;
using CyclesWatchRequest = ::dokime::host::v1::CyclesWatchRequest;
using CyclesWatchResponse = ::dokime::host::v1::CyclesWatchResponse;
using CyclesWatchService = ::dokime::host::v1::Cycles;
using CyclesCreateRequest = ::dokime::host::v1::CyclesCreateRequest;
using CyclesCreateResponse = ::dokime::host::v1::CyclesCreateResponse;
using CyclesCreateService = ::dokime::host::v1::Cycles;
using CyclesSealRequest = ::dokime::host::v1::CyclesSealRequest;
using CyclesSealResponse = ::dokime::host::v1::CyclesSealResponse;
using CyclesSealService = ::dokime::host::v1::Cycles;
using BackendOverviewGetRequest = ::dokime::host::v1::backend::OverviewGetRequest;
using BackendOverviewGetResponse = ::dokime::host::v1::backend::OverviewGetResponse;
using BackendOverviewGetService = ::dokime::host::v1::backend::Overview;
using BackendOverviewWatchRequest = ::dokime::host::v1::backend::OverviewWatchRequest;
using BackendOverviewWatchResponse = ::dokime::host::v1::backend::OverviewWatchResponse;
using BackendOverviewWatchService = ::dokime::host::v1::backend::Overview;
using BackendAppearanceRequest = ::dokime::host::v1::BackendAppearanceRequest;
using BackendAppearanceResponse = ::dokime::host::v1::BackendAppearanceResponse;
using BackendAppearanceService = ::dokime::host::v1::Backend;
using RecordSchemasGetRequest = ::dokime::host::v1::RecordSchemasGetRequest;
using RecordSchemasGetResponse = ::dokime::host::v1::RecordSchemasGetResponse;
using RecordSchemasGetService = ::dokime::host::v1::RecordSchemas;
using RecordSchemasListRequest = ::dokime::host::v1::RecordSchemasListRequest;
using RecordSchemasListResponse = ::dokime::host::v1::RecordSchemasListResponse;
using RecordSchemasListService = ::dokime::host::v1::RecordSchemas;
using ExecutionsGetRequest = ::dokime::host::v1::ExecutionsGetRequest;
using ExecutionsGetResponse = ::dokime::host::v1::ExecutionsGetResponse;
using ExecutionsGetService = ::dokime::host::v1::Executions;
using ExecutionsSetRequest = ::dokime::host::v1::ExecutionsSetRequest;
using ExecutionsSetResponse = ::dokime::host::v1::ExecutionsSetResponse;
using ExecutionsSetService = ::dokime::host::v1::Executions;
using ExecutionsListRequest = ::dokime::host::v1::ExecutionsListRequest;
using ExecutionsListResponse = ::dokime::host::v1::ExecutionsListResponse;
using ExecutionsListService = ::dokime::host::v1::Executions;
using ExecutionsWatchRequest = ::dokime::host::v1::ExecutionsWatchRequest;
using ExecutionsWatchResponse = ::dokime::host::v1::ExecutionsWatchResponse;
using ExecutionsWatchService = ::dokime::host::v1::Executions;
using ExecutionsCreateRequest = ::dokime::host::v1::ExecutionsCreateRequest;
using ExecutionsCreateResponse = ::dokime::host::v1::ExecutionsCreateResponse;
using ExecutionsCreateService = ::dokime::host::v1::Executions;
using ExecutionsStartRequest = ::dokime::host::v1::ExecutionsStartRequest;
using ExecutionsStartResponse = ::dokime::host::v1::ExecutionsStartResponse;
using ExecutionsStartService = ::dokime::host::v1::Executions;
using ExecutionsStopRequest = ::dokime::host::v1::ExecutionsStopRequest;
using ExecutionsStopResponse = ::dokime::host::v1::ExecutionsStopResponse;
using ExecutionsStopService = ::dokime::host::v1::Executions;
using ExecutionsCancelRequest = ::dokime::host::v1::ExecutionsCancelRequest;
using ExecutionsCancelResponse = ::dokime::host::v1::ExecutionsCancelResponse;
using ExecutionsCancelService = ::dokime::host::v1::Executions;
using ExecutionsDeleteRequest = ::dokime::host::v1::ExecutionsDeleteRequest;
using ExecutionsDeleteResponse = ::dokime::host::v1::ExecutionsDeleteResponse;
using ExecutionsDeleteService = ::dokime::host::v1::Executions;
using ExecutionsParametersRequest = ::dokime::host::v1::ExecutionsParametersRequest;
using ExecutionsParametersResponse = ::dokime::host::v1::ExecutionsParametersResponse;
using ExecutionsParametersService = ::dokime::host::v1::Executions;
using ExecutionsStepsRequest = ::dokime::host::v1::ExecutionsStepsRequest;
using ExecutionsStepsResponse = ::dokime::host::v1::ExecutionsStepsResponse;
using ExecutionsStepsService = ::dokime::host::v1::Executions;
using CasesCreateRequest = ::dokime::host::v1::CasesCreateRequest;
using CasesCreateResponse = ::dokime::host::v1::CasesCreateResponse;
using CasesCreateService = ::dokime::host::v1::Cases;
using CasesGetRequest = ::dokime::host::v1::CasesGetRequest;
using CasesGetResponse = ::dokime::host::v1::CasesGetResponse;
using CasesGetService = ::dokime::host::v1::Cases;
using CasesListRequest = ::dokime::host::v1::CasesListRequest;
using CasesListResponse = ::dokime::host::v1::CasesListResponse;
using CasesListService = ::dokime::host::v1::Cases;
using CasesWatchRequest = ::dokime::host::v1::CasesWatchRequest;
using CasesWatchResponse = ::dokime::host::v1::CasesWatchResponse;
using CasesWatchService = ::dokime::host::v1::Cases;
using CasesDeleteRequest = ::dokime::host::v1::CasesDeleteRequest;
using CasesDeleteResponse = ::dokime::host::v1::CasesDeleteResponse;
using CasesDeleteService = ::dokime::host::v1::Cases;
using CasesParametersRequest = ::dokime::host::v1::CasesParametersRequest;
using CasesParametersResponse = ::dokime::host::v1::CasesParametersResponse;
using CasesParametersService = ::dokime::host::v1::Cases;
using ContentArtifactsGetRequest = ::dokime::host::v1::content::ArtifactsGetRequest;
using ContentArtifactsGetResponse = ::dokime::host::v1::content::ArtifactsGetResponse;
using ContentArtifactsGetService = ::dokime::host::v1::content::Artifacts;
using ContentArtifactsCreateRequest = ::dokime::host::v1::content::ArtifactsCreateRequest;
using ContentArtifactsCreateResponse = ::dokime::host::v1::content::ArtifactsCreateResponse;
using ContentArtifactsCreateService = ::dokime::host::v1::content::Artifacts;
using ContentArtifactsListRequest = ::dokime::host::v1::content::ArtifactsListRequest;
using ContentArtifactsListResponse = ::dokime::host::v1::content::ArtifactsListResponse;
using ContentArtifactsListService = ::dokime::host::v1::content::Artifacts;
using ContentArtifactsWatchRequest = ::dokime::host::v1::content::ArtifactsWatchRequest;
using ContentArtifactsWatchResponse = ::dokime::host::v1::content::ArtifactsWatchResponse;
using ContentArtifactsWatchService = ::dokime::host::v1::content::Artifacts;
using ContentExecutionFilesGetRequest = ::dokime::host::v1::content::ExecutionFilesGetRequest;
using ContentExecutionFilesGetResponse = ::dokime::host::v1::content::ExecutionFilesGetResponse;
using ContentExecutionFilesGetService = ::dokime::host::v1::content::ExecutionFiles;
using ContentExecutionFilesListRequest = ::dokime::host::v1::content::ExecutionFilesListRequest;
using ContentExecutionFilesListResponse = ::dokime::host::v1::content::ExecutionFilesListResponse;
using ContentExecutionFilesListService = ::dokime::host::v1::content::ExecutionFiles;
using ContentExecutionFilesWatchRequest = ::dokime::host::v1::content::ExecutionFilesWatchRequest;
using ContentExecutionFilesWatchResponse = ::dokime::host::v1::content::ExecutionFilesWatchResponse;
using ContentExecutionFilesWatchService = ::dokime::host::v1::content::ExecutionFiles;
using PlotsSnapshotsListRequest = ::dokime::host::v1::plots::SnapshotsListRequest;
using PlotsSnapshotsListResponse = ::dokime::host::v1::plots::SnapshotsListResponse;
using PlotsSnapshotsListService = ::dokime::host::v1::plots::Snapshots;
using PlotsSnapshotsWatchRequest = ::dokime::host::v1::plots::SnapshotsWatchRequest;
using PlotsSnapshotsWatchResponse = ::dokime::host::v1::plots::SnapshotsWatchResponse;
using PlotsSnapshotsWatchService = ::dokime::host::v1::plots::Snapshots;
using DocumentsGetRequest = ::dokime::host::v1::DocumentsGetRequest;
using DocumentsGetResponse = ::dokime::host::v1::DocumentsGetResponse;
using DocumentsGetService = ::dokime::host::v1::Documents;
using DocumentsWatchRequest = ::dokime::host::v1::DocumentsWatchRequest;
using DocumentsWatchResponse = ::dokime::host::v1::DocumentsWatchResponse;
using DocumentsWatchService = ::dokime::host::v1::Documents;
using EvidenceNotesGetRequest = ::dokime::host::v1::evidence::NotesGetRequest;
using EvidenceNotesGetResponse = ::dokime::host::v1::evidence::NotesGetResponse;
using EvidenceNotesGetService = ::dokime::host::v1::evidence::Notes;
using EvidenceNotesSetRequest = ::dokime::host::v1::evidence::NotesSetRequest;
using EvidenceNotesSetResponse = ::dokime::host::v1::evidence::NotesSetResponse;
using EvidenceNotesSetService = ::dokime::host::v1::evidence::Notes;
using EvidenceNotesListRequest = ::dokime::host::v1::evidence::NotesListRequest;
using EvidenceNotesListResponse = ::dokime::host::v1::evidence::NotesListResponse;
using EvidenceNotesListService = ::dokime::host::v1::evidence::Notes;
using EvidenceNotesWatchRequest = ::dokime::host::v1::evidence::NotesWatchRequest;
using EvidenceNotesWatchResponse = ::dokime::host::v1::evidence::NotesWatchResponse;
using EvidenceNotesWatchService = ::dokime::host::v1::evidence::Notes;
using EvidenceNotesCreateRequest = ::dokime::host::v1::evidence::NotesCreateRequest;
using EvidenceNotesCreateResponse = ::dokime::host::v1::evidence::NotesCreateResponse;
using EvidenceNotesCreateService = ::dokime::host::v1::evidence::Notes;
using EvidenceHealthEventsListRequest = ::dokime::host::v1::evidence::HealthEventsListRequest;
using EvidenceHealthEventsListResponse = ::dokime::host::v1::evidence::HealthEventsListResponse;
using EvidenceHealthEventsListService = ::dokime::host::v1::evidence::HealthEvents;
using EvidenceHealthEventsWatchRequest = ::dokime::host::v1::evidence::HealthEventsWatchRequest;
using EvidenceHealthEventsWatchResponse = ::dokime::host::v1::evidence::HealthEventsWatchResponse;
using EvidenceHealthEventsWatchService = ::dokime::host::v1::evidence::HealthEvents;
using EvidenceRecordsListRequest = ::dokime::host::v1::evidence::RecordsListRequest;
using EvidenceRecordsListResponse = ::dokime::host::v1::evidence::RecordsListResponse;
using EvidenceRecordsListService = ::dokime::host::v1::evidence::Records;
using EvidenceRecordsWatchRequest = ::dokime::host::v1::evidence::RecordsWatchRequest;
using EvidenceRecordsWatchResponse = ::dokime::host::v1::evidence::RecordsWatchResponse;
using EvidenceRecordsWatchService = ::dokime::host::v1::evidence::Records;
using EvidenceSamplesListRequest = ::dokime::host::v1::evidence::SamplesListRequest;
using EvidenceSamplesListResponse = ::dokime::host::v1::evidence::SamplesListResponse;
using EvidenceSamplesListService = ::dokime::host::v1::evidence::Samples;
using EvidenceSamplesWatchRequest = ::dokime::host::v1::evidence::SamplesWatchRequest;
using EvidenceSamplesWatchResponse = ::dokime::host::v1::evidence::SamplesWatchResponse;
using EvidenceSamplesWatchService = ::dokime::host::v1::evidence::Samples;
using EvidenceTraceabilityGetRequest = ::dokime::host::v1::evidence::TraceabilityGetRequest;
using EvidenceTraceabilityGetResponse = ::dokime::host::v1::evidence::TraceabilityGetResponse;
using EvidenceTraceabilityGetService = ::dokime::host::v1::evidence::Traceability;
using EvidenceTraceabilityWatchRequest = ::dokime::host::v1::evidence::TraceabilityWatchRequest;
using EvidenceTraceabilityWatchResponse = ::dokime::host::v1::evidence::TraceabilityWatchResponse;
using EvidenceTraceabilityWatchService = ::dokime::host::v1::evidence::Traceability;
using EvidenceStatisticsRequest = ::dokime::host::v1::EvidenceStatisticsRequest;
using EvidenceStatisticsResponse = ::dokime::host::v1::EvidenceStatisticsResponse;
using EvidenceStatisticsService = ::dokime::host::v1::Evidence;
using EvidenceArchiveRequest = ::dokime::host::v1::EvidenceArchiveRequest;
using EvidenceArchiveResponse = ::dokime::host::v1::EvidenceArchiveResponse;
using EvidenceArchiveService = ::dokime::host::v1::Evidence;
using ExecutionsCommandsGetRequest = ::dokime::host::v1::executions::CommandsGetRequest;
using ExecutionsCommandsGetResponse = ::dokime::host::v1::executions::CommandsGetResponse;
using ExecutionsCommandsGetService = ::dokime::host::v1::executions::Commands;
using ExecutionsCommandsCreateRequest = ::dokime::host::v1::executions::CommandsCreateRequest;
using ExecutionsCommandsCreateResponse = ::dokime::host::v1::executions::CommandsCreateResponse;
using ExecutionsCommandsCreateService = ::dokime::host::v1::executions::Commands;
using ExecutionsProgressGetRequest = ::dokime::host::v1::executions::ProgressGetRequest;
using ExecutionsProgressGetResponse = ::dokime::host::v1::executions::ProgressGetResponse;
using ExecutionsProgressGetService = ::dokime::host::v1::executions::Progress;
using ExecutionsProgressWatchRequest = ::dokime::host::v1::executions::ProgressWatchRequest;
using ExecutionsProgressWatchResponse = ::dokime::host::v1::executions::ProgressWatchResponse;
using ExecutionsProgressWatchService = ::dokime::host::v1::executions::Progress;
using ExecutionsSchemasGetRequest = ::dokime::host::v1::executions::SchemasGetRequest;
using ExecutionsSchemasGetResponse = ::dokime::host::v1::executions::SchemasGetResponse;
using ExecutionsSchemasGetService = ::dokime::host::v1::executions::Schemas;
using FilesUploadsCreateRequest = ::dokime::host::v1::files::UploadsCreateRequest;
using FilesUploadsCreateResponse = ::dokime::host::v1::files::UploadsCreateResponse;
using FilesUploadsCreateService = ::dokime::host::v1::files::Uploads;
using FilesUploadsGetRequest = ::dokime::host::v1::files::UploadsGetRequest;
using FilesUploadsGetResponse = ::dokime::host::v1::files::UploadsGetResponse;
using FilesUploadsGetService = ::dokime::host::v1::files::Uploads;
using FilesUploadsResumeRequest = ::dokime::host::v1::files::UploadsResumeRequest;
using FilesUploadsResumeResponse = ::dokime::host::v1::files::UploadsResumeResponse;
using FilesUploadsResumeService = ::dokime::host::v1::files::Uploads;
using FilesUploadsAppendRequest = ::dokime::host::v1::files::UploadsAppendRequest;
using FilesUploadsAppendResponse = ::dokime::host::v1::files::UploadsAppendResponse;
using FilesUploadsAppendService = ::dokime::host::v1::files::Uploads;
using FilesUploadsPublishRequest = ::dokime::host::v1::files::UploadsPublishRequest;
using FilesUploadsPublishResponse = ::dokime::host::v1::files::UploadsPublishResponse;
using FilesUploadsPublishService = ::dokime::host::v1::files::Uploads;
using FilesUploadsDeleteRequest = ::dokime::host::v1::files::UploadsDeleteRequest;
using FilesUploadsDeleteResponse = ::dokime::host::v1::files::UploadsDeleteResponse;
using FilesUploadsDeleteService = ::dokime::host::v1::files::Uploads;
using FilesGetRequest = ::dokime::host::v1::FilesGetRequest;
using FilesGetResponse = ::dokime::host::v1::FilesGetResponse;
using FilesGetService = ::dokime::host::v1::Files;
using FilesReadRequest = ::dokime::host::v1::FilesReadRequest;
using FilesReadResponse = ::dokime::host::v1::FilesReadResponse;
using FilesReadService = ::dokime::host::v1::Files;
using FilesWatchRequest = ::dokime::host::v1::FilesWatchRequest;
using FilesWatchResponse = ::dokime::host::v1::FilesWatchResponse;
using FilesWatchService = ::dokime::host::v1::Files;
using FilesDeleteRequest = ::dokime::host::v1::FilesDeleteRequest;
using FilesDeleteResponse = ::dokime::host::v1::FilesDeleteResponse;
using FilesDeleteService = ::dokime::host::v1::Files;
using MediaCapturesCreateRequest = ::dokime::host::v1::media::CapturesCreateRequest;
using MediaCapturesCreateResponse = ::dokime::host::v1::media::CapturesCreateResponse;
using MediaCapturesCreateService = ::dokime::host::v1::media::Captures;
using MediaCapturesGetRequest = ::dokime::host::v1::media::CapturesGetRequest;
using MediaCapturesGetResponse = ::dokime::host::v1::media::CapturesGetResponse;
using MediaCapturesGetService = ::dokime::host::v1::media::Captures;
using MediaCapturesWatchRequest = ::dokime::host::v1::media::CapturesWatchRequest;
using MediaCapturesWatchResponse = ::dokime::host::v1::media::CapturesWatchResponse;
using MediaCapturesWatchService = ::dokime::host::v1::media::Captures;
using MediaCapturesStopRequest = ::dokime::host::v1::media::CapturesStopRequest;
using MediaCapturesStopResponse = ::dokime::host::v1::media::CapturesStopResponse;
using MediaCapturesStopService = ::dokime::host::v1::media::Captures;
using PlotsSettingsGetRequest = ::dokime::host::v1::plots::SettingsGetRequest;
using PlotsSettingsGetResponse = ::dokime::host::v1::plots::SettingsGetResponse;
using PlotsSettingsGetService = ::dokime::host::v1::plots::Settings;
using PlotsSettingsSetRequest = ::dokime::host::v1::plots::SettingsSetRequest;
using PlotsSettingsSetResponse = ::dokime::host::v1::plots::SettingsSetResponse;
using PlotsSettingsSetService = ::dokime::host::v1::plots::Settings;
using PluginsSecretsListRequest = ::dokime::host::v1::plugins::SecretsListRequest;
using PluginsSecretsListResponse = ::dokime::host::v1::plugins::SecretsListResponse;
using PluginsSecretsListService = ::dokime::host::v1::plugins::Secrets;
using PluginsSecretsSetRequest = ::dokime::host::v1::plugins::SecretsSetRequest;
using PluginsSecretsSetResponse = ::dokime::host::v1::plugins::SecretsSetResponse;
using PluginsSecretsSetService = ::dokime::host::v1::plugins::Secrets;
using PluginsSecretsDeleteRequest = ::dokime::host::v1::plugins::SecretsDeleteRequest;
using PluginsSecretsDeleteResponse = ::dokime::host::v1::plugins::SecretsDeleteResponse;
using PluginsSecretsDeleteService = ::dokime::host::v1::plugins::Secrets;
using PromptDefinitionsGetRequest = ::dokime::host::v1::PromptDefinitionsGetRequest;
using PromptDefinitionsGetResponse = ::dokime::host::v1::PromptDefinitionsGetResponse;
using PromptDefinitionsGetService = ::dokime::host::v1::PromptDefinitions;
using PromptDefinitionsListRequest = ::dokime::host::v1::PromptDefinitionsListRequest;
using PromptDefinitionsListResponse = ::dokime::host::v1::PromptDefinitionsListResponse;
using PromptDefinitionsListService = ::dokime::host::v1::PromptDefinitions;
using PromptsGetRequest = ::dokime::host::v1::PromptsGetRequest;
using PromptsGetResponse = ::dokime::host::v1::PromptsGetResponse;
using PromptsGetService = ::dokime::host::v1::Prompts;
using PromptsListRequest = ::dokime::host::v1::PromptsListRequest;
using PromptsListResponse = ::dokime::host::v1::PromptsListResponse;
using PromptsListService = ::dokime::host::v1::Prompts;
using PromptsWatchRequest = ::dokime::host::v1::PromptsWatchRequest;
using PromptsWatchResponse = ::dokime::host::v1::PromptsWatchResponse;
using PromptsWatchService = ::dokime::host::v1::Prompts;
using PromptsAnswersCreateRequest = ::dokime::host::v1::prompts::AnswersCreateRequest;
using PromptsAnswersCreateResponse = ::dokime::host::v1::prompts::AnswersCreateResponse;
using PromptsAnswersCreateService = ::dokime::host::v1::prompts::Answers;
using ReportsGetRequest = ::dokime::host::v1::ReportsGetRequest;
using ReportsGetResponse = ::dokime::host::v1::ReportsGetResponse;
using ReportsGetService = ::dokime::host::v1::Reports;
using ReportsCreateRequest = ::dokime::host::v1::ReportsCreateRequest;
using ReportsCreateResponse = ::dokime::host::v1::ReportsCreateResponse;
using ReportsCreateService = ::dokime::host::v1::Reports;
using ReportsPublishRequest = ::dokime::host::v1::ReportsPublishRequest;
using ReportsPublishResponse = ::dokime::host::v1::ReportsPublishResponse;
using ReportsPublishService = ::dokime::host::v1::Reports;
using SecretsGetRequest = ::dokime::host::v1::SecretsGetRequest;
using SecretsGetResponse = ::dokime::host::v1::SecretsGetResponse;
using SecretsGetService = ::dokime::host::v1::Secrets;
using SecretsSetRequest = ::dokime::host::v1::SecretsSetRequest;
using SecretsSetResponse = ::dokime::host::v1::SecretsSetResponse;
using SecretsSetService = ::dokime::host::v1::Secrets;
using SecretsListRequest = ::dokime::host::v1::SecretsListRequest;
using SecretsListResponse = ::dokime::host::v1::SecretsListResponse;
using SecretsListService = ::dokime::host::v1::Secrets;
using SecretsWatchRequest = ::dokime::host::v1::SecretsWatchRequest;
using SecretsWatchResponse = ::dokime::host::v1::SecretsWatchResponse;
using SecretsWatchService = ::dokime::host::v1::Secrets;
using SecretsDeleteRequest = ::dokime::host::v1::SecretsDeleteRequest;
using SecretsDeleteResponse = ::dokime::host::v1::SecretsDeleteResponse;
using SecretsDeleteService = ::dokime::host::v1::Secrets;
using SecretsExportRequest = ::dokime::host::v1::SecretsExportRequest;
using SecretsExportResponse = ::dokime::host::v1::SecretsExportResponse;
using SecretsExportService = ::dokime::host::v1::Secrets;
using SecretsImportRequest = ::dokime::host::v1::SecretsImportRequest;
using SecretsImportResponse = ::dokime::host::v1::SecretsImportResponse;
using SecretsImportService = ::dokime::host::v1::Secrets;
using SettingsGetRequest = ::dokime::host::v1::SettingsGetRequest;
using SettingsGetResponse = ::dokime::host::v1::SettingsGetResponse;
using SettingsGetService = ::dokime::host::v1::Settings;
using SettingsSetRequest = ::dokime::host::v1::SettingsSetRequest;
using SettingsSetResponse = ::dokime::host::v1::SettingsSetResponse;
using SettingsSetService = ::dokime::host::v1::Settings;
using SettingsSchemaRequest = ::dokime::host::v1::SettingsSchemaRequest;
using SettingsSchemaResponse = ::dokime::host::v1::SettingsSchemaResponse;
using SettingsSchemaService = ::dokime::host::v1::Settings;
using StorageSourcesGetRequest = ::dokime::host::v1::storage::SourcesGetRequest;
using StorageSourcesGetResponse = ::dokime::host::v1::storage::SourcesGetResponse;
using StorageSourcesGetService = ::dokime::host::v1::storage::Sources;
using StorageSourcesSetRequest = ::dokime::host::v1::storage::SourcesSetRequest;
using StorageSourcesSetResponse = ::dokime::host::v1::storage::SourcesSetResponse;
using StorageSourcesSetService = ::dokime::host::v1::storage::Sources;
using StreamSchemasGetRequest = ::dokime::host::v1::StreamSchemasGetRequest;
using StreamSchemasGetResponse = ::dokime::host::v1::StreamSchemasGetResponse;
using StreamSchemasGetService = ::dokime::host::v1::StreamSchemas;
using StreamSchemasListRequest = ::dokime::host::v1::StreamSchemasListRequest;
using StreamSchemasListResponse = ::dokime::host::v1::StreamSchemasListResponse;
using StreamSchemasListService = ::dokime::host::v1::StreamSchemas;
using SuitesGetRequest = ::dokime::host::v1::SuitesGetRequest;
using SuitesGetResponse = ::dokime::host::v1::SuitesGetResponse;
using SuitesGetService = ::dokime::host::v1::Suites;
using SuitesListRequest = ::dokime::host::v1::SuitesListRequest;
using SuitesListResponse = ::dokime::host::v1::SuitesListResponse;
using SuitesListService = ::dokime::host::v1::Suites;
using AccessCertificatesCreateRequest = ::harmos::host::v1::access::CertificatesCreateRequest;
using AccessCertificatesCreateResponse = ::harmos::host::v1::access::CertificatesCreateResponse;
using AccessCertificatesCreateService = ::harmos::host::v1::access::Certificates;
using AccessListenerGetRequest = ::harmos::host::v1::access::ListenerGetRequest;
using AccessListenerGetResponse = ::harmos::host::v1::access::ListenerGetResponse;
using AccessListenerGetService = ::harmos::host::v1::access::Listener;
using AccessListenerSetRequest = ::harmos::host::v1::access::ListenerSetRequest;
using AccessListenerSetResponse = ::harmos::host::v1::access::ListenerSetResponse;
using AccessListenerSetService = ::harmos::host::v1::access::Listener;
using AccessPairingsCreateRequest = ::harmos::host::v1::access::PairingsCreateRequest;
using AccessPairingsCreateResponse = ::harmos::host::v1::access::PairingsCreateResponse;
using AccessPairingsCreateService = ::harmos::host::v1::access::Pairings;
using AccessPairingsListRequest = ::harmos::host::v1::access::PairingsListRequest;
using AccessPairingsListResponse = ::harmos::host::v1::access::PairingsListResponse;
using AccessPairingsListService = ::harmos::host::v1::access::Pairings;
using AccessPairingsDeleteRequest = ::harmos::host::v1::access::PairingsDeleteRequest;
using AccessPairingsDeleteResponse = ::harmos::host::v1::access::PairingsDeleteResponse;
using AccessPairingsDeleteService = ::harmos::host::v1::access::Pairings;
using InvocationsCreateRequest = ::harmos::host::v1::InvocationsCreateRequest;
using InvocationsCreateResponse = ::harmos::host::v1::InvocationsCreateResponse;
using InvocationsCreateService = ::harmos::host::v1::Invocations;
using InvocationsGetRequest = ::harmos::host::v1::InvocationsGetRequest;
using InvocationsGetResponse = ::harmos::host::v1::InvocationsGetResponse;
using InvocationsGetService = ::harmos::host::v1::Invocations;
using InvocationsCancelRequest = ::harmos::host::v1::InvocationsCancelRequest;
using InvocationsCancelResponse = ::harmos::host::v1::InvocationsCancelResponse;
using InvocationsCancelService = ::harmos::host::v1::Invocations;
using InvocationsWatchRequest = ::harmos::host::v1::InvocationsWatchRequest;
using InvocationsWatchResponse = ::harmos::host::v1::InvocationsWatchResponse;
using InvocationsWatchService = ::harmos::host::v1::Invocations;
using OperationsListRequest = ::harmos::host::v1::OperationsListRequest;
using OperationsListResponse = ::harmos::host::v1::OperationsListResponse;
using OperationsListService = ::harmos::host::v1::Operations;
using OperationsGetRequest = ::harmos::host::v1::OperationsGetRequest;
using OperationsGetResponse = ::harmos::host::v1::OperationsGetResponse;
using OperationsGetService = ::harmos::host::v1::Operations;
using PluginsSettingsListRequest = ::harmos::host::v1::plugins::SettingsListRequest;
using PluginsSettingsListResponse = ::harmos::host::v1::plugins::SettingsListResponse;
using PluginsSettingsListService = ::harmos::host::v1::plugins::Settings;
using PluginsSettingsSetRequest = ::harmos::host::v1::plugins::SettingsSetRequest;
using PluginsSettingsSetResponse = ::harmos::host::v1::plugins::SettingsSetResponse;
using PluginsSettingsSetService = ::harmos::host::v1::plugins::Settings;
using PluginsGetRequest = ::harmos::host::v1::PluginsGetRequest;
using PluginsGetResponse = ::harmos::host::v1::PluginsGetResponse;
using PluginsGetService = ::harmos::host::v1::Plugins;
using PluginsListRequest = ::harmos::host::v1::PluginsListRequest;
using PluginsListResponse = ::harmos::host::v1::PluginsListResponse;
using PluginsListService = ::harmos::host::v1::Plugins;
using PluginsWatchRequest = ::harmos::host::v1::PluginsWatchRequest;
using PluginsWatchResponse = ::harmos::host::v1::PluginsWatchResponse;
using PluginsWatchService = ::harmos::host::v1::Plugins;
using PluginsCreateRequest = ::harmos::host::v1::PluginsCreateRequest;
using PluginsCreateResponse = ::harmos::host::v1::PluginsCreateResponse;
using PluginsCreateService = ::harmos::host::v1::Plugins;
using PluginsSetRequest = ::harmos::host::v1::PluginsSetRequest;
using PluginsSetResponse = ::harmos::host::v1::PluginsSetResponse;
using PluginsSetService = ::harmos::host::v1::Plugins;
using PluginsDeleteRequest = ::harmos::host::v1::PluginsDeleteRequest;
using PluginsDeleteResponse = ::harmos::host::v1::PluginsDeleteResponse;
using PluginsDeleteService = ::harmos::host::v1::Plugins;
using PluginsRestartRequest = ::harmos::host::v1::PluginsRestartRequest;
using PluginsRestartResponse = ::harmos::host::v1::PluginsRestartResponse;
using PluginsRestartService = ::harmos::host::v1::Plugins;
using PluginsReplaceRequest = ::harmos::host::v1::PluginsReplaceRequest;
using PluginsReplaceResponse = ::harmos::host::v1::PluginsReplaceResponse;
using PluginsReplaceService = ::harmos::host::v1::Plugins;
using SchemasListRequest = ::harmos::host::v1::SchemasListRequest;
using SchemasListResponse = ::harmos::host::v1::SchemasListResponse;
using SchemasListService = ::harmos::host::v1::Schemas;
using StatusDiagnosticsGetRequest = ::harmos::host::v1::status::DiagnosticsGetRequest;
using StatusDiagnosticsGetResponse = ::harmos::host::v1::status::DiagnosticsGetResponse;
using StatusDiagnosticsGetService = ::harmos::host::v1::status::Diagnostics;
using StatusLogsListRequest = ::harmos::host::v1::status::LogsListRequest;
using StatusLogsListResponse = ::harmos::host::v1::status::LogsListResponse;
using StatusLogsListService = ::harmos::host::v1::status::Logs;
using StatusLogsWatchRequest = ::harmos::host::v1::status::LogsWatchRequest;
using StatusLogsWatchResponse = ::harmos::host::v1::status::LogsWatchResponse;
using StatusLogsWatchService = ::harmos::host::v1::status::Logs;
using StatusTraceEventsListRequest = ::harmos::host::v1::status::TraceEventsListRequest;
using StatusTraceEventsListResponse = ::harmos::host::v1::status::TraceEventsListResponse;
using StatusTraceEventsListService = ::harmos::host::v1::status::TraceEvents;
using StatusTraceEventsWatchRequest = ::harmos::host::v1::status::TraceEventsWatchRequest;
using StatusTraceEventsWatchResponse = ::harmos::host::v1::status::TraceEventsWatchResponse;
using StatusTraceEventsWatchService = ::harmos::host::v1::status::TraceEvents;
using StatusGetRequest = ::harmos::host::v1::StatusGetRequest;
using StatusGetResponse = ::harmos::host::v1::StatusGetResponse;
using StatusGetService = ::harmos::host::v1::Status;
using StatusWatchRequest = ::harmos::host::v1::StatusWatchRequest;
using StatusWatchResponse = ::harmos::host::v1::StatusWatchResponse;
using StatusWatchService = ::harmos::host::v1::Status;
using StreamsListRequest = ::harmos::host::v1::StreamsListRequest;
using StreamsListResponse = ::harmos::host::v1::StreamsListResponse;
using StreamsListService = ::harmos::host::v1::Streams;
using TimeGetRequest = ::harmos::host::v1::TimeGetRequest;
using TimeGetResponse = ::harmos::host::v1::TimeGetResponse;
using TimeGetService = ::harmos::host::v1::Time;
using TimeSleepRequest = ::harmos::host::v1::TimeSleepRequest;
using TimeSleepResponse = ::harmos::host::v1::TimeSleepResponse;
using TimeSleepService = ::harmos::host::v1::Time;
} // namespace dokime::detail
