/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <algorithm>
#include <cmath>
#include <fstream>
#include <regex>
#include <set>

// Third Party Imports
#include <google/protobuf/struct.pb.h>
#include <google/protobuf/util/json_util.h>

// Local Imports
#include "dokime/detail/client.hpp"

using google::protobuf::Struct;
using google::protobuf::util::JsonStringToMessage;
using grpc::Channel;
using grpc::ChannelCredentials;
using grpc::CreateCustomChannel;
using grpc::InsecureChannelCredentials;
using grpc::SslCredentials;
using grpc::SslCredentialsOptions;
using harmos::OperationError;
using harmos::model::Failure;
using std::invalid_argument;
using std::make_shared;
using std::move;
using std::runtime_error;
using std::shared_ptr;
using std::string;
using std::vector;
using std::chrono::duration_cast;
using std::chrono::system_clock;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
Client connect(ConnectionSettings settings) {
  if (!std::isfinite(settings.connect_timeout.count()) || settings.connect_timeout.count() <= 0)
    throw invalid_argument("Connect: Supply a Positive Finite Connection Timeout");
  const auto values = detail::configuration::load();
  const auto selected = detail::configuration::resolve("DOKIME_REMOTE", settings.remote, values);
  vector<string> endpoints{selected};
  if (selected.ends_with(".pair")) {
    std::ifstream input(selected, std::ios::binary);
    if (!input)
      throw invalid_argument("Connect: Open a Readable Pairing File Exported by the Intended Host");
    string content(1048577, '\0');
    input.read(content.data(), static_cast<std::streamsize>(content.size()));
    content.resize(static_cast<std::size_t>(input.gcount()));
    if (content.size() > 1048576 || input.bad())
      throw invalid_argument("Connect: Supply a Readable Pairing File of at Most 1 MiB");
    Struct pairing;
    const auto parsed = JsonStringToMessage(content, &pairing);
    const auto &fields = pairing.fields();
    const std::set<string> required{"version", "endpoints", "backend_identifier", "label", "certificate_pem", "access_key"};
    std::set<string> actual;
    for (const auto &[key, value] : fields) {
      (void)value;
      actual.insert(key);
    }
    if (!parsed.ok() || actual != required || !fields.at("version").has_number_value() || fields.at("version").number_value() != 1)
      throw invalid_argument("Connect: Export a Valid Version 1 Pairing File");
    const auto &identity = fields.at("backend_identifier");
    const auto &label = fields.at("label");
    if (!identity.has_string_value() || identity.string_value().empty() || identity.string_value().size() > 128 ||
        std::any_of(identity.string_value().begin(), identity.string_value().end(), [](unsigned char value) {
          return value <= 32;
        }))
      throw invalid_argument("Connect: Export a Pairing With a Valid Backend Identifier");
    if (!label.has_string_value() || label.string_value().empty() || label.string_value().size() > 256 ||
        std::all_of(label.string_value().begin(), label.string_value().end(),
                    [](unsigned char value) {
                      return value <= 32;
                    }) ||
        std::any_of(label.string_value().begin(), label.string_value().end(), [](unsigned char value) {
          return value < 32;
        }))
      throw invalid_argument("Connect: Export a Pairing With a Valid Host Label");
    settings.expected_backend_id = identity.string_value();
    const auto &certificate = fields.at("certificate_pem");
    const auto &key = fields.at("access_key");
    if ((!certificate.has_null_value() && !certificate.has_string_value()) || (!key.has_null_value() && !key.has_string_value()))
      throw invalid_argument("Connect: Export a Pairing With Valid Credentials");
    settings.certificate_pem = certificate.has_string_value() ? std::optional<string>(certificate.string_value()) : std::nullopt;
    settings.access_key = key.has_string_value() ? std::optional<string>(key.string_value()) : std::nullopt;
    if (settings.access_key && (settings.access_key->size() < 32 || settings.access_key->size() > 4096 ||
                                std::any_of(settings.access_key->begin(), settings.access_key->end(), [](unsigned char value) {
                                  return value < 33 || value > 126;
                                })))
      throw invalid_argument("Connect: Export a Pairing With a Valid Access Key");
    if (!fields.at("endpoints").has_list_value())
      throw invalid_argument("Connect: Export a Pairing With an Ordered Endpoint List");
    endpoints.clear();
    std::set<string> unique;
    for (const auto &endpoint : fields.at("endpoints").list_value().values()) {
      if (!endpoint.has_string_value() || !unique.insert(endpoint.string_value()).second)
        throw invalid_argument("Connect: Export a Pairing With Distinct Endpoint Strings");
      endpoints.push_back(endpoint.string_value());
    }
    if (endpoints.empty() || endpoints.size() > 32)
      throw invalid_argument("Connect: Export a Pairing With Between 1 and 32 Endpoints");
  } else if (!selected.starts_with("http://") || (settings.certificate_pem && !settings.certificate_pem->empty())) {
    throw invalid_argument("Connect: Use a Direct HTTP Origin or a Pinned HTTPS Pairing File");
  }
  const std::regex origin(R"(^(http|https)://(\[[0-9a-fA-F:.]+\]|[^\s/:@?#]+):([0-9]+)$)");
  vector<Failure> failures;
  for (const auto &endpoint : endpoints) {
    try {
      if (std::any_of(endpoint.begin(), endpoint.end(), [](unsigned char value) {
            return value <= 32 || value == 127;
          }))
        throw invalid_argument("Remove Whitespace and Control Characters From the Endpoint");
      std::smatch match;
      if (!std::regex_match(endpoint, match, origin))
        throw invalid_argument("Supply an HTTP(S) Origin With an Explicit Port");
      const auto port = std::stoul(match[3].str());
      if (!port || port > 65535)
        throw invalid_argument("Supply a Port Between 1 and 65535");
      shared_ptr<ChannelCredentials> credentials;
      if (match[1] == "https") {
        if (!settings.certificate_pem || settings.certificate_pem->empty())
          throw invalid_argument("Export a Pairing With the Pinned HTTPS Certificate");
        SslCredentialsOptions tls;
        tls.pem_root_certs = *settings.certificate_pem;
        credentials = SslCredentials(tls);
      } else {
        if (settings.certificate_pem && !settings.certificate_pem->empty())
          throw invalid_argument("Select HTTPS for a Pairing With a Pinned Certificate");
        credentials = InsecureChannelCredentials();
      }
      grpc::ChannelArguments arguments;
      const auto maximum = harmos::transfer_message_bytes({});
      arguments.SetMaxReceiveMessageSize(maximum);
      arguments.SetMaxSendMessageSize(maximum);
      const auto channel = CreateCustomChannel(match[2].str() + ":" + match[3].str(), credentials, arguments);
      const auto deadline = system_clock::now() + duration_cast<system_clock::duration>(settings.connect_timeout);
      auto state = make_shared<detail::ClientState>(channel);
      state->access_key = settings.access_key.value_or("");
      if (settings.expected_backend_id) {
        client::StatusGetOptions options;
        options.context = state->context();
        options.context->set_deadline(deadline);
        const auto result = state->transport->status.get(options);
        if (result.status().identity().host_id() != *settings.expected_backend_id)
          throw invalid_argument("Export a Pairing From the Intended Host; Its Identity Differs");
      }
      if (!channel->WaitForConnected(deadline))
        throw runtime_error("Check the Host Listener; the Connection Budget Expired");
      return Client(move(state));
    } catch (const OperationError &error) {
      auto failure = error.value;
      (*failure.mutable_details())["connection_endpoint"] = endpoint;
      failures.push_back(move(failure));
    } catch (const std::exception &error) {
      Failure failure;
      failure.set_classification(dynamic_cast<const invalid_argument *>(&error) ? harmos::CLASSIFICATION_REFUSED
                                                                                : harmos::CLASSIFICATION_TIMED_OUT);
      failure.set_code("connection_failed");
      failure.set_message(error.what());
      failure.set_action("Check the Host Listener or Export a New Pairing");
      failure.set_operation("connect");
      (*failure.mutable_details())["connection_endpoint"] = endpoint;
      failures.push_back(move(failure));
    }
  }
  Failure failure;
  failure.set_classification(failures.front().classification());
  failure.set_code("connection_failed");
  failure.set_action("Check the Host Listener or Export a New Pairing");
  failure.set_operation("connect");
  for (const auto &cause : failures) {
    failure.set_message(failure.message() + cause.details().at("connection_endpoint") + ": " + cause.message() + "; ");
    if (cause.classification() != failure.classification())
      failure.set_classification(harmos::CLASSIFICATION_TRANSIENT);
  }
  auto *tail = &failure;
  for (const auto &cause : failures) {
    *tail->mutable_cause() = cause;
    tail = tail->mutable_cause();
    while (tail->has_cause())
      tail = tail->mutable_cause();
  }
  throw OperationError(move(failure));
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
