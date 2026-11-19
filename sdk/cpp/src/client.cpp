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
#include <fstream>
#include <limits>
#include <openssl/evp.h>
#include <random>
#include <set>
#include <sstream>

using google::protobuf::FileDescriptorSet;
using grpc::ClientContext;
using harmos::FileMetadata;
using harmos::FileReference;

using std::current_exception;
using std::ifstream;
using std::invalid_argument;
using std::make_shared;
using std::numeric_limits;
using std::ostringstream;
using std::rethrow_exception;
using std::runtime_error;
using std::set;
using std::shared_ptr;
using std::streamsize;
using std::string;
using std::throw_with_nested;
using std::to_string;
using std::unique_ptr;
using std::filesystem::path;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::client {
FileReference upload(detail::Transport &client, const path &path, const std::function<void(ClientContext &)> &configure) {
  ifstream source(path, std::ios::binary);
  if (!source)
    throw runtime_error("Upload File: Open Its Path");
  std::random_device random;
  ostringstream identity;
  for (int index = 0; index < 4; ++index)
    identity << std::hex << random();
  const auto request_id = identity.str();
  FileMetadata metadata;
  metadata.set_expected_length(std::filesystem::file_size(path));
  FilesUploadsCreateOptions opening;
  opening.request_id = request_id + "/create";
  opening.context = make_shared<ClientContext>();
  if (configure)
    configure(*opening.context);
  auto created = client.files.uploads.create(metadata, opening);
  if (!created.has_upload())
    throw runtime_error("Upload File: Missing Upload");
  auto upload = created.upload();
  try {
    if (!created.chunk_bytes() || created.chunk_bytes() >= created.message_bytes() ||
        created.chunk_bytes() > static_cast<uint64_t>(numeric_limits<streamsize>::max()))
      throw runtime_error("Upload File: Invalid Transfer Limits");
    unique_ptr<EVP_MD_CTX, decltype(&EVP_MD_CTX_free)> digest(EVP_MD_CTX_new(), EVP_MD_CTX_free);
    if (!digest || EVP_DigestInit_ex(digest.get(), EVP_sha256(), nullptr) != 1)
      throw runtime_error("Upload File: Initialize SHA-256");
    string buffer(created.chunk_bytes(), '\0');
    uint64_t offset = 0;
    while (source) {
      source.read(buffer.data(), static_cast<streamsize>(buffer.size()));
      const auto size = source.gcount();
      if (!size)
        break;
      FilesUploadsAppendOptions append;
      append.request_id = request_id + "/append/" + to_string(offset);
      append.context = make_shared<ClientContext>();
      if (configure)
        configure(*append.context);
      const auto reply = client.files.uploads.append(upload, offset, string(buffer.data(), static_cast<size_t>(size)), append);
      offset += static_cast<uint64_t>(size);
      if (!reply.has_upload() || reply.upload().accepted_length() != offset)
        throw runtime_error("Upload File: Invalid Accepted Offset");
      upload = reply.upload();
      if (EVP_DigestUpdate(digest.get(), buffer.data(), static_cast<size_t>(size)) != 1)
        throw runtime_error("Upload File: Hash Its Chunk");
    }
    if (!source.eof())
      throw runtime_error("Upload File: Read Its Path");
    unsigned char bytes[EVP_MAX_MD_SIZE];
    unsigned int length = 0;
    if (EVP_DigestFinal_ex(digest.get(), bytes, &length) != 1)
      throw runtime_error("Upload File: Finish SHA-256");
    const string checksum(reinterpret_cast<const char *>(bytes), length);
    FilesUploadsPublishOptions publish;
    publish.request_id = request_id + "/publish";
    publish.context = make_shared<ClientContext>();
    if (configure)
      configure(*publish.context);
    const auto reply = client.files.uploads.publish(upload, offset, checksum, publish);
    if (!reply.has_upload() || reply.upload().length() != offset || reply.upload().sha256() != checksum)
      throw runtime_error("Upload File: Published Length or SHA-256 Changed");
    return reply.upload();
  } catch (...) {
    const auto cause = current_exception();
    FilesUploadsDeleteOptions abandon;
    abandon.request_id = request_id + "/abandon";
    abandon.context = make_shared<ClientContext>();
    if (configure)
      configure(*abandon.context);
    try {
      client.files.uploads.delete_(upload, abandon);
    } catch (...) {
      throw_with_nested(runtime_error("Upload File: Abandon Failed Upload"));
    }
    rethrow_exception(cause);
  }
}

string FrozenSchemaCache::message_name(const ExecutionDetail &detail, const string &identity) {
  for (const auto &record : detail.schemas().records())
    if (record.record_type() == identity && !record.message_name().empty())
      return record.message_name();
  for (const auto &stream : detail.schemas().streams())
    if (stream.stream() == identity && !stream.message_name().empty())
      return stream.message_name();
  throw runtime_error("Decode Evidence: Missing Frozen Binding");
}
shared_ptr<const FileDescriptorSet> FrozenSchemaCache::get(detail::Transport &client, const ExecutionDetail &detail,
                                                           const std::function<void(ClientContext &)> &configure) {
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
    request.context = make_shared<ClientContext>();
    if (configure)
      configure(*request.context);
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
} // namespace dokime::client

/* === Private Functions === */

/* === Entry Point === */
