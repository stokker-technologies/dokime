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
#include "dokime/host/v1/secrets.pb.h"
#include "dokime/plugin.hpp"
#include "harmos/host/v1/plugins/settings.pb.h"
#include <cmath>

using dokime::host::v1::SecretsGetRequest;
using dokime::host::v1::SecretsGetResponse;
using google::protobuf::Duration;
using harmos::host::v1::plugins::SettingsListRequest;
using harmos::host::v1::plugins::SettingsListResponse;
using harmos::plugin::Operation;
using std::invalid_argument;
using std::map;
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
  if (!std::isfinite(duration.count()) || duration.count() < 0 || duration.count() > 315576000000.0)
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
  const Operation<SecretsGetRequest, SecretsGetResponse> operation("dokime.host.v1.Secrets/Get", harmos::VISIBILITY_PUBLIC);
  return transport_.call(operation, request).get().secret();
}
map<string, SettingValue> Context::settings() {
  const Operation<SettingsListRequest, SettingsListResponse> operation("harmos.plugin.v1.plugins.Settings/List", harmos::VISIBILITY_PUBLIC);
  const auto result = transport_.call(operation, SettingsListRequest{}).get();
  map<string, SettingValue> values;
  for (const auto &[name, value] : result.value().values()) {
    SettingValue converted;
    if (value.has_flag())
      converted.value = value.flag();
    else if (value.has_integer())
      converted.value = value.integer();
    else if (value.has_number())
      converted.value = value.number();
    else if (value.has_text())
      converted.value = value.text();
    else if (value.has_duration())
      converted.value = duration<double>(value.duration().seconds()) + duration<double>(nanoseconds(value.duration().nanos()));
    else
      throw invalid_argument("Read Plugin Settings: Store a Value for Every Declared Setting");
    values.emplace(name, std::move(converted));
  }
  return values;
}
} // namespace dokime::operations

/* === Private Functions === */

/* === Entry Point === */
