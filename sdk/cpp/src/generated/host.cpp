// @generated from protocol descriptors; do not edit.
#include "dokime/dokime.hpp"
#include "dokime/host/v1/backend.pb.h"
#include "dokime/host/v1/content/artifacts.pb.h"
#include "dokime/host/v1/documents.pb.h"
#include "dokime/host/v1/effects.pb.h"
#include "dokime/host/v1/evidence.pb.h"
#include "dokime/host/v1/evidence/logs.pb.h"
#include "dokime/host/v1/evidence/records.pb.h"
#include "dokime/host/v1/evidence/traceability.pb.h"
#include "dokime/host/v1/executions/progress.pb.h"
#include "dokime/host/v1/files.pb.h"
#include "dokime/host/v1/files/captures.pb.h"
#include "dokime/host/v1/files/uploads.pb.h"
#include "dokime/host/v1/prompts.pb.h"
#include "dokime/host/v1/secrets.pb.h"
#include "harmos/plugin/v1/invocations.pb.h"
#include "harmos/plugin/v1/operations.pb.h"
#include "harmos/plugin/v1/plugins/settings.pb.h"
#include "harmos/plugin/v1/plugins/state.pb.h"
#include "harmos/plugin/v1/time.pb.h"
using ::dokime::CapturedRecordSelection;
using ::dokime::ExecutionProgressValue;
using ::dokime::FileSource;
using ::dokime::Output;
using ::dokime::Severity;
using ::dokime::TraceabilityEntry;
using ::dokime::Value;
using ::dokime::host::v1::BackendEnvironmentRequest;
using ::dokime::host::v1::BackendEnvironmentResponse;
using ::dokime::host::v1::DocumentsAppendPartsRequest;
using ::dokime::host::v1::DocumentsAppendPartsResponse;
using ::dokime::host::v1::EffectsCreateRequest;
using ::dokime::host::v1::EffectsCreateResponse;
using ::dokime::host::v1::EffectsGetRequest;
using ::dokime::host::v1::EffectsGetResponse;
using ::dokime::host::v1::EffectsSetRequest;
using ::dokime::host::v1::EffectsSetResponse;
using ::dokime::host::v1::EvidenceStatisticsRequest;
using ::dokime::host::v1::EvidenceStatisticsResponse;
using ::dokime::host::v1::FilesCaptureRequest;
using ::dokime::host::v1::FilesCaptureResponse;
using ::dokime::host::v1::FilesDeleteRequest;
using ::dokime::host::v1::FilesDeleteResponse;
using ::dokime::host::v1::FilesGetRequest;
using ::dokime::host::v1::FilesGetResponse;
using ::dokime::host::v1::FilesReadRequest;
using ::dokime::host::v1::FilesReadResponse;
using ::dokime::host::v1::FilesWatchRequest;
using ::dokime::host::v1::FilesWatchResponse;
using ::dokime::host::v1::PromptsCreateRequest;
using ::dokime::host::v1::PromptsCreateResponse;
using ::dokime::host::v1::SecretsGetRequest;
using ::dokime::host::v1::SecretsGetResponse;
using ::dokime::host::v1::content::ArtifactsCreateRequest;
using ::dokime::host::v1::content::ArtifactsCreateResponse;
using ::dokime::host::v1::evidence::LogsAppendRequest;
using ::dokime::host::v1::evidence::LogsAppendResponse;
using ::dokime::host::v1::evidence::RecordsAppendRequest;
using ::dokime::host::v1::evidence::RecordsAppendResponse;
using ::dokime::host::v1::evidence::RecordsListRequest;
using ::dokime::host::v1::evidence::RecordsListResponse;
using ::dokime::host::v1::evidence::TraceabilityGetRequest;
using ::dokime::host::v1::evidence::TraceabilityGetResponse;
using ::dokime::host::v1::evidence::TraceabilitySetRequest;
using ::dokime::host::v1::evidence::TraceabilitySetResponse;
using ::dokime::host::v1::executions::ProgressGetRequest;
using ::dokime::host::v1::executions::ProgressGetResponse;
using ::dokime::host::v1::executions::ProgressSetRequest;
using ::dokime::host::v1::executions::ProgressSetResponse;
using ::dokime::host::v1::files::CapturesStartRequest;
using ::dokime::host::v1::files::CapturesStartResponse;
using ::dokime::host::v1::files::CapturesStopRequest;
using ::dokime::host::v1::files::CapturesStopResponse;
using ::dokime::host::v1::files::UploadsAppendRequest;
using ::dokime::host::v1::files::UploadsAppendResponse;
using ::dokime::host::v1::files::UploadsCreateRequest;
using ::dokime::host::v1::files::UploadsCreateResponse;
using ::dokime::host::v1::files::UploadsDeleteRequest;
using ::dokime::host::v1::files::UploadsDeleteResponse;
using ::dokime::host::v1::files::UploadsGetRequest;
using ::dokime::host::v1::files::UploadsGetResponse;
using ::dokime::host::v1::files::UploadsPublishRequest;
using ::dokime::host::v1::files::UploadsPublishResponse;
using ::dokime::host::v1::files::UploadsResumeRequest;
using ::dokime::host::v1::files::UploadsResumeResponse;
using ::dokime::model::Window;
using ::dokime::plugin::CaptureMode;
using ::dokime::plugin::EffectWritable;
using ::google::protobuf::Duration;
using ::google::protobuf::Timestamp;
using ::harmos::DiscoveredOperation;
using ::harmos::FileMetadata;
using ::harmos::FileReference;
using ::harmos::InstanceState;
using ::harmos::Receipt;
using ::harmos::Upload;
using ::harmos::host::v1::InvocationsCancelRequest;
using ::harmos::host::v1::InvocationsCancelResponse;
using ::harmos::host::v1::InvocationsGetRequest;
using ::harmos::host::v1::InvocationsGetResponse;
using ::harmos::host::v1::InvocationsWatchRequest;
using ::harmos::host::v1::InvocationsWatchResponse;
using ::harmos::host::v1::OperationsGetRequest;
using ::harmos::host::v1::OperationsGetResponse;
using ::harmos::host::v1::OperationsListRequest;
using ::harmos::host::v1::OperationsListResponse;
using ::harmos::host::v1::TimeGetRequest;
using ::harmos::host::v1::TimeGetResponse;
using ::harmos::host::v1::TimeSleepRequest;
using ::harmos::host::v1::TimeSleepResponse;
using ::harmos::host::v1::plugins::SettingsListRequest;
using ::harmos::host::v1::plugins::SettingsListResponse;
using ::harmos::host::v1::plugins::StateGetRequest;
using ::harmos::host::v1::plugins::StateGetResponse;
using ::harmos::host::v1::plugins::StatePutRequest;
using ::harmos::host::v1::plugins::StatePutResponse;
using std::invalid_argument;
using std::optional;
using std::runtime_error;
using std::size_t;
using std::string;
using std::uint32_t;
using std::uint64_t;
using std::vector;
namespace dokime::templates {
using ::harmos::VISIBILITY_PUBLIC;
using ::harmos::plugin::Operation;
using ::harmos::plugin::Watching;
Timestamp HostTimeResource::get(HostTimeGetOptions options) {
  TimeGetRequest request;
  auto operation = Operation<TimeGetRequest, TimeGetResponse>("harmos.plugin.v1.Time/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_now()) {
    throw runtime_error("Read TimeGet: Missing Payload");
  }
  return std::move(*response.mutable_now());
}
TimeSleepResponse HostTimeResource::sleep(const Duration &duration, HostTimeSleepOptions options) {
  TimeSleepRequest request;
  *request.mutable_duration() = duration;
  auto operation = Operation<TimeSleepRequest, TimeSleepResponse>("harmos.plugin.v1.Time/Sleep", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::SecretsGetResponse HostSecretsResource::get(const string &name, HostSecretsGetOptions options) {
  SecretsGetRequest request;
  request.set_name(name);
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<SecretsGetRequest, SecretsGetResponse>("dokime.host.v1.Secrets/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::PromptsCreateResponse HostPromptsResource::create(const string &prompt, const string &prompt_type,
                                                                    const std::map<string, Value> &values,
                                                                    HostPromptsCreateOptions options) {
  PromptsCreateRequest request;
  request.set_prompt(prompt);
  request.set_prompt_type(prompt_type);
  for (const auto &[key, item] : values) {
    (*request.mutable_values())[key] = item;
  }
  if (options.definition) {
    *request.mutable_definition() = *options.definition;
  }
  auto operation = Operation<PromptsCreateRequest, PromptsCreateResponse>("dokime.host.v1.Prompts/Create", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
StateGetResponse HostPluginsStateResource::get(HostPluginsStateGetOptions options) {
  StateGetRequest request;
  auto operation = Operation<StateGetRequest, StateGetResponse>("harmos.plugin.v1.plugins.State/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
uint64_t HostPluginsStateResource::put(const uint64_t &expected_revision, const string &value, HostPluginsStatePutOptions options) {
  StatePutRequest request;
  request.set_expected_revision(expected_revision);
  request.set_value(value);
  auto operation = Operation<StatePutRequest, StatePutResponse>("harmos.plugin.v1.plugins.State/Put", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response.revision();
}
SettingsListResponse HostPluginsSettingsResource::list(HostPluginsSettingsListOptions options) {
  SettingsListRequest request;
  if (options.name) {
    request.set_name(*options.name);
  }
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<SettingsListRequest, SettingsListResponse>("harmos.plugin.v1.plugins.Settings/List", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
vector<DiscoveredOperation> HostOperationsResource::list(HostOperationsListOptions options) {
  OperationsListRequest request;
  auto operation = Operation<OperationsListRequest, OperationsListResponse>("harmos.plugin.v1.Operations/List", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return vector<DiscoveredOperation>(response.operations().begin(), response.operations().end());
}
DiscoveredOperation HostOperationsResource::get(const string &id, HostOperationsGetOptions options) {
  OperationsGetRequest request;
  request.set_id(id);
  auto operation = Operation<OperationsGetRequest, OperationsGetResponse>("harmos.plugin.v1.Operations/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_operation()) {
    throw runtime_error("Read OperationsGet: Missing Payload");
  }
  return std::move(*response.mutable_operation());
}
InstanceState HostInvocationsResource::get(const uint64_t &instance, HostInvocationsGetOptions options) {
  InvocationsGetRequest request;
  request.set_instance(instance);
  auto operation = Operation<InvocationsGetRequest, InvocationsGetResponse>("harmos.plugin.v1.Invocations/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_invocation()) {
    throw runtime_error("Read InvocationsGet: Missing Payload");
  }
  return std::move(*response.mutable_invocation());
}
InstanceState HostInvocationsResource::cancel(const uint64_t &instance, const string &reason, HostInvocationsCancelOptions options) {
  InvocationsCancelRequest request;
  request.set_instance(instance);
  request.set_reason(reason);
  auto operation = Operation<InvocationsCancelRequest, InvocationsCancelResponse>("harmos.plugin.v1.Invocations/Cancel", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_invocation()) {
    throw runtime_error("Read InvocationsCancel: Missing Payload");
  }
  return std::move(*response.mutable_invocation());
}
HostInvocationsWatch HostInvocationsResource::watch(const uint64_t &instance, size_t max_item_bytes, HostInvocationsWatchOptions options) {
  InvocationsWatchRequest request;
  request.set_instance(instance);
  auto operation = Operation<InvocationsWatchRequest, InvocationsWatchResponse>("harmos.plugin.v1.Invocations/Watch", VISIBILITY_PUBLIC);
  (void)options;
  operation.watch();
  auto incoming = std::make_shared<Watching<InvocationsWatchResponse>>(context.watch(operation, request, max_item_bytes));
  return HostInvocationsWatch([incoming]() -> optional<InstanceState> {
    auto response = incoming->next();
    if (!response) {
      return {};
    }
    if (!response->has_invocation()) {
      throw runtime_error("Read InvocationsWatch: Missing Payload");
    }
    return std::move(*response->mutable_invocation());
  });
}
::dokime::UploadsCreateResponse HostFilesUploadsResource::create(const FileMetadata &metadata, HostFilesUploadsCreateOptions options) {
  UploadsCreateRequest request;
  *request.mutable_metadata() = metadata;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<UploadsCreateRequest, UploadsCreateResponse>("dokime.host.v1.files.Uploads/Create", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::UploadsGetResponse HostFilesUploadsResource::get(const Upload &upload, HostFilesUploadsGetOptions options) {
  UploadsGetRequest request;
  *request.mutable_upload() = upload;
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<UploadsGetRequest, UploadsGetResponse>("dokime.host.v1.files.Uploads/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::UploadsResumeResponse HostFilesUploadsResource::resume(const Upload &upload, HostFilesUploadsResumeOptions options) {
  UploadsResumeRequest request;
  *request.mutable_upload() = upload;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<UploadsResumeRequest, UploadsResumeResponse>("dokime.host.v1.files.Uploads/Resume", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::UploadsAppendResponse HostFilesUploadsResource::append(const Upload &upload, const uint64_t &offset, const string &data,
                                                                 HostFilesUploadsAppendOptions options) {
  UploadsAppendRequest request;
  *request.mutable_upload() = upload;
  request.set_offset(offset);
  request.set_data(data);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<UploadsAppendRequest, UploadsAppendResponse>("dokime.host.v1.files.Uploads/Append", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::UploadsPublishResponse HostFilesUploadsResource::publish(const Upload &upload, const uint64_t &length, const string &sha256,
                                                                   HostFilesUploadsPublishOptions options) {
  UploadsPublishRequest request;
  *request.mutable_upload() = upload;
  request.set_length(length);
  request.set_sha256(sha256);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<UploadsPublishRequest, UploadsPublishResponse>("dokime.host.v1.files.Uploads/Publish", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::UploadsDeleteResponse HostFilesUploadsResource::delete_(const Upload &upload, HostFilesUploadsDeleteOptions options) {
  UploadsDeleteRequest request;
  *request.mutable_upload() = upload;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<UploadsDeleteRequest, UploadsDeleteResponse>("dokime.host.v1.files.Uploads/Delete", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
uint64_t HostFilesCapturesResource::start(const FileSource &source, const string &output, const CaptureMode &mode,
                                          HostFilesCapturesStartOptions options) {
  CapturesStartRequest request;
  *request.mutable_source() = source;
  request.set_output(output);
  *request.mutable_mode() = mode;
  auto operation = Operation<CapturesStartRequest, CapturesStartResponse>("dokime.host.v1.files.Captures/Start", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response.capture_id();
}
::dokime::plugin::host::files::CapturesStopResponse HostFilesCapturesResource::stop(const uint64_t &capture_id,
                                                                                    HostFilesCapturesStopOptions options) {
  CapturesStopRequest request;
  request.set_capture_id(capture_id);
  auto operation = Operation<CapturesStopRequest, CapturesStopResponse>("dokime.host.v1.files.Captures/Stop", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::FilesGetResponse HostFilesResource::get(const FileReference &file, HostFilesGetOptions options) {
  FilesGetRequest request;
  *request.mutable_file() = file;
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<FilesGetRequest, FilesGetResponse>("dokime.host.v1.Files/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::FilesReadResponse HostFilesResource::read(const FileReference &file, const uint64_t &offset, const uint64_t &max_bytes,
                                                    HostFilesReadOptions options) {
  FilesReadRequest request;
  *request.mutable_file() = file;
  request.set_offset(offset);
  request.set_max_bytes(max_bytes);
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<FilesReadRequest, FilesReadResponse>("dokime.host.v1.Files/Read", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
HostFilesWatch HostFilesResource::watch(const uint64_t &offset, size_t max_item_bytes, HostFilesWatchOptions options) {
  FilesWatchRequest request;
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
  auto operation = Operation<FilesWatchRequest, FilesWatchResponse>("dokime.host.v1.Files/Watch", VISIBILITY_PUBLIC);
  (void)options;
  operation.watch();
  auto incoming = std::make_shared<Watching<FilesWatchResponse>>(context.watch(operation, request, max_item_bytes));
  return HostFilesWatch([incoming]() -> optional<::dokime::FilesWatchResponse> {
    auto response = incoming->next();
    if (!response) {
      return {};
    }
    return *response;
  });
}
Receipt HostFilesResource::delete_(const FileReference &file, HostFilesDeleteOptions options) {
  FilesDeleteRequest request;
  *request.mutable_file() = file;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<FilesDeleteRequest, FilesDeleteResponse>("dokime.host.v1.Files/Delete", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_receipt()) {
    throw runtime_error("Read FilesDelete: Missing Payload");
  }
  return std::move(*response.mutable_receipt());
}
::dokime::plugin::FilesCaptureResponse HostFilesResource::capture(const FileSource &source, const string &output,
                                                                  HostFilesCaptureOptions options) {
  FilesCaptureRequest request;
  *request.mutable_source() = source;
  request.set_output(output);
  auto operation = Operation<FilesCaptureRequest, FilesCaptureResponse>("dokime.host.v1.Files/Capture", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::ProgressGetResponse HostExecutionsProgressResource::get(HostExecutionsProgressGetOptions options) {
  ProgressGetRequest request;
  if (options.name) {
    request.set_name(*options.name);
  }
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<ProgressGetRequest, ProgressGetResponse>("dokime.host.v1.executions.Progress/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::ProgressSetResponse HostExecutionsProgressResource::set(const ExecutionProgressValue &value,
                                                                          const string &expected_revision,
                                                                          HostExecutionsProgressSetOptions options) {
  ProgressSetRequest request;
  if (options.name) {
    request.set_name(*options.name);
  }
  *request.mutable_value() = value;
  request.set_expected_revision(expected_revision);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<ProgressSetRequest, ProgressSetResponse>("dokime.host.v1.executions.Progress/Set", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::TraceabilityGetResponse HostEvidenceTraceabilityResource::get(HostEvidenceTraceabilityGetOptions options) {
  TraceabilityGetRequest request;
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
  auto operation =
      Operation<TraceabilityGetRequest, TraceabilityGetResponse>("dokime.host.v1.evidence.Traceability/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::TraceabilitySetResponse HostEvidenceTraceabilityResource::set(const TraceabilityEntry &value,
                                                                                const string &expected_revision, const string &entry,
                                                                                HostEvidenceTraceabilitySetOptions options) {
  TraceabilitySetRequest request;
  if (options.name) {
    request.set_name(*options.name);
  }
  *request.mutable_value() = value;
  request.set_expected_revision(expected_revision);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  request.set_entry(entry);
  auto operation =
      Operation<TraceabilitySetRequest, TraceabilitySetResponse>("dokime.host.v1.evidence.Traceability/Set", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::RecordsListResponse HostEvidenceRecordsResource::list(const string &record_type, const vector<uint32_t> &versions,
                                                                const Window &between, const string &step,
                                                                const CapturedRecordSelection &selection,
                                                                HostEvidenceRecordsListOptions options) {
  RecordsListRequest request;
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
  auto operation = Operation<RecordsListRequest, RecordsListResponse>("dokime.host.v1.evidence.Records/List", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::RecordsAppendResponse HostEvidenceRecordsResource::append(const string &execution, const string &kind,
                                                                            const string &payload, const Timestamp &source_at,
                                                                            HostEvidenceRecordsAppendOptions options) {
  RecordsAppendRequest request;
  request.set_execution(execution);
  request.set_kind(kind);
  request.set_payload(payload);
  *request.mutable_source_at() = source_at;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<RecordsAppendRequest, RecordsAppendResponse>("dokime.host.v1.evidence.Records/Append", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
Receipt HostEvidenceLogsResource::append(const string &execution, const Severity &severity, const string &message, const Timestamp &at,
                                         HostEvidenceLogsAppendOptions options) {
  LogsAppendRequest request;
  request.set_execution(execution);
  request.set_severity(severity);
  request.set_message(message);
  *request.mutable_at() = at;
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<LogsAppendRequest, LogsAppendResponse>("dokime.host.v1.evidence.Logs/Append", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_receipt()) {
    throw runtime_error("Read EvidenceLogsAppend: Missing Payload");
  }
  return std::move(*response.mutable_receipt());
}
::dokime::EvidenceStatisticsResponse HostEvidenceResource::statistics(const string &stream, const vector<uint32_t> &versions,
                                                                      const Window &between, const vector<Output> &outputs,
                                                                      const string &field, const string &step,
                                                                      HostEvidenceStatisticsOptions options) {
  EvidenceStatisticsRequest request;
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
  auto operation =
      Operation<EvidenceStatisticsRequest, EvidenceStatisticsResponse>("dokime.host.v1.Evidence/Statistics", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::EffectsCreateResponse HostEffectsResource::create(const string &resource, const string &inverse_operation,
                                                                    const string &inverse, const string &description,
                                                                    HostEffectsCreateOptions options) {
  EffectsCreateRequest request;
  request.set_resource(resource);
  request.set_inverse_operation(inverse_operation);
  request.set_inverse(inverse);
  request.set_description(description);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<EffectsCreateRequest, EffectsCreateResponse>("dokime.host.v1.Effects/Create", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::EffectsGetResponse HostEffectsResource::get(const string &name, HostEffectsGetOptions options) {
  EffectsGetRequest request;
  request.set_name(name);
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation = Operation<EffectsGetRequest, EffectsGetResponse>("dokime.host.v1.Effects/Get", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::EffectsSetResponse HostEffectsResource::set(const string &name, const EffectWritable &value,
                                                              const string &expected_revision, HostEffectsSetOptions options) {
  EffectsSetRequest request;
  request.set_name(name);
  *request.mutable_value() = value;
  request.set_expected_revision(expected_revision);
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation = Operation<EffectsSetRequest, EffectsSetResponse>("dokime.host.v1.Effects/Set", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
Receipt HostDocumentsResource::append_parts(const string &execution, const string &document, const string &section,
                                            HostDocumentsAppendPartsOptions options) {
  DocumentsAppendPartsRequest request;
  request.set_execution(execution);
  request.set_document(document);
  request.set_section(section);
  if (options.subsection) {
    *request.mutable_subsection() = *options.subsection;
  }
  if (options.card) {
    *request.mutable_card() = *options.card;
  }
  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());
  auto operation =
      Operation<DocumentsAppendPartsRequest, DocumentsAppendPartsResponse>("dokime.host.v1.Documents/AppendParts", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  if (!response.has_receipt()) {
    throw runtime_error("Read DocumentsAppendParts: Missing Payload");
  }
  return std::move(*response.mutable_receipt());
}
::dokime::ArtifactsCreateResponse HostContentArtifactsResource::create(const FileReference &file, const string &filename,
                                                                       const string &content_type,
                                                                       HostContentArtifactsCreateOptions options) {
  ArtifactsCreateRequest request;
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
  auto operation = Operation<ArtifactsCreateRequest, ArtifactsCreateResponse>("dokime.host.v1.content.Artifacts/Create", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
::dokime::plugin::BackendEnvironmentResponse HostBackendResource::environment(HostBackendEnvironmentOptions options) {
  BackendEnvironmentRequest request;
  if (options.at) {
    *request.mutable_at() = *options.at;
  }
  auto operation =
      Operation<BackendEnvironmentRequest, BackendEnvironmentResponse>("dokime.host.v1.Backend/Environment", VISIBILITY_PUBLIC);
  auto response = context.call(operation, request, options.policy).get();
  return response;
}
} // namespace dokime::templates
namespace dokime::templates {
HostRoot Context::host() {
  return {{*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}, {*this}};
}
} // namespace dokime::templates
namespace dokime {
Registrar &Registrar::add(templates::HostOperation operation) {
  using ::harmos::VISIBILITY_PUBLIC;
  using ::harmos::plugin::Dependency;
  using ::harmos::plugin::Operation;
  using templates::HostOperation;
  switch (operation) {
  case HostOperation::BackendEnvironment:
    return add(Dependency{
        Operation<BackendEnvironmentRequest, BackendEnvironmentResponse>("dokime.host.v1.Backend/Environment", VISIBILITY_PUBLIC)});
  case HostOperation::ContentArtifactsCreate:
    return add(Dependency{
        Operation<ArtifactsCreateRequest, ArtifactsCreateResponse>("dokime.host.v1.content.Artifacts/Create", VISIBILITY_PUBLIC)});
  case HostOperation::DocumentsAppendParts:
    return add(Dependency{
        Operation<DocumentsAppendPartsRequest, DocumentsAppendPartsResponse>("dokime.host.v1.Documents/AppendParts", VISIBILITY_PUBLIC)});
  case HostOperation::EffectsCreate:
    return add(Dependency{Operation<EffectsCreateRequest, EffectsCreateResponse>("dokime.host.v1.Effects/Create", VISIBILITY_PUBLIC)});
  case HostOperation::EffectsGet:
    return add(Dependency{Operation<EffectsGetRequest, EffectsGetResponse>("dokime.host.v1.Effects/Get", VISIBILITY_PUBLIC)});
  case HostOperation::EffectsSet:
    return add(Dependency{Operation<EffectsSetRequest, EffectsSetResponse>("dokime.host.v1.Effects/Set", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceLogsAppend:
    return add(Dependency{Operation<LogsAppendRequest, LogsAppendResponse>("dokime.host.v1.evidence.Logs/Append", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceRecordsList:
    return add(Dependency{Operation<RecordsListRequest, RecordsListResponse>("dokime.host.v1.evidence.Records/List", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceRecordsAppend:
    return add(
        Dependency{Operation<RecordsAppendRequest, RecordsAppendResponse>("dokime.host.v1.evidence.Records/Append", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceTraceabilityGet:
    return add(Dependency{
        Operation<TraceabilityGetRequest, TraceabilityGetResponse>("dokime.host.v1.evidence.Traceability/Get", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceTraceabilitySet:
    return add(Dependency{
        Operation<TraceabilitySetRequest, TraceabilitySetResponse>("dokime.host.v1.evidence.Traceability/Set", VISIBILITY_PUBLIC)});
  case HostOperation::EvidenceStatistics:
    return add(Dependency{
        Operation<EvidenceStatisticsRequest, EvidenceStatisticsResponse>("dokime.host.v1.Evidence/Statistics", VISIBILITY_PUBLIC)});
  case HostOperation::ExecutionsProgressGet:
    return add(Dependency{Operation<ProgressGetRequest, ProgressGetResponse>("dokime.host.v1.executions.Progress/Get", VISIBILITY_PUBLIC)});
  case HostOperation::ExecutionsProgressSet:
    return add(Dependency{Operation<ProgressSetRequest, ProgressSetResponse>("dokime.host.v1.executions.Progress/Set", VISIBILITY_PUBLIC)});
  case HostOperation::FilesCapturesStart:
    return add(
        Dependency{Operation<CapturesStartRequest, CapturesStartResponse>("dokime.host.v1.files.Captures/Start", VISIBILITY_PUBLIC)});
  case HostOperation::FilesCapturesStop:
    return add(Dependency{Operation<CapturesStopRequest, CapturesStopResponse>("dokime.host.v1.files.Captures/Stop", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsCreate:
    return add(
        Dependency{Operation<UploadsCreateRequest, UploadsCreateResponse>("dokime.host.v1.files.Uploads/Create", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsGet:
    return add(Dependency{Operation<UploadsGetRequest, UploadsGetResponse>("dokime.host.v1.files.Uploads/Get", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsResume:
    return add(
        Dependency{Operation<UploadsResumeRequest, UploadsResumeResponse>("dokime.host.v1.files.Uploads/Resume", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsAppend:
    return add(
        Dependency{Operation<UploadsAppendRequest, UploadsAppendResponse>("dokime.host.v1.files.Uploads/Append", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsPublish:
    return add(
        Dependency{Operation<UploadsPublishRequest, UploadsPublishResponse>("dokime.host.v1.files.Uploads/Publish", VISIBILITY_PUBLIC)});
  case HostOperation::FilesUploadsDelete:
    return add(
        Dependency{Operation<UploadsDeleteRequest, UploadsDeleteResponse>("dokime.host.v1.files.Uploads/Delete", VISIBILITY_PUBLIC)});
  case HostOperation::FilesGet:
    return add(Dependency{Operation<FilesGetRequest, FilesGetResponse>("dokime.host.v1.Files/Get", VISIBILITY_PUBLIC)});
  case HostOperation::FilesRead:
    return add(Dependency{Operation<FilesReadRequest, FilesReadResponse>("dokime.host.v1.Files/Read", VISIBILITY_PUBLIC)});
  case HostOperation::FilesWatch:
    return add(Dependency{Operation<FilesWatchRequest, FilesWatchResponse>("dokime.host.v1.Files/Watch", VISIBILITY_PUBLIC).watch()});
  case HostOperation::FilesDelete:
    return add(Dependency{Operation<FilesDeleteRequest, FilesDeleteResponse>("dokime.host.v1.Files/Delete", VISIBILITY_PUBLIC)});
  case HostOperation::FilesCapture:
    return add(Dependency{Operation<FilesCaptureRequest, FilesCaptureResponse>("dokime.host.v1.Files/Capture", VISIBILITY_PUBLIC)});
  case HostOperation::PromptsCreate:
    return add(Dependency{Operation<PromptsCreateRequest, PromptsCreateResponse>("dokime.host.v1.Prompts/Create", VISIBILITY_PUBLIC)});
  case HostOperation::SecretsGet:
    return add(Dependency{Operation<SecretsGetRequest, SecretsGetResponse>("dokime.host.v1.Secrets/Get", VISIBILITY_PUBLIC)});
  case HostOperation::InvocationsGet:
    return add(Dependency{Operation<InvocationsGetRequest, InvocationsGetResponse>("harmos.plugin.v1.Invocations/Get", VISIBILITY_PUBLIC)});
  case HostOperation::InvocationsCancel:
    return add(Dependency{
        Operation<InvocationsCancelRequest, InvocationsCancelResponse>("harmos.plugin.v1.Invocations/Cancel", VISIBILITY_PUBLIC)});
  case HostOperation::InvocationsWatch:
    return add(Dependency{
        Operation<InvocationsWatchRequest, InvocationsWatchResponse>("harmos.plugin.v1.Invocations/Watch", VISIBILITY_PUBLIC).watch()});
  case HostOperation::OperationsList:
    return add(Dependency{Operation<OperationsListRequest, OperationsListResponse>("harmos.plugin.v1.Operations/List", VISIBILITY_PUBLIC)});
  case HostOperation::OperationsGet:
    return add(Dependency{Operation<OperationsGetRequest, OperationsGetResponse>("harmos.plugin.v1.Operations/Get", VISIBILITY_PUBLIC)});
  case HostOperation::PluginsSettingsList:
    return add(
        Dependency{Operation<SettingsListRequest, SettingsListResponse>("harmos.plugin.v1.plugins.Settings/List", VISIBILITY_PUBLIC)});
  case HostOperation::PluginsStateGet:
    return add(Dependency{Operation<StateGetRequest, StateGetResponse>("harmos.plugin.v1.plugins.State/Get", VISIBILITY_PUBLIC)});
  case HostOperation::PluginsStatePut:
    return add(Dependency{Operation<StatePutRequest, StatePutResponse>("harmos.plugin.v1.plugins.State/Put", VISIBILITY_PUBLIC)});
  case HostOperation::TimeGet:
    return add(Dependency{Operation<TimeGetRequest, TimeGetResponse>("harmos.plugin.v1.Time/Get", VISIBILITY_PUBLIC)});
  case HostOperation::TimeSleep:
    return add(Dependency{Operation<TimeSleepRequest, TimeSleepResponse>("harmos.plugin.v1.Time/Sleep", VISIBILITY_PUBLIC)});
  default:
    throw invalid_argument("Declare Host Requirement: Unknown Operation");
  }
}
} // namespace dokime
