// @generated from protocol descriptors; do not edit.
#pragma once
namespace dokime::templates {
enum class HostOperation {
  BackendEnvironment,
  ContentArtifactsCreate,
  DocumentsAppendParts,
  EffectsCreate,
  EffectsGet,
  EffectsSet,
  EvidenceLogsAppend,
  EvidenceRecordsList,
  EvidenceRecordsAppend,
  EvidenceTraceabilityGet,
  EvidenceTraceabilitySet,
  EvidenceStatistics,
  ExecutionsProgressGet,
  ExecutionsProgressSet,
  FilesCapturesStart,
  FilesCapturesStop,
  FilesUploadsCreate,
  FilesUploadsGet,
  FilesUploadsResume,
  FilesUploadsAppend,
  FilesUploadsPublish,
  FilesUploadsDelete,
  FilesGet,
  FilesRead,
  FilesWatch,
  FilesDelete,
  FilesCapture,
  PromptsCreate,
  SecretsGet,
  InvocationsGet,
  InvocationsCancel,
  InvocationsWatch,
  OperationsList,
  OperationsGet,
  PluginsSettingsList,
  TimeGet,
  TimeSleep,
};
struct HostBackendEnvironmentOptions {
  std::optional<::harmos::Position> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostContentArtifactsCreateOptions {
  std::optional<::dokime::ArtifactScope> scope;
  std::optional<std::string> request_id;
  std::optional<::dokime::Metadata> metadata;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostDocumentsAppendPartsOptions {
  std::optional<::dokime::model::Section> subsection;
  std::optional<::dokime::Card> card;
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEffectsCreateOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEffectsGetOptions {
  std::optional<::harmos::Position> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEffectsSetOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceLogsAppendOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceRecordsListOptions {
  std::optional<::dokime::RecordScope> scope;
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::PageRequest> page;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceRecordsAppendOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceTraceabilityGetOptions {
  std::optional<::dokime::EvidenceScope> scope;
  std::optional<::harmos::PageRequest> page;
  std::optional<::harmos::ReadCut> at;
  std::optional<std::string> entry;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceTraceabilitySetOptions {
  std::optional<std::string> name;
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostEvidenceStatisticsOptions {
  std::optional<::dokime::EvidenceScope> scope;
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostExecutionsProgressGetOptions {
  std::optional<std::string> name;
  std::optional<::harmos::Position> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostExecutionsProgressSetOptions {
  std::optional<std::string> name;
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesCapturesStartOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesCapturesStopOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsCreateOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsResumeOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsAppendOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsPublishOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesUploadsDeleteOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesReadOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesWatchOptions {
  std::optional<::harmos::FileReference> file;
  std::optional<::harmos::Upload> upload;
  std::optional<::harmos::ReadCut> at;
};
class HostFilesWatch {
public:
  explicit HostFilesWatch(std::function<std::optional<::dokime::FilesWatchResponse>()> next_value) : next_(std::move(next_value)) {}
  std::optional<::dokime::FilesWatchResponse> next() {
    return next_();
  }

private:
  std::function<std::optional<::dokime::FilesWatchResponse>()> next_;
};
struct HostFilesDeleteOptions {
  std::optional<std::string> request_id;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostFilesCaptureOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostPromptsCreateOptions {
  std::optional<::dokime::Prompt> definition;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostSecretsGetOptions {
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostInvocationsGetOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostInvocationsCancelOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostInvocationsWatchOptions {};
class HostInvocationsWatch {
public:
  explicit HostInvocationsWatch(std::function<std::optional<::harmos::InstanceState>()> next_value) : next_(std::move(next_value)) {}
  std::optional<::harmos::InstanceState> next() {
    return next_();
  }

private:
  std::function<std::optional<::harmos::InstanceState>()> next_;
};
struct HostOperationsListOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostOperationsGetOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostPluginsSettingsListOptions {
  std::optional<std::string> name;
  std::optional<::harmos::ReadCut> at;
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostTimeGetOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostTimeSleepOptions {
  std::optional<::harmos::OperationPolicy> policy;
};
struct HostTimeResource {
  Context context;
  HostTimeResource(Context context_value) : context(context_value) {}
  ::google::protobuf::Timestamp get(HostTimeGetOptions options = {});
  ::harmos::host::v1::TimeSleepResponse sleep(const ::google::protobuf::Duration &duration, HostTimeSleepOptions options = {});
};
struct HostSecretsResource {
  Context context;
  HostSecretsResource(Context context_value) : context(context_value) {}
  ::dokime::SecretsGetResponse get(const std::string &name, HostSecretsGetOptions options = {});
};
struct HostPromptsResource {
  Context context;
  HostPromptsResource(Context context_value) : context(context_value) {}
  ::dokime::plugin::PromptsCreateResponse create(const std::string &prompt, const std::string &prompt_type,
                                                 const std::map<std::string, ::dokime::Value> &values,
                                                 HostPromptsCreateOptions options = {});
};
struct HostPluginsSettingsResource {
  Context context;
  HostPluginsSettingsResource(Context context_value) : context(context_value) {}
  ::harmos::host::v1::plugins::SettingsListResponse list(HostPluginsSettingsListOptions options = {});
};
struct HostPluginsResource {
  Context context;
  HostPluginsSettingsResource settings;
  HostPluginsResource(Context context_value) : context(context_value), settings(context_value) {}
};
struct HostOperationsResource {
  Context context;
  HostOperationsResource(Context context_value) : context(context_value) {}
  std::vector<::harmos::DiscoveredOperation> list(HostOperationsListOptions options = {});
  ::harmos::DiscoveredOperation get(const std::string &id, HostOperationsGetOptions options = {});
};
struct HostInvocationsResource {
  Context context;
  HostInvocationsResource(Context context_value) : context(context_value) {}
  ::harmos::InstanceState get(const std::uint64_t &instance, HostInvocationsGetOptions options = {});
  ::harmos::InstanceState cancel(const std::uint64_t &instance, const std::string &reason, HostInvocationsCancelOptions options = {});
  HostInvocationsWatch watch(const std::uint64_t &instance, std::size_t max_item_bytes, HostInvocationsWatchOptions options = {});
};
struct HostFilesUploadsResource {
  Context context;
  HostFilesUploadsResource(Context context_value) : context(context_value) {}
  ::dokime::UploadsCreateResponse create(const ::harmos::FileMetadata &metadata, HostFilesUploadsCreateOptions options = {});
  ::dokime::UploadsGetResponse get(const ::harmos::Upload &upload, HostFilesUploadsGetOptions options = {});
  ::dokime::UploadsResumeResponse resume(const ::harmos::Upload &upload, HostFilesUploadsResumeOptions options = {});
  ::dokime::UploadsAppendResponse append(const ::harmos::Upload &upload, const std::uint64_t &offset, const std::string &data,
                                         HostFilesUploadsAppendOptions options = {});
  ::dokime::UploadsPublishResponse publish(const ::harmos::Upload &upload, const std::uint64_t &length, const std::string &sha256,
                                           HostFilesUploadsPublishOptions options = {});
  ::dokime::UploadsDeleteResponse delete_(const ::harmos::Upload &upload, HostFilesUploadsDeleteOptions options = {});
};
struct HostFilesCapturesResource {
  Context context;
  HostFilesCapturesResource(Context context_value) : context(context_value) {}
  std::uint64_t start(const ::dokime::FileSource &source, const std::string &output, const ::dokime::plugin::CaptureMode &mode,
                      HostFilesCapturesStartOptions options = {});
  ::dokime::plugin::host::files::CapturesStopResponse stop(const std::uint64_t &capture_id, HostFilesCapturesStopOptions options = {});
};
struct HostFilesResource {
  Context context;
  HostFilesCapturesResource captures;
  HostFilesUploadsResource uploads;
  HostFilesResource(Context context_value) : context(context_value), captures(context_value), uploads(context_value) {}
  ::dokime::FilesGetResponse get(const ::harmos::FileReference &file, HostFilesGetOptions options = {});
  ::dokime::FilesReadResponse read(const ::harmos::FileReference &file, const std::uint64_t &offset, const std::uint64_t &max_bytes,
                                   HostFilesReadOptions options = {});
  HostFilesWatch watch(const std::uint64_t &offset, std::size_t max_item_bytes, HostFilesWatchOptions options = {});
  ::harmos::Receipt delete_(const ::harmos::FileReference &file, HostFilesDeleteOptions options = {});
  ::dokime::plugin::FilesCaptureResponse capture(const ::dokime::FileSource &source, const std::string &output,
                                                 HostFilesCaptureOptions options = {});
};
struct HostExecutionsProgressResource {
  Context context;
  HostExecutionsProgressResource(Context context_value) : context(context_value) {}
  ::dokime::ProgressGetResponse get(HostExecutionsProgressGetOptions options = {});
  ::dokime::plugin::ProgressSetResponse set(const ::dokime::ExecutionProgressValue &value, const std::string &expected_revision,
                                            HostExecutionsProgressSetOptions options = {});
};
struct HostExecutionsResource {
  Context context;
  HostExecutionsProgressResource progress;
  HostExecutionsResource(Context context_value) : context(context_value), progress(context_value) {}
};
struct HostEvidenceTraceabilityResource {
  Context context;
  HostEvidenceTraceabilityResource(Context context_value) : context(context_value) {}
  ::dokime::TraceabilityGetResponse get(HostEvidenceTraceabilityGetOptions options = {});
  ::dokime::plugin::TraceabilitySetResponse set(const ::dokime::TraceabilityEntry &value, const std::string &expected_revision,
                                                const std::string &entry, HostEvidenceTraceabilitySetOptions options = {});
};
struct HostEvidenceRecordsResource {
  Context context;
  HostEvidenceRecordsResource(Context context_value) : context(context_value) {}
  ::dokime::RecordsListResponse list(const std::string &record_type, const std::vector<std::uint32_t> &versions,
                                     const ::dokime::model::Window &between, const std::string &step,
                                     const ::dokime::CapturedRecordSelection &selection, HostEvidenceRecordsListOptions options = {});
  ::dokime::plugin::RecordsAppendResponse append(const std::string &execution, const std::string &kind, const std::string &payload,
                                                 const ::google::protobuf::Timestamp &source_at,
                                                 HostEvidenceRecordsAppendOptions options = {});
};
struct HostEvidenceLogsResource {
  Context context;
  HostEvidenceLogsResource(Context context_value) : context(context_value) {}
  ::harmos::Receipt append(const std::string &execution, const ::dokime::Severity &severity, const std::string &message,
                           const ::google::protobuf::Timestamp &at, HostEvidenceLogsAppendOptions options = {});
};
struct HostEvidenceResource {
  Context context;
  HostEvidenceLogsResource logs;
  HostEvidenceRecordsResource records;
  HostEvidenceTraceabilityResource traceability;
  HostEvidenceResource(Context context_value)
      : context(context_value), logs(context_value), records(context_value), traceability(context_value) {}
  ::dokime::EvidenceStatisticsResponse statistics(const std::string &stream, const std::vector<std::uint32_t> &versions,
                                                  const ::dokime::model::Window &between, const std::vector<::dokime::Output> &outputs,
                                                  const std::string &field, const std::string &step,
                                                  HostEvidenceStatisticsOptions options = {});
};
struct HostEffectsResource {
  Context context;
  HostEffectsResource(Context context_value) : context(context_value) {}
  ::dokime::plugin::EffectsCreateResponse create(const std::string &resource, const std::string &inverse_operation,
                                                 const std::string &inverse, const std::string &description,
                                                 HostEffectsCreateOptions options = {});
  ::dokime::plugin::EffectsGetResponse get(const std::string &name, HostEffectsGetOptions options = {});
  ::dokime::plugin::EffectsSetResponse set(const std::string &name, const ::dokime::plugin::EffectWritable &value,
                                           const std::string &expected_revision, HostEffectsSetOptions options = {});
};
struct HostDocumentsResource {
  Context context;
  HostDocumentsResource(Context context_value) : context(context_value) {}
  ::harmos::Receipt append_parts(const std::string &execution, const std::string &document, const std::string &section,
                                 HostDocumentsAppendPartsOptions options = {});
};
struct HostContentArtifactsResource {
  Context context;
  HostContentArtifactsResource(Context context_value) : context(context_value) {}
  ::dokime::ArtifactsCreateResponse create(const ::harmos::FileReference &file, const std::string &filename,
                                           const std::string &content_type, HostContentArtifactsCreateOptions options = {});
};
struct HostContentResource {
  Context context;
  HostContentArtifactsResource artifacts;
  HostContentResource(Context context_value) : context(context_value), artifacts(context_value) {}
};
struct HostBackendResource {
  Context context;
  HostBackendResource(Context context_value) : context(context_value) {}
  ::dokime::plugin::BackendEnvironmentResponse environment(HostBackendEnvironmentOptions options = {});
};
struct HostRoot {
  HostBackendResource backend;
  HostContentResource content;
  HostDocumentsResource documents;
  HostEffectsResource effects;
  HostEvidenceResource evidence;
  HostExecutionsResource executions;
  HostFilesResource files;
  HostInvocationsResource invocations;
  HostOperationsResource operations;
  HostPluginsResource plugins;
  HostPromptsResource prompts;
  HostSecretsResource secrets;
  HostTimeResource time;
};
} // namespace dokime::templates
