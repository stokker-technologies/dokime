/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/context.hpp"
#include "dokime/plugin/host/v1/evidence/traceability.pb.h"
#include "dokime/plugin/host/v1/executions/progress.pb.h"
#include <random>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
namespace {
std::string request_id() {
  std::random_device random;
  return std::to_string(random()) + ":" + std::to_string(random()) + ":" + std::to_string(random()) + ":" + std::to_string(random());
}

const harmos::Operation<::dokime::plugin::host::v1::PromptsCreateRequest, ::dokime::plugin::host::v1::PromptsCreateResponse>
    prompt_call("dokime.plugin.host.v1.Prompts/Create", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::ReadSettings, ::dokime::plugin::host::v1::SavedSettings>
    settings_call("dokime.settings.read", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::ReadSecret, ::dokime::plugin::host::v1::Secret>
    secret_call("dokime.secrets.read", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::evidence::RecordsAppendRequest,
                        ::dokime::plugin::host::v1::evidence::RecordsAppendResponse>
    record_call("dokime.plugin.host.v1.evidence.Records/Append", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::evidence::RecordsListRequest, ::dokime::plugin::host::v1::evidence::RecordsListResponse>
    records_call("dokime.plugin.host.v1.evidence.Records/List", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::EvidenceStatisticsRequest, ::dokime::plugin::host::v1::EvidenceStatisticsResponse>
    statistics_call("dokime.plugin.host.v1.Evidence/Statistics", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::executions::ProgressGetRequest,
                        ::dokime::plugin::host::v1::executions::ProgressGetResponse>
    progress_get("dokime.plugin.host.v1.executions.Progress/Get", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::executions::ProgressSetRequest,
                        ::dokime::plugin::host::v1::executions::ProgressSetResponse>
    progress_set("dokime.plugin.host.v1.executions.Progress/Set", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::evidence::LogsAppendRequest, ::dokime::plugin::host::v1::evidence::LogsAppendResponse>
    log_call("dokime.plugin.host.v1.evidence.Logs/Append", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::content::ArtifactsCreateRequest,
                        ::dokime::plugin::host::v1::content::ArtifactsCreateResponse>
    artifact_call("dokime.plugin.host.v1.content.Artifacts/Create", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::evidence::TraceabilityGetRequest,
                        ::dokime::plugin::host::v1::evidence::TraceabilityGetResponse>
    traceability_get("dokime.plugin.host.v1.evidence.Traceability/Get", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::evidence::TraceabilitySetRequest,
                        ::dokime::plugin::host::v1::evidence::TraceabilitySetResponse>
    traceability_set("dokime.plugin.host.v1.evidence.Traceability/Set", harmos::wire::VISIBILITY_PUBLIC);
const harmos::Operation<::dokime::plugin::host::v1::DocumentsAppendPartsRequest, ::dokime::plugin::host::v1::DocumentsAppendPartsResponse>
    document_call("dokime.plugin.host.v1.Documents/AppendParts", harmos::wire::VISIBILITY_PUBLIC);
} // namespace

std::shared_ptr<detail::Resources> detail::Executions::open(const std::string &execution) {
  std::lock_guard lock(mutex);
  auto &resources = values[execution];
  if (!resources)
    resources = std::make_shared<Resources>();
  return resources;
}
void detail::Executions::close(const std::string &execution) {
  std::lock_guard lock(mutex);
  values.erase(execution);
}
void detail::Executions::clear() {
  std::unordered_map<std::string, std::shared_ptr<Resources>> released;
  {
    std::lock_guard lock(mutex);
    values.swap(released);
  }
}
Context::Context(harmos::Context &transport, v1::TestCase test_case, std::string item, std::shared_ptr<detail::Resources> resources)
    : transport_(transport), test_case_(std::move(test_case)), item_(std::move(item)), resources_(std::move(resources)) {}

::dokime::plugin::host::v1::PromptsCreateResponse Context::prompt(::dokime::plugin::host::v1::PromptsCreateRequest request) {
  return operation(prompt_call, std::move(request));
}
::dokime::plugin::host::v1::SavedSettings Context::settings() {
  return operation(settings_call, ::dokime::plugin::host::v1::ReadSettings{});
}
::dokime::plugin::host::v1::Secret Context::secret(std::string identifier) {
  ::dokime::plugin::host::v1::ReadSecret request;
  request.set_identifier(std::move(identifier));
  return operation(secret_call, std::move(request));
}
::dokime::plugin::host::v1::evidence::RecordsAppendResponse Context::record(std::string kind, const google::protobuf::Message &value) {
  ::dokime::plugin::host::v1::evidence::RecordsAppendRequest request;
  request.set_execution(execution());
  request.set_kind(std::move(kind));
  request.set_payload(harmos::detail::encode(value));
  request.set_request_id(request_id());
  return operation(record_call, std::move(request));
}
::dokime::plugin::host::v1::evidence::RecordsListResponse
Context::records(std::string kind, std::string step, harmos::wire::PageRequest page, std::optional<harmos::wire::Position> at) {
  ::dokime::plugin::host::v1::evidence::RecordsListRequest request;
  request.set_execution(execution());
  request.set_kind(std::move(kind));
  request.set_step(std::move(step));
  *request.mutable_page() = std::move(page);
  if (at)
    *request.mutable_at() = std::move(*at);
  return operation(records_call, std::move(request));
}
::dokime::plugin::host::v1::EvidenceStatisticsResponse Context::statistics(std::string stream, std::string field, std::string step) {
  ::dokime::plugin::host::v1::EvidenceStatisticsRequest request;
  request.set_execution(execution());
  request.set_stream(std::move(stream));
  request.set_field(std::move(field));
  request.set_step(std::move(step));
  return operation(statistics_call, std::move(request));
}
void Context::progress(std::uint64_t completed, std::uint64_t total, std::string label) {
  ::dokime::plugin::host::v1::executions::ProgressGetRequest get;
  get.set_name(execution());
  const auto current = operation(progress_get, std::move(get));
  ::dokime::plugin::host::v1::executions::ProgressSetRequest request;
  request.set_name(execution());
  request.mutable_value()->set_completed(completed);
  request.mutable_value()->set_total(total);
  request.mutable_value()->set_label(std::move(label));
  request.set_expected_revision(current.revision());
  request.set_request_id(request_id());
  operation(progress_set, std::move(request));
}
void Context::log(v1::Severity severity, std::string message) {
  ::dokime::plugin::host::v1::evidence::LogsAppendRequest request;
  request.set_execution(execution());
  request.set_severity(severity);
  request.set_message(std::move(message));
  request.set_request_id(request_id());
  operation(log_call, std::move(request));
}
::dokime::plugin::host::v1::content::ArtifactsCreateResponse
Context::artifact(::dokime::plugin::host::v1::content::ArtifactsCreateRequest request) {
  request.set_execution(execution());
  return operation(artifact_call, std::move(request));
}
void Context::traceability(v1::TraceabilityEntry value) {
  ::dokime::plugin::host::v1::evidence::TraceabilityGetRequest get;
  get.set_name(execution());
  get.set_entry(value.metadata().identifier());
  const auto current = operation(traceability_get, std::move(get));
  ::dokime::plugin::host::v1::evidence::TraceabilitySetRequest request;
  request.set_name(execution());
  request.set_entry(value.metadata().identifier());
  *request.mutable_value() = std::move(value);
  request.set_expected_revision(current.revision());
  request.set_request_id(request_id());
  operation(traceability_set, std::move(request));
}
void Context::document(::dokime::plugin::host::v1::DocumentsAppendPartsRequest request) {
  request.set_execution(execution());
  operation(document_call, std::move(request));
}
void Context::declare_requirements(harmos::Plugin &plugin) {
  plugin.requires_operation(prompt_call)
      .requires_operation(settings_call)
      .requires_operation(secret_call)
      .requires_operation(record_call)
      .requires_operation(records_call)
      .requires_operation(statistics_call)
      .requires_operation(progress_get)
      .requires_operation(progress_set)
      .requires_operation(log_call)
      .requires_operation(artifact_call)
      .requires_operation(traceability_get)
      .requires_operation(traceability_set)
      .requires_operation(document_call)
      .requires_operation(harmos::GET_TIME)
      .requires_operation(harmos::SLEEP);
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
