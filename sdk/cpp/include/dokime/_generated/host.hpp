// @generated from protocol descriptors; do not edit.
#pragma once
namespace dokime::plugin {
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
  std::optional<::dokime::Section> subsection;
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
  std::optional<::dokime::EvidenceRecordScope> scope;
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
  explicit HostFilesWatch(std::function<std::optional<::dokime::FilesWatchResponse>()> next) : next_(std::move(next)) {}
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
  explicit HostInvocationsWatch(std::function<std::optional<::harmos::InstanceState>()> next) : next_(std::move(next)) {}
  std::optional<::harmos::InstanceState> next() {
    return next_();
  }

private:
  std::function<std::optional<::harmos::InstanceState>()> next_;
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
  HostTimeResource(Context value) : context(value) {}
  ::google::protobuf::Timestamp get(HostTimeGetOptions options = {});
  ::harmos::host::v1::TimeSleepResponse sleep(const ::google::protobuf::Duration &duration, HostTimeSleepOptions options = {});
};
struct HostSecretsResource {
  Context context;
  HostSecretsResource(Context value) : context(value) {}
  ::dokime::SecretsGetResponse get(const std::string &name, HostSecretsGetOptions options = {});
};
struct HostPromptsResource {
  Context context;
  HostPromptsResource(Context value) : context(value) {}
  ::dokime::plugin::PromptsCreateResponse create(const std::string &prompt, const std::string &prompt_type,
                                                 const std::map<std::string, ::dokime::Value> &values,
                                                 HostPromptsCreateOptions options = {});
};
struct HostPluginsSettingsResource {
  Context context;
  HostPluginsSettingsResource(Context value) : context(value) {}
  ::harmos::host::v1::plugins::SettingsListResponse list(HostPluginsSettingsListOptions options = {});
};
struct HostPluginsResource {
  Context context;
  HostPluginsSettingsResource settings;
  HostPluginsResource(Context value) : context(value), settings(value) {}
};
struct HostInvocationsResource {
  Context context;
  HostInvocationsResource(Context value) : context(value) {}
  ::harmos::InstanceState get(const std::uint64_t &instance, HostInvocationsGetOptions options = {});
  ::harmos::InstanceState cancel(const std::uint64_t &instance, const std::string &reason, HostInvocationsCancelOptions options = {});
  HostInvocationsWatch watch(const std::uint64_t &instance, std::size_t max_item_bytes, HostInvocationsWatchOptions options = {});
};
struct HostFilesUploadsResource {
  Context context;
  HostFilesUploadsResource(Context value) : context(value) {}
  ::dokime::FilesUploadsCreateResponse create(const ::harmos::FileMetadata &metadata, HostFilesUploadsCreateOptions options = {});
  ::dokime::FilesUploadsGetResponse get(const ::harmos::Upload &upload, HostFilesUploadsGetOptions options = {});
  ::dokime::FilesUploadsResumeResponse resume(const ::harmos::Upload &upload, HostFilesUploadsResumeOptions options = {});
  ::dokime::FilesUploadsAppendResponse append(const ::harmos::Upload &upload, const std::uint64_t &offset, const std::string &data,
                                              HostFilesUploadsAppendOptions options = {});
  ::dokime::FilesUploadsPublishResponse publish(const ::harmos::Upload &upload, const std::uint64_t &length, const std::string &sha256,
                                                HostFilesUploadsPublishOptions options = {});
  ::dokime::FilesUploadsDeleteResponse delete_(const ::harmos::Upload &upload, HostFilesUploadsDeleteOptions options = {});
};
struct HostFilesCapturesResource {
  Context context;
  HostFilesCapturesResource(Context value) : context(value) {}
  std::uint64_t start(const ::dokime::FileSource &source, const std::string &output, const ::dokime::plugin::FilesCaptureMode &mode,
                      HostFilesCapturesStartOptions options = {});
  ::dokime::plugin::FilesCapturesStopResponse stop(const std::uint64_t &capture_id, HostFilesCapturesStopOptions options = {});
};
struct HostFilesResource {
  Context context;
  HostFilesCapturesResource captures;
  HostFilesUploadsResource uploads;
  HostFilesResource(Context value) : context(value), captures(value), uploads(value) {}
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
  HostExecutionsProgressResource(Context value) : context(value) {}
  ::dokime::ExecutionsProgressGetResponse get(HostExecutionsProgressGetOptions options = {});
  ::dokime::plugin::ExecutionsProgressSetResponse set(const ::dokime::ExecutionProgressValue &value, const std::string &expected_revision,
                                                      HostExecutionsProgressSetOptions options = {});
};
struct HostExecutionsResource {
  Context context;
  HostExecutionsProgressResource progress;
  HostExecutionsResource(Context value) : context(value), progress(value) {}
};
struct HostEvidenceTraceabilityResource {
  Context context;
  HostEvidenceTraceabilityResource(Context value) : context(value) {}
  ::dokime::EvidenceTraceabilityGetResponse get(HostEvidenceTraceabilityGetOptions options = {});
  ::dokime::plugin::EvidenceTraceabilitySetResponse set(const ::dokime::TraceabilityEntry &value, const std::string &expected_revision,
                                                        const std::string &entry, HostEvidenceTraceabilitySetOptions options = {});
};
struct HostEvidenceRecordsResource {
  Context context;
  HostEvidenceRecordsResource(Context value) : context(value) {}
  ::dokime::EvidenceRecordsListResponse list(const std::string &record_type, const std::vector<std::uint32_t> &versions,
                                             const ::dokime::Window &between, const std::string &step,
                                             const ::dokime::EvidenceCapturedRecordSelection &selection,
                                             HostEvidenceRecordsListOptions options = {});
  ::dokime::plugin::EvidenceRecordsAppendResponse append(const std::string &execution, const std::string &kind, const std::string &payload,
                                                         const ::google::protobuf::Timestamp &source_at,
                                                         HostEvidenceRecordsAppendOptions options = {});
};
struct HostEvidenceLogsResource {
  Context context;
  HostEvidenceLogsResource(Context value) : context(value) {}
  ::harmos::Receipt append(const std::string &execution, const ::dokime::Severity &severity, const std::string &message,
                           const ::google::protobuf::Timestamp &at, HostEvidenceLogsAppendOptions options = {});
};
struct HostEvidenceResource {
  Context context;
  HostEvidenceLogsResource logs;
  HostEvidenceRecordsResource records;
  HostEvidenceTraceabilityResource traceability;
  HostEvidenceResource(Context value) : context(value), logs(value), records(value), traceability(value) {}
  ::dokime::EvidenceStatisticsResponse statistics(const std::string &stream, const std::vector<std::uint32_t> &versions,
                                                  const ::dokime::Window &between, const std::vector<::dokime::Output> &outputs,
                                                  const std::string &field, const std::string &step,
                                                  HostEvidenceStatisticsOptions options = {});
};
struct HostEffectsResource {
  Context context;
  HostEffectsResource(Context value) : context(value) {}
  ::dokime::plugin::EffectsCreateResponse create(const std::string &resource, const std::string &inverse_operation,
                                                 const std::string &inverse, const std::string &description,
                                                 HostEffectsCreateOptions options = {});
  ::dokime::plugin::EffectsGetResponse get(const std::string &name, HostEffectsGetOptions options = {});
  ::dokime::plugin::EffectsSetResponse set(const std::string &name, const ::dokime::plugin::EffectWritable &value,
                                           const std::string &expected_revision, HostEffectsSetOptions options = {});
};
struct HostDocumentsResource {
  Context context;
  HostDocumentsResource(Context value) : context(value) {}
  ::harmos::Receipt append_parts(const std::string &execution, const std::string &document, const std::string &section,
                                 HostDocumentsAppendPartsOptions options = {});
};
struct HostContentArtifactsResource {
  Context context;
  HostContentArtifactsResource(Context value) : context(value) {}
  ::dokime::ContentArtifactsCreateResponse create(const ::harmos::FileReference &file, const std::string &filename,
                                                  const std::string &content_type, HostContentArtifactsCreateOptions options = {});
};
struct HostContentResource {
  Context context;
  HostContentArtifactsResource artifacts;
  HostContentResource(Context value) : context(value), artifacts(value) {}
};
struct HostBackendResource {
  Context context;
  HostBackendResource(Context value) : context(value) {}
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
  HostPluginsResource plugins;
  HostPromptsResource prompts;
  HostSecretsResource secrets;
  HostTimeResource time;
};
} // namespace dokime::plugin
