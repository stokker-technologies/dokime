// @generated from protocol descriptors; do not edit.
#pragma once
#include "client_wire.hpp"
#include "dokime/messages.hpp"
#include "harmos/client_transport.hpp"
#include "harmos/messages.hpp"
namespace dokime {
namespace client {
struct AggregatesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct AnalysesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesGetOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CyclesSealOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct BackendOverviewGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct BackendOverviewWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct BackendAppearanceOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct RecordSchemasGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct RecordSchemasListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsGetOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsStartOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsStopOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsCancelOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsParametersOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsStepsOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct CasesParametersOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentArtifactsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentArtifactsCreateOptions {
  std::optional<::dokime::ArtifactScope> scope;
  std::optional<std::string> request_id;
  std::optional<::dokime::Metadata> metadata;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentArtifactsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentArtifactsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentExecutionFilesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentExecutionFilesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ContentExecutionFilesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PlotsSnapshotsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PlotsSnapshotsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct DocumentsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct DocumentsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceNotesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceNotesSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceNotesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceNotesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceNotesCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceHealthEventsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceHealthEventsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceRecordsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceRecordsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceSamplesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceSamplesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceTraceabilityGetOptions {
  std::optional<::harmos::PageRequest> page;
  std::optional<::harmos::ReadCut> at;
  std::optional<std::string> entry;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceTraceabilityWatchOptions {
  std::optional<::harmos::PageRequest> page;
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceStatisticsOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct EvidenceArchiveOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsCommandsGetOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsCommandsCreateOptions {
  std::optional<std::string> request_id;
  std::optional<std::string> step_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsProgressGetOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsProgressWatchOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ExecutionsSchemasGetOptions {
  std::optional<::harmos::Position> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsResumeOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsAppendOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsPublishOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesUploadsDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesReadOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct MediaCapturesCreateOptions {
  std::optional<std::uint32_t> input;
  std::optional<std::uint32_t> camera;
  std::optional<std::uint32_t> display;
  std::optional<std::uint32_t> maximum_seconds;
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct MediaCapturesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct MediaCapturesWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct MediaCapturesStopOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PlotsSettingsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PlotsSettingsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSecretsListOptions {
  std::optional<std::string> name;
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSecretsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSecretsDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptDefinitionsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptDefinitionsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptsListOptions {
  std::optional<std::string> cycle_id;
  std::optional<std::string> execution_id;
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptsWatchOptions {
  std::optional<std::string> cycle_id;
  std::optional<std::string> execution_id;
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PromptsAnswersCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ReportsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ReportsCreateOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ReportsPublishOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsWatchOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsExportOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SecretsImportOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SettingsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SettingsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SettingsSchemaOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StorageSourcesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StorageSourcesSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StreamSchemasGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StreamSchemasListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SuitesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SuitesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessCertificatesCreateOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessListenerGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessListenerSetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessPairingsCreateOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessPairingsListOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct AccessPairingsDeleteOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct InvocationsCreateOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct InvocationsGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct InvocationsCancelOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct InvocationsWatchOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct OperationsListOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct OperationsGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSettingsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSettingsSetOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsWatchOptions {
  std::optional<std::string> name;
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsCreateOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsSetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsDeleteOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsRestartOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct PluginsReplaceOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SchemasListOptions {
  std::optional<std::string> schema;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusDiagnosticsGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusLogsListOptions {
  std::optional<::harmos::Position> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusLogsWatchOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusTraceEventsListOptions {
  std::optional<::harmos::Position> at;
  std::optional<::harmos::PageRequest> page;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusTraceEventsWatchOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct StatusWatchOptions {
  std::optional<::harmos::Position> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct StreamsListOptions {
  std::optional<std::string> schema;
  std::shared_ptr<grpc::ClientContext> context;
};
struct TimeGetOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct TimeSleepOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
} // namespace client
struct AccessCertificatesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessCertificatesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  std::string create(client::AccessCertificatesCreateOptions options = {}) {
    detail::AccessCertificatesCreateRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessCertificatesCreateResponse response;
    const auto status = detail::AccessCertificatesCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.certificates.create: " + status.error_message());
    }
    return response.certificate_pem();
  }
};
struct AccessListenerResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessListenerResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::client::ListenerGetResponse get(client::AccessListenerGetOptions options = {}) {
    detail::AccessListenerGetRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessListenerGetResponse response;
    const auto status = detail::AccessListenerGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.listener.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::ListenerSetResponse set(const ::harmos::client::ListenerValue &value, const std::string &expected_revision,
                                            client::AccessListenerSetOptions options = {}) {
    detail::AccessListenerSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessListenerSetResponse response;
    const auto status = detail::AccessListenerSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.listener.set: " + status.error_message());
    }
    return response;
  }
};
struct AccessPairingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessPairingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::client::PairingsCreateResponse create(const std::string &label, client::AccessPairingsCreateOptions options = {}) {
    detail::AccessPairingsCreateRequest request;
    request.set_label(label);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessPairingsCreateResponse response;
    const auto status = detail::AccessPairingsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.pairings.create: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::PairingsListResponse list(client::AccessPairingsListOptions options = {}) {
    detail::AccessPairingsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessPairingsListResponse response;
    const auto status = detail::AccessPairingsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.pairings.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::PairingsDeleteResponse delete_(const std::string &name, client::AccessPairingsDeleteOptions options = {}) {
    detail::AccessPairingsDeleteRequest request;
    request.set_name(name);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AccessPairingsDeleteResponse response;
    const auto status = detail::AccessPairingsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call access.pairings.delete: " + status.error_message());
    }
    return response;
  }
};
struct BackendOverviewResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit BackendOverviewResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::BackendOverviewGetResponse get(client::BackendOverviewGetOptions options = {}) {
    detail::BackendOverviewGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::BackendOverviewGetResponse response;
    const auto status = detail::BackendOverviewGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call backend.overview.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::BackendOverviewWatchResponse> watch(client::BackendOverviewWatchOptions options = {}) {
    detail::BackendOverviewWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::BackendOverviewWatchResponse>>(
        detail::BackendOverviewWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::BackendOverviewWatchResponse> {
              detail::BackendOverviewWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read backend.overview.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct ContentArtifactsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentArtifactsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::ContentArtifactsGetResponse get(const std::string &name, client::ContentArtifactsGetOptions options = {}) {
    detail::ContentArtifactsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ContentArtifactsGetResponse response;
    const auto status = detail::ContentArtifactsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call content.artifacts.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ContentArtifactsCreateResponse create(const ::harmos::FileReference &file, const std::string &filename,
                                                          const std::string &content_type,
                                                          client::ContentArtifactsCreateOptions options = {}) {
    detail::ContentArtifactsCreateRequest request;
    *request.mutable_file() = file;
    if (options.scope) {
      *request.mutable_scope() = *options.scope;
    }
    request.set_filename(filename);
    request.set_content_type(content_type);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    if (options.metadata) {
      *request.mutable_metadata() = *options.metadata;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ContentArtifactsCreateResponse response;
    const auto status = detail::ContentArtifactsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call content.artifacts.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ContentArtifactsListResponse list(const ::dokime::ArtifactScope &scope,
                                                      client::ContentArtifactsListOptions options = {}) {
    detail::ContentArtifactsListRequest request;
    *request.mutable_scope() = scope;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ContentArtifactsListResponse response;
    const auto status = detail::ContentArtifactsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call content.artifacts.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ContentArtifactsWatchResponse> watch(const ::dokime::ArtifactScope &scope,
                                                                                client::ContentArtifactsWatchOptions options = {}) {
    detail::ContentArtifactsWatchRequest request;
    *request.mutable_scope() = scope;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::ContentArtifactsWatchResponse>>(
        detail::ContentArtifactsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::ContentArtifactsWatchResponse> {
              detail::ContentArtifactsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read content.artifacts.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct ContentExecutionFilesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentExecutionFilesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::ContentExecutionFilesGetResponse get(const std::string &parent, const std::string &name,
                                                         client::ContentExecutionFilesGetOptions options = {}) {
    detail::ContentExecutionFilesGetRequest request;
    request.set_parent(parent);
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ContentExecutionFilesGetResponse response;
    const auto status = detail::ContentExecutionFilesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call content.execution_files.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ContentExecutionFilesListResponse list(const std::string &parent,
                                                           client::ContentExecutionFilesListOptions options = {}) {
    detail::ContentExecutionFilesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ContentExecutionFilesListResponse response;
    const auto status = detail::ContentExecutionFilesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call content.execution_files.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ContentExecutionFilesWatchResponse>
  watch(const std::string &parent, client::ContentExecutionFilesWatchOptions options = {}) {
    detail::ContentExecutionFilesWatchRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::ContentExecutionFilesWatchResponse>>(
        detail::ContentExecutionFilesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::ContentExecutionFilesWatchResponse> {
              detail::ContentExecutionFilesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read content.execution_files.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceHealthEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceHealthEventsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::EvidenceHealthEventsListResponse list(const ::dokime::client::EvidenceAnnotationSelection &selection,
                                                          client::EvidenceHealthEventsListOptions options = {}) {
    detail::EvidenceHealthEventsListRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceHealthEventsListResponse response;
    const auto status = detail::EvidenceHealthEventsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.health_events.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::EvidenceHealthEventsWatchResponse>
  watch(const ::dokime::client::EvidenceAnnotationSelection &selection, client::EvidenceHealthEventsWatchOptions options = {}) {
    detail::EvidenceHealthEventsWatchRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::EvidenceHealthEventsWatchResponse>>(
        detail::EvidenceHealthEventsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::EvidenceHealthEventsWatchResponse> {
              detail::EvidenceHealthEventsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read evidence.health_events.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceNotesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceNotesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::EvidenceNotesGetResponse get(const std::string &name, client::EvidenceNotesGetOptions options = {}) {
    detail::EvidenceNotesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceNotesGetResponse response;
    const auto status = detail::EvidenceNotesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.notes.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::EvidenceNotesSetResponse set(const std::string &name, const ::dokime::client::EvidenceNoteWritable &value,
                                                 const std::string &expected_revision, client::EvidenceNotesSetOptions options = {}) {
    detail::EvidenceNotesSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceNotesSetResponse response;
    const auto status = detail::EvidenceNotesSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.notes.set: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::EvidenceNotesListResponse list(const ::dokime::client::EvidenceAnnotationSelection &selection,
                                                   client::EvidenceNotesListOptions options = {}) {
    detail::EvidenceNotesListRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceNotesListResponse response;
    const auto status = detail::EvidenceNotesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.notes.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::EvidenceNotesWatchResponse> watch(const ::dokime::client::EvidenceAnnotationSelection &selection,
                                                                             client::EvidenceNotesWatchOptions options = {}) {
    detail::EvidenceNotesWatchRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::EvidenceNotesWatchResponse>>(
        detail::EvidenceNotesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::EvidenceNotesWatchResponse> {
              detail::EvidenceNotesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read evidence.notes.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::EvidenceNotesCreateResponse create(const ::dokime::client::EvidenceAnnotationOwner &owner,
                                                       const ::dokime::client::EvidenceNoteWritable &value,
                                                       client::EvidenceNotesCreateOptions options = {}) {
    detail::EvidenceNotesCreateRequest request;
    *request.mutable_owner() = owner;
    *request.mutable_value() = value;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceNotesCreateResponse response;
    const auto status = detail::EvidenceNotesCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.notes.create: " + status.error_message());
    }
    return response;
  }
};
struct EvidenceRecordsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceRecordsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::EvidenceRecordsListResponse list(const ::dokime::EvidenceRecordScope &scope, const std::string &record_type,
                                             const std::vector<std::uint32_t> &versions, const ::dokime::Window &between,
                                             const std::string &step, const ::dokime::EvidenceCapturedRecordSelection &selection,
                                             client::EvidenceRecordsListOptions options = {}) {
    detail::EvidenceRecordsListRequest request;
    *request.mutable_scope() = scope;
    request.set_record_type(record_type);
    request.mutable_versions()->Assign(versions.begin(), versions.end());
    *request.mutable_between() = between;
    request.set_step(step);
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceRecordsListResponse response;
    const auto status = detail::EvidenceRecordsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.records.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::EvidenceRecordsWatchResponse>
  watch(const ::dokime::EvidenceRecordScope &scope, const std::string &record_type, const std::vector<std::uint32_t> &versions,
        const ::dokime::Window &between, const std::string &step, const ::dokime::EvidenceCapturedRecordSelection &selection,
        client::EvidenceRecordsWatchOptions options = {}) {
    detail::EvidenceRecordsWatchRequest request;
    *request.mutable_scope() = scope;
    request.set_record_type(record_type);
    request.mutable_versions()->Assign(versions.begin(), versions.end());
    *request.mutable_between() = between;
    request.set_step(step);
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::EvidenceRecordsWatchResponse>>(
        detail::EvidenceRecordsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::EvidenceRecordsWatchResponse> {
              detail::EvidenceRecordsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read evidence.records.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceSamplesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceSamplesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::EvidenceSamplesListResponse list(const ::dokime::EvidenceScope &scope, const std::vector<std::string> &streams,
                                                     const std::vector<std::uint32_t> &versions, const std::vector<std::string> &fields,
                                                     const std::vector<std::string> &sources, const std::vector<std::string> &signals,
                                                     const ::dokime::Window &between, const ::dokime::client::EvidenceBucketRequest &bucket,
                                                     const bool &latest, client::EvidenceSamplesListOptions options = {}) {
    detail::EvidenceSamplesListRequest request;
    *request.mutable_scope() = scope;
    request.mutable_streams()->Assign(streams.begin(), streams.end());
    request.mutable_versions()->Assign(versions.begin(), versions.end());
    request.mutable_fields()->Assign(fields.begin(), fields.end());
    request.mutable_sources()->Assign(sources.begin(), sources.end());
    request.mutable_signals()->Assign(signals.begin(), signals.end());
    *request.mutable_between() = between;
    *request.mutable_bucket() = bucket;
    request.set_latest(latest);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceSamplesListResponse response;
    const auto status = detail::EvidenceSamplesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.samples.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::EvidenceSamplesWatchResponse>
  watch(const ::dokime::EvidenceScope &scope, const std::vector<std::string> &streams, const std::vector<std::uint32_t> &versions,
        const std::vector<std::string> &fields, const std::vector<std::string> &sources, const std::vector<std::string> &signals,
        const ::dokime::Window &between, const ::dokime::client::EvidenceBucketRequest &bucket, const bool &latest,
        client::EvidenceSamplesWatchOptions options = {}) {
    detail::EvidenceSamplesWatchRequest request;
    *request.mutable_scope() = scope;
    request.mutable_streams()->Assign(streams.begin(), streams.end());
    request.mutable_versions()->Assign(versions.begin(), versions.end());
    request.mutable_fields()->Assign(fields.begin(), fields.end());
    request.mutable_sources()->Assign(sources.begin(), sources.end());
    request.mutable_signals()->Assign(signals.begin(), signals.end());
    *request.mutable_between() = between;
    *request.mutable_bucket() = bucket;
    request.set_latest(latest);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::EvidenceSamplesWatchResponse>>(
        detail::EvidenceSamplesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::EvidenceSamplesWatchResponse> {
              detail::EvidenceSamplesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read evidence.samples.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceTraceabilityResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceTraceabilityResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::EvidenceTraceabilityGetResponse get(const ::dokime::EvidenceScope &scope, client::EvidenceTraceabilityGetOptions options = {}) {
    detail::EvidenceTraceabilityGetRequest request;
    *request.mutable_scope() = scope;
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.entry) {
      request.set_entry(*options.entry);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceTraceabilityGetResponse response;
    const auto status = detail::EvidenceTraceabilityGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.traceability.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::EvidenceTraceabilityWatchResponse> watch(const ::dokime::EvidenceScope &scope,
                                                                                    client::EvidenceTraceabilityWatchOptions options = {}) {
    detail::EvidenceTraceabilityWatchRequest request;
    *request.mutable_scope() = scope;
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::EvidenceTraceabilityWatchResponse>>(
        detail::EvidenceTraceabilityWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::EvidenceTraceabilityWatchResponse> {
              detail::EvidenceTraceabilityWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read evidence.traceability.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct ExecutionsCommandsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsCommandsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::ExecutionsCommandsGetResponse get(const std::string &name, const std::string &parent,
                                                      client::ExecutionsCommandsGetOptions options = {}) {
    detail::ExecutionsCommandsGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsCommandsGetResponse response;
    const auto status = detail::ExecutionsCommandsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.commands.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsCommandsCreateResponse create(const std::string &parent, const std::string &command_id,
                                                            const ::dokime::client::ExecutionsExecutionCommandAction &action,
                                                            client::ExecutionsCommandsCreateOptions options = {}) {
    detail::ExecutionsCommandsCreateRequest request;
    request.set_parent(parent);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_command_id(command_id);
    request.set_action(action);
    if (options.step_id) {
      request.set_step_id(*options.step_id);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsCommandsCreateResponse response;
    const auto status = detail::ExecutionsCommandsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.commands.create: " + status.error_message());
    }
    return response;
  }
};
struct ExecutionsProgressResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsProgressResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::ExecutionsProgressGetResponse get(const std::string &name, client::ExecutionsProgressGetOptions options = {}) {
    detail::ExecutionsProgressGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsProgressGetResponse response;
    const auto status = detail::ExecutionsProgressGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.progress.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ExecutionsProgressWatchResponse> watch(const std::string &name,
                                                                                  client::ExecutionsProgressWatchOptions options = {}) {
    detail::ExecutionsProgressWatchRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::ExecutionsProgressWatchResponse>>(
        detail::ExecutionsProgressWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::ExecutionsProgressWatchResponse> {
              detail::ExecutionsProgressWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read executions.progress.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct ExecutionsSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::ExecutionsSchemasGetResponse get(const std::string &name, client::ExecutionsSchemasGetOptions options = {}) {
    detail::ExecutionsSchemasGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsSchemasGetResponse response;
    const auto status = detail::ExecutionsSchemasGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.schemas.get: " + status.error_message());
    }
    return response;
  }
};
struct FilesUploadsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit FilesUploadsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::FilesUploadsCreateResponse create(const ::harmos::FileMetadata &metadata, client::FilesUploadsCreateOptions options = {}) {
    detail::FilesUploadsCreateRequest request;
    *request.mutable_metadata() = metadata;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsCreateResponse response;
    const auto status = detail::FilesUploadsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesUploadsGetResponse get(const ::harmos::Upload &upload, client::FilesUploadsGetOptions options = {}) {
    detail::FilesUploadsGetRequest request;
    *request.mutable_upload() = upload;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsGetResponse response;
    const auto status = detail::FilesUploadsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesUploadsResumeResponse resume(const ::harmos::Upload &upload, client::FilesUploadsResumeOptions options = {}) {
    detail::FilesUploadsResumeRequest request;
    *request.mutable_upload() = upload;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsResumeResponse response;
    const auto status = detail::FilesUploadsResumeService::NewStub(channel)->Resume(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.resume: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesUploadsAppendResponse append(const ::harmos::Upload &upload, const std::uint64_t &offset, const std::string &data,
                                              client::FilesUploadsAppendOptions options = {}) {
    detail::FilesUploadsAppendRequest request;
    *request.mutable_upload() = upload;
    request.set_offset(offset);
    request.set_data(data);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsAppendResponse response;
    const auto status = detail::FilesUploadsAppendService::NewStub(channel)->Append(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.append: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesUploadsPublishResponse publish(const ::harmos::Upload &upload, const std::uint64_t &length, const std::string &sha256,
                                                client::FilesUploadsPublishOptions options = {}) {
    detail::FilesUploadsPublishRequest request;
    *request.mutable_upload() = upload;
    request.set_length(length);
    request.set_sha256(sha256);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsPublishResponse response;
    const auto status = detail::FilesUploadsPublishService::NewStub(channel)->Publish(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.publish: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesUploadsDeleteResponse delete_(const ::harmos::Upload &upload, client::FilesUploadsDeleteOptions options = {}) {
    detail::FilesUploadsDeleteRequest request;
    *request.mutable_upload() = upload;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesUploadsDeleteResponse response;
    const auto status = detail::FilesUploadsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.uploads.delete: " + status.error_message());
    }
    return response;
  }
};
struct MediaCapturesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit MediaCapturesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::MediaCapturesCreateResponse create(const ::dokime::ArtifactScope &scope, const ::dokime::client::MediaCaptureKind &kind,
                                                       const ::dokime::client::MediaRecordingAudio &audio,
                                                       client::MediaCapturesCreateOptions options = {}) {
    detail::MediaCapturesCreateRequest request;
    *request.mutable_scope() = scope;
    request.set_kind(kind);
    if (options.input) {
      request.set_input(*options.input);
    }
    if (options.camera) {
      request.set_camera(*options.camera);
    }
    if (options.display) {
      request.set_display(*options.display);
    }
    *request.mutable_audio() = audio;
    if (options.maximum_seconds) {
      request.set_maximum_seconds(*options.maximum_seconds);
    }
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::MediaCapturesCreateResponse response;
    const auto status = detail::MediaCapturesCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call media.captures.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::MediaCapturesGetResponse get(const std::string &name, client::MediaCapturesGetOptions options = {}) {
    detail::MediaCapturesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::MediaCapturesGetResponse response;
    const auto status = detail::MediaCapturesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call media.captures.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::MediaCapturesWatchResponse> watch(const std::string &name,
                                                                             client::MediaCapturesWatchOptions options = {}) {
    detail::MediaCapturesWatchRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::MediaCapturesWatchResponse>>(
        detail::MediaCapturesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::MediaCapturesWatchResponse> {
              detail::MediaCapturesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read media.captures.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::MediaCapturesStopResponse stop(const std::string &name, client::MediaCapturesStopOptions options = {}) {
    detail::MediaCapturesStopRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::MediaCapturesStopResponse response;
    const auto status = detail::MediaCapturesStopService::NewStub(channel)->Stop(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call media.captures.stop: " + status.error_message());
    }
    return response;
  }
};
struct PlotsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::PlotsSettingsGetResponse get(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                                 const std::string &plot_id, client::PlotsSettingsGetOptions options = {}) {
    detail::PlotsSettingsGetRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_plot_id(plot_id);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PlotsSettingsGetResponse response;
    const auto status = detail::PlotsSettingsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plots.settings.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::PlotsSettingsSetResponse set(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                                 const std::string &plot_id, const ::dokime::client::PlotsPlotSettingsValue &value,
                                                 const std::string &expected_revision, client::PlotsSettingsSetOptions options = {}) {
    detail::PlotsSettingsSetRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_plot_id(plot_id);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PlotsSettingsSetResponse response;
    const auto status = detail::PlotsSettingsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plots.settings.set: " + status.error_message());
    }
    return response;
  }
};
struct PlotsSnapshotsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSnapshotsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::PlotsSnapshotsListResponse list(const std::vector<::dokime::client::PlotsPlotRequest> &requests,
                                                    client::PlotsSnapshotsListOptions options = {}) {
    detail::PlotsSnapshotsListRequest request;
    request.mutable_requests()->Assign(requests.begin(), requests.end());
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PlotsSnapshotsListResponse response;
    const auto status = detail::PlotsSnapshotsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plots.snapshots.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::PlotsSnapshotsWatchResponse>
  watch(const std::vector<::dokime::client::PlotsPlotRequest> &requests, client::PlotsSnapshotsWatchOptions options = {}) {
    detail::PlotsSnapshotsWatchRequest request;
    request.mutable_requests()->Assign(requests.begin(), requests.end());
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::PlotsSnapshotsWatchResponse>>(
        detail::PlotsSnapshotsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::PlotsSnapshotsWatchResponse> {
              detail::PlotsSnapshotsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read plots.snapshots.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct PluginsSecretsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PluginsSecretsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::PluginsSecretsListResponse list(client::PluginsSecretsListOptions options = {}) {
    detail::PluginsSecretsListRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSecretsListResponse response;
    const auto status = detail::PluginsSecretsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.secrets.list: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::PluginsSecretsSetResponse set(const std::string &name,
                                                  const std::vector<::dokime::client::PluginsSecretBindingValue> &bindings,
                                                  const std::string &expected_revision, client::PluginsSecretsSetOptions options = {}) {
    detail::PluginsSecretsSetRequest request;
    request.set_name(name);
    request.mutable_bindings()->Assign(bindings.begin(), bindings.end());
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSecretsSetResponse response;
    const auto status = detail::PluginsSecretsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.secrets.set: " + status.error_message());
    }
    return response;
  }
  ::harmos::Receipt delete_(const std::string &name, const std::vector<std::string> &identifiers, const std::string &expected_revision,
                            client::PluginsSecretsDeleteOptions options = {}) {
    detail::PluginsSecretsDeleteRequest request;
    request.set_name(name);
    request.mutable_identifiers()->Assign(identifiers.begin(), identifiers.end());
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSecretsDeleteResponse response;
    const auto status = detail::PluginsSecretsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.secrets.delete: " + status.error_message());
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read plugins.secrets.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
};
struct PluginsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PluginsSettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::SettingsListResponse list(const std::string &name, client::PluginsSettingsListOptions options = {}) {
    detail::PluginsSettingsListRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSettingsListResponse response;
    const auto status = detail::PluginsSettingsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.settings.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::SettingsSetResponse set(const std::string &name, const ::harmos::SettingValues &value,
                                            const std::string &expected_revision, client::PluginsSettingsSetOptions options = {}) {
    detail::PluginsSettingsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSettingsSetResponse response;
    const auto status = detail::PluginsSettingsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.settings.set: " + status.error_message());
    }
    return response;
  }
};
struct PromptsAnswersResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptsAnswersResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::PromptsAnswersCreateResponse create(const std::string &parent, const std::string &action,
                                                        const ::dokime::MapValue &values, const bool &accepted,
                                                        client::PromptsAnswersCreateOptions options = {}) {
    detail::PromptsAnswersCreateRequest request;
    request.set_parent(parent);
    request.set_action(action);
    *request.mutable_values() = values;
    request.set_accepted(accepted);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PromptsAnswersCreateResponse response;
    const auto status = detail::PromptsAnswersCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call prompts.answers.create: " + status.error_message());
    }
    return response;
  }
};
struct StatusDiagnosticsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusDiagnosticsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::client::DiagnosticsGetResponse get(const std::string &name, const std::uint64_t &offset, const std::uint64_t &max_bytes,
                                               client::StatusDiagnosticsGetOptions options = {}) {
    detail::StatusDiagnosticsGetRequest request;
    request.set_name(name);
    request.set_offset(offset);
    request.set_max_bytes(max_bytes);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StatusDiagnosticsGetResponse response;
    const auto status = detail::StatusDiagnosticsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call status.diagnostics.get: " + status.error_message());
    }
    return response;
  }
};
struct StatusLogsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusLogsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::client::LogsListResponse list(client::StatusLogsListOptions options = {}) {
    detail::StatusLogsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StatusLogsListResponse response;
    const auto status = detail::StatusLogsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call status.logs.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::LogsWatchResponse> watch(const std::uint64_t &max_bytes, const std::uint32_t &max_events,
                                                                    client::StatusLogsWatchOptions options = {}) {
    detail::StatusLogsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    request.set_max_bytes(max_bytes);
    request.set_max_events(max_events);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::StatusLogsWatchResponse>>(
        detail::StatusLogsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::harmos::client::LogsWatchResponse> {
              detail::StatusLogsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read status.logs.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct StatusTraceEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusTraceEventsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::harmos::client::TraceEventsListResponse list(client::StatusTraceEventsListOptions options = {}) {
    detail::StatusTraceEventsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StatusTraceEventsListResponse response;
    const auto status = detail::StatusTraceEventsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call status.trace_events.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::TraceEventsWatchResponse> watch(const std::uint64_t &max_bytes, const std::uint32_t &max_events,
                                                                           client::StatusTraceEventsWatchOptions options = {}) {
    detail::StatusTraceEventsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    request.set_max_bytes(max_bytes);
    request.set_max_events(max_events);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::StatusTraceEventsWatchResponse>>(
        detail::StatusTraceEventsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::harmos::client::TraceEventsWatchResponse> {
              detail::StatusTraceEventsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read status.trace_events.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct StorageSourcesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StorageSourcesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::StorageSourcesGetResponse get(client::StorageSourcesGetOptions options = {}) {
    detail::StorageSourcesGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StorageSourcesGetResponse response;
    const auto status = detail::StorageSourcesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call storage.sources.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::StorageSourcesSetResponse set(const ::dokime::client::StorageStorageSources &value,
                                                  const std::string &expected_revision, client::StorageSourcesSetOptions options = {}) {
    detail::StorageSourcesSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StorageSourcesSetResponse response;
    const auto status = detail::StorageSourcesSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call storage.sources.set: " + status.error_message());
    }
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
  ::dokime::client::AggregatesGetResponse get(const ::dokime::client::QuerySource &source,
                                              const std::vector<::dokime::client::QueryFilter> &filters,
                                              const std::vector<std::string> &group_by,
                                              const std::vector<::dokime::client::QueryAggregate> &aggregates, const std::uint32_t &limit,
                                              client::AggregatesGetOptions options = {}) {
    detail::AggregatesGetRequest request;
    request.set_source(source);
    request.mutable_filters()->Assign(filters.begin(), filters.end());
    request.mutable_group_by()->Assign(group_by.begin(), group_by.end());
    request.mutable_aggregates()->Assign(aggregates.begin(), aggregates.end());
    request.set_limit(limit);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AggregatesGetResponse response;
    const auto status = detail::AggregatesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call aggregates.get: " + status.error_message());
    }
    return response;
  }
};
struct AnalysesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AnalysesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::AnalysesListResponse list(client::AnalysesListOptions options = {}) {
    detail::AnalysesListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::AnalysesListResponse response;
    const auto status = detail::AnalysesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call analyses.list: " + status.error_message());
    }
    return response;
  }
};
struct BackendResource {
  std::shared_ptr<grpc::Channel> channel;
  BackendOverviewResource overview;
  explicit BackendResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), overview(channel) {}
  ::dokime::client::BackendAppearanceResponse appearance(client::BackendAppearanceOptions options = {}) {
    detail::BackendAppearanceRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::BackendAppearanceResponse response;
    const auto status = detail::BackendAppearanceService::NewStub(channel)->Appearance(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call backend.appearance: " + status.error_message());
    }
    return response;
  }
};
struct CasesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CasesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::CasesCreateResponse create(const ::dokime::client::SelectionPlan &plan, const bool &validate_only,
                                               client::CasesCreateOptions options = {}) {
    detail::CasesCreateRequest request;
    *request.mutable_plan() = plan;
    request.set_validate_only(validate_only);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CasesCreateResponse response;
    const auto status = detail::CasesCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cases.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::CasesGetResponse get(const std::string &name, const std::string &parent, client::CasesGetOptions options = {}) {
    detail::CasesGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CasesGetResponse response;
    const auto status = detail::CasesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cases.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::CasesListResponse list(const std::string &parent, client::CasesListOptions options = {}) {
    detail::CasesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CasesListResponse response;
    const auto status = detail::CasesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cases.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::CasesWatchResponse> watch(const std::string &parent, client::CasesWatchOptions options = {}) {
    detail::CasesWatchRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::CasesWatchResponse>>(
        detail::CasesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::CasesWatchResponse> {
              detail::CasesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read cases.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::harmos::Receipt delete_(const std::string &name, const std::string &parent, client::CasesDeleteOptions options = {}) {
    detail::CasesDeleteRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CasesDeleteResponse response;
    const auto status = detail::CasesDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cases.delete: " + status.error_message());
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read cases.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
  ::dokime::client::CasesParametersResponse parameters(const std::string &name, const std::string &parent,
                                                       client::CasesParametersOptions options = {}) {
    detail::CasesParametersRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CasesParametersResponse response;
    const auto status = detail::CasesParametersService::NewStub(channel)->Parameters(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cases.parameters: " + status.error_message());
    }
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
  ::dokime::client::CyclesGetResponse get(const std::string &name, client::CyclesGetOptions options = {}) {
    detail::CyclesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CyclesGetResponse response;
    const auto status = detail::CyclesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cycles.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::CyclesSetResponse set(const std::string &name, const ::dokime::client::CycleWritable &value,
                                          const std::string &expected_revision, client::CyclesSetOptions options = {}) {
    detail::CyclesSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CyclesSetResponse response;
    const auto status = detail::CyclesSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cycles.set: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::CyclesListResponse list(const ::dokime::client::CycleSelection &selection, const ::dokime::client::CycleFilter &filter,
                                            const ::dokime::client::CycleOrder &order, client::CyclesListOptions options = {}) {
    detail::CyclesListRequest request;
    *request.mutable_selection() = selection;
    *request.mutable_filter() = filter;
    *request.mutable_order() = order;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CyclesListResponse response;
    const auto status = detail::CyclesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cycles.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::CyclesWatchResponse> watch(const ::dokime::client::CycleSelection &selection,
                                                                      const ::dokime::client::CycleFilter &filter,
                                                                      const ::dokime::client::CycleOrder &order,
                                                                      client::CyclesWatchOptions options = {}) {
    detail::CyclesWatchRequest request;
    *request.mutable_selection() = selection;
    *request.mutable_filter() = filter;
    *request.mutable_order() = order;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::CyclesWatchResponse>>(
        detail::CyclesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::CyclesWatchResponse> {
              detail::CyclesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read cycles.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::CyclesCreateResponse create(const std::vector<::dokime::Origin> &origins, const ::dokime::Metadata &metadata,
                                                const bool &continue_automatically, const std::string &source_cycle,
                                                client::CyclesCreateOptions options = {}) {
    detail::CyclesCreateRequest request;
    request.mutable_origins()->Assign(origins.begin(), origins.end());
    *request.mutable_metadata() = metadata;
    request.set_continue_automatically(continue_automatically);
    request.set_source_cycle(source_cycle);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CyclesCreateResponse response;
    const auto status = detail::CyclesCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cycles.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::CyclesSealResponse seal(const std::string &name, client::CyclesSealOptions options = {}) {
    detail::CyclesSealRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::CyclesSealResponse response;
    const auto status = detail::CyclesSealService::NewStub(channel)->Seal(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call cycles.seal: " + status.error_message());
    }
    return response;
  }
};
struct DocumentsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit DocumentsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::DocumentsGetResponse get(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                             const bool &structure_only, const std::vector<std::string> &card_ids,
                                             const std::string &page_token, const std::uint64_t &max_bytes, const std::uint32_t &max_frames,
                                             client::DocumentsGetOptions options = {}) {
    detail::DocumentsGetRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_structure_only(structure_only);
    request.mutable_card_ids()->Assign(card_ids.begin(), card_ids.end());
    request.set_page_token(page_token);
    request.set_max_bytes(max_bytes);
    request.set_max_frames(max_frames);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::DocumentsGetResponse response;
    const auto status = detail::DocumentsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call documents.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::DocumentsWatchResponse>
  watch(const ::dokime::client::DocumentScope &scope, const std::string &document_id, const bool &structure_only,
        const std::vector<std::string> &card_ids, const std::string &page_token, const std::uint64_t &max_bytes,
        const std::uint32_t &max_frames, client::DocumentsWatchOptions options = {}) {
    detail::DocumentsWatchRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_structure_only(structure_only);
    request.mutable_card_ids()->Assign(card_ids.begin(), card_ids.end());
    request.set_page_token(page_token);
    request.set_max_bytes(max_bytes);
    request.set_max_frames(max_frames);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::DocumentsWatchResponse>>(
        detail::DocumentsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::DocumentsWatchResponse> {
              detail::DocumentsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read documents.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
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
  ::dokime::EvidenceStatisticsResponse statistics(const ::dokime::EvidenceScope &scope, const std::string &stream,
                                                  const std::vector<std::uint32_t> &versions, const ::dokime::Window &between,
                                                  const std::vector<::dokime::Output> &outputs, const std::string &field,
                                                  const std::string &step, client::EvidenceStatisticsOptions options = {}) {
    detail::EvidenceStatisticsRequest request;
    *request.mutable_scope() = scope;
    request.set_stream(stream);
    request.mutable_versions()->Assign(versions.begin(), versions.end());
    *request.mutable_between() = between;
    request.mutable_outputs()->Assign(outputs.begin(), outputs.end());
    request.set_field(field);
    request.set_step(step);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceStatisticsResponse response;
    const auto status = detail::EvidenceStatisticsService::NewStub(channel)->Statistics(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.statistics: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::EvidenceArchiveResponse archive(const ::dokime::client::EvidenceSubject &subject,
                                                    client::EvidenceArchiveOptions options = {}) {
    detail::EvidenceArchiveRequest request;
    *request.mutable_subject() = subject;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::EvidenceArchiveResponse response;
    const auto status = detail::EvidenceArchiveService::NewStub(channel)->Archive(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call evidence.archive: " + status.error_message());
    }
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
  ::dokime::client::ExecutionsGetResponse get(const std::string &name, client::ExecutionsGetOptions options = {}) {
    detail::ExecutionsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsGetResponse response;
    const auto status = detail::ExecutionsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsSetResponse set(const std::string &name, const ::dokime::client::ExecutionWritable &value,
                                              const std::string &expected_revision, client::ExecutionsSetOptions options = {}) {
    detail::ExecutionsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsSetResponse response;
    const auto status = detail::ExecutionsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.set: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsListResponse list(const std::string &parent, const ::dokime::client::ExecutionSelection &selection,
                                                const ::dokime::client::ExecutionFilter &filter,
                                                const ::dokime::client::ExecutionOrder &order, client::ExecutionsListOptions options = {}) {
    detail::ExecutionsListRequest request;
    request.set_parent(parent);
    *request.mutable_selection() = selection;
    *request.mutable_filter() = filter;
    *request.mutable_order() = order;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsListResponse response;
    const auto status = detail::ExecutionsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ExecutionsWatchResponse>
  watch(const std::string &parent, const ::dokime::client::ExecutionSelection &selection, const ::dokime::client::ExecutionFilter &filter,
        const ::dokime::client::ExecutionOrder &order, client::ExecutionsWatchOptions options = {}) {
    detail::ExecutionsWatchRequest request;
    request.set_parent(parent);
    *request.mutable_selection() = selection;
    *request.mutable_filter() = filter;
    *request.mutable_order() = order;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::ExecutionsWatchResponse>>(
        detail::ExecutionsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::ExecutionsWatchResponse> {
              detail::ExecutionsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read executions.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::ExecutionsCreateResponse create(const std::string &parent,
                                                    const std::vector<::dokime::client::ExecutionBinding> &executions,
                                                    client::ExecutionsCreateOptions options = {}) {
    detail::ExecutionsCreateRequest request;
    request.set_parent(parent);
    request.mutable_executions()->Assign(executions.begin(), executions.end());
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsCreateResponse response;
    const auto status = detail::ExecutionsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsStartResponse start(const std::string &name, const ::dokime::client::ExecutionMode &mode,
                                                  client::ExecutionsStartOptions options = {}) {
    detail::ExecutionsStartRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_mode(mode);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsStartResponse response;
    const auto status = detail::ExecutionsStartService::NewStub(channel)->Start(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.start: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsStopResponse stop(const std::string &name, client::ExecutionsStopOptions options = {}) {
    detail::ExecutionsStopRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsStopResponse response;
    const auto status = detail::ExecutionsStopService::NewStub(channel)->Stop(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.stop: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsCancelResponse cancel(const std::string &name, client::ExecutionsCancelOptions options = {}) {
    detail::ExecutionsCancelRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsCancelResponse response;
    const auto status = detail::ExecutionsCancelService::NewStub(channel)->Cancel(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.cancel: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsDeleteResponse delete_(const std::string &name, client::ExecutionsDeleteOptions options = {}) {
    detail::ExecutionsDeleteRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsDeleteResponse response;
    const auto status = detail::ExecutionsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.delete: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsParametersResponse parameters(const std::string &name, const ::dokime::client::ExecutionParameterSet &set,
                                                            client::ExecutionsParametersOptions options = {}) {
    detail::ExecutionsParametersRequest request;
    request.set_name(name);
    request.set_set(set);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsParametersResponse response;
    const auto status = detail::ExecutionsParametersService::NewStub(channel)->Parameters(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.parameters: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ExecutionsStepsResponse steps(const std::string &name, client::ExecutionsStepsOptions options = {}) {
    detail::ExecutionsStepsRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ExecutionsStepsResponse response;
    const auto status = detail::ExecutionsStepsService::NewStub(channel)->Steps(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call executions.steps: " + status.error_message());
    }
    return response;
  }
};
struct FilesResource {
  std::shared_ptr<grpc::Channel> channel;
  FilesUploadsResource uploads;
  explicit FilesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), uploads(channel) {}
  ::dokime::FilesGetResponse get(const ::harmos::FileReference &file, client::FilesGetOptions options = {}) {
    detail::FilesGetRequest request;
    *request.mutable_file() = file;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesGetResponse response;
    const auto status = detail::FilesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::FilesReadResponse read(const ::harmos::FileReference &file, const std::uint64_t &offset, const std::uint64_t &max_bytes,
                                   client::FilesReadOptions options = {}) {
    detail::FilesReadRequest request;
    *request.mutable_file() = file;
    request.set_offset(offset);
    request.set_max_bytes(max_bytes);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesReadResponse response;
    const auto status = detail::FilesReadService::NewStub(channel)->Read(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.read: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::FilesWatchResponse> watch(const ::harmos::FileReference &file, const std::uint64_t &offset,
                                                             client::FilesWatchOptions options = {}) {
    detail::FilesWatchRequest request;
    *request.mutable_file() = file;
    request.set_offset(offset);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::FilesWatchResponse>>(
        detail::FilesWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::FilesWatchResponse> {
              detail::FilesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read files.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::harmos::Receipt delete_(const ::harmos::FileReference &file, client::FilesDeleteOptions options = {}) {
    detail::FilesDeleteRequest request;
    *request.mutable_file() = file;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::FilesDeleteResponse response;
    const auto status = detail::FilesDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call files.delete: " + status.error_message());
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read files.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
};
struct InvocationsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit InvocationsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  std::string create(const std::string &operation, const std::string &fingerprint, const std::string &payload,
                     client::InvocationsCreateOptions options = {}) {
    detail::InvocationsCreateRequest request;
    request.set_operation(operation);
    request.set_fingerprint(fingerprint);
    request.set_payload(payload);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::InvocationsCreateResponse response;
    const auto status = detail::InvocationsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call invocations.create: " + status.error_message());
    }
    return response.payload();
  }
  ::harmos::InstanceState get(const std::uint64_t &instance, client::InvocationsGetOptions options = {}) {
    detail::InvocationsGetRequest request;
    request.set_instance(instance);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::InvocationsGetResponse response;
    const auto status = detail::InvocationsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call invocations.get: " + status.error_message());
    }
    if (!response.has_invocation()) {
      throw std::runtime_error("Read invocations.get: Missing Payload");
    }
    return std::move(*response.mutable_invocation());
  }
  ::harmos::InstanceState cancel(const std::uint64_t &instance, const std::string &reason, client::InvocationsCancelOptions options = {}) {
    detail::InvocationsCancelRequest request;
    request.set_instance(instance);
    request.set_reason(reason);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::InvocationsCancelResponse response;
    const auto status = detail::InvocationsCancelService::NewStub(channel)->Cancel(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call invocations.cancel: " + status.error_message());
    }
    if (!response.has_invocation()) {
      throw std::runtime_error("Read invocations.cancel: Missing Payload");
    }
    return std::move(*response.mutable_invocation());
  }
  ::harmos::ClientStream<::harmos::InstanceState> watch(const std::uint64_t &instance, client::InvocationsWatchOptions options = {}) {
    detail::InvocationsWatchRequest request;
    request.set_instance(instance);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::InvocationsWatchResponse>>(
        detail::InvocationsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::harmos::InstanceState> {
              detail::InvocationsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read invocations.watch: " + status.error_message());
                }
                return {};
              }
              if (!response.has_invocation()) {
                throw std::runtime_error("Read invocations.watch: Missing Payload");
              }
              return std::move(*response.mutable_invocation());
            }};
  }
};
struct MediaResource {
  MediaCapturesResource captures;
  explicit MediaResource(std::shared_ptr<grpc::Channel> channel) : captures(channel) {}
};
struct OperationsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit OperationsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  std::vector<::harmos::DiscoveredOperation> list(client::OperationsListOptions options = {}) {
    detail::OperationsListRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::OperationsListResponse response;
    const auto status = detail::OperationsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call operations.list: " + status.error_message());
    }
    return std::vector<::harmos::DiscoveredOperation>(response.operations().begin(), response.operations().end());
  }
  ::harmos::DiscoveredOperation get(const std::string &id, client::OperationsGetOptions options = {}) {
    detail::OperationsGetRequest request;
    request.set_id(id);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::OperationsGetResponse response;
    const auto status = detail::OperationsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call operations.get: " + status.error_message());
    }
    if (!response.has_operation()) {
      throw std::runtime_error("Read operations.get: Missing Payload");
    }
    return std::move(*response.mutable_operation());
  }
};
struct PlotsResource {
  PlotsSettingsResource settings;
  PlotsSnapshotsResource snapshots;
  explicit PlotsResource(std::shared_ptr<grpc::Channel> channel) : settings(channel), snapshots(channel) {}
};
struct PluginsResource {
  std::shared_ptr<grpc::Channel> channel;
  PluginsSecretsResource secrets;
  PluginsSettingsResource settings;
  explicit PluginsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), secrets(channel), settings(channel) {}
  ::harmos::client::PluginsGetResponse get(const std::string &name, client::PluginsGetOptions options = {}) {
    detail::PluginsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsGetResponse response;
    const auto status = detail::PluginsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::PluginsListResponse list(client::PluginsListOptions options = {}) {
    detail::PluginsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsListResponse response;
    const auto status = detail::PluginsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::PluginsWatchResponse> watch(client::PluginsWatchOptions options = {}) {
    detail::PluginsWatchRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::PluginsWatchResponse>>(
        detail::PluginsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::harmos::client::PluginsWatchResponse> {
              detail::PluginsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read plugins.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::harmos::client::PluginsCreateResponse create(const std::string &name, const ::harmos::client::PluginPlacement &placement,
                                                 const ::harmos::client::PluginValue &value, client::PluginsCreateOptions options = {}) {
    detail::PluginsCreateRequest request;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_name(name);
    *request.mutable_placement() = placement;
    *request.mutable_value() = value;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsCreateResponse response;
    const auto status = detail::PluginsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.create: " + status.error_message());
    }
    return response;
  }
  ::harmos::client::PluginsSetResponse set(const std::string &name, const ::harmos::client::PluginValue &value,
                                           const std::string &expected_revision, client::PluginsSetOptions options = {}) {
    detail::PluginsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsSetResponse response;
    const auto status = detail::PluginsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.set: " + status.error_message());
    }
    return response;
  }
  ::harmos::Receipt delete_(const std::string &name, const std::string &expected_revision, client::PluginsDeleteOptions options = {}) {
    detail::PluginsDeleteRequest request;
    request.set_name(name);
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsDeleteResponse response;
    const auto status = detail::PluginsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.delete: " + status.error_message());
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read plugins.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
  ::harmos::client::PluginEntry restart(const std::string &name, const std::string &expected_revision,
                                        client::PluginsRestartOptions options = {}) {
    detail::PluginsRestartRequest request;
    request.set_name(name);
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsRestartResponse response;
    const auto status = detail::PluginsRestartService::NewStub(channel)->Restart(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.restart: " + status.error_message());
    }
    if (!response.has_entry()) {
      throw std::runtime_error("Read plugins.restart: Missing Payload");
    }
    return std::move(*response.mutable_entry());
  }
  ::harmos::client::PluginsReplaceResponse replace(const std::string &name, const ::harmos::client::PluginPlacement &placement,
                                                   const std::string &expected_revision, client::PluginsReplaceOptions options = {}) {
    detail::PluginsReplaceRequest request;
    request.set_name(name);
    *request.mutable_placement() = placement;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PluginsReplaceResponse response;
    const auto status = detail::PluginsReplaceService::NewStub(channel)->Replace(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call plugins.replace: " + status.error_message());
    }
    return response;
  }
};
struct PromptDefinitionsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptDefinitionsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::PromptDefinitionsGetResponse get(const std::string &name, const std::string &parent,
                                                     const std::uint32_t &schema_version,
                                                     client::PromptDefinitionsGetOptions options = {}) {
    detail::PromptDefinitionsGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PromptDefinitionsGetResponse response;
    const auto status = detail::PromptDefinitionsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call prompt_definitions.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::PromptDefinitionsListResponse list(const std::string &parent, client::PromptDefinitionsListOptions options = {}) {
    detail::PromptDefinitionsListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PromptDefinitionsListResponse response;
    const auto status = detail::PromptDefinitionsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call prompt_definitions.list: " + status.error_message());
    }
    return response;
  }
};
struct PromptsResource {
  std::shared_ptr<grpc::Channel> channel;
  PromptsAnswersResource answers;
  explicit PromptsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel), answers(channel) {}
  ::dokime::client::PromptsGetResponse get(const std::string &name, client::PromptsGetOptions options = {}) {
    detail::PromptsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PromptsGetResponse response;
    const auto status = detail::PromptsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call prompts.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::PromptsListResponse list(const std::string &parent, const bool &pending_only, client::PromptsListOptions options = {}) {
    detail::PromptsListRequest request;
    request.set_parent(parent);
    if (options.cycle_id) {
      request.set_cycle_id(*options.cycle_id);
    }
    if (options.execution_id) {
      request.set_execution_id(*options.execution_id);
    }
    request.set_pending_only(pending_only);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::PromptsListResponse response;
    const auto status = detail::PromptsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call prompts.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::PromptsWatchResponse> watch(const std::string &parent, const bool &pending_only,
                                                                       client::PromptsWatchOptions options = {}) {
    detail::PromptsWatchRequest request;
    request.set_parent(parent);
    if (options.cycle_id) {
      request.set_cycle_id(*options.cycle_id);
    }
    if (options.execution_id) {
      request.set_execution_id(*options.execution_id);
    }
    request.set_pending_only(pending_only);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::PromptsWatchResponse>>(
        detail::PromptsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::PromptsWatchResponse> {
              detail::PromptsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read prompts.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct RecordSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit RecordSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::RecordSchemasGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                                 client::RecordSchemasGetOptions options = {}) {
    detail::RecordSchemasGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::RecordSchemasGetResponse response;
    const auto status = detail::RecordSchemasGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call record_schemas.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::RecordSchemasListResponse list(const std::string &parent, client::RecordSchemasListOptions options = {}) {
    detail::RecordSchemasListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::RecordSchemasListResponse response;
    const auto status = detail::RecordSchemasListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call record_schemas.list: " + status.error_message());
    }
    return response;
  }
};
struct ReportsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ReportsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::ReportsGetResponse get(const ::dokime::client::ReportSubject &subject, const std::string &name,
                                           client::ReportsGetOptions options = {}) {
    detail::ReportsGetRequest request;
    *request.mutable_subject() = subject;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ReportsGetResponse response;
    const auto status = detail::ReportsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call reports.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ReportsCreateResponse create(const ::dokime::client::ReportSubject &subject, const std::string &document_id,
                                                 client::ReportsCreateOptions options = {}) {
    detail::ReportsCreateRequest request;
    *request.mutable_subject() = subject;
    request.set_document_id(document_id);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ReportsCreateResponse response;
    const auto status = detail::ReportsCreateService::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call reports.create: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::ReportsPublishResponse publish(const std::string &name, const ::dokime::client::ReportSubject &subject,
                                                   const std::string &expected_revision, client::ReportsPublishOptions options = {}) {
    detail::ReportsPublishRequest request;
    request.set_name(name);
    *request.mutable_subject() = subject;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::ReportsPublishResponse response;
    const auto status = detail::ReportsPublishService::NewStub(channel)->Publish(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call reports.publish: " + status.error_message());
    }
    return response;
  }
};
struct SchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  std::vector<::harmos::client::SchemaDescription> list(client::SchemasListOptions options = {}) {
    detail::SchemasListRequest request;
    if (options.schema) {
      request.set_schema(*options.schema);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SchemasListResponse response;
    const auto status = detail::SchemasListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call schemas.list: " + status.error_message());
    }
    return std::vector<::harmos::client::SchemaDescription>(response.schemas().begin(), response.schemas().end());
  }
};
struct SecretsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SecretsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::SecretsGetResponse get(const std::string &name, client::SecretsGetOptions options = {}) {
    detail::SecretsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsGetResponse response;
    const auto status = detail::SecretsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SecretsSetResponse set(const std::string &name, const ::dokime::SecretWritable &value,
                                           const std::string &expected_revision, client::SecretsSetOptions options = {}) {
    detail::SecretsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsSetResponse response;
    const auto status = detail::SecretsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.set: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SecretsListResponse list(client::SecretsListOptions options = {}) {
    detail::SecretsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsListResponse response;
    const auto status = detail::SecretsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.list: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::SecretsWatchResponse> watch(client::SecretsWatchOptions options = {}) {
    detail::SecretsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::SecretsWatchResponse>>(
        detail::SecretsWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::dokime::client::SecretsWatchResponse> {
              detail::SecretsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read secrets.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::SecretsDeleteResponse delete_(const std::vector<std::string> &names, const std::string &expected_revision,
                                                  client::SecretsDeleteOptions options = {}) {
    detail::SecretsDeleteRequest request;
    request.mutable_names()->Assign(names.begin(), names.end());
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsDeleteResponse response;
    const auto status = detail::SecretsDeleteService::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.delete: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SecretsExportResponse export(const std::vector<std::string> &names, const std::uint64_t &generation,
                                                 client::SecretsExportOptions options = {}) {
    detail::SecretsExportRequest request;
    request.mutable_names()->Assign(names.begin(), names.end());
    request.set_generation(generation);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsExportResponse response;
    const auto status = detail::SecretsExportService::NewStub(channel)->Export(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.export: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SecretsImportResponse import(const std::string &data, const std::vector<::dokime::client::SecretImportChoice> &choices,
                                                 const std::uint64_t &generation, const bool &validate_only,
                                                 client::SecretsImportOptions options = {}) {
    detail::SecretsImportRequest request;
    request.set_data(data);
    request.mutable_choices()->Assign(choices.begin(), choices.end());
    request.set_generation(generation);
    request.set_validate_only(validate_only);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SecretsImportResponse response;
    const auto status = detail::SecretsImportService::NewStub(channel)->Import(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call secrets.import: " + status.error_message());
    }
    return response;
  }
};
struct SettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SettingsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::SettingsGetResponse get(client::SettingsGetOptions options = {}) {
    detail::SettingsGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SettingsGetResponse response;
    const auto status = detail::SettingsGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call settings.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SettingsSetResponse set(const ::dokime::client::ApplicationSettings &value, const std::string &expected_revision,
                                            client::SettingsSetOptions options = {}) {
    detail::SettingsSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SettingsSetResponse response;
    const auto status = detail::SettingsSetService::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call settings.set: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SettingsSchemaResponse schema(client::SettingsSchemaOptions options = {}) {
    detail::SettingsSchemaRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SettingsSchemaResponse response;
    const auto status = detail::SettingsSchemaService::NewStub(channel)->Schema(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call settings.schema: " + status.error_message());
    }
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
  ::harmos::client::StatusGetResponse get(client::StatusGetOptions options = {}) {
    detail::StatusGetRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StatusGetResponse response;
    const auto status = detail::StatusGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call status.get: " + status.error_message());
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::StatusWatchResponse> watch(client::StatusWatchOptions options = {}) {
    detail::StatusWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::StatusWatchResponse>>(
        detail::StatusWatchService::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader]() -> std::optional<::harmos::client::StatusWatchResponse> {
              detail::StatusWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw std::runtime_error("Read status.watch: " + status.error_message());
                }
                return {};
              }
              return response;
            }};
  }
};
struct StorageResource {
  StorageSourcesResource sources;
  explicit StorageResource(std::shared_ptr<grpc::Channel> channel) : sources(channel) {}
};
struct StreamSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StreamSchemasResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::StreamSchemasGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                                 client::StreamSchemasGetOptions options = {}) {
    detail::StreamSchemasGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StreamSchemasGetResponse response;
    const auto status = detail::StreamSchemasGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call stream_schemas.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::StreamSchemasListResponse list(const std::string &parent, client::StreamSchemasListOptions options = {}) {
    detail::StreamSchemasListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StreamSchemasListResponse response;
    const auto status = detail::StreamSchemasListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call stream_schemas.list: " + status.error_message());
    }
    return response;
  }
};
struct StreamsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StreamsResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  std::vector<::harmos::client::StreamIdentity> list(client::StreamsListOptions options = {}) {
    detail::StreamsListRequest request;
    if (options.schema) {
      request.set_schema(*options.schema);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::StreamsListResponse response;
    const auto status = detail::StreamsListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call streams.list: " + status.error_message());
    }
    return std::vector<::harmos::client::StreamIdentity>(response.streams().begin(), response.streams().end());
  }
};
struct SuitesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SuitesResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::dokime::client::SuitesGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                          client::SuitesGetOptions options = {}) {
    detail::SuitesGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SuitesGetResponse response;
    const auto status = detail::SuitesGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call suites.get: " + status.error_message());
    }
    return response;
  }
  ::dokime::client::SuitesListResponse list(const std::string &parent, client::SuitesListOptions options = {}) {
    detail::SuitesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::SuitesListResponse response;
    const auto status = detail::SuitesListService::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call suites.list: " + status.error_message());
    }
    return response;
  }
};
struct TimeResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit TimeResource(std::shared_ptr<grpc::Channel> channel) : channel(channel) {}
  ::google::protobuf::Timestamp get(client::TimeGetOptions options = {}) {
    detail::TimeGetRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::TimeGetResponse response;
    const auto status = detail::TimeGetService::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call time.get: " + status.error_message());
    }
    if (!response.has_now()) {
      throw std::runtime_error("Read time.get: Missing Payload");
    }
    return std::move(*response.mutable_now());
  }
  ::harmos::TimeSleepResponse sleep(const ::google::protobuf::Duration &duration, client::TimeSleepOptions options = {}) {
    detail::TimeSleepRequest request;
    *request.mutable_duration() = duration;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::TimeSleepResponse response;
    const auto status = detail::TimeSleepService::NewStub(channel)->Sleep(context.get(), request, &response);
    if (!status.ok()) {
      throw std::runtime_error("Call time.sleep: " + status.error_message());
    }
    return response;
  }
};
struct Client {
  AccessResource access;
  AggregatesResource aggregates;
  AnalysesResource analyses;
  BackendResource backend;
  CasesResource cases;
  ContentResource content;
  CyclesResource cycles;
  DocumentsResource documents;
  EvidenceResource evidence;
  ExecutionsResource executions;
  FilesResource files;
  InvocationsResource invocations;
  MediaResource media;
  OperationsResource operations;
  PlotsResource plots;
  PluginsResource plugins;
  PromptDefinitionsResource prompt_definitions;
  PromptsResource prompts;
  RecordSchemasResource record_schemas;
  ReportsResource reports;
  SchemasResource schemas;
  SecretsResource secrets;
  SettingsResource settings;
  StatusResource status;
  StorageResource storage;
  StreamSchemasResource stream_schemas;
  StreamsResource streams;
  SuitesResource suites;
  TimeResource time;
  explicit Client(std::shared_ptr<grpc::Channel> channel)
      : access(channel), aggregates(channel), analyses(channel), backend(channel), cases(channel), content(channel), cycles(channel),
        documents(channel), evidence(channel), executions(channel), files(channel), invocations(channel), media(channel),
        operations(channel), plots(channel), plugins(channel), prompt_definitions(channel), prompts(channel), record_schemas(channel),
        reports(channel), schemas(channel), secrets(channel), settings(channel), status(channel), storage(channel), stream_schemas(channel),
        streams(channel), suites(channel), time(channel) {}
  static Client connect(std::string address, const ::harmos::StorageTransferSettings &transfer = {},
                        std::optional<int> message_bytes = {}) {
    const auto maximum = ::harmos::transfer_message_bytes(transfer, message_bytes);
    grpc::ChannelArguments arguments;
    arguments.SetMaxSendMessageSize(maximum);
    arguments.SetMaxReceiveMessageSize(maximum);
    if (address.starts_with("http://")) {
      address.erase(0, 7);
    }
    return Client(grpc::CreateCustomChannel(address, grpc::InsecureChannelCredentials(), arguments));
  }
};
} // namespace dokime
