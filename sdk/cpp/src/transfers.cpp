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

// Third Party Imports

// Local Imports
#include "dokime/detail/transfers.hpp"
#include "dokime/detail/client.hpp"

using std::make_shared;
using std::move;
using std::optional;
using std::runtime_error;
using std::shared_ptr;
using std::string;
using std::to_string;
using std::uint64_t;
using std::uint8_t;
using std::vector;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail {
UploadOperations uploads(const shared_ptr<ClientState> &client) {
  return {[client](const harmos::FileMetadata &metadata, const string &identity) {
            client::FilesUploadsCreateOptions options;
            options.request_id = identity;
            options.context = client->context();
            return client->transport->files.uploads.create(metadata, options);
          },
          [client](const harmos::Upload &upload, uint64_t offset, const string &data, const string &identity) {
            client::FilesUploadsAppendOptions options;
            options.request_id = identity;
            options.context = client->context();
            return client->transport->files.uploads.append(upload, offset, data, options);
          },
          [client](const harmos::Upload &upload, uint64_t length, const string &digest, const string &identity) {
            client::FilesUploadsPublishOptions options;
            options.request_id = identity;
            options.context = client->context();
            return client->transport->files.uploads.publish(upload, length, digest, options);
          },
          [client](const harmos::Upload &upload, const string &identity) {
            client::FilesUploadsDeleteOptions options;
            options.request_id = identity;
            options.context = client->context();
            client->transport->files.uploads.delete_(upload, options);
          }};
}
harmos::FileReference upload_bytes(const UploadOperations &uploads, const vector<uint8_t> &data, const harmos::FileMetadata &metadata,
                                   const string &identity) {
  const auto created = uploads.create(metadata, identity + "/create");
  auto handle = created.upload();
  bool published = false;
  try {
    if (!created.has_upload() || !created.chunk_bytes() || created.chunk_bytes() >= created.message_bytes())
      throw runtime_error("Upload Content: Read a Valid Upload Handle and Host Transfer Limits");
    for (std::size_t offset = 0; offset < data.size();) {
      const auto length = std::min<std::size_t>(created.chunk_bytes(), data.size() - offset);
      const string chunk(data.begin() + offset, data.begin() + offset + length);
      const auto result = uploads.append(handle, offset, chunk, identity + "/append/" + to_string(offset));
      if (!result.has_upload() || result.upload().id() != handle.id() ||
          result.upload().history().SerializeAsString() != handle.history().SerializeAsString() ||
          result.upload().writer_generation() != handle.writer_generation() || result.upload().accepted_length() != offset + length)
        throw runtime_error("Upload Content: Preserve the Acknowledged Upload Fence and Offset");
      handle = result.upload();
      offset += length;
    }
    const auto digest = harmos::plugin::detail::sha256(string(data.begin(), data.end()));
    const auto result = uploads.publish(handle, data.size(), digest, identity + "/publish");
    published = true;
    if (!result.has_upload() || result.upload().length() != data.size() || result.upload().sha256() != digest)
      throw runtime_error("Upload Content: Read the Published Content Length and Digest");
    return result.upload();
  } catch (...) {
    const auto cause = std::current_exception();
    if (!published && created.has_upload()) {
      try {
        uploads.abandon(handle, identity + "/abandon");
      } catch (...) {
        std::throw_with_nested(runtime_error("Upload Content: Resolve the Failed Upload Cleanup"));
      }
    }
    std::rethrow_exception(cause);
  }
}
vector<uint8_t> download(const shared_ptr<ClientState> &client, const harmos::FileReference &file) {
  client::FilesGetOptions options;
  options.context = client->context();
  const auto limits = client->transport->files.get(file, options);
  if (!limits.chunk_bytes() || file.sha256().size() != 32)
    throw runtime_error("Download Evidence: Read Valid Transfer Bounds and a SHA-256 Digest");
  vector<uint8_t> data;
  while (data.size() < file.length()) {
    client::FilesReadOptions reading;
    reading.context = client->context();
    const auto page = client->transport->files.read(file, data.size(), limits.chunk_bytes(), reading);
    if (page.total_length() != file.length() || page.sha256() != file.sha256() || page.data().empty() ||
        page.data().size() > limits.chunk_bytes() || page.data().size() > file.length() - data.size())
      throw runtime_error("Download Evidence: Read the Retained File Identity and Requested Range");
    data.insert(data.end(), page.data().begin(), page.data().end());
  }
  if (harmos::plugin::detail::sha256(string(data.begin(), data.end())) != file.sha256())
    throw runtime_error("Download Evidence: Obtain Content Matching the Retained SHA-256 Digest");
  return data;
}
Artifact artifact_value(const ArtifactMetadata &value) {
  Artifact result;
  result.identifier = value.artifact_id();
  result.name = value.filename();
  result.mime = value.content_type();
  result.size_bytes = value.size_bytes();
  if (value.sha256().size() != 64)
    throw runtime_error("Read Artifact: Obtain a Complete SHA-256 Digest");
  for (std::size_t index = 0; index < value.sha256().size(); index += 2) {
    const auto pair = value.sha256().substr(index, 2);
    if (pair.find_first_not_of("0123456789abcdefABCDEF") != string::npos)
      throw runtime_error("Read Artifact: Obtain a Hexadecimal SHA-256 Digest");
    result.sha256.push_back(static_cast<uint8_t>(std::stoul(pair, nullptr, 16)));
  }
  if (!value.scope().cycle_id().empty())
    result.cycle = value.scope().cycle_id();
  if (!value.scope().execution_id().empty())
    result.execution = value.scope().execution_id();
  return result;
}
Artifact publish_artifact(const shared_ptr<ClientState> &client, const ArtifactScope &scope, const string &name,
                          const vector<uint8_t> &data, const string &mime, optional<string> request_id) {
  const auto identity = mutation_identity(move(request_id));
  harmos::FileMetadata metadata;
  metadata.set_scope(scope.execution_id().empty() ? "cycle/" + scope.cycle_id() : "execution/" + scope.execution_id());
  metadata.set_media_type(mime);
  metadata.set_expected_length(data.size());
  const auto file = upload_bytes(uploads(client), data, metadata, identity);
  client::ContentArtifactsCreateOptions options;
  options.scope = scope;
  options.request_id = identity + "/attach";
  options.context = client->context();
  const auto result = client->transport->content.artifacts.create(file, name, mime, options);
  return artifact_value(result.artifact());
}
} // namespace dokime::detail

/* === Private Functions === */

/* === Entry Point === */
