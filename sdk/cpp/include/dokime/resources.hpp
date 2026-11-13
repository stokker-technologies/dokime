// @generated from the protocol descriptor table; do not edit.
#pragma once
#include <grpcpp/grpcpp.h>
#include <memory>
#include <stdexcept>
#pragma push_macro("AdvanceProjection")
#undef AdvanceProjection
#pragma push_macro("Appearance")
#undef Appearance
#pragma push_macro("AppendFileBytes")
#undef AppendFileBytes
#pragma push_macro("AppendObjectRevisions")
#undef AppendObjectRevisions
#pragma push_macro("AppendRecords")
#undef AppendRecords
#pragma push_macro("AppendSamples")
#undef AppendSamples
#pragma push_macro("Archive")
#undef Archive
#pragma push_macro("Cancel")
#undef Cancel
#pragma push_macro("Capabilities")
#undef Capabilities
#pragma push_macro("CaptureFile")
#undef CaptureFile
#pragma push_macro("Create")
#undef Create
#pragma push_macro("CreateFileProduction")
#undef CreateFileProduction
#pragma push_macro("CreateFileWriter")
#undef CreateFileWriter
#pragma push_macro("CreateSnapshot")
#undef CreateSnapshot
#pragma push_macro("Delete")
#undef Delete
#pragma push_macro("DeleteFileProduction")
#undef DeleteFileProduction
#pragma push_macro("DeleteSnapshot")
#undef DeleteSnapshot
#pragma push_macro("Export")
#undef Export
#pragma push_macro("Get")
#undef Get
#pragma push_macro("GetFileBytes")
#undef GetFileBytes
#pragma push_macro("GetFileHash")
#undef GetFileHash
#pragma push_macro("GetFileProduction")
#undef GetFileProduction
#pragma push_macro("GetIndexDefinition")
#undef GetIndexDefinition
#pragma push_macro("GetObject")
#undef GetObject
#pragma push_macro("GetProjectionDefinition")
#undef GetProjectionDefinition
#pragma push_macro("GetProjectionStatus")
#undef GetProjectionStatus
#pragma push_macro("Import")
#undef Import
#pragma push_macro("Invoke")
#undef Invoke
#pragma push_macro("List")
#undef List
#pragma push_macro("ListColumnRows")
#undef ListColumnRows
#pragma push_macro("ListDirectoryEntries")
#undef ListDirectoryEntries
#pragma push_macro("ListIndexEntries")
#undef ListIndexEntries
#pragma push_macro("ListProjectionRows")
#undef ListProjectionRows
#pragma push_macro("ListRows")
#undef ListRows
#pragma push_macro("Parameters")
#undef Parameters
#pragma push_macro("Publish")
#undef Publish
#pragma push_macro("PublishFileProduction")
#undef PublishFileProduction
#pragma push_macro("Restart")
#undef Restart
#pragma push_macro("Schema")
#undef Schema
#pragma push_macro("Seal")
#undef Seal
#pragma push_macro("Set")
#undef Set
#pragma push_macro("SetIndexDefinition")
#undef SetIndexDefinition
#pragma push_macro("SetObject")
#undef SetObject
#pragma push_macro("SetProjectionDefinition")
#undef SetProjectionDefinition
#pragma push_macro("Sleep")
#undef Sleep
#pragma push_macro("Start")
#undef Start
#pragma push_macro("Statistics")
#undef Statistics
#pragma push_macro("Steps")
#undef Steps
#pragma push_macro("Stop")
#undef Stop
#pragma push_macro("Watch")
#undef Watch
#pragma push_macro("WatchDirectoryEntries")
#undef WatchDirectoryEntries
#pragma push_macro("WatchFileBytes")
#undef WatchFileBytes
#pragma push_macro("WatchFileProduction")
#undef WatchFileProduction
#pragma push_macro("WatchRows")
#undef WatchRows
#include "dokime/client/v1/aggregates.grpc.pb.h"
#include "dokime/client/v1/analyses.grpc.pb.h"
#include "dokime/client/v1/backend.grpc.pb.h"
#include "dokime/client/v1/backend/overview.grpc.pb.h"
#include "dokime/client/v1/cases.grpc.pb.h"
#include "dokime/client/v1/content/artifacts.grpc.pb.h"
#include "dokime/client/v1/content/execution_files.grpc.pb.h"
#include "dokime/client/v1/cycles.grpc.pb.h"
#include "dokime/client/v1/documents.grpc.pb.h"
#include "dokime/client/v1/evidence.grpc.pb.h"
#include "dokime/client/v1/evidence/health_events.grpc.pb.h"
#include "dokime/client/v1/evidence/notes.grpc.pb.h"
#include "dokime/client/v1/evidence/records.grpc.pb.h"
#include "dokime/client/v1/evidence/samples.grpc.pb.h"
#include "dokime/client/v1/evidence/traceability.grpc.pb.h"
#include "dokime/client/v1/executions.grpc.pb.h"
#include "dokime/client/v1/executions/commands.grpc.pb.h"
#include "dokime/client/v1/executions/progress.grpc.pb.h"
#include "dokime/client/v1/executions/schemas.grpc.pb.h"
#include "dokime/client/v1/media/captures.grpc.pb.h"
#include "dokime/client/v1/plots/settings.grpc.pb.h"
#include "dokime/client/v1/plots/snapshots.grpc.pb.h"
#include "dokime/client/v1/plugins/settings.grpc.pb.h"
#include "dokime/client/v1/prompt_definitions.grpc.pb.h"
#include "dokime/client/v1/prompts.grpc.pb.h"
#include "dokime/client/v1/prompts/answers.grpc.pb.h"
#include "dokime/client/v1/record_schemas.grpc.pb.h"
#include "dokime/client/v1/reports.grpc.pb.h"
#include "dokime/client/v1/secrets.grpc.pb.h"
#include "dokime/client/v1/settings.grpc.pb.h"
#include "dokime/client/v1/storage/sources.grpc.pb.h"
#include "dokime/client/v1/stream_schemas.grpc.pb.h"
#include "dokime/client/v1/suites.grpc.pb.h"
#include "harmos/v1/access/certificates.grpc.pb.h"
#include "harmos/v1/access/listener.grpc.pb.h"
#include "harmos/v1/access/pairings.grpc.pb.h"
#include "harmos/v1/catalog_entries.grpc.pb.h"
#include "harmos/v1/invocations.grpc.pb.h"
#include "harmos/v1/plugins.grpc.pb.h"
#include "harmos/v1/status.grpc.pb.h"
#include "harmos/v1/status/diagnostics.grpc.pb.h"
#include "harmos/v1/status/logs.grpc.pb.h"
#include "harmos/v1/status/trace_events.grpc.pb.h"
#include "harmos/v1/storage.grpc.pb.h"
#include "harmos/v1/time.grpc.pb.h"
namespace dokime::resources {
struct AccessCertificatesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessCertificatesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::access::CertificatesCreateResponse create(grpc::ClientContext &context,
                                                          const ::harmos::v1::access::CertificatesCreateRequest &request) {
    ::harmos::v1::access::CertificatesCreateResponse response;
    const auto status = ::harmos::v1::access::Certificates::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
};
struct AccessListenerResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessListenerResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::access::ListenerGetResponse get(grpc::ClientContext &context, const ::harmos::v1::access::ListenerGetRequest &request) {
    ::harmos::v1::access::ListenerGetResponse response;
    const auto status = ::harmos::v1::access::Listener::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::harmos::v1::access::ListenerSetResponse set(grpc::ClientContext &context, const ::harmos::v1::access::ListenerSetRequest &request) {
    ::harmos::v1::access::ListenerSetResponse response;
    const auto status = ::harmos::v1::access::Listener::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
};
struct AccessPairingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessPairingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::access::PairingsCreateResponse create(grpc::ClientContext &context,
                                                      const ::harmos::v1::access::PairingsCreateRequest &request) {
    ::harmos::v1::access::PairingsCreateResponse response;
    const auto status = ::harmos::v1::access::Pairings::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::harmos::v1::access::PairingsListResponse list(grpc::ClientContext &context, const ::harmos::v1::access::PairingsListRequest &request) {
    ::harmos::v1::access::PairingsListResponse response;
    const auto status = ::harmos::v1::access::Pairings::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  ::harmos::v1::access::PairingsDeleteResponse delete_(grpc::ClientContext &context,
                                                       const ::harmos::v1::access::PairingsDeleteRequest &request) {
    ::harmos::v1::access::PairingsDeleteResponse response;
    const auto status = ::harmos::v1::access::Pairings::NewStub(channel)->Delete(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Delete: ") + status.error_message());
    return response;
  }
};
struct BackendOverviewResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit BackendOverviewResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::backend::OverviewGetResponse get(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::backend::OverviewGetRequest &request) {
    ::dokime::client::v1::backend::OverviewGetResponse response;
    const auto status = ::dokime::client::v1::backend::Overview::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::backend::OverviewWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::backend::OverviewWatchRequest &request) {
    return ::dokime::client::v1::backend::Overview::NewStub(channel)->Watch(&context, request);
  }
};
struct ContentArtifactsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentArtifactsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::content::ArtifactsGetResponse get(grpc::ClientContext &context,
                                                          const ::dokime::client::v1::content::ArtifactsGetRequest &request) {
    ::dokime::client::v1::content::ArtifactsGetResponse response;
    const auto status = ::dokime::client::v1::content::Artifacts::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::content::ArtifactsCreateResponse create(grpc::ClientContext &context,
                                                                const ::dokime::client::v1::content::ArtifactsCreateRequest &request) {
    ::dokime::client::v1::content::ArtifactsCreateResponse response;
    const auto status = ::dokime::client::v1::content::Artifacts::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::content::ArtifactsListResponse list(grpc::ClientContext &context,
                                                            const ::dokime::client::v1::content::ArtifactsListRequest &request) {
    ::dokime::client::v1::content::ArtifactsListResponse response;
    const auto status = ::dokime::client::v1::content::Artifacts::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::content::ArtifactsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::content::ArtifactsWatchRequest &request) {
    return ::dokime::client::v1::content::Artifacts::NewStub(channel)->Watch(&context, request);
  }
};
struct ContentExecutionFilesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentExecutionFilesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::content::ExecutionFilesGetResponse get(grpc::ClientContext &context,
                                                               const ::dokime::client::v1::content::ExecutionFilesGetRequest &request) {
    ::dokime::client::v1::content::ExecutionFilesGetResponse response;
    const auto status = ::dokime::client::v1::content::ExecutionFiles::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::content::ExecutionFilesListResponse list(grpc::ClientContext &context,
                                                                 const ::dokime::client::v1::content::ExecutionFilesListRequest &request) {
    ::dokime::client::v1::content::ExecutionFilesListResponse response;
    const auto status = ::dokime::client::v1::content::ExecutionFiles::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::content::ExecutionFilesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::content::ExecutionFilesWatchRequest &request) {
    return ::dokime::client::v1::content::ExecutionFiles::NewStub(channel)->Watch(&context, request);
  }
};
struct EvidenceHealthEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceHealthEventsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::evidence::HealthEventsListResponse list(grpc::ClientContext &context,
                                                                const ::dokime::client::v1::evidence::HealthEventsListRequest &request) {
    ::dokime::client::v1::evidence::HealthEventsListResponse response;
    const auto status = ::dokime::client::v1::evidence::HealthEvents::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::evidence::HealthEventsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::evidence::HealthEventsWatchRequest &request) {
    return ::dokime::client::v1::evidence::HealthEvents::NewStub(channel)->Watch(&context, request);
  }
};
struct EvidenceNotesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceNotesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::evidence::NotesGetResponse get(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::evidence::NotesGetRequest &request) {
    ::dokime::client::v1::evidence::NotesGetResponse response;
    const auto status = ::dokime::client::v1::evidence::Notes::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::evidence::NotesSetResponse set(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::evidence::NotesSetRequest &request) {
    ::dokime::client::v1::evidence::NotesSetResponse response;
    const auto status = ::dokime::client::v1::evidence::Notes::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::evidence::NotesListResponse list(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::evidence::NotesListRequest &request) {
    ::dokime::client::v1::evidence::NotesListResponse response;
    const auto status = ::dokime::client::v1::evidence::Notes::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::evidence::NotesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::evidence::NotesWatchRequest &request) {
    return ::dokime::client::v1::evidence::Notes::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::evidence::NotesCreateResponse create(grpc::ClientContext &context,
                                                             const ::dokime::client::v1::evidence::NotesCreateRequest &request) {
    ::dokime::client::v1::evidence::NotesCreateResponse response;
    const auto status = ::dokime::client::v1::evidence::Notes::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
};
struct EvidenceRecordsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceRecordsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::evidence::RecordsListResponse list(grpc::ClientContext &context,
                                                           const ::dokime::client::v1::evidence::RecordsListRequest &request) {
    ::dokime::client::v1::evidence::RecordsListResponse response;
    const auto status = ::dokime::client::v1::evidence::Records::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::evidence::RecordsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::evidence::RecordsWatchRequest &request) {
    return ::dokime::client::v1::evidence::Records::NewStub(channel)->Watch(&context, request);
  }
};
struct EvidenceSamplesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceSamplesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::evidence::SamplesListResponse list(grpc::ClientContext &context,
                                                           const ::dokime::client::v1::evidence::SamplesListRequest &request) {
    ::dokime::client::v1::evidence::SamplesListResponse response;
    const auto status = ::dokime::client::v1::evidence::Samples::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::evidence::SamplesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::evidence::SamplesWatchRequest &request) {
    return ::dokime::client::v1::evidence::Samples::NewStub(channel)->Watch(&context, request);
  }
};
struct EvidenceTraceabilityResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceTraceabilityResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::evidence::TraceabilityGetResponse get(grpc::ClientContext &context,
                                                              const ::dokime::client::v1::evidence::TraceabilityGetRequest &request) {
    ::dokime::client::v1::evidence::TraceabilityGetResponse response;
    const auto status = ::dokime::client::v1::evidence::Traceability::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::evidence::TraceabilityWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::evidence::TraceabilityWatchRequest &request) {
    return ::dokime::client::v1::evidence::Traceability::NewStub(channel)->Watch(&context, request);
  }
};
struct ExecutionsCommandsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsCommandsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::executions::CommandsGetResponse get(grpc::ClientContext &context,
                                                            const ::dokime::client::v1::executions::CommandsGetRequest &request) {
    ::dokime::client::v1::executions::CommandsGetResponse response;
    const auto status = ::dokime::client::v1::executions::Commands::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::executions::CommandsCreateResponse create(grpc::ClientContext &context,
                                                                  const ::dokime::client::v1::executions::CommandsCreateRequest &request) {
    ::dokime::client::v1::executions::CommandsCreateResponse response;
    const auto status = ::dokime::client::v1::executions::Commands::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
};
struct ExecutionsProgressResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsProgressResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::executions::ProgressGetResponse get(grpc::ClientContext &context,
                                                            const ::dokime::client::v1::executions::ProgressGetRequest &request) {
    ::dokime::client::v1::executions::ProgressGetResponse response;
    const auto status = ::dokime::client::v1::executions::Progress::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::executions::ProgressWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::executions::ProgressWatchRequest &request) {
    return ::dokime::client::v1::executions::Progress::NewStub(channel)->Watch(&context, request);
  }
};
struct ExecutionsSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::executions::SchemasGetResponse get(grpc::ClientContext &context,
                                                           const ::dokime::client::v1::executions::SchemasGetRequest &request) {
    ::dokime::client::v1::executions::SchemasGetResponse response;
    const auto status = ::dokime::client::v1::executions::Schemas::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
};
struct MediaCapturesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit MediaCapturesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::media::CapturesCreateResponse create(grpc::ClientContext &context,
                                                             const ::dokime::client::v1::media::CapturesCreateRequest &request) {
    ::dokime::client::v1::media::CapturesCreateResponse response;
    const auto status = ::dokime::client::v1::media::Captures::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::media::CapturesGetResponse get(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::media::CapturesGetRequest &request) {
    ::dokime::client::v1::media::CapturesGetResponse response;
    const auto status = ::dokime::client::v1::media::Captures::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::media::CapturesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::media::CapturesWatchRequest &request) {
    return ::dokime::client::v1::media::Captures::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::media::CapturesStopResponse stop(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::media::CapturesStopRequest &request) {
    ::dokime::client::v1::media::CapturesStopResponse response;
    const auto status = ::dokime::client::v1::media::Captures::NewStub(channel)->Stop(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Stop: ") + status.error_message());
    return response;
  }
};
struct PlotsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::plots::SettingsGetResponse get(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::plots::SettingsGetRequest &request) {
    ::dokime::client::v1::plots::SettingsGetResponse response;
    const auto status = ::dokime::client::v1::plots::Settings::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::plots::SettingsSetResponse set(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::plots::SettingsSetRequest &request) {
    ::dokime::client::v1::plots::SettingsSetResponse response;
    const auto status = ::dokime::client::v1::plots::Settings::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
};
struct PlotsSnapshotsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSnapshotsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::plots::SnapshotsListResponse list(grpc::ClientContext &context,
                                                          const ::dokime::client::v1::plots::SnapshotsListRequest &request) {
    ::dokime::client::v1::plots::SnapshotsListResponse response;
    const auto status = ::dokime::client::v1::plots::Snapshots::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::plots::SnapshotsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::plots::SnapshotsWatchRequest &request) {
    return ::dokime::client::v1::plots::Snapshots::NewStub(channel)->Watch(&context, request);
  }
};
struct PluginsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PluginsSettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::plugins::SettingsGetResponse get(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::plugins::SettingsGetRequest &request) {
    ::dokime::client::v1::plugins::SettingsGetResponse response;
    const auto status = ::dokime::client::v1::plugins::Settings::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::plugins::SettingsSetResponse set(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::plugins::SettingsSetRequest &request) {
    ::dokime::client::v1::plugins::SettingsSetResponse response;
    const auto status = ::dokime::client::v1::plugins::Settings::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
};
struct PromptsAnswersResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptsAnswersResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::prompts::AnswersCreateResponse create(grpc::ClientContext &context,
                                                              const ::dokime::client::v1::prompts::AnswersCreateRequest &request) {
    ::dokime::client::v1::prompts::AnswersCreateResponse response;
    const auto status = ::dokime::client::v1::prompts::Answers::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
};
struct StatusDiagnosticsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusDiagnosticsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::status::DiagnosticsGetResponse get(grpc::ClientContext &context,
                                                   const ::harmos::v1::status::DiagnosticsGetRequest &request) {
    ::harmos::v1::status::DiagnosticsGetResponse response;
    const auto status = ::harmos::v1::status::Diagnostics::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
};
struct StatusLogsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusLogsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::status::LogsListResponse list(grpc::ClientContext &context, const ::harmos::v1::status::LogsListRequest &request) {
    ::harmos::v1::status::LogsListResponse response;
    const auto status = ::harmos::v1::status::Logs::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::status::LogsWatchResponse>>
  watch(grpc::ClientContext &context, const ::harmos::v1::status::LogsWatchRequest &request) {
    return ::harmos::v1::status::Logs::NewStub(channel)->Watch(&context, request);
  }
};
struct StatusTraceEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusTraceEventsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::status::TraceEventsListResponse list(grpc::ClientContext &context,
                                                     const ::harmos::v1::status::TraceEventsListRequest &request) {
    ::harmos::v1::status::TraceEventsListResponse response;
    const auto status = ::harmos::v1::status::TraceEvents::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::status::TraceEventsWatchResponse>>
  watch(grpc::ClientContext &context, const ::harmos::v1::status::TraceEventsWatchRequest &request) {
    return ::harmos::v1::status::TraceEvents::NewStub(channel)->Watch(&context, request);
  }
};
struct StorageSourcesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StorageSourcesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::storage::SourcesGetResponse get(grpc::ClientContext &context,
                                                        const ::dokime::client::v1::storage::SourcesGetRequest &request) {
    ::dokime::client::v1::storage::SourcesGetResponse response;
    const auto status = ::dokime::client::v1::storage::Sources::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::storage::SourcesSetResponse set(grpc::ClientContext &context,
                                                        const ::dokime::client::v1::storage::SourcesSetRequest &request) {
    ::dokime::client::v1::storage::SourcesSetResponse response;
    const auto status = ::dokime::client::v1::storage::Sources::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
};
struct AccessResource {
  AccessCertificatesResource certificates;
  AccessListenerResource listener;
  AccessPairingsResource pairings;
  explicit AccessResource(std::shared_ptr<grpc::Channel> channel) : certificates(channel), listener(channel), pairings(channel) {}
};
struct AggregatesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AggregatesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::AggregatesGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::AggregatesGetRequest &request) {
    ::dokime::client::v1::AggregatesGetResponse response;
    const auto status = ::dokime::client::v1::Aggregates::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
};
struct AnalysesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AnalysesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::AnalysesListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::AnalysesListRequest &request) {
    ::dokime::client::v1::AnalysesListResponse response;
    const auto status = ::dokime::client::v1::Analyses::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct BackendResource {
  std::shared_ptr<grpc::Channel> channel;
  BackendOverviewResource overview;
  explicit BackendResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), overview(channel) {}
  ::dokime::client::v1::BackendAppearanceResponse appearance(grpc::ClientContext &context,
                                                             const ::dokime::client::v1::BackendAppearanceRequest &request) {
    ::dokime::client::v1::BackendAppearanceResponse response;
    const auto status = ::dokime::client::v1::Backend::NewStub(channel)->Appearance(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Appearance: ") + status.error_message());
    return response;
  }
};
struct CasesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CasesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::CasesCreateResponse create(grpc::ClientContext &context, const ::dokime::client::v1::CasesCreateRequest &request) {
    ::dokime::client::v1::CasesCreateResponse response;
    const auto status = ::dokime::client::v1::Cases::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CasesGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::CasesGetRequest &request) {
    ::dokime::client::v1::CasesGetResponse response;
    const auto status = ::dokime::client::v1::Cases::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CasesListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::CasesListRequest &request) {
    ::dokime::client::v1::CasesListResponse response;
    const auto status = ::dokime::client::v1::Cases::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::CasesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::CasesWatchRequest &request) {
    return ::dokime::client::v1::Cases::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::CasesDeleteResponse delete_(grpc::ClientContext &context, const ::dokime::client::v1::CasesDeleteRequest &request) {
    ::dokime::client::v1::CasesDeleteResponse response;
    const auto status = ::dokime::client::v1::Cases::NewStub(channel)->Delete(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Delete: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CasesParametersResponse parameters(grpc::ClientContext &context,
                                                           const ::dokime::client::v1::CasesParametersRequest &request) {
    ::dokime::client::v1::CasesParametersResponse response;
    const auto status = ::dokime::client::v1::Cases::NewStub(channel)->Parameters(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Parameters: ") + status.error_message());
    return response;
  }
};
struct CatalogEntriesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CatalogEntriesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::CatalogEntriesListResponse list(grpc::ClientContext &context, const ::harmos::v1::CatalogEntriesListRequest &request) {
    ::harmos::v1::CatalogEntriesListResponse response;
    const auto status = ::harmos::v1::CatalogEntries::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct ContentResource {
  ContentArtifactsResource artifacts;
  ContentExecutionFilesResource execution_files;
  explicit ContentResource(std::shared_ptr<grpc::Channel> channel) : artifacts(channel), execution_files(channel) {}
};
struct CyclesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CyclesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::CyclesGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::CyclesGetRequest &request) {
    ::dokime::client::v1::CyclesGetResponse response;
    const auto status = ::dokime::client::v1::Cycles::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CyclesSetResponse set(grpc::ClientContext &context, const ::dokime::client::v1::CyclesSetRequest &request) {
    ::dokime::client::v1::CyclesSetResponse response;
    const auto status = ::dokime::client::v1::Cycles::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CyclesListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::CyclesListRequest &request) {
    ::dokime::client::v1::CyclesListResponse response;
    const auto status = ::dokime::client::v1::Cycles::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::CyclesWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::CyclesWatchRequest &request) {
    return ::dokime::client::v1::Cycles::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::CyclesCreateResponse create(grpc::ClientContext &context,
                                                    const ::dokime::client::v1::CyclesCreateRequest &request) {
    ::dokime::client::v1::CyclesCreateResponse response;
    const auto status = ::dokime::client::v1::Cycles::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::CyclesSealResponse seal(grpc::ClientContext &context, const ::dokime::client::v1::CyclesSealRequest &request) {
    ::dokime::client::v1::CyclesSealResponse response;
    const auto status = ::dokime::client::v1::Cycles::NewStub(channel)->Seal(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Seal: ") + status.error_message());
    return response;
  }
};
struct DocumentsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit DocumentsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::DocumentsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::DocumentsGetRequest &request) {
    ::dokime::client::v1::DocumentsGetResponse response;
    const auto status = ::dokime::client::v1::Documents::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::DocumentsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::DocumentsWatchRequest &request) {
    return ::dokime::client::v1::Documents::NewStub(channel)->Watch(&context, request);
  }
};
struct EvidenceResource {
  std::shared_ptr<grpc::Channel> channel;
  EvidenceHealthEventsResource health_events;
  EvidenceNotesResource notes;
  EvidenceRecordsResource records;
  EvidenceSamplesResource samples;
  EvidenceTraceabilityResource traceability;
  explicit EvidenceResource(std::shared_ptr<grpc::Channel> channel)
      : channel(channel), health_events(channel), notes(channel), records(channel), samples(channel), traceability(channel) {}
  ::dokime::client::v1::EvidenceStatisticsResponse statistics(grpc::ClientContext &context,
                                                              const ::dokime::client::v1::EvidenceStatisticsRequest &request) {
    ::dokime::client::v1::EvidenceStatisticsResponse response;
    const auto status = ::dokime::client::v1::Evidence::NewStub(channel)->Statistics(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Statistics: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::EvidenceArchiveResponse archive(grpc::ClientContext &context,
                                                        const ::dokime::client::v1::EvidenceArchiveRequest &request) {
    ::dokime::client::v1::EvidenceArchiveResponse response;
    const auto status = ::dokime::client::v1::Evidence::NewStub(channel)->Archive(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Archive: ") + status.error_message());
    return response;
  }
};
struct ExecutionsResource {
  std::shared_ptr<grpc::Channel> channel;
  ExecutionsCommandsResource commands;
  ExecutionsProgressResource progress;
  ExecutionsSchemasResource schemas;
  explicit ExecutionsResource(std::shared_ptr<grpc::Channel> channel)
      : channel(channel), commands(channel), progress(channel), schemas(channel) {}
  ::dokime::client::v1::ExecutionsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::ExecutionsGetRequest &request) {
    ::dokime::client::v1::ExecutionsGetResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsSetResponse set(grpc::ClientContext &context, const ::dokime::client::v1::ExecutionsSetRequest &request) {
    ::dokime::client::v1::ExecutionsSetResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsListResponse list(grpc::ClientContext &context,
                                                    const ::dokime::client::v1::ExecutionsListRequest &request) {
    ::dokime::client::v1::ExecutionsListResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::ExecutionsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::ExecutionsWatchRequest &request) {
    return ::dokime::client::v1::Executions::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::ExecutionsCreateResponse create(grpc::ClientContext &context,
                                                        const ::dokime::client::v1::ExecutionsCreateRequest &request) {
    ::dokime::client::v1::ExecutionsCreateResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsStartResponse start(grpc::ClientContext &context,
                                                      const ::dokime::client::v1::ExecutionsStartRequest &request) {
    ::dokime::client::v1::ExecutionsStartResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Start(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Start: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsStopResponse stop(grpc::ClientContext &context,
                                                    const ::dokime::client::v1::ExecutionsStopRequest &request) {
    ::dokime::client::v1::ExecutionsStopResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Stop(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Stop: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsCancelResponse cancel(grpc::ClientContext &context,
                                                        const ::dokime::client::v1::ExecutionsCancelRequest &request) {
    ::dokime::client::v1::ExecutionsCancelResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Cancel(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Cancel: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsDeleteResponse delete_(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::ExecutionsDeleteRequest &request) {
    ::dokime::client::v1::ExecutionsDeleteResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Delete(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Delete: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsParametersResponse parameters(grpc::ClientContext &context,
                                                                const ::dokime::client::v1::ExecutionsParametersRequest &request) {
    ::dokime::client::v1::ExecutionsParametersResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Parameters(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Parameters: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ExecutionsStepsResponse steps(grpc::ClientContext &context,
                                                      const ::dokime::client::v1::ExecutionsStepsRequest &request) {
    ::dokime::client::v1::ExecutionsStepsResponse response;
    const auto status = ::dokime::client::v1::Executions::NewStub(channel)->Steps(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Steps: ") + status.error_message());
    return response;
  }
};
struct InvocationsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit InvocationsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::InvocationsInvokeResponse invoke(grpc::ClientContext &context, const ::harmos::v1::InvocationsInvokeRequest &request) {
    ::harmos::v1::InvocationsInvokeResponse response;
    const auto status = ::harmos::v1::Invocations::NewStub(channel)->Invoke(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Invoke: ") + status.error_message());
    return response;
  }
  ::harmos::v1::InvocationsGetResponse get(grpc::ClientContext &context, const ::harmos::v1::InvocationsGetRequest &request) {
    ::harmos::v1::InvocationsGetResponse response;
    const auto status = ::harmos::v1::Invocations::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::harmos::v1::InvocationsCancelResponse cancel(grpc::ClientContext &context, const ::harmos::v1::InvocationsCancelRequest &request) {
    ::harmos::v1::InvocationsCancelResponse response;
    const auto status = ::harmos::v1::Invocations::NewStub(channel)->Cancel(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Cancel: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::InvocationsWatchResponse>> watch(grpc::ClientContext &context,
                                                                                    const ::harmos::v1::InvocationsWatchRequest &request) {
    return ::harmos::v1::Invocations::NewStub(channel)->Watch(&context, request);
  }
};
struct MediaResource {
  MediaCapturesResource captures;
  explicit MediaResource(std::shared_ptr<grpc::Channel> channel) : captures(channel) {}
};
struct PlotsResource {
  PlotsSettingsResource settings;
  PlotsSnapshotsResource snapshots;
  explicit PlotsResource(std::shared_ptr<grpc::Channel> channel) : settings(channel), snapshots(channel) {}
};
struct PluginsResource {
  std::shared_ptr<grpc::Channel> channel;
  PluginsSettingsResource settings;
  explicit PluginsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), settings(channel) {}
  ::harmos::v1::PluginsGetResponse get(grpc::ClientContext &context, const ::harmos::v1::PluginsGetRequest &request) {
    ::harmos::v1::PluginsGetResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::harmos::v1::PluginsListResponse list(grpc::ClientContext &context, const ::harmos::v1::PluginsListRequest &request) {
    ::harmos::v1::PluginsListResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::PluginsWatchResponse>> watch(grpc::ClientContext &context,
                                                                                const ::harmos::v1::PluginsWatchRequest &request) {
    return ::harmos::v1::Plugins::NewStub(channel)->Watch(&context, request);
  }
  ::harmos::v1::PluginsCreateResponse create(grpc::ClientContext &context, const ::harmos::v1::PluginsCreateRequest &request) {
    ::harmos::v1::PluginsCreateResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::harmos::v1::PluginsSetResponse set(grpc::ClientContext &context, const ::harmos::v1::PluginsSetRequest &request) {
    ::harmos::v1::PluginsSetResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::harmos::v1::PluginsDeleteResponse delete_(grpc::ClientContext &context, const ::harmos::v1::PluginsDeleteRequest &request) {
    ::harmos::v1::PluginsDeleteResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->Delete(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Delete: ") + status.error_message());
    return response;
  }
  ::harmos::v1::PluginsRestartResponse restart(grpc::ClientContext &context, const ::harmos::v1::PluginsRestartRequest &request) {
    ::harmos::v1::PluginsRestartResponse response;
    const auto status = ::harmos::v1::Plugins::NewStub(channel)->Restart(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Restart: ") + status.error_message());
    return response;
  }
};
struct PromptDefinitionsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptDefinitionsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::PromptDefinitionsGetResponse get(grpc::ClientContext &context,
                                                         const ::dokime::client::v1::PromptDefinitionsGetRequest &request) {
    ::dokime::client::v1::PromptDefinitionsGetResponse response;
    const auto status = ::dokime::client::v1::PromptDefinitions::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::PromptDefinitionsListResponse list(grpc::ClientContext &context,
                                                           const ::dokime::client::v1::PromptDefinitionsListRequest &request) {
    ::dokime::client::v1::PromptDefinitionsListResponse response;
    const auto status = ::dokime::client::v1::PromptDefinitions::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct PromptsResource {
  std::shared_ptr<grpc::Channel> channel;
  PromptsAnswersResource answers;
  explicit PromptsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), answers(channel) {}
  ::dokime::client::v1::PromptsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::PromptsGetRequest &request) {
    ::dokime::client::v1::PromptsGetResponse response;
    const auto status = ::dokime::client::v1::Prompts::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::PromptsListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::PromptsListRequest &request) {
    ::dokime::client::v1::PromptsListResponse response;
    const auto status = ::dokime::client::v1::Prompts::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::PromptsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::PromptsWatchRequest &request) {
    return ::dokime::client::v1::Prompts::NewStub(channel)->Watch(&context, request);
  }
};
struct RecordSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit RecordSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::RecordSchemasGetResponse get(grpc::ClientContext &context,
                                                     const ::dokime::client::v1::RecordSchemasGetRequest &request) {
    ::dokime::client::v1::RecordSchemasGetResponse response;
    const auto status = ::dokime::client::v1::RecordSchemas::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::RecordSchemasListResponse list(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::RecordSchemasListRequest &request) {
    ::dokime::client::v1::RecordSchemasListResponse response;
    const auto status = ::dokime::client::v1::RecordSchemas::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct ReportsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ReportsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::ReportsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::ReportsGetRequest &request) {
    ::dokime::client::v1::ReportsGetResponse response;
    const auto status = ::dokime::client::v1::Reports::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ReportsCreateResponse create(grpc::ClientContext &context,
                                                     const ::dokime::client::v1::ReportsCreateRequest &request) {
    ::dokime::client::v1::ReportsCreateResponse response;
    const auto status = ::dokime::client::v1::Reports::NewStub(channel)->Create(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Create: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::ReportsPublishResponse publish(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::ReportsPublishRequest &request) {
    ::dokime::client::v1::ReportsPublishResponse response;
    const auto status = ::dokime::client::v1::Reports::NewStub(channel)->Publish(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Publish: ") + status.error_message());
    return response;
  }
};
struct SecretsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SecretsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::SecretsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::SecretsGetRequest &request) {
    ::dokime::client::v1::SecretsGetResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SecretsSetResponse set(grpc::ClientContext &context, const ::dokime::client::v1::SecretsSetRequest &request) {
    ::dokime::client::v1::SecretsSetResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SecretsListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::SecretsListRequest &request) {
    ::dokime::client::v1::SecretsListResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::dokime::client::v1::SecretsWatchResponse>>
  watch(grpc::ClientContext &context, const ::dokime::client::v1::SecretsWatchRequest &request) {
    return ::dokime::client::v1::Secrets::NewStub(channel)->Watch(&context, request);
  }
  ::dokime::client::v1::SecretsDeleteResponse delete_(grpc::ClientContext &context,
                                                      const ::dokime::client::v1::SecretsDeleteRequest &request) {
    ::dokime::client::v1::SecretsDeleteResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->Delete(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Delete: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SecretsExportResponse export_(grpc::ClientContext &context,
                                                      const ::dokime::client::v1::SecretsExportRequest &request) {
    ::dokime::client::v1::SecretsExportResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->Export(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Export: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SecretsImportResponse import(grpc::ClientContext &context,
                                                     const ::dokime::client::v1::SecretsImportRequest &request) {
    ::dokime::client::v1::SecretsImportResponse response;
    const auto status = ::dokime::client::v1::Secrets::NewStub(channel)->Import(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Import: ") + status.error_message());
    return response;
  }
};
struct SettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::SettingsGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::SettingsGetRequest &request) {
    ::dokime::client::v1::SettingsGetResponse response;
    const auto status = ::dokime::client::v1::Settings::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SettingsSetResponse set(grpc::ClientContext &context, const ::dokime::client::v1::SettingsSetRequest &request) {
    ::dokime::client::v1::SettingsSetResponse response;
    const auto status = ::dokime::client::v1::Settings::NewStub(channel)->Set(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Set: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SettingsSchemaResponse schema(grpc::ClientContext &context,
                                                      const ::dokime::client::v1::SettingsSchemaRequest &request) {
    ::dokime::client::v1::SettingsSchemaResponse response;
    const auto status = ::dokime::client::v1::Settings::NewStub(channel)->Schema(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Schema: ") + status.error_message());
    return response;
  }
};
struct StatusResource {
  std::shared_ptr<grpc::Channel> channel;
  StatusDiagnosticsResource diagnostics;
  StatusLogsResource logs;
  StatusTraceEventsResource trace_events;
  explicit StatusResource(std::shared_ptr<grpc::Channel> channel)
      : channel(channel), diagnostics(channel), logs(channel), trace_events(channel) {}
  ::harmos::v1::StatusGetResponse get(grpc::ClientContext &context, const ::harmos::v1::StatusGetRequest &request) {
    ::harmos::v1::StatusGetResponse response;
    const auto status = ::harmos::v1::Status::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::StatusWatchResponse>> watch(grpc::ClientContext &context,
                                                                               const ::harmos::v1::StatusWatchRequest &request) {
    return ::harmos::v1::Status::NewStub(channel)->Watch(&context, request);
  }
};
struct StorageResource {
  std::shared_ptr<grpc::Channel> channel;
  StorageSourcesResource sources;
  explicit StorageResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), sources(channel) {}
  ::harmos::v1::StorageCreateFileProductionResponse
  create_file_production(grpc::ClientContext &context, const ::harmos::v1::StorageCreateFileProductionRequest &request) {
    ::harmos::v1::StorageCreateFileProductionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->CreateFileProduction(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("CreateFileProduction: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetFileProductionResponse get_file_production(grpc::ClientContext &context,
                                                                     const ::harmos::v1::StorageGetFileProductionRequest &request) {
    ::harmos::v1::StorageGetFileProductionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetFileProduction(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetFileProduction: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageAppendFileBytesResponse append_file_bytes(grpc::ClientContext &context,
                                                                 const ::harmos::v1::StorageAppendFileBytesRequest &request) {
    ::harmos::v1::StorageAppendFileBytesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->AppendFileBytes(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("AppendFileBytes: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageCreateFileWriterResponse create_file_writer(grpc::ClientContext &context,
                                                                   const ::harmos::v1::StorageCreateFileWriterRequest &request) {
    ::harmos::v1::StorageCreateFileWriterResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->CreateFileWriter(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("CreateFileWriter: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StoragePublishFileProductionResponse
  publish_file_production(grpc::ClientContext &context, const ::harmos::v1::StoragePublishFileProductionRequest &request) {
    ::harmos::v1::StoragePublishFileProductionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->PublishFileProduction(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("PublishFileProduction: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageDeleteFileProductionResponse
  delete_file_production(grpc::ClientContext &context, const ::harmos::v1::StorageDeleteFileProductionRequest &request) {
    ::harmos::v1::StorageDeleteFileProductionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->DeleteFileProduction(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("DeleteFileProduction: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetFileBytesResponse get_file_bytes(grpc::ClientContext &context,
                                                           const ::harmos::v1::StorageGetFileBytesRequest &request) {
    ::harmos::v1::StorageGetFileBytesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetFileBytes(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetFileBytes: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageCapabilitiesResponse capabilities(grpc::ClientContext &context,
                                                         const ::harmos::v1::StorageCapabilitiesRequest &request) {
    ::harmos::v1::StorageCapabilitiesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->Capabilities(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Capabilities: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageCreateSnapshotResponse create_snapshot(grpc::ClientContext &context,
                                                              const ::harmos::v1::StorageCreateSnapshotRequest &request) {
    ::harmos::v1::StorageCreateSnapshotResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->CreateSnapshot(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("CreateSnapshot: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageDeleteSnapshotResponse delete_snapshot(grpc::ClientContext &context,
                                                              const ::harmos::v1::StorageDeleteSnapshotRequest &request) {
    ::harmos::v1::StorageDeleteSnapshotResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->DeleteSnapshot(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("DeleteSnapshot: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageListColumnRowsResponse list_column_rows(grpc::ClientContext &context,
                                                               const ::harmos::v1::StorageListColumnRowsRequest &request) {
    ::harmos::v1::StorageListColumnRowsResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->ListColumnRows(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("ListColumnRows: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageListIndexEntriesResponse list_index_entries(grpc::ClientContext &context,
                                                                   const ::harmos::v1::StorageListIndexEntriesRequest &request) {
    ::harmos::v1::StorageListIndexEntriesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->ListIndexEntries(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("ListIndexEntries: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageSetIndexDefinitionResponse set_index_definition(grpc::ClientContext &context,
                                                                       const ::harmos::v1::StorageSetIndexDefinitionRequest &request) {
    ::harmos::v1::StorageSetIndexDefinitionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->SetIndexDefinition(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("SetIndexDefinition: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageSetProjectionDefinitionResponse
  set_projection_definition(grpc::ClientContext &context, const ::harmos::v1::StorageSetProjectionDefinitionRequest &request) {
    ::harmos::v1::StorageSetProjectionDefinitionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->SetProjectionDefinition(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("SetProjectionDefinition: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageAdvanceProjectionResponse advance_projection(grpc::ClientContext &context,
                                                                    const ::harmos::v1::StorageAdvanceProjectionRequest &request) {
    ::harmos::v1::StorageAdvanceProjectionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->AdvanceProjection(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("AdvanceProjection: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageListProjectionRowsResponse list_projection_rows(grpc::ClientContext &context,
                                                                       const ::harmos::v1::StorageListProjectionRowsRequest &request) {
    ::harmos::v1::StorageListProjectionRowsResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->ListProjectionRows(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("ListProjectionRows: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetProjectionStatusResponse get_projection_status(grpc::ClientContext &context,
                                                                         const ::harmos::v1::StorageGetProjectionStatusRequest &request) {
    ::harmos::v1::StorageGetProjectionStatusResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetProjectionStatus(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetProjectionStatus: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageSetObjectResponse set_object(grpc::ClientContext &context, const ::harmos::v1::StorageSetObjectRequest &request) {
    ::harmos::v1::StorageSetObjectResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->SetObject(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("SetObject: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageListRowsResponse list_rows(grpc::ClientContext &context, const ::harmos::v1::StorageListRowsRequest &request) {
    ::harmos::v1::StorageListRowsResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->ListRows(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("ListRows: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::StorageWatchRowsResponse>>
  watch_rows(grpc::ClientContext &context, const ::harmos::v1::StorageWatchRowsRequest &request) {
    return ::harmos::v1::Storage::NewStub(channel)->WatchRows(&context, request);
  }
  ::harmos::v1::StorageAppendRecordsResponse append_records(grpc::ClientContext &context,
                                                            const ::harmos::v1::StorageAppendRecordsRequest &request) {
    ::harmos::v1::StorageAppendRecordsResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->AppendRecords(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("AppendRecords: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageAppendSamplesResponse append_samples(grpc::ClientContext &context,
                                                            const ::harmos::v1::StorageAppendSamplesRequest &request) {
    ::harmos::v1::StorageAppendSamplesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->AppendSamples(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("AppendSamples: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageAppendObjectRevisionsResponse
  append_object_revisions(grpc::ClientContext &context, const ::harmos::v1::StorageAppendObjectRevisionsRequest &request) {
    ::harmos::v1::StorageAppendObjectRevisionsResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->AppendObjectRevisions(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("AppendObjectRevisions: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetFileHashResponse get_file_hash(grpc::ClientContext &context,
                                                         const ::harmos::v1::StorageGetFileHashRequest &request) {
    ::harmos::v1::StorageGetFileHashResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetFileHash(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetFileHash: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageCaptureFileResponse capture_file(grpc::ClientContext &context,
                                                        const ::harmos::v1::StorageCaptureFileRequest &request) {
    ::harmos::v1::StorageCaptureFileResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->CaptureFile(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("CaptureFile: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::StorageWatchFileBytesResponse>>
  watch_file_bytes(grpc::ClientContext &context, const ::harmos::v1::StorageWatchFileBytesRequest &request) {
    return ::harmos::v1::Storage::NewStub(channel)->WatchFileBytes(&context, request);
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::StorageWatchFileProductionResponse>>
  watch_file_production(grpc::ClientContext &context, const ::harmos::v1::StorageWatchFileProductionRequest &request) {
    return ::harmos::v1::Storage::NewStub(channel)->WatchFileProduction(&context, request);
  }
  ::harmos::v1::StorageListDirectoryEntriesResponse
  list_directory_entries(grpc::ClientContext &context, const ::harmos::v1::StorageListDirectoryEntriesRequest &request) {
    ::harmos::v1::StorageListDirectoryEntriesResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->ListDirectoryEntries(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("ListDirectoryEntries: ") + status.error_message());
    return response;
  }
  std::unique_ptr<grpc::ClientReader<::harmos::v1::StorageWatchDirectoryEntriesResponse>>
  watch_directory_entries(grpc::ClientContext &context, const ::harmos::v1::StorageWatchDirectoryEntriesRequest &request) {
    return ::harmos::v1::Storage::NewStub(channel)->WatchDirectoryEntries(&context, request);
  }
  ::harmos::v1::StorageGetIndexDefinitionResponse get_index_definition(grpc::ClientContext &context,
                                                                       const ::harmos::v1::StorageGetIndexDefinitionRequest &request) {
    ::harmos::v1::StorageGetIndexDefinitionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetIndexDefinition(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetIndexDefinition: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetProjectionDefinitionResponse
  get_projection_definition(grpc::ClientContext &context, const ::harmos::v1::StorageGetProjectionDefinitionRequest &request) {
    ::harmos::v1::StorageGetProjectionDefinitionResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetProjectionDefinition(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetProjectionDefinition: ") + status.error_message());
    return response;
  }
  ::harmos::v1::StorageGetObjectResponse get_object(grpc::ClientContext &context, const ::harmos::v1::StorageGetObjectRequest &request) {
    ::harmos::v1::StorageGetObjectResponse response;
    const auto status = ::harmos::v1::Storage::NewStub(channel)->GetObject(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("GetObject: ") + status.error_message());
    return response;
  }
};
struct StreamSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StreamSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::StreamSchemasGetResponse get(grpc::ClientContext &context,
                                                     const ::dokime::client::v1::StreamSchemasGetRequest &request) {
    ::dokime::client::v1::StreamSchemasGetResponse response;
    const auto status = ::dokime::client::v1::StreamSchemas::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::StreamSchemasListResponse list(grpc::ClientContext &context,
                                                       const ::dokime::client::v1::StreamSchemasListRequest &request) {
    ::dokime::client::v1::StreamSchemasListResponse response;
    const auto status = ::dokime::client::v1::StreamSchemas::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct SuitesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SuitesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::v1::SuitesGetResponse get(grpc::ClientContext &context, const ::dokime::client::v1::SuitesGetRequest &request) {
    ::dokime::client::v1::SuitesGetResponse response;
    const auto status = ::dokime::client::v1::Suites::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::dokime::client::v1::SuitesListResponse list(grpc::ClientContext &context, const ::dokime::client::v1::SuitesListRequest &request) {
    ::dokime::client::v1::SuitesListResponse response;
    const auto status = ::dokime::client::v1::Suites::NewStub(channel)->List(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("List: ") + status.error_message());
    return response;
  }
};
struct TimeResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit TimeResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::v1::TimeGetResponse get(grpc::ClientContext &context, const ::harmos::v1::TimeGetRequest &request) {
    ::harmos::v1::TimeGetResponse response;
    const auto status = ::harmos::v1::Time::NewStub(channel)->Get(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Get: ") + status.error_message());
    return response;
  }
  ::harmos::v1::TimeSleepResponse sleep(grpc::ClientContext &context, const ::harmos::v1::TimeSleepRequest &request) {
    ::harmos::v1::TimeSleepResponse response;
    const auto status = ::harmos::v1::Time::NewStub(channel)->Sleep(&context, request, &response);
    if (!status.ok())
      throw std::runtime_error(std::string("Sleep: ") + status.error_message());
    return response;
  }
};
struct Client {
  AccessResource access;
  AggregatesResource aggregates;
  AnalysesResource analyses;
  BackendResource backend;
  CasesResource cases;
  CatalogEntriesResource catalog_entries;
  ContentResource content;
  CyclesResource cycles;
  DocumentsResource documents;
  EvidenceResource evidence;
  ExecutionsResource executions;
  InvocationsResource invocations;
  MediaResource media;
  PlotsResource plots;
  PluginsResource plugins;
  PromptDefinitionsResource prompt_definitions;
  PromptsResource prompts;
  RecordSchemasResource record_schemas;
  ReportsResource reports;
  SecretsResource secrets;
  SettingsResource settings;
  StatusResource status;
  StorageResource storage;
  StreamSchemasResource stream_schemas;
  SuitesResource suites;
  TimeResource time;
  explicit Client(std::shared_ptr<grpc::Channel> channel)
      : access(channel), aggregates(channel), analyses(channel), backend(channel), cases(channel), catalog_entries(channel),
        content(channel), cycles(channel), documents(channel), evidence(channel), executions(channel), invocations(channel), media(channel),
        plots(channel), plugins(channel), prompt_definitions(channel), prompts(channel), record_schemas(channel), reports(channel),
        secrets(channel), settings(channel), status(channel), storage(channel), stream_schemas(channel), suites(channel), time(channel) {}
};
} // namespace dokime::resources
#pragma pop_macro("WatchRows")
#pragma pop_macro("WatchFileProduction")
#pragma pop_macro("WatchFileBytes")
#pragma pop_macro("WatchDirectoryEntries")
#pragma pop_macro("Watch")
#pragma pop_macro("Stop")
#pragma pop_macro("Steps")
#pragma pop_macro("Statistics")
#pragma pop_macro("Start")
#pragma pop_macro("Sleep")
#pragma pop_macro("SetProjectionDefinition")
#pragma pop_macro("SetObject")
#pragma pop_macro("SetIndexDefinition")
#pragma pop_macro("Set")
#pragma pop_macro("Seal")
#pragma pop_macro("Schema")
#pragma pop_macro("Restart")
#pragma pop_macro("PublishFileProduction")
#pragma pop_macro("Publish")
#pragma pop_macro("Parameters")
#pragma pop_macro("ListRows")
#pragma pop_macro("ListProjectionRows")
#pragma pop_macro("ListIndexEntries")
#pragma pop_macro("ListDirectoryEntries")
#pragma pop_macro("ListColumnRows")
#pragma pop_macro("List")
#pragma pop_macro("Invoke")
#pragma pop_macro("Import")
#pragma pop_macro("GetProjectionStatus")
#pragma pop_macro("GetProjectionDefinition")
#pragma pop_macro("GetObject")
#pragma pop_macro("GetIndexDefinition")
#pragma pop_macro("GetFileProduction")
#pragma pop_macro("GetFileHash")
#pragma pop_macro("GetFileBytes")
#pragma pop_macro("Get")
#pragma pop_macro("Export")
#pragma pop_macro("DeleteSnapshot")
#pragma pop_macro("DeleteFileProduction")
#pragma pop_macro("Delete")
#pragma pop_macro("CreateSnapshot")
#pragma pop_macro("CreateFileWriter")
#pragma pop_macro("CreateFileProduction")
#pragma pop_macro("Create")
#pragma pop_macro("CaptureFile")
#pragma pop_macro("Capabilities")
#pragma pop_macro("Cancel")
#pragma pop_macro("Archive")
#pragma pop_macro("AppendSamples")
#pragma pop_macro("AppendRecords")
#pragma pop_macro("AppendObjectRevisions")
#pragma pop_macro("AppendFileBytes")
#pragma pop_macro("Appearance")
#pragma pop_macro("AdvanceProjection")
