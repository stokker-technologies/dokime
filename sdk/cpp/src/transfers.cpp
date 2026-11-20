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

// Third Party Imports
#include "harmos/client_transport.hpp"

// Local Imports
#include "dokime/detail/transfers.hpp"

using std::invalid_argument;
using std::move;
using std::optional;
using std::runtime_error;
using std::string;
using std::to_string;
using std::uint8_t;
using std::vector;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail {
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
string mutation_identity(optional<string> request_id) {
  if (request_id && request_id->empty())
    throw invalid_argument("Mutate Resource: Supply a Nonempty Request Identity");
  return request_id ? *request_id : harmos::detail::request_id();
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
} // namespace dokime::detail

/* === Private Functions === */

/* === Entry Point === */
