/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <functional>
#include <set>
#include <string_view>

// Third Party Imports
#include "harmos/harmos.hpp"

// Local Imports
#include "dokime/detail/client.hpp"
#include "dokime/detail/schemas.hpp"

using dokime::client::ExecutionDetail;
using dokime::client::ExecutionsGetOptions;
using dokime::client::ExecutionsSchemasGetOptions;
using dokime::detail::Transport;
using google::protobuf::FileDescriptorSet;
using grpc::ClientContext;

using std::function;
using std::invalid_argument;
using std::make_shared;
using std::runtime_error;
using std::set;
using std::shared_ptr;
using std::string;
using std::string_view;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail {
string FrozenSchemaCache::message_name(const ExecutionDetail &detail, const string &identity) {
  for (const auto &record : detail.schemas().records())
    if (record.record_type() == identity && !record.message_name().empty())
      return record.message_name();
  for (const auto &stream : detail.schemas().streams())
    if (stream.stream() == identity && !stream.message_name().empty())
      return stream.message_name();
  throw runtime_error("Decode Evidence: Missing Frozen Binding");
}
shared_ptr<const FileDescriptorSet> FrozenSchemaCache::get(Transport &client, const ExecutionDetail &detail,
                                                           const function<shared_ptr<ClientContext>()> &configure) {
  if (!detail.has_schemas() || detail.schemas().digest().size() != 32)
    throw invalid_argument("Read Schemas: Supply a SHA-256 Digest");
  const auto &digest = detail.schemas().digest();
  if (const auto found = sets_.find(digest); found != sets_.end())
    return found->second;
  if (!detail.has_summary())
    throw invalid_argument("Read Schemas: Supply an Execution Identity");
  ExecutionsSchemasGetOptions request;
  request.page.emplace();
  request.page->set_page_size(1);
  if (detail.has_resolved_at())
    request.at = detail.resolved_at();
  auto types = make_shared<FileDescriptorSet>();
  set<string> tokens;
  while (true) {
    request.context = configure ? configure() : make_shared<ClientContext>();
    const auto page = client.executions.schemas.get(detail.summary().execution_id(), request);
    if (page.digest() != digest || !page.has_resolved_at() ||
        (request.at.has_value() && page.resolved_at().SerializeAsString() != request.at->SerializeAsString()))
      throw runtime_error("Read Schemas: Descriptor Digest or Read Cut Changed");
    if (!page.has_types())
      throw runtime_error("Read Schemas: Missing Descriptor Page");
    if (page.records_size() != detail.schemas().records_size() || page.streams_size() != detail.schemas().streams_size())
      throw runtime_error("Read Schemas: Frozen Message Bindings Changed");
    for (int index = 0; index < page.records_size(); ++index)
      if (page.records(index).SerializeAsString() != detail.schemas().records(index).SerializeAsString())
        throw runtime_error("Read Schemas: Frozen Record Binding Changed");
    for (int index = 0; index < page.streams_size(); ++index)
      if (page.streams(index).SerializeAsString() != detail.schemas().streams(index).SerializeAsString())
        throw runtime_error("Read Schemas: Frozen Stream Binding Changed");
    types->mutable_file()->MergeFrom(page.types().file());
    if (page.next_page_token().empty())
      break;
    if (!tokens.insert(page.next_page_token()).second)
      throw runtime_error("Read Schemas: Repeated Descriptor Page");
    request.at = page.resolved_at();
    request.page->set_page_token(page.next_page_token());
  }
  if (harmos::plugin::detail::sha256(types->SerializeAsString()) != digest)
    throw runtime_error("Read Schemas: Frozen Descriptor Digest Does Not Match");
  sets_.emplace(digest, types);
  return types;
}
void validate_schema(const shared_ptr<ExecutionState> &execution, const string &identity, string_view message) {
  auto &client = *execution->client;
  client.context();
  if (!client.schemas)
    client.schemas = make_shared<FrozenSchemaCache>();
  if (!execution->detail.has_schemas()) {
    ExecutionsGetOptions options;
    options.context = client.context();
    execution->observe(client.transport->executions.get(execution->detail.summary().execution_id(), options).execution());
  }
  const auto configure = [&] {
    return client.context();
  };
  client.schemas->get(*client.transport, execution->detail, configure);
  const auto binding = FrozenSchemaCache::message_name(execution->detail, identity);
  if (binding != message)
    throw invalid_argument("Decode Evidence: Select a Row Type Matching the Frozen Message Binding");
}
} // namespace dokime::detail

/* === Private Functions === */

/* === Entry Point === */
