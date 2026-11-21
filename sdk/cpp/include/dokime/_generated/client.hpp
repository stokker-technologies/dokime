// @generated from protocol descriptors; do not edit.
#pragma once
#include "detail/client.hpp"
#include "dokime/errors.hpp"
#include "dokime/messages.hpp"
#include "harmos/client_transport.hpp"
#include "harmos/messages.hpp"
namespace dokime {
namespace client {
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
struct AggregatesGetOptions {
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
struct ExecutionsSealOptions {
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
  std::optional<::dokime::RecordScope> scope;
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
  std::optional<::dokime::EvidenceScope> scope;
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
  std::optional<::dokime::EvidenceScope> scope;
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
  std::optional<std::string> name;
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
  std::optional<::harmos::FileReference> file;
  std::optional<::harmos::Upload> upload;
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct FilesDeleteOptions {
  std::optional<std::string> request_id;
  std::shared_ptr<grpc::ClientContext> context;
};
struct InvocationsCreateOptions {
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
struct OperationsListOptions {
  std::shared_ptr<grpc::ClientContext> context;
};
struct OperationsGetOptions {
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
struct PluginsSettingsListOptions {
  std::optional<std::string> name;
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
  std::optional<std::string> origin;
  std::optional<std::string> sdk_version;
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
struct SuitesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct SuitesListOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ReportsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::shared_ptr<grpc::ClientContext> context;
};
struct ReportsCreateOptions {
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
} // namespace client
struct AccessCertificatesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessCertificatesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  std::string create(client::AccessCertificatesCreateOptions options = {}) {
    detail::packages::harmos::host::access::CertificatesCreateRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::CertificatesCreateResponse response;
    const auto status = detail::packages::dokime::host::access::Certificates::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.certificates.create", status, *context);
    }
    return response.certificate_pem();
  }
};
struct AccessListenerResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessListenerResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::harmos::client::ListenerGetResponse get(client::AccessListenerGetOptions options = {}) {
    detail::packages::harmos::host::access::ListenerGetRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::ListenerGetResponse response;
    const auto status = detail::packages::dokime::host::access::Listener::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.listener.get", status, *context);
    }
    return response;
  }
  ::harmos::client::ListenerSetResponse set(const ::harmos::client::ListenerValue &value, const std::string &expected_revision,
                                            client::AccessListenerSetOptions options = {}) {
    detail::packages::harmos::host::access::ListenerSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::ListenerSetResponse response;
    const auto status = detail::packages::dokime::host::access::Listener::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.listener.set", status, *context);
    }
    return response;
  }
};
struct AccessPairingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AccessPairingsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::harmos::client::PairingsCreateResponse create(const std::string &label, client::AccessPairingsCreateOptions options = {}) {
    detail::packages::harmos::host::access::PairingsCreateRequest request;
    request.set_label(label);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::PairingsCreateResponse response;
    const auto status = detail::packages::dokime::host::access::Pairings::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.pairings.create", status, *context);
    }
    return response;
  }
  ::harmos::client::PairingsListResponse list(client::AccessPairingsListOptions options = {}) {
    detail::packages::harmos::host::access::PairingsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::PairingsListResponse response;
    const auto status = detail::packages::dokime::host::access::Pairings::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.pairings.list", status, *context);
    }
    return response;
  }
  ::harmos::client::PairingsDeleteResponse delete_(const std::string &name, client::AccessPairingsDeleteOptions options = {}) {
    detail::packages::harmos::host::access::PairingsDeleteRequest request;
    request.set_name(name);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::access::PairingsDeleteResponse response;
    const auto status = detail::packages::dokime::host::access::Pairings::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("access.pairings.delete", status, *context);
    }
    return response;
  }
};
struct BackendOverviewResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit BackendOverviewResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::OverviewGetResponse get(client::BackendOverviewGetOptions options = {}) {
    detail::packages::dokime::host::backend::OverviewGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::backend::OverviewGetResponse response;
    const auto status = detail::packages::dokime::host::backend::Overview::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("backend.overview.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::OverviewWatchResponse> watch(client::BackendOverviewWatchOptions options = {}) {
    detail::packages::dokime::host::backend::OverviewWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::backend::OverviewWatchResponse>>(
        detail::packages::dokime::host::backend::Overview::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::OverviewWatchResponse> {
              detail::packages::dokime::host::backend::OverviewWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("backend.overview.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct ContentArtifactsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentArtifactsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::ArtifactsGetResponse get(const std::string &name, client::ContentArtifactsGetOptions options = {}) {
    detail::packages::dokime::host::content::ArtifactsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::content::ArtifactsGetResponse response;
    const auto status = detail::packages::dokime::host::content::Artifacts::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("content.artifacts.get", status, *context);
    }
    return response;
  }
  ::dokime::ArtifactsCreateResponse create(const ::harmos::FileReference &file, const std::string &filename,
                                           const std::string &content_type, client::ContentArtifactsCreateOptions options = {}) {
    detail::packages::dokime::host::content::ArtifactsCreateRequest request;
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
    detail::packages::dokime::host::content::ArtifactsCreateResponse response;
    const auto status = detail::packages::dokime::host::content::Artifacts::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("content.artifacts.create", status, *context);
    }
    return response;
  }
  ::dokime::client::ArtifactsListResponse list(const ::dokime::ArtifactScope &scope, client::ContentArtifactsListOptions options = {}) {
    detail::packages::dokime::host::content::ArtifactsListRequest request;
    *request.mutable_scope() = scope;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::content::ArtifactsListResponse response;
    const auto status = detail::packages::dokime::host::content::Artifacts::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("content.artifacts.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ArtifactsWatchResponse> watch(const ::dokime::ArtifactScope &scope,
                                                                         client::ContentArtifactsWatchOptions options = {}) {
    detail::packages::dokime::host::content::ArtifactsWatchRequest request;
    *request.mutable_scope() = scope;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::content::ArtifactsWatchResponse>>(
        detail::packages::dokime::host::content::Artifacts::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::ArtifactsWatchResponse> {
              detail::packages::dokime::host::content::ArtifactsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("content.artifacts.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct ContentExecutionFilesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ContentExecutionFilesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::ExecutionFilesGetResponse get(const std::string &parent, const std::string &name,
                                                  client::ContentExecutionFilesGetOptions options = {}) {
    detail::packages::dokime::host::content::ExecutionFilesGetRequest request;
    request.set_parent(parent);
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::content::ExecutionFilesGetResponse response;
    const auto status = detail::packages::dokime::host::content::ExecutionFiles::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("content.execution_files.get", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionFilesListResponse list(const std::string &parent, client::ContentExecutionFilesListOptions options = {}) {
    detail::packages::dokime::host::content::ExecutionFilesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::content::ExecutionFilesListResponse response;
    const auto status = detail::packages::dokime::host::content::ExecutionFiles::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("content.execution_files.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ExecutionFilesWatchResponse> watch(const std::string &parent,
                                                                              client::ContentExecutionFilesWatchOptions options = {}) {
    detail::packages::dokime::host::content::ExecutionFilesWatchRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::content::ExecutionFilesWatchResponse>>(
        detail::packages::dokime::host::content::ExecutionFiles::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::ExecutionFilesWatchResponse> {
              detail::packages::dokime::host::content::ExecutionFilesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("content.execution_files.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceHealthEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceHealthEventsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::HealthEventsListResponse list(const ::dokime::client::AnnotationSelection &selection,
                                                  client::EvidenceHealthEventsListOptions options = {}) {
    detail::packages::dokime::host::evidence::HealthEventsListRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::evidence::HealthEventsListResponse response;
    const auto status = detail::packages::dokime::host::evidence::HealthEvents::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.health_events.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::HealthEventsWatchResponse> watch(const ::dokime::client::AnnotationSelection &selection,
                                                                            client::EvidenceHealthEventsWatchOptions options = {}) {
    detail::packages::dokime::host::evidence::HealthEventsWatchRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::evidence::HealthEventsWatchResponse>>(
        detail::packages::dokime::host::evidence::HealthEvents::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::HealthEventsWatchResponse> {
              detail::packages::dokime::host::evidence::HealthEventsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("evidence.health_events.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceNotesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceNotesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::NotesGetResponse get(const std::string &name, client::EvidenceNotesGetOptions options = {}) {
    detail::packages::dokime::host::evidence::NotesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::evidence::NotesGetResponse response;
    const auto status = detail::packages::dokime::host::evidence::Notes::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.notes.get", status, *context);
    }
    return response;
  }
  ::dokime::client::NotesSetResponse set(const std::string &name, const ::dokime::client::NoteWritable &value,
                                         const std::string &expected_revision, client::EvidenceNotesSetOptions options = {}) {
    detail::packages::dokime::host::evidence::NotesSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::evidence::NotesSetResponse response;
    const auto status = detail::packages::dokime::host::evidence::Notes::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.notes.set", status, *context);
    }
    return response;
  }
  ::dokime::client::NotesListResponse list(const ::dokime::client::AnnotationSelection &selection,
                                           client::EvidenceNotesListOptions options = {}) {
    detail::packages::dokime::host::evidence::NotesListRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::evidence::NotesListResponse response;
    const auto status = detail::packages::dokime::host::evidence::Notes::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.notes.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::NotesWatchResponse> watch(const ::dokime::client::AnnotationSelection &selection,
                                                                     client::EvidenceNotesWatchOptions options = {}) {
    detail::packages::dokime::host::evidence::NotesWatchRequest request;
    *request.mutable_selection() = selection;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::evidence::NotesWatchResponse>>(
        detail::packages::dokime::host::evidence::Notes::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::NotesWatchResponse> {
              detail::packages::dokime::host::evidence::NotesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("evidence.notes.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::NotesCreateResponse create(const ::dokime::client::AnnotationOwner &owner, const ::dokime::client::NoteWritable &value,
                                               client::EvidenceNotesCreateOptions options = {}) {
    detail::packages::dokime::host::evidence::NotesCreateRequest request;
    *request.mutable_owner() = owner;
    *request.mutable_value() = value;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::evidence::NotesCreateResponse response;
    const auto status = detail::packages::dokime::host::evidence::Notes::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.notes.create", status, *context);
    }
    return response;
  }
};
struct EvidenceRecordsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceRecordsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::RecordsListResponse list(const std::string &record_type, const std::vector<std::uint32_t> &versions,
                                     const ::dokime::model::Window &between, const std::string &step,
                                     const ::dokime::CapturedRecordSelection &selection, client::EvidenceRecordsListOptions options = {}) {
    detail::packages::dokime::host::evidence::RecordsListRequest request;
    if (options.scope) {
      *request.mutable_scope() = *options.scope;
    }
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
    detail::packages::dokime::host::evidence::RecordsListResponse response;
    const auto status = detail::packages::dokime::host::evidence::Records::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.records.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::RecordsWatchResponse> watch(const ::dokime::RecordScope &scope, const std::string &record_type,
                                                                       const std::vector<std::uint32_t> &versions,
                                                                       const ::dokime::model::Window &between, const std::string &step,
                                                                       const ::dokime::CapturedRecordSelection &selection,
                                                                       client::EvidenceRecordsWatchOptions options = {}) {
    detail::packages::dokime::host::evidence::RecordsWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::evidence::RecordsWatchResponse>>(
        detail::packages::dokime::host::evidence::Records::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::RecordsWatchResponse> {
              detail::packages::dokime::host::evidence::RecordsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("evidence.records.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceSamplesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceSamplesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::SamplesListResponse list(const ::dokime::EvidenceScope &scope, const std::vector<std::string> &streams,
                                             const std::vector<std::uint32_t> &versions, const std::vector<std::string> &fields,
                                             const std::vector<std::string> &sources, const std::vector<std::string> &signals,
                                             const ::dokime::model::Window &between, const ::dokime::client::BucketRequest &bucket,
                                             const bool &latest, client::EvidenceSamplesListOptions options = {}) {
    detail::packages::dokime::host::evidence::SamplesListRequest request;
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
    detail::packages::dokime::host::evidence::SamplesListResponse response;
    const auto status = detail::packages::dokime::host::evidence::Samples::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.samples.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::SamplesWatchResponse>
  watch(const ::dokime::EvidenceScope &scope, const std::vector<std::string> &streams, const std::vector<std::uint32_t> &versions,
        const std::vector<std::string> &fields, const std::vector<std::string> &sources, const std::vector<std::string> &signals,
        const ::dokime::model::Window &between, const ::dokime::client::BucketRequest &bucket, const bool &latest,
        client::EvidenceSamplesWatchOptions options = {}) {
    detail::packages::dokime::host::evidence::SamplesWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::evidence::SamplesWatchResponse>>(
        detail::packages::dokime::host::evidence::Samples::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::SamplesWatchResponse> {
              detail::packages::dokime::host::evidence::SamplesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("evidence.samples.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct EvidenceTraceabilityResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit EvidenceTraceabilityResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::TraceabilityGetResponse get(client::EvidenceTraceabilityGetOptions options = {}) {
    detail::packages::dokime::host::evidence::TraceabilityGetRequest request;
    if (options.scope) {
      *request.mutable_scope() = *options.scope;
    }
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
    detail::packages::dokime::host::evidence::TraceabilityGetResponse response;
    const auto status = detail::packages::dokime::host::evidence::Traceability::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.traceability.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::TraceabilityWatchResponse> watch(const ::dokime::EvidenceScope &scope,
                                                                            client::EvidenceTraceabilityWatchOptions options = {}) {
    detail::packages::dokime::host::evidence::TraceabilityWatchRequest request;
    *request.mutable_scope() = scope;
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::evidence::TraceabilityWatchResponse>>(
        detail::packages::dokime::host::evidence::Traceability::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::TraceabilityWatchResponse> {
              detail::packages::dokime::host::evidence::TraceabilityWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("evidence.traceability.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct ExecutionsCommandsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsCommandsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::CommandsGetResponse get(const std::string &name, const std::string &parent,
                                            client::ExecutionsCommandsGetOptions options = {}) {
    detail::packages::dokime::host::executions::CommandsGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::executions::CommandsGetResponse response;
    const auto status = detail::packages::dokime::host::executions::Commands::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.commands.get", status, *context);
    }
    return response;
  }
  ::dokime::client::CommandsCreateResponse create(const std::string &parent, const std::string &command_id,
                                                  const ::dokime::client::ExecutionCommandAction &action,
                                                  client::ExecutionsCommandsCreateOptions options = {}) {
    detail::packages::dokime::host::executions::CommandsCreateRequest request;
    request.set_parent(parent);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_command_id(command_id);
    request.set_action(action);
    if (options.step_id) {
      request.set_step_id(*options.step_id);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::executions::CommandsCreateResponse response;
    const auto status = detail::packages::dokime::host::executions::Commands::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.commands.create", status, *context);
    }
    return response;
  }
};
struct ExecutionsProgressResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsProgressResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::ProgressGetResponse get(client::ExecutionsProgressGetOptions options = {}) {
    detail::packages::dokime::host::executions::ProgressGetRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::executions::ProgressGetResponse response;
    const auto status = detail::packages::dokime::host::executions::Progress::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.progress.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ProgressWatchResponse> watch(const std::string &name,
                                                                        client::ExecutionsProgressWatchOptions options = {}) {
    detail::packages::dokime::host::executions::ProgressWatchRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::executions::ProgressWatchResponse>>(
        detail::packages::dokime::host::executions::Progress::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::ProgressWatchResponse> {
              detail::packages::dokime::host::executions::ProgressWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("executions.progress.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct ExecutionsSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ExecutionsSchemasResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::SchemasGetResponse get(const std::string &name, client::ExecutionsSchemasGetOptions options = {}) {
    detail::packages::dokime::host::executions::SchemasGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::executions::SchemasGetResponse response;
    const auto status = detail::packages::dokime::host::executions::Schemas::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.schemas.get", status, *context);
    }
    return response;
  }
};
struct FilesUploadsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit FilesUploadsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::UploadsCreateResponse create(const ::harmos::FileMetadata &metadata, client::FilesUploadsCreateOptions options = {}) {
    detail::packages::dokime::host::files::UploadsCreateRequest request;
    *request.mutable_metadata() = metadata;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsCreateResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.create", status, *context);
    }
    return response;
  }
  ::dokime::UploadsGetResponse get(const ::harmos::Upload &upload, client::FilesUploadsGetOptions options = {}) {
    detail::packages::dokime::host::files::UploadsGetRequest request;
    *request.mutable_upload() = upload;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsGetResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.get", status, *context);
    }
    return response;
  }
  ::dokime::UploadsResumeResponse resume(const ::harmos::Upload &upload, client::FilesUploadsResumeOptions options = {}) {
    detail::packages::dokime::host::files::UploadsResumeRequest request;
    *request.mutable_upload() = upload;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsResumeResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Resume(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.resume", status, *context);
    }
    return response;
  }
  ::dokime::UploadsAppendResponse append(const ::harmos::Upload &upload, const std::uint64_t &offset, const std::string &data,
                                         client::FilesUploadsAppendOptions options = {}) {
    detail::packages::dokime::host::files::UploadsAppendRequest request;
    *request.mutable_upload() = upload;
    request.set_offset(offset);
    request.set_data(data);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsAppendResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Append(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.append", status, *context);
    }
    return response;
  }
  ::dokime::UploadsPublishResponse publish(const ::harmos::Upload &upload, const std::uint64_t &length, const std::string &sha256,
                                           client::FilesUploadsPublishOptions options = {}) {
    detail::packages::dokime::host::files::UploadsPublishRequest request;
    *request.mutable_upload() = upload;
    request.set_length(length);
    request.set_sha256(sha256);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsPublishResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Publish(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.publish", status, *context);
    }
    return response;
  }
  ::dokime::UploadsDeleteResponse delete_(const ::harmos::Upload &upload, client::FilesUploadsDeleteOptions options = {}) {
    detail::packages::dokime::host::files::UploadsDeleteRequest request;
    *request.mutable_upload() = upload;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::files::UploadsDeleteResponse response;
    const auto status = detail::packages::dokime::host::files::Uploads::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.uploads.delete", status, *context);
    }
    return response;
  }
};
struct MediaCapturesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit MediaCapturesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::CapturesCreateResponse create(const ::dokime::ArtifactScope &scope, const ::dokime::client::CaptureKind &kind,
                                                  const ::dokime::client::RecordingAudio &audio,
                                                  client::MediaCapturesCreateOptions options = {}) {
    detail::packages::dokime::host::media::CapturesCreateRequest request;
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
    detail::packages::dokime::host::media::CapturesCreateResponse response;
    const auto status = detail::packages::dokime::host::media::Captures::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("media.captures.create", status, *context);
    }
    return response;
  }
  ::dokime::client::CapturesGetResponse get(const std::string &name, client::MediaCapturesGetOptions options = {}) {
    detail::packages::dokime::host::media::CapturesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::media::CapturesGetResponse response;
    const auto status = detail::packages::dokime::host::media::Captures::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("media.captures.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::CapturesWatchResponse> watch(const std::string &name,
                                                                        client::MediaCapturesWatchOptions options = {}) {
    detail::packages::dokime::host::media::CapturesWatchRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::media::CapturesWatchResponse>>(
        detail::packages::dokime::host::media::Captures::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::CapturesWatchResponse> {
              detail::packages::dokime::host::media::CapturesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("media.captures.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::host::media::CapturesStopResponse stop(const std::string &name, client::MediaCapturesStopOptions options = {}) {
    detail::packages::dokime::host::media::CapturesStopRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::media::CapturesStopResponse response;
    const auto status = detail::packages::dokime::host::media::Captures::NewStub(channel)->Stop(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("media.captures.stop", status, *context);
    }
    return response;
  }
};
struct PlotsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSettingsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::host::plots::SettingsGetResponse get(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                                         const std::string &plot_id, client::PlotsSettingsGetOptions options = {}) {
    detail::packages::dokime::host::plots::SettingsGetRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_plot_id(plot_id);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plots::SettingsGetResponse response;
    const auto status = detail::packages::dokime::host::plots::Settings::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plots.settings.get", status, *context);
    }
    return response;
  }
  ::dokime::client::host::plots::SettingsSetResponse set(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                                         const std::string &plot_id, const ::dokime::client::PlotSettingsValue &value,
                                                         const std::string &expected_revision,
                                                         client::PlotsSettingsSetOptions options = {}) {
    detail::packages::dokime::host::plots::SettingsSetRequest request;
    *request.mutable_scope() = scope;
    request.set_document_id(document_id);
    request.set_plot_id(plot_id);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plots::SettingsSetResponse response;
    const auto status = detail::packages::dokime::host::plots::Settings::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plots.settings.set", status, *context);
    }
    return response;
  }
};
struct PlotsSnapshotsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PlotsSnapshotsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::SnapshotsListResponse list(const std::vector<::dokime::client::PlotRequest> &requests,
                                               client::PlotsSnapshotsListOptions options = {}) {
    detail::packages::dokime::host::plots::SnapshotsListRequest request;
    request.mutable_requests()->Assign(requests.begin(), requests.end());
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plots::SnapshotsListResponse response;
    const auto status = detail::packages::dokime::host::plots::Snapshots::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plots.snapshots.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::SnapshotsWatchResponse> watch(const std::vector<::dokime::client::PlotRequest> &requests,
                                                                         client::PlotsSnapshotsWatchOptions options = {}) {
    detail::packages::dokime::host::plots::SnapshotsWatchRequest request;
    request.mutable_requests()->Assign(requests.begin(), requests.end());
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::plots::SnapshotsWatchResponse>>(
        detail::packages::dokime::host::plots::Snapshots::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::SnapshotsWatchResponse> {
              detail::packages::dokime::host::plots::SnapshotsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("plots.snapshots.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct PluginsSecretsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PluginsSecretsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::host::plugins::SecretsListResponse list(client::PluginsSecretsListOptions options = {}) {
    detail::packages::dokime::host::plugins::SecretsListRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plugins::SecretsListResponse response;
    const auto status = detail::packages::dokime::host::plugins::Secrets::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.secrets.list", status, *context);
    }
    return response;
  }
};
struct PluginsSettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PluginsSettingsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::host::plugins::SettingsListResponse list(client::PluginsSettingsListOptions options = {}) {
    detail::packages::dokime::host::plugins::SettingsListRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plugins::SettingsListResponse response;
    const auto status = detail::packages::dokime::host::plugins::Settings::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.settings.list", status, *context);
    }
    return response;
  }
  ::dokime::client::host::plugins::SettingsSetResponse set(const std::string &name, const ::harmos::SettingValues &value,
                                                           const std::string &expected_revision,
                                                           client::PluginsSettingsSetOptions options = {}) {
    detail::packages::dokime::host::plugins::SettingsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::plugins::SettingsSetResponse response;
    const auto status = detail::packages::dokime::host::plugins::Settings::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.settings.set", status, *context);
    }
    return response;
  }
};
struct PromptsAnswersResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptsAnswersResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::AnswersCreateResponse create(const std::string &parent, const std::string &action, const ::dokime::MapValue &values,
                                                 const bool &accepted, client::PromptsAnswersCreateOptions options = {}) {
    detail::packages::dokime::host::prompts::AnswersCreateRequest request;
    request.set_parent(parent);
    request.set_action(action);
    *request.mutable_values() = values;
    request.set_accepted(accepted);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::prompts::AnswersCreateResponse response;
    const auto status = detail::packages::dokime::host::prompts::Answers::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("prompts.answers.create", status, *context);
    }
    return response;
  }
};
struct StatusDiagnosticsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusDiagnosticsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::harmos::client::DiagnosticsGetResponse get(const std::string &name, const std::uint64_t &offset, const std::uint64_t &max_bytes,
                                               client::StatusDiagnosticsGetOptions options = {}) {
    detail::packages::harmos::host::status::DiagnosticsGetRequest request;
    request.set_name(name);
    request.set_offset(offset);
    request.set_max_bytes(max_bytes);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::status::DiagnosticsGetResponse response;
    const auto status = detail::packages::dokime::host::status::Diagnostics::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("status.diagnostics.get", status, *context);
    }
    return response;
  }
};
struct StatusLogsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusLogsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::harmos::client::LogsListResponse list(client::StatusLogsListOptions options = {}) {
    detail::packages::harmos::host::status::LogsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::status::LogsListResponse response;
    const auto status = detail::packages::dokime::host::status::Logs::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("status.logs.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::LogsWatchResponse> watch(const std::uint64_t &max_bytes, const std::uint32_t &max_events,
                                                                    client::StatusLogsWatchOptions options = {}) {
    detail::packages::harmos::host::status::LogsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    request.set_max_bytes(max_bytes);
    request.set_max_events(max_events);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::harmos::host::status::LogsWatchResponse>>(
        detail::packages::dokime::host::status::Logs::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::harmos::client::LogsWatchResponse> {
              detail::packages::harmos::host::status::LogsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("status.logs.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct StatusTraceEventsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StatusTraceEventsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::harmos::client::TraceEventsListResponse list(client::StatusTraceEventsListOptions options = {}) {
    detail::packages::harmos::host::status::TraceEventsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    if (options.page) {
      *request.mutable_page() = *options.page;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::status::TraceEventsListResponse response;
    const auto status = detail::packages::dokime::host::status::TraceEvents::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("status.trace_events.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::harmos::client::TraceEventsWatchResponse> watch(const std::uint64_t &max_bytes, const std::uint32_t &max_events,
                                                                           client::StatusTraceEventsWatchOptions options = {}) {
    detail::packages::harmos::host::status::TraceEventsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    request.set_max_bytes(max_bytes);
    request.set_max_events(max_events);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::harmos::host::status::TraceEventsWatchResponse>>(
        detail::packages::dokime::host::status::TraceEvents::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::harmos::client::TraceEventsWatchResponse> {
              detail::packages::harmos::host::status::TraceEventsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("status.trace_events.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct StorageSourcesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StorageSourcesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::SourcesGetResponse get(client::StorageSourcesGetOptions options = {}) {
    detail::packages::dokime::host::storage::SourcesGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::storage::SourcesGetResponse response;
    const auto status = detail::packages::dokime::host::storage::Sources::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("storage.sources.get", status, *context);
    }
    return response;
  }
  ::dokime::client::SourcesSetResponse set(const ::dokime::client::StorageSources &value, const std::string &expected_revision,
                                           client::StorageSourcesSetOptions options = {}) {
    detail::packages::dokime::host::storage::SourcesSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::storage::SourcesSetResponse response;
    const auto status = detail::packages::dokime::host::storage::Sources::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("storage.sources.set", status, *context);
    }
    return response;
  }
};
struct AccessResource {
  AccessCertificatesResource certificates;
  AccessListenerResource listener;
  AccessPairingsResource pairings;
  explicit AccessResource(std::shared_ptr<grpc::Channel> connection)
      : certificates(connection), listener(connection), pairings(connection) {}
};
struct AggregatesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit AggregatesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::AggregatesGetResponse get(const ::dokime::client::QuerySource &source,
                                              const std::vector<::dokime::client::QueryFilter> &filters,
                                              const std::vector<std::string> &group_by,
                                              const std::vector<::dokime::client::QueryAggregate> &aggregates, const std::uint32_t &limit,
                                              client::AggregatesGetOptions options = {}) {
    detail::packages::dokime::host::AggregatesGetRequest request;
    request.set_source(source);
    request.mutable_filters()->Assign(filters.begin(), filters.end());
    request.mutable_group_by()->Assign(group_by.begin(), group_by.end());
    request.mutable_aggregates()->Assign(aggregates.begin(), aggregates.end());
    request.set_limit(limit);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::AggregatesGetResponse response;
    const auto status = detail::packages::dokime::host::Aggregates::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("aggregates.get", status, *context);
    }
    return response;
  }
};
struct BackendResource {
  std::shared_ptr<grpc::Channel> channel;
  BackendOverviewResource overview;
  explicit BackendResource(std::shared_ptr<grpc::Channel> connection) : channel(connection), overview(connection) {}
  ::dokime::client::BackendAppearanceResponse appearance(client::BackendAppearanceOptions options = {}) {
    detail::packages::dokime::host::BackendAppearanceRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::BackendAppearanceResponse response;
    const auto status = detail::packages::dokime::host::Backend::NewStub(channel)->Appearance(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("backend.appearance", status, *context);
    }
    return response;
  }
};
struct CasesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CasesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::CasesCreateResponse create(const ::dokime::client::SelectionPlan &plan, const bool &validate_only,
                                               client::CasesCreateOptions options = {}) {
    detail::packages::dokime::host::CasesCreateRequest request;
    *request.mutable_plan() = plan;
    request.set_validate_only(validate_only);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CasesCreateResponse response;
    const auto status = detail::packages::dokime::host::Cases::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cases.create", status, *context);
    }
    return response;
  }
  ::dokime::client::CasesGetResponse get(const std::string &name, const std::string &parent, client::CasesGetOptions options = {}) {
    detail::packages::dokime::host::CasesGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CasesGetResponse response;
    const auto status = detail::packages::dokime::host::Cases::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cases.get", status, *context);
    }
    return response;
  }
  ::dokime::client::CasesListResponse list(const std::string &parent, client::CasesListOptions options = {}) {
    detail::packages::dokime::host::CasesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CasesListResponse response;
    const auto status = detail::packages::dokime::host::Cases::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cases.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::CasesWatchResponse> watch(const std::string &parent, client::CasesWatchOptions options = {}) {
    detail::packages::dokime::host::CasesWatchRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::CasesWatchResponse>>(
        detail::packages::dokime::host::Cases::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::CasesWatchResponse> {
              detail::packages::dokime::host::CasesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("cases.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::harmos::Receipt delete_(const std::string &name, const std::string &parent, client::CasesDeleteOptions options = {}) {
    detail::packages::dokime::host::CasesDeleteRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CasesDeleteResponse response;
    const auto status = detail::packages::dokime::host::Cases::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cases.delete", status, *context);
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read cases.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
  ::dokime::client::CasesParametersResponse parameters(const std::string &name, const std::string &parent,
                                                       client::CasesParametersOptions options = {}) {
    detail::packages::dokime::host::CasesParametersRequest request;
    request.set_name(name);
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CasesParametersResponse response;
    const auto status = detail::packages::dokime::host::Cases::NewStub(channel)->Parameters(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cases.parameters", status, *context);
    }
    return response;
  }
};
struct ContentResource {
  ContentArtifactsResource artifacts;
  ContentExecutionFilesResource execution_files;
  explicit ContentResource(std::shared_ptr<grpc::Channel> connection) : artifacts(connection), execution_files(connection) {}
};
struct CyclesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit CyclesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::CyclesGetResponse get(const std::string &name, client::CyclesGetOptions options = {}) {
    detail::packages::dokime::host::CyclesGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CyclesGetResponse response;
    const auto status = detail::packages::dokime::host::Cycles::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cycles.get", status, *context);
    }
    return response;
  }
  ::dokime::client::CyclesSetResponse set(const std::string &name, const ::dokime::client::CycleWritable &value,
                                          const std::string &expected_revision, client::CyclesSetOptions options = {}) {
    detail::packages::dokime::host::CyclesSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CyclesSetResponse response;
    const auto status = detail::packages::dokime::host::Cycles::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cycles.set", status, *context);
    }
    return response;
  }
  ::dokime::client::CyclesListResponse list(const ::dokime::client::CycleSelection &selection, const ::dokime::client::CycleFilter &filter,
                                            const ::dokime::client::CycleOrder &order, client::CyclesListOptions options = {}) {
    detail::packages::dokime::host::CyclesListRequest request;
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
    detail::packages::dokime::host::CyclesListResponse response;
    const auto status = detail::packages::dokime::host::Cycles::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cycles.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::CyclesWatchResponse> watch(const ::dokime::client::CycleSelection &selection,
                                                                      const ::dokime::client::CycleFilter &filter,
                                                                      const ::dokime::client::CycleOrder &order,
                                                                      client::CyclesWatchOptions options = {}) {
    detail::packages::dokime::host::CyclesWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::CyclesWatchResponse>>(
        detail::packages::dokime::host::Cycles::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::CyclesWatchResponse> {
              detail::packages::dokime::host::CyclesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("cycles.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::CyclesCreateResponse create(const std::vector<::dokime::Origin> &origins, const ::dokime::Metadata &metadata,
                                                const bool &continue_automatically, const std::string &source_cycle,
                                                client::CyclesCreateOptions options = {}) {
    detail::packages::dokime::host::CyclesCreateRequest request;
    request.mutable_origins()->Assign(origins.begin(), origins.end());
    *request.mutable_metadata() = metadata;
    request.set_continue_automatically(continue_automatically);
    request.set_source_cycle(source_cycle);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CyclesCreateResponse response;
    const auto status = detail::packages::dokime::host::Cycles::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cycles.create", status, *context);
    }
    return response;
  }
  ::dokime::client::CyclesSealResponse seal(const std::string &name, client::CyclesSealOptions options = {}) {
    detail::packages::dokime::host::CyclesSealRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::CyclesSealResponse response;
    const auto status = detail::packages::dokime::host::Cycles::NewStub(channel)->Seal(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("cycles.seal", status, *context);
    }
    return response;
  }
};
struct DocumentsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit DocumentsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::DocumentsGetResponse get(const ::dokime::client::DocumentScope &scope, const std::string &document_id,
                                             const bool &structure_only, const std::vector<std::string> &card_ids,
                                             const std::string &page_token, const std::uint64_t &max_bytes, const std::uint32_t &max_frames,
                                             client::DocumentsGetOptions options = {}) {
    detail::packages::dokime::host::DocumentsGetRequest request;
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
    detail::packages::dokime::host::DocumentsGetResponse response;
    const auto status = detail::packages::dokime::host::Documents::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("documents.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::DocumentsWatchResponse>
  watch(const ::dokime::client::DocumentScope &scope, const std::string &document_id, const bool &structure_only,
        const std::vector<std::string> &card_ids, const std::string &page_token, const std::uint64_t &max_bytes,
        const std::uint32_t &max_frames, client::DocumentsWatchOptions options = {}) {
    detail::packages::dokime::host::DocumentsWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::DocumentsWatchResponse>>(
        detail::packages::dokime::host::Documents::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::DocumentsWatchResponse> {
              detail::packages::dokime::host::DocumentsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("documents.watch", status, *context);
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
  explicit EvidenceResource(std::shared_ptr<grpc::Channel> connection)
      : channel(connection), health_events(connection), notes(connection), records(connection), samples(connection),
        traceability(connection) {}
  ::dokime::EvidenceStatisticsResponse statistics(const std::string &stream, const std::vector<std::uint32_t> &versions,
                                                  const ::dokime::model::Window &between, const std::vector<::dokime::Output> &outputs,
                                                  const std::string &field, const std::string &step,
                                                  client::EvidenceStatisticsOptions options = {}) {
    detail::packages::dokime::host::EvidenceStatisticsRequest request;
    if (options.scope) {
      *request.mutable_scope() = *options.scope;
    }
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
    detail::packages::dokime::host::EvidenceStatisticsResponse response;
    const auto status = detail::packages::dokime::host::Evidence::NewStub(channel)->Statistics(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.statistics", status, *context);
    }
    return response;
  }
  ::dokime::client::EvidenceArchiveResponse archive(const ::dokime::client::EvidenceSubject &subject,
                                                    client::EvidenceArchiveOptions options = {}) {
    detail::packages::dokime::host::EvidenceArchiveRequest request;
    *request.mutable_subject() = subject;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::EvidenceArchiveResponse response;
    const auto status = detail::packages::dokime::host::Evidence::NewStub(channel)->Archive(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("evidence.archive", status, *context);
    }
    return response;
  }
};
struct ExecutionsResource {
  std::shared_ptr<grpc::Channel> channel;
  ExecutionsCommandsResource commands;
  ExecutionsProgressResource progress;
  ExecutionsSchemasResource schemas;
  explicit ExecutionsResource(std::shared_ptr<grpc::Channel> connection)
      : channel(connection), commands(connection), progress(connection), schemas(connection) {}
  ::dokime::client::ExecutionsGetResponse get(const std::string &name, client::ExecutionsGetOptions options = {}) {
    detail::packages::dokime::host::ExecutionsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsGetResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.get", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsSetResponse set(const std::string &name, const ::dokime::client::ExecutionWritable &value,
                                              const std::string &expected_revision, client::ExecutionsSetOptions options = {}) {
    detail::packages::dokime::host::ExecutionsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsSetResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.set", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsListResponse list(const std::string &parent, const ::dokime::client::ExecutionSelection &selection,
                                                const ::dokime::client::ExecutionFilter &filter,
                                                const ::dokime::client::ExecutionOrder &order, client::ExecutionsListOptions options = {}) {
    detail::packages::dokime::host::ExecutionsListRequest request;
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
    detail::packages::dokime::host::ExecutionsListResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::ExecutionsWatchResponse>
  watch(const std::string &parent, const ::dokime::client::ExecutionSelection &selection, const ::dokime::client::ExecutionFilter &filter,
        const ::dokime::client::ExecutionOrder &order, client::ExecutionsWatchOptions options = {}) {
    detail::packages::dokime::host::ExecutionsWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::ExecutionsWatchResponse>>(
        detail::packages::dokime::host::Executions::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::ExecutionsWatchResponse> {
              detail::packages::dokime::host::ExecutionsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("executions.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::ExecutionsCreateResponse create(const std::string &parent,
                                                    const std::vector<::dokime::client::ExecutionBinding> &executions,
                                                    client::ExecutionsCreateOptions options = {}) {
    detail::packages::dokime::host::ExecutionsCreateRequest request;
    request.set_parent(parent);
    request.mutable_executions()->Assign(executions.begin(), executions.end());
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsCreateResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.create", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsStartResponse start(const std::string &name, const ::dokime::client::ExecutionMode &mode,
                                                  client::ExecutionsStartOptions options = {}) {
    detail::packages::dokime::host::ExecutionsStartRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_mode(mode);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsStartResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Start(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.start", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsStopResponse stop(const std::string &name, client::ExecutionsStopOptions options = {}) {
    detail::packages::dokime::host::ExecutionsStopRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsStopResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Stop(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.stop", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsCancelResponse cancel(const std::string &name, client::ExecutionsCancelOptions options = {}) {
    detail::packages::dokime::host::ExecutionsCancelRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsCancelResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Cancel(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.cancel", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsSealResponse seal(const std::string &name, client::ExecutionsSealOptions options = {}) {
    detail::packages::dokime::host::ExecutionsSealRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsSealResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Seal(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.seal", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsDeleteResponse delete_(const std::string &name, client::ExecutionsDeleteOptions options = {}) {
    detail::packages::dokime::host::ExecutionsDeleteRequest request;
    request.set_name(name);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsDeleteResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.delete", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsParametersResponse parameters(const std::string &name,
                                                            const ::dokime::client::ExecutionParameterSet &set_value,
                                                            client::ExecutionsParametersOptions options = {}) {
    detail::packages::dokime::host::ExecutionsParametersRequest request;
    request.set_name(name);
    request.set_set(set_value);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsParametersResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Parameters(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.parameters", status, *context);
    }
    return response;
  }
  ::dokime::client::ExecutionsStepsResponse steps(const std::string &name, client::ExecutionsStepsOptions options = {}) {
    detail::packages::dokime::host::ExecutionsStepsRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ExecutionsStepsResponse response;
    const auto status = detail::packages::dokime::host::Executions::NewStub(channel)->Steps(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("executions.steps", status, *context);
    }
    return response;
  }
};
struct FilesResource {
  std::shared_ptr<grpc::Channel> channel;
  FilesUploadsResource uploads;
  explicit FilesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection), uploads(connection) {}
  ::dokime::FilesGetResponse get(const ::harmos::FileReference &file, client::FilesGetOptions options = {}) {
    detail::packages::dokime::host::FilesGetRequest request;
    *request.mutable_file() = file;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::FilesGetResponse response;
    const auto status = detail::packages::dokime::host::Files::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.get", status, *context);
    }
    return response;
  }
  ::dokime::FilesReadResponse read(const ::harmos::FileReference &file, const std::uint64_t &offset, const std::uint64_t &max_bytes,
                                   client::FilesReadOptions options = {}) {
    detail::packages::dokime::host::FilesReadRequest request;
    *request.mutable_file() = file;
    request.set_offset(offset);
    request.set_max_bytes(max_bytes);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::FilesReadResponse response;
    const auto status = detail::packages::dokime::host::Files::NewStub(channel)->Read(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.read", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::FilesWatchResponse> watch(const std::uint64_t &offset, client::FilesWatchOptions options = {}) {
    detail::packages::dokime::host::FilesWatchRequest request;
    if (options.file) {
      *request.mutable_file() = *options.file;
    }
    if (options.upload) {
      *request.mutable_upload() = *options.upload;
    }
    request.set_offset(offset);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::FilesWatchResponse>>(
        detail::packages::dokime::host::Files::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::FilesWatchResponse> {
              detail::packages::dokime::host::FilesWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("files.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::harmos::Receipt delete_(const ::harmos::FileReference &file, client::FilesDeleteOptions options = {}) {
    detail::packages::dokime::host::FilesDeleteRequest request;
    *request.mutable_file() = file;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::FilesDeleteResponse response;
    const auto status = detail::packages::dokime::host::Files::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("files.delete", status, *context);
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read files.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
};
struct InvocationsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit InvocationsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  std::string create(const std::string &operation_value, const std::string &fingerprint, const std::string &payload,
                     client::InvocationsCreateOptions options = {}) {
    detail::packages::harmos::host::InvocationsCreateRequest request;
    request.set_operation(operation_value);
    request.set_fingerprint(fingerprint);
    request.set_payload(payload);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::InvocationsCreateResponse response;
    const auto status = detail::packages::dokime::host::Invocations::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("invocations.create", status, *context);
    }
    return response.payload();
  }
};
struct MediaResource {
  MediaCapturesResource captures;
  explicit MediaResource(std::shared_ptr<grpc::Channel> connection) : captures(connection) {}
};
struct OperationsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit OperationsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  std::vector<::harmos::DiscoveredOperation> list(client::OperationsListOptions options = {}) {
    detail::packages::harmos::host::OperationsListRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::OperationsListResponse response;
    const auto status = detail::packages::dokime::host::Operations::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("operations.list", status, *context);
    }
    return std::vector<::harmos::DiscoveredOperation>(response.operations().begin(), response.operations().end());
  }
  ::harmos::DiscoveredOperation get(const std::string &id, client::OperationsGetOptions options = {}) {
    detail::packages::harmos::host::OperationsGetRequest request;
    request.set_id(id);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::harmos::host::OperationsGetResponse response;
    const auto status = detail::packages::dokime::host::Operations::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("operations.get", status, *context);
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
  explicit PlotsResource(std::shared_ptr<grpc::Channel> connection) : settings(connection), snapshots(connection) {}
};
struct PluginsResource {
  std::shared_ptr<grpc::Channel> channel;
  PluginsSecretsResource secrets;
  PluginsSettingsResource settings;
  explicit PluginsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection), secrets(connection), settings(connection) {}
  ::dokime::client::host::PluginsGetResponse get(const std::string &name, client::PluginsGetOptions options = {}) {
    detail::packages::dokime::host::PluginsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsGetResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.get", status, *context);
    }
    return response;
  }
  ::dokime::client::host::PluginsListResponse list(client::PluginsListOptions options = {}) {
    detail::packages::dokime::host::PluginsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsListResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::host::PluginsWatchResponse> watch(client::PluginsWatchOptions options = {}) {
    detail::packages::dokime::host::PluginsWatchRequest request;
    if (options.name) {
      request.set_name(*options.name);
    }
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::PluginsWatchResponse>>(
        detail::packages::dokime::host::Plugins::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::host::PluginsWatchResponse> {
              detail::packages::dokime::host::PluginsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("plugins.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::host::PluginsCreateResponse create(const std::string &name, const ::dokime::client::host::PluginPlacement &placement,
                                                       const ::dokime::client::host::PluginValue &value,
                                                       client::PluginsCreateOptions options = {}) {
    detail::packages::dokime::host::PluginsCreateRequest request;
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    request.set_name(name);
    *request.mutable_placement() = placement;
    *request.mutable_value() = value;
    if (options.origin) {
      request.set_origin(*options.origin);
    }
    if (options.sdk_version) {
      request.set_sdk_version(*options.sdk_version);
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsCreateResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.create", status, *context);
    }
    return response;
  }
  ::dokime::client::host::PluginsSetResponse set(const std::string &name, const ::dokime::client::host::PluginValue &value,
                                                 const std::string &expected_revision, client::PluginsSetOptions options = {}) {
    detail::packages::dokime::host::PluginsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsSetResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.set", status, *context);
    }
    return response;
  }
  ::harmos::Receipt delete_(const std::string &name, const std::string &expected_revision, client::PluginsDeleteOptions options = {}) {
    detail::packages::dokime::host::PluginsDeleteRequest request;
    request.set_name(name);
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsDeleteResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.delete", status, *context);
    }
    if (!response.has_receipt()) {
      throw std::runtime_error("Read plugins.delete: Missing Payload");
    }
    return std::move(*response.mutable_receipt());
  }
  ::dokime::client::host::PluginEntry restart(const std::string &name, const std::string &expected_revision,
                                              client::PluginsRestartOptions options = {}) {
    detail::packages::dokime::host::PluginsRestartRequest request;
    request.set_name(name);
    request.set_expected_revision(expected_revision);
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsRestartResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Restart(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.restart", status, *context);
    }
    if (!response.has_entry()) {
      throw std::runtime_error("Read plugins.restart: Missing Payload");
    }
    return std::move(*response.mutable_entry());
  }
  ::dokime::client::host::PluginsReplaceResponse replace(const std::string &name, const ::dokime::client::host::PluginPlacement &placement,
                                                         const std::string &expected_revision, client::PluginsReplaceOptions options = {}) {
    detail::packages::dokime::host::PluginsReplaceRequest request;
    request.set_name(name);
    *request.mutable_placement() = placement;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PluginsReplaceResponse response;
    const auto status = detail::packages::dokime::host::Plugins::NewStub(channel)->Replace(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("plugins.replace", status, *context);
    }
    return response;
  }
};
struct PromptDefinitionsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit PromptDefinitionsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::PromptDefinitionsGetResponse get(const std::string &name, const std::string &parent,
                                                     const std::uint32_t &schema_version,
                                                     client::PromptDefinitionsGetOptions options = {}) {
    detail::packages::dokime::host::PromptDefinitionsGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PromptDefinitionsGetResponse response;
    const auto status = detail::packages::dokime::host::PromptDefinitions::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("prompt_definitions.get", status, *context);
    }
    return response;
  }
  ::dokime::client::PromptDefinitionsListResponse list(const std::string &parent, client::PromptDefinitionsListOptions options = {}) {
    detail::packages::dokime::host::PromptDefinitionsListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PromptDefinitionsListResponse response;
    const auto status = detail::packages::dokime::host::PromptDefinitions::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("prompt_definitions.list", status, *context);
    }
    return response;
  }
};
struct PromptsResource {
  std::shared_ptr<grpc::Channel> channel;
  PromptsAnswersResource answers;
  explicit PromptsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection), answers(connection) {}
  ::dokime::client::PromptsGetResponse get(const std::string &name, client::PromptsGetOptions options = {}) {
    detail::packages::dokime::host::PromptsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::PromptsGetResponse response;
    const auto status = detail::packages::dokime::host::Prompts::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("prompts.get", status, *context);
    }
    return response;
  }
  ::dokime::client::PromptsListResponse list(const std::string &parent, const bool &pending_only, client::PromptsListOptions options = {}) {
    detail::packages::dokime::host::PromptsListRequest request;
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
    detail::packages::dokime::host::PromptsListResponse response;
    const auto status = detail::packages::dokime::host::Prompts::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("prompts.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::PromptsWatchResponse> watch(const std::string &parent, const bool &pending_only,
                                                                       client::PromptsWatchOptions options = {}) {
    detail::packages::dokime::host::PromptsWatchRequest request;
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
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::PromptsWatchResponse>>(
        detail::packages::dokime::host::Prompts::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::PromptsWatchResponse> {
              detail::packages::dokime::host::PromptsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("prompts.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct RecordSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit RecordSchemasResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::RecordSchemasGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                                 client::RecordSchemasGetOptions options = {}) {
    detail::packages::dokime::host::RecordSchemasGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::RecordSchemasGetResponse response;
    const auto status = detail::packages::dokime::host::RecordSchemas::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("record_schemas.get", status, *context);
    }
    return response;
  }
  ::dokime::client::RecordSchemasListResponse list(const std::string &parent, client::RecordSchemasListOptions options = {}) {
    detail::packages::dokime::host::RecordSchemasListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::RecordSchemasListResponse response;
    const auto status = detail::packages::dokime::host::RecordSchemas::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("record_schemas.list", status, *context);
    }
    return response;
  }
};
struct ReportsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit ReportsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::ReportsGetResponse get(const ::dokime::client::ReportSubject &subject, const std::string &name,
                                           client::ReportsGetOptions options = {}) {
    detail::packages::dokime::host::ReportsGetRequest request;
    *request.mutable_subject() = subject;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ReportsGetResponse response;
    const auto status = detail::packages::dokime::host::Reports::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("reports.get", status, *context);
    }
    return response;
  }
  ::dokime::client::ReportsCreateResponse create(const ::dokime::client::ReportSubject &subject, const std::string &document_id,
                                                 client::ReportsCreateOptions options = {}) {
    detail::packages::dokime::host::ReportsCreateRequest request;
    *request.mutable_subject() = subject;
    request.set_document_id(document_id);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ReportsCreateResponse response;
    const auto status = detail::packages::dokime::host::Reports::NewStub(channel)->Create(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("reports.create", status, *context);
    }
    return response;
  }
  ::dokime::client::ReportsPublishResponse publish(const std::string &name, const ::dokime::client::ReportSubject &subject,
                                                   const std::string &expected_revision, client::ReportsPublishOptions options = {}) {
    detail::packages::dokime::host::ReportsPublishRequest request;
    request.set_name(name);
    *request.mutable_subject() = subject;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::ReportsPublishResponse response;
    const auto status = detail::packages::dokime::host::Reports::NewStub(channel)->Publish(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("reports.publish", status, *context);
    }
    return response;
  }
};
struct SecretsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SecretsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::SecretsGetResponse get(const std::string &name, client::SecretsGetOptions options = {}) {
    detail::packages::dokime::host::SecretsGetRequest request;
    request.set_name(name);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsGetResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.get", status, *context);
    }
    return response;
  }
  ::dokime::client::SecretsSetResponse set(const std::string &name, const ::dokime::SecretWritable &value,
                                           const std::string &expected_revision, client::SecretsSetOptions options = {}) {
    detail::packages::dokime::host::SecretsSetRequest request;
    request.set_name(name);
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsSetResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.set", status, *context);
    }
    return response;
  }
  ::dokime::client::host::SecretsListResponse list(client::SecretsListOptions options = {}) {
    detail::packages::dokime::host::SecretsListRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsListResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.list", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::SecretsWatchResponse> watch(client::SecretsWatchOptions options = {}) {
    detail::packages::dokime::host::SecretsWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::SecretsWatchResponse>>(
        detail::packages::dokime::host::Secrets::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::SecretsWatchResponse> {
              detail::packages::dokime::host::SecretsWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("secrets.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
  ::dokime::client::SecretsDeleteResponse delete_(const std::vector<std::string> &names, const std::string &expected_revision,
                                                  client::SecretsDeleteOptions options = {}) {
    detail::packages::dokime::host::SecretsDeleteRequest request;
    request.mutable_names()->Assign(names.begin(), names.end());
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsDeleteResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->Delete(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.delete", status, *context);
    }
    return response;
  }
  ::dokime::client::SecretsExportResponse export_(const std::vector<std::string> &names, const std::uint64_t &generation,
                                                  client::SecretsExportOptions options = {}) {
    detail::packages::dokime::host::SecretsExportRequest request;
    request.mutable_names()->Assign(names.begin(), names.end());
    request.set_generation(generation);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsExportResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->Export(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.export", status, *context);
    }
    return response;
  }
  ::dokime::client::SecretsImportResponse import_(const std::string &data, const std::vector<::dokime::client::SecretImportChoice> &choices,
                                                  const std::uint64_t &generation, const bool &validate_only,
                                                  client::SecretsImportOptions options = {}) {
    detail::packages::dokime::host::SecretsImportRequest request;
    request.set_data(data);
    request.mutable_choices()->Assign(choices.begin(), choices.end());
    request.set_generation(generation);
    request.set_validate_only(validate_only);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SecretsImportResponse response;
    const auto status = detail::packages::dokime::host::Secrets::NewStub(channel)->Import(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("secrets.import", status, *context);
    }
    return response;
  }
};
struct SettingsResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SettingsResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::host::SettingsGetResponse get(client::SettingsGetOptions options = {}) {
    detail::packages::dokime::host::SettingsGetRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SettingsGetResponse response;
    const auto status = detail::packages::dokime::host::Settings::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("settings.get", status, *context);
    }
    return response;
  }
  ::dokime::client::host::SettingsSetResponse set(const ::dokime::client::ApplicationSettings &value, const std::string &expected_revision,
                                                  client::SettingsSetOptions options = {}) {
    detail::packages::dokime::host::SettingsSetRequest request;
    *request.mutable_value() = value;
    request.set_expected_revision(expected_revision);
    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SettingsSetResponse response;
    const auto status = detail::packages::dokime::host::Settings::NewStub(channel)->Set(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("settings.set", status, *context);
    }
    return response;
  }
  ::dokime::client::SettingsSchemaResponse schema(client::SettingsSchemaOptions options = {}) {
    detail::packages::dokime::host::SettingsSchemaRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SettingsSchemaResponse response;
    const auto status = detail::packages::dokime::host::Settings::NewStub(channel)->Schema(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("settings.schema", status, *context);
    }
    return response;
  }
};
struct StatusResource {
  std::shared_ptr<grpc::Channel> channel;
  StatusDiagnosticsResource diagnostics;
  StatusLogsResource logs;
  StatusTraceEventsResource trace_events;
  explicit StatusResource(std::shared_ptr<grpc::Channel> connection)
      : channel(connection), diagnostics(connection), logs(connection), trace_events(connection) {}
  ::dokime::client::host::StatusGetResponse get(client::StatusGetOptions options = {}) {
    detail::packages::harmos::host::StatusGetRequest request;
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::StatusGetResponse response;
    const auto status = detail::packages::dokime::host::Status::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("status.get", status, *context);
    }
    return response;
  }
  ::harmos::ClientStream<::dokime::client::host::StatusWatchResponse> watch(client::StatusWatchOptions options = {}) {
    detail::packages::harmos::host::StatusWatchRequest request;
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    auto reader = std::shared_ptr<grpc::ClientReader<detail::packages::dokime::host::StatusWatchResponse>>(
        detail::packages::dokime::host::Status::NewStub(channel)->Watch(context.get(), request));
    return {context, [reader, context]() -> std::optional<::dokime::client::host::StatusWatchResponse> {
              detail::packages::dokime::host::StatusWatchResponse response;
              if (!reader->Read(&response)) {
                const auto status = reader->Finish();
                if (!status.ok()) {
                  throw ::dokime::detail::rpc_error("status.watch", status, *context);
                }
                return {};
              }
              return response;
            }};
  }
};
struct StorageResource {
  StorageSourcesResource sources;
  explicit StorageResource(std::shared_ptr<grpc::Channel> connection) : sources(connection) {}
};
struct StreamSchemasResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit StreamSchemasResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::StreamSchemasGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                                 client::StreamSchemasGetOptions options = {}) {
    detail::packages::dokime::host::StreamSchemasGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::StreamSchemasGetResponse response;
    const auto status = detail::packages::dokime::host::StreamSchemas::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("stream_schemas.get", status, *context);
    }
    return response;
  }
  ::dokime::client::StreamSchemasListResponse list(const std::string &parent, client::StreamSchemasListOptions options = {}) {
    detail::packages::dokime::host::StreamSchemasListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::StreamSchemasListResponse response;
    const auto status = detail::packages::dokime::host::StreamSchemas::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("stream_schemas.list", status, *context);
    }
    return response;
  }
};
struct SuitesResource {
  std::shared_ptr<grpc::Channel> channel;
  explicit SuitesResource(std::shared_ptr<grpc::Channel> connection) : channel(connection) {}
  ::dokime::client::SuitesGetResponse get(const std::string &name, const std::string &parent, const std::uint32_t &schema_version,
                                          client::SuitesGetOptions options = {}) {
    detail::packages::dokime::host::SuitesGetRequest request;
    request.set_name(name);
    request.set_parent(parent);
    request.set_schema_version(schema_version);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SuitesGetResponse response;
    const auto status = detail::packages::dokime::host::Suites::NewStub(channel)->Get(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("suites.get", status, *context);
    }
    return response;
  }
  ::dokime::client::SuitesListResponse list(const std::string &parent, client::SuitesListOptions options = {}) {
    detail::packages::dokime::host::SuitesListRequest request;
    request.set_parent(parent);
    if (options.at) {
      *request.mutable_at() = *options.at;
    }
    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();
    detail::packages::dokime::host::SuitesListResponse response;
    const auto status = detail::packages::dokime::host::Suites::NewStub(channel)->List(context.get(), request, &response);
    if (!status.ok()) {
      throw ::dokime::detail::rpc_error("suites.list", status, *context);
    }
    return response;
  }
};
namespace detail {
struct Transport {
  AccessResource access;
  AggregatesResource aggregates;
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
  SecretsResource secrets;
  SettingsResource settings;
  StatusResource status;
  StorageResource storage;
  StreamSchemasResource stream_schemas;
  SuitesResource suites;
  explicit Transport(std::shared_ptr<grpc::Channel> connection)
      : access(connection), aggregates(connection), backend(connection), cases(connection), content(connection), cycles(connection),
        documents(connection), evidence(connection), executions(connection), files(connection), invocations(connection), media(connection),
        operations(connection), plots(connection), plugins(connection), prompt_definitions(connection), prompts(connection),
        record_schemas(connection), reports(connection), secrets(connection), settings(connection), status(connection), storage(connection),
        stream_schemas(connection), suites(connection) {}
  static Transport connect(std::string address, const ::harmos::StorageTransferSettings &transfer = {},
                           std::optional<int> message_bytes = {}) {
    const auto maximum = ::harmos::transfer_message_bytes(transfer, message_bytes);
    grpc::ChannelArguments arguments;
    arguments.SetMaxSendMessageSize(maximum);
    arguments.SetMaxReceiveMessageSize(maximum);
    if (address.starts_with("http://")) {
      address.erase(0, 7);
    }
    return Transport(grpc::CreateCustomChannel(address, grpc::InsecureChannelCredentials(), arguments));
  }
};
} // namespace detail
} // namespace dokime
