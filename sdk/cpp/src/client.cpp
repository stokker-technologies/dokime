/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/client.hpp"
#include "harmos/harmos.hpp"
#include <set>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
Client Client::connect(std::string address, std::shared_ptr<grpc::ChannelCredentials> credentials) {
  return Client(grpc::CreateChannel(std::move(address), std::move(credentials)));
}
std::shared_ptr<const google::protobuf::FileDescriptorSet>
FrozenSchemaCache::get(const Client &client, const ::dokime::client::v1::ExecutionDetail &detail,
                       const std::function<void(grpc::ClientContext &)> &configure) {
  if (!detail.has_schemas() || detail.schemas().digest().size() != 32)
    throw std::invalid_argument("Read Schemas: Supply a SHA-256 Digest");
  const auto &digest = detail.schemas().digest();
  if (const auto found = sets_.find(digest); found != sets_.end())
    return found->second;
  if (!detail.has_summary())
    throw std::invalid_argument("Read Schemas: Supply an Execution Identity");
  ::dokime::client::v1::executions::SchemasGetRequest request;
  request.set_name(detail.summary().execution_id());
  request.mutable_page()->set_page_size(1);
  if (detail.has_resolved_at())
    *request.mutable_at() = detail.resolved_at();
  auto types = std::make_shared<google::protobuf::FileDescriptorSet>();
  auto stub = client.stub<::dokime::client::v1::executions::Schemas>();
  std::set<std::string> tokens;
  while (true) {
    grpc::ClientContext context;
    if (configure)
      configure(context);
    ::dokime::client::v1::executions::SchemasGetResponse page;
    const auto status = stub->Get(&context, request, &page);
    if (!status.ok())
      throw RpcError(status);
    if (page.digest() != digest || !page.has_resolved_at() ||
        (request.has_at() && page.resolved_at().SerializeAsString() != request.at().SerializeAsString()))
      throw std::runtime_error("Read Schemas: Descriptor Digest or Read Cut Changed");
    if (!page.has_types())
      throw std::runtime_error("Read Schemas: Missing Descriptor Page");
    types->mutable_file()->MergeFrom(page.types().file());
    if (page.next_page_token().empty())
      break;
    if (!tokens.insert(page.next_page_token()).second)
      throw std::runtime_error("Read Schemas: Repeated Descriptor Page");
    *request.mutable_at() = page.resolved_at();
    request.mutable_page()->set_page_token(page.next_page_token());
  }
  if (harmos::detail::sha256(types->SerializeAsString()) != digest)
    throw std::runtime_error("Read Schemas: Frozen Descriptor Digest Does Not Match");
  sets_.emplace(digest, types);
  return types;
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
