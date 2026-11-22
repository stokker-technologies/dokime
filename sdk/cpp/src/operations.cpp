/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/operations.hpp"
#include "dokime/host/v1/evidence/records.pb.h"
#include "dokime/host/v1/secrets.pb.h"
#include "dokime/plugin.hpp"
#include "harmos/client_transport.hpp"
#include <cmath>
#include <exception>

using dokime::host::v1::SecretsGetRequest;
using dokime::host::v1::SecretsGetResponse;
using dokime::host::v1::evidence::RecordsAppendRequest;
using dokime::host::v1::evidence::RecordsAppendResponse;
using google::protobuf::Duration;
using harmos::SettingValue;
using harmos::VISIBILITY_PUBLIC;
using harmos::plugin::Operation;
using std::abs;
using std::current_exception;
using std::exception_ptr;
using std::invalid_argument;
using std::isfinite;
using std::lock_guard;
using std::move;
using std::rethrow_exception;
using std::string;
using std::chrono::duration;
using std::chrono::duration_cast;
using std::chrono::nanoseconds;
using std::chrono::seconds;
using std::chrono::system_clock;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::operations {
Duration detail::encode_duration(duration<double> duration) {
  if (!isfinite(duration.count()) || duration.count() < 0 || duration.count() > 315576000000.0)
    throw invalid_argument("Configure Duration: Supply Finite Nonnegative Seconds Within the Protobuf Duration Range");
  const auto whole = duration_cast<seconds>(duration);
  const auto fraction = duration_cast<nanoseconds>(duration - whole);
  Duration request;
  request.set_seconds(whole.count());
  request.set_nanos(static_cast<int>(fraction.count()));
  return request;
}
void Context::sleep(duration<double> duration) {
  transport_.sleep(detail::encode_duration(duration));
}
system_clock::time_point Context::now() {
  const auto value = transport_.now();
  return system_clock::time_point(duration_cast<system_clock::duration>(seconds(value.seconds()) + nanoseconds(value.nanos())));
}
string Context::secret(const string &identifier) {
  bool declared = false;
  const auto catalog = plugin().declarations();
  for (const auto &section : catalog.sections()) {
    if (section.type() != SecretsDeclarations::descriptor()->full_name())
      continue;
    SecretsDeclarations declarations;
    if (!declarations.ParseFromString(section.payload()))
      throw invalid_argument("Read Secret Declarations: Rebuild the Plugin Catalog");
    for (const auto &secret : declarations.secrets())
      declared = declared || secret.metadata().identifier() == identifier;
  }
  if (!declared)
    throw invalid_argument("Read Secret: Declare the Secret Slot on This Plugin");
  SecretsGetRequest request;
  request.set_name(identifier);
  const Operation<SecretsGetRequest, SecretsGetResponse> operation("dokime.host.v1.Secrets/Get", VISIBILITY_PUBLIC);
  return transport_.operation(operation, request).get().secret();
}
std::uint64_t Context::append_record(const string &identity, const string &payload) {
  RecordsAppendRequest request;
  request.set_kind(identity);
  request.set_payload(payload);
  *request.mutable_source_at() = transport_.now();
  request.set_request_id(harmos::detail::request_id());
  const Operation<RecordsAppendRequest, RecordsAppendResponse> operation("dokime.host.v1.evidence.Records/Append", VISIBILITY_PUBLIC);
  return transport_.operation(operation, request).get().ordinal();
}
SettingValue Settings::encode(duration<double> value) {
  SettingValue result;
  auto encoded = detail::encode_duration(duration<double>(abs(value.count())));
  if (value.count() < 0) {
    encoded.set_seconds(-encoded.seconds());
    encoded.set_nanos(-encoded.nanos());
  }
  *result.mutable_duration() = move(encoded);
  return result;
}
} // namespace dokime::operations

namespace dokime::detail {
void PluginState::activate(operations::Context &context) {
  closed = false;
  snapshots.clear();
  if (settings.empty())
    return;
  const auto saved = context.transport_.settings();
  decltype(snapshots) loaded;
  for (const auto &[type, create] : settings)
    loaded.emplace(type, create(saved.value()));
  snapshots = move(loaded);
}
void PluginState::close() {
  closed = true;
  for (const auto &[_, entry] : resources) {
    lock_guard lock(entry->gate);
    entry->value.reset();
  }
  decltype(opened) retired;
  {
    lock_guard lock(opened_gate);
    opened.swap(retired);
  }
  exception_ptr failure;
  for (auto value = retired.rbegin(); value != retired.rend(); ++value) {
    try {
      (*value)->close();
    } catch (...) {
      if (!failure)
        failure = current_exception();
    }
  }
  snapshots.clear();
  if (failure)
    rethrow_exception(failure);
}
} // namespace dokime::detail

/* === Private Functions === */

/* === Entry Point === */
