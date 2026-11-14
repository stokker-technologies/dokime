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
#include <fstream>
#include <openssl/evp.h>
#include <random>
#include <sstream>
#include <limits>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
::harmos::storage::v1::FileReference Files::upload(const std::filesystem::path &path, const std::function<void(grpc::ClientContext &)> &configure) {
  std::ifstream source(path, std::ios::binary);
  if (!source) throw std::runtime_error("Upload File: Open Its Path");
  std::random_device random;
  std::ostringstream identity;
  for (int index = 0; index < 4; ++index) identity << std::hex << random();
  const auto request_id = identity.str();
  ::dokime::client::v1::files::UploadsCreateRequest opening;
  opening.mutable_metadata()->set_expected_length(std::filesystem::file_size(path));
  opening.set_request_id(request_id + "/create");
  grpc::ClientContext opening_context;
  if (configure) configure(opening_context);
  auto created = uploads.create(opening_context, opening);
  if (!created.has_production() || !created.production().has_session()) throw std::runtime_error("Upload File: Missing Writer Session");
  const auto session = created.production().session();
  try {
    if (!created.chunk_bytes() || created.chunk_bytes() >= created.message_bytes() || created.chunk_bytes() > static_cast<uint64_t>(std::numeric_limits<std::streamsize>::max())) throw std::runtime_error("Upload File: Invalid Transfer Limits");
    std::unique_ptr<EVP_MD_CTX, decltype(&EVP_MD_CTX_free)> digest(EVP_MD_CTX_new(), EVP_MD_CTX_free);
    if (!digest || EVP_DigestInit_ex(digest.get(), EVP_sha256(), nullptr) != 1) throw std::runtime_error("Upload File: Initialize SHA-256");
    std::string buffer(created.chunk_bytes(), '\0');
    uint64_t offset = 0;
    while (source) {
      source.read(buffer.data(), static_cast<std::streamsize>(buffer.size()));
      const auto size = source.gcount();
      if (!size) break;
      ::dokime::client::v1::files::UploadsAppendRequest append;
      *append.mutable_session() = session;
      append.set_offset(offset);
      append.set_data(buffer.data(), static_cast<size_t>(size));
      append.set_request_id(request_id + "/append/" + std::to_string(offset));
      grpc::ClientContext context;
      if (configure) configure(context);
      const auto reply = uploads.append(context, append);
      offset += static_cast<uint64_t>(size);
      if (!reply.has_production() || reply.production().length() != offset) throw std::runtime_error("Upload File: Invalid Accepted Offset");
      if (EVP_DigestUpdate(digest.get(), buffer.data(), static_cast<size_t>(size)) != 1) throw std::runtime_error("Upload File: Hash Its Chunk");
    }
    if (!source.eof()) throw std::runtime_error("Upload File: Read Its Path");
    unsigned char bytes[EVP_MAX_MD_SIZE];
    unsigned int length = 0;
    if (EVP_DigestFinal_ex(digest.get(), bytes, &length) != 1) throw std::runtime_error("Upload File: Finish SHA-256");
    ::dokime::client::v1::files::UploadsPublishRequest publish;
    *publish.mutable_session() = session;
    publish.set_length(offset);
    publish.set_sha256(bytes, length);
    publish.set_request_id(request_id + "/publish");
    grpc::ClientContext context;
    if (configure) configure(context);
    const auto reply = uploads.publish(context, publish);
    if (!reply.has_file() || reply.file().length() != offset || reply.file().sha256() != publish.sha256()) throw std::runtime_error("Upload File: Published Length or SHA-256 Changed");
    return reply.file();
  } catch (...) {
    ::dokime::client::v1::files::UploadsDeleteRequest abandon;
    *abandon.mutable_session() = session;
    abandon.set_request_id(request_id + "/abandon");
    grpc::ClientContext context;
    if (configure) configure(context);
    uploads.delete_(context, abandon);
    throw;
  }
}

Client Client::connect(std::string address, std::shared_ptr<grpc::ChannelCredentials> credentials) {
  return Client(grpc::CreateChannel(std::move(address), std::move(credentials)));
}
std::string FrozenSchemaCache::message_name(const ::dokime::client::v1::ExecutionDetail &detail, const std::string &identity) {
  for (const auto &record : detail.schemas().records()) if (record.record_type() == identity && !record.message_name().empty()) return record.message_name();
  for (const auto &stream : detail.schemas().streams()) if (stream.stream() == identity && !stream.message_name().empty()) return stream.message_name();
  throw std::runtime_error("Decode Evidence: Missing Frozen Binding");
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
    if (page.records_size() != detail.schemas().records_size() || page.streams_size() != detail.schemas().streams_size()) throw std::runtime_error("Read Schemas: Frozen Message Bindings Changed");
    for (int index = 0; index < page.records_size(); ++index) if (page.records(index).SerializeAsString() != detail.schemas().records(index).SerializeAsString()) throw std::runtime_error("Read Schemas: Frozen Record Binding Changed");
    for (int index = 0; index < page.streams_size(); ++index) if (page.streams(index).SerializeAsString() != detail.schemas().streams(index).SerializeAsString()) throw std::runtime_error("Read Schemas: Frozen Stream Binding Changed");
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
