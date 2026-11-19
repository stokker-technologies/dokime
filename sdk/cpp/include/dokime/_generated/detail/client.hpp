// @generated from protocol descriptors; do not edit.
#pragma once
#include "dokime/host/v1/access/certificates.grpc.pb.h"
#include "dokime/host/v1/access/listener.grpc.pb.h"
#include "dokime/host/v1/access/pairings.grpc.pb.h"
#include "dokime/host/v1/aggregates.grpc.pb.h"
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
#include "dokime/host/v1/invocations.grpc.pb.h"
#include "dokime/host/v1/media/captures.grpc.pb.h"
#include "dokime/host/v1/operations.grpc.pb.h"
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
#include "dokime/host/v1/status.grpc.pb.h"
#include "dokime/host/v1/status/diagnostics.grpc.pb.h"
#include "dokime/host/v1/status/logs.grpc.pb.h"
#include "dokime/host/v1/status/trace_events.grpc.pb.h"
#include "dokime/host/v1/storage/sources.grpc.pb.h"
#include "dokime/host/v1/stream_schemas.grpc.pb.h"
#include "dokime/host/v1/suites.grpc.pb.h"
namespace dokime::detail::packages {
namespace harmos::host::access {
using ::harmos::host::v1::access::CertificatesCreateRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::CertificatesCreateResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Certificates;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::ListenerGetRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::ListenerGetResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Listener;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::ListenerSetRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::ListenerSetResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Listener;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsCreateRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsCreateResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Pairings;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsListRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsListResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Pairings;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsDeleteRequest;
}
namespace harmos::host::access {
using ::harmos::host::v1::access::PairingsDeleteResponse;
}
namespace dokime::host::access {
using ::dokime::host::v1::access::Pairings;
}
namespace dokime::host {
using ::dokime::host::v1::AggregatesGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::AggregatesGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Aggregates;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesSetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesSetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesCreateRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesCreateResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesSealRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CyclesSealResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cycles;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::OverviewGetRequest;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::OverviewGetResponse;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::Overview;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::OverviewWatchRequest;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::OverviewWatchResponse;
}
namespace dokime::host::backend {
using ::dokime::host::v1::backend::Overview;
}
namespace dokime::host {
using ::dokime::host::v1::BackendAppearanceRequest;
}
namespace dokime::host {
using ::dokime::host::v1::BackendAppearanceResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Backend;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemasGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemasGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemas;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemasListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemasListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::RecordSchemas;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsSetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsSetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsCreateRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsCreateResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStartRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStartResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStopRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStopResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsCancelRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsCancelResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsSealRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsSealResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsDeleteRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsDeleteResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsParametersRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsParametersResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStepsRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ExecutionsStepsResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Executions;
}
namespace dokime::host {
using ::dokime::host::v1::CasesCreateRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesCreateResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host {
using ::dokime::host::v1::CasesGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host {
using ::dokime::host::v1::CasesListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host {
using ::dokime::host::v1::CasesWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host {
using ::dokime::host::v1::CasesDeleteRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesDeleteResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host {
using ::dokime::host::v1::CasesParametersRequest;
}
namespace dokime::host {
using ::dokime::host::v1::CasesParametersResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Cases;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsGetRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsGetResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::Artifacts;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsCreateRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsCreateResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::Artifacts;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsListRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsListResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::Artifacts;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsWatchRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ArtifactsWatchResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::Artifacts;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesGetRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesGetResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFiles;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesListRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesListResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFiles;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesWatchRequest;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFilesWatchResponse;
}
namespace dokime::host::content {
using ::dokime::host::v1::content::ExecutionFiles;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SnapshotsListRequest;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SnapshotsListResponse;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::Snapshots;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SnapshotsWatchRequest;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SnapshotsWatchResponse;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::Snapshots;
}
namespace dokime::host {
using ::dokime::host::v1::DocumentsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::DocumentsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Documents;
}
namespace dokime::host {
using ::dokime::host::v1::DocumentsWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::DocumentsWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Documents;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesGetRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesGetResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Notes;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesSetRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesSetResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Notes;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesListRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesListResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Notes;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesWatchRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesWatchResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Notes;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesCreateRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::NotesCreateResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Notes;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEventsListRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEventsListResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEvents;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEventsWatchRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEventsWatchResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::HealthEvents;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::RecordsListRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::RecordsListResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Records;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::RecordsWatchRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::RecordsWatchResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Records;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::SamplesListRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::SamplesListResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Samples;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::SamplesWatchRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::SamplesWatchResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Samples;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::TraceabilityGetRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::TraceabilityGetResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Traceability;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::TraceabilityWatchRequest;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::TraceabilityWatchResponse;
}
namespace dokime::host::evidence {
using ::dokime::host::v1::evidence::Traceability;
}
namespace dokime::host {
using ::dokime::host::v1::EvidenceStatisticsRequest;
}
namespace dokime::host {
using ::dokime::host::v1::EvidenceStatisticsResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Evidence;
}
namespace dokime::host {
using ::dokime::host::v1::EvidenceArchiveRequest;
}
namespace dokime::host {
using ::dokime::host::v1::EvidenceArchiveResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Evidence;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::CommandsGetRequest;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::CommandsGetResponse;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::Commands;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::CommandsCreateRequest;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::CommandsCreateResponse;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::Commands;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::ProgressGetRequest;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::ProgressGetResponse;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::Progress;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::ProgressWatchRequest;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::ProgressWatchResponse;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::Progress;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::SchemasGetRequest;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::SchemasGetResponse;
}
namespace dokime::host::executions {
using ::dokime::host::v1::executions::Schemas;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsCreateRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsCreateResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsGetRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsGetResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsResumeRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsResumeResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsAppendRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsAppendResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsPublishRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsPublishResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsDeleteRequest;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::UploadsDeleteResponse;
}
namespace dokime::host::files {
using ::dokime::host::v1::files::Uploads;
}
namespace dokime::host {
using ::dokime::host::v1::FilesGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::FilesGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Files;
}
namespace dokime::host {
using ::dokime::host::v1::FilesReadRequest;
}
namespace dokime::host {
using ::dokime::host::v1::FilesReadResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Files;
}
namespace dokime::host {
using ::dokime::host::v1::FilesWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::FilesWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Files;
}
namespace dokime::host {
using ::dokime::host::v1::FilesDeleteRequest;
}
namespace dokime::host {
using ::dokime::host::v1::FilesDeleteResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Files;
}
namespace harmos::host {
using ::harmos::host::v1::InvocationsCreateRequest;
}
namespace harmos::host {
using ::harmos::host::v1::InvocationsCreateResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Invocations;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesCreateRequest;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesCreateResponse;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::Captures;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesGetRequest;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesGetResponse;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::Captures;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesWatchRequest;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesWatchResponse;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::Captures;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesStopRequest;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::CapturesStopResponse;
}
namespace dokime::host::media {
using ::dokime::host::v1::media::Captures;
}
namespace harmos::host {
using ::harmos::host::v1::OperationsListRequest;
}
namespace harmos::host {
using ::harmos::host::v1::OperationsListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Operations;
}
namespace harmos::host {
using ::harmos::host::v1::OperationsGetRequest;
}
namespace harmos::host {
using ::harmos::host::v1::OperationsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Operations;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SettingsGetRequest;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SettingsGetResponse;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::Settings;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SettingsSetRequest;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::SettingsSetResponse;
}
namespace dokime::host::plots {
using ::dokime::host::v1::plots::Settings;
}
namespace dokime::host::plugins {
using ::dokime::host::v1::plugins::SecretsListRequest;
}
namespace dokime::host::plugins {
using ::dokime::host::v1::plugins::SecretsListResponse;
}
namespace dokime::host::plugins {
using ::dokime::host::v1::plugins::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitionsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitionsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitions;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitionsListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitionsListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::PromptDefinitions;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Prompts;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Prompts;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::PromptsWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Prompts;
}
namespace dokime::host::prompts {
using ::dokime::host::v1::prompts::AnswersCreateRequest;
}
namespace dokime::host::prompts {
using ::dokime::host::v1::prompts::AnswersCreateResponse;
}
namespace dokime::host::prompts {
using ::dokime::host::v1::prompts::Answers;
}
namespace dokime::host {
using ::dokime::host::v1::SuitesGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SuitesGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Suites;
}
namespace dokime::host {
using ::dokime::host::v1::SuitesListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SuitesListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Suites;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Reports;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsCreateRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsCreateResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Reports;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsPublishRequest;
}
namespace dokime::host {
using ::dokime::host::v1::ReportsPublishResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Reports;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsSetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsSetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsWatchRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsDeleteRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsDeleteResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsExportRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsExportResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsImportRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SecretsImportResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Secrets;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Settings;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsSetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsSetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Settings;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsSchemaRequest;
}
namespace dokime::host {
using ::dokime::host::v1::SettingsSchemaResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Settings;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::DiagnosticsGetRequest;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::DiagnosticsGetResponse;
}
namespace dokime::host::status {
using ::dokime::host::v1::status::Diagnostics;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::LogsWatchRequest;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::LogsWatchResponse;
}
namespace dokime::host::status {
using ::dokime::host::v1::status::Logs;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::TraceEventsListRequest;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::TraceEventsListResponse;
}
namespace dokime::host::status {
using ::dokime::host::v1::status::TraceEvents;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::TraceEventsWatchRequest;
}
namespace harmos::host::status {
using ::harmos::host::v1::status::TraceEventsWatchResponse;
}
namespace dokime::host::status {
using ::dokime::host::v1::status::TraceEvents;
}
namespace harmos::host {
using ::harmos::host::v1::StatusGetRequest;
}
namespace harmos::host {
using ::harmos::host::v1::StatusGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Status;
}
namespace harmos::host {
using ::harmos::host::v1::StatusWatchRequest;
}
namespace harmos::host {
using ::harmos::host::v1::StatusWatchResponse;
}
namespace dokime::host {
using ::dokime::host::v1::Status;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::SourcesGetRequest;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::SourcesGetResponse;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::Sources;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::SourcesSetRequest;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::SourcesSetResponse;
}
namespace dokime::host::storage {
using ::dokime::host::v1::storage::Sources;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemasGetRequest;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemasGetResponse;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemas;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemasListRequest;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemasListResponse;
}
namespace dokime::host {
using ::dokime::host::v1::StreamSchemas;
}
} // namespace dokime::detail::packages
