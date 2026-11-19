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
#include <catch2/catch_test_macros.hpp>

// Local Imports
#include "dokime/detail/transfers.hpp"
#include "dokime/plugin.hpp"

using dokime::UploadsAppendResponse;
using dokime::UploadsCreateResponse;
using dokime::UploadsPublishResponse;
using dokime::detail::upload_bytes;
using dokime::detail::UploadOperations;
using harmos::FileMetadata;
using harmos::Upload;
using std::string;
using std::vector;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace {
class UploadHost {
public:
  string data;
  vector<string> identities;
  bool wrong_fence = false;
  bool wrong_digest = false;
  bool abandoned = false;
  UploadOperations operations() {
    return {[&](const FileMetadata &, const string &identity) {
              identities.push_back(identity);
              UploadsCreateResponse result;
              result.set_chunk_bytes(2);
              result.set_message_bytes(100);
              result.mutable_upload()->set_id("upload");
              result.mutable_upload()->mutable_history()->set_store("store");
              result.mutable_upload()->set_writer_generation(3);
              return result;
            },
            [&](const Upload &upload, std::uint64_t offset, const string &chunk, const string &identity) {
              identities.push_back(identity);
              if (offset != data.size())
                throw std::invalid_argument("Append Upload: Retain the Accepted Offset");
              data += chunk;
              UploadsAppendResponse result;
              *result.mutable_upload() = upload;
              result.mutable_upload()->set_accepted_length(data.size());
              if (wrong_fence)
                result.mutable_upload()->set_writer_generation(upload.writer_generation() + 1);
              return result;
            },
            [&](const Upload &, std::uint64_t length, const string &digest, const string &identity) {
              identities.push_back(identity);
              UploadsPublishResponse result;
              result.mutable_upload()->set_length(length);
              result.mutable_upload()->set_sha256(wrong_digest ? string(32, 'x') : digest);
              return result;
            },
            [&](const Upload &, const string &identity) {
              identities.push_back(identity);
              abandoned = true;
            }};
  }
};
} // namespace

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

TEST_CASE("Shared Uploads Preserve Fences, Offsets and Mutation Identities") {
  UploadHost host;
  FileMetadata metadata;
  metadata.set_expected_length(5);
  const auto file = upload_bytes(host.operations(), {'a', 'b', 'c', 'd', 'e'}, metadata, "upload-request");
  REQUIRE(host.data == "abcde");
  REQUIRE(file.length() == host.data.size());
  REQUIRE(file.sha256() == harmos::plugin::detail::sha256(host.data));
  REQUIRE(host.identities == vector<string>{"upload-request/create", "upload-request/append/0", "upload-request/append/2",
                                            "upload-request/append/4", "upload-request/publish"});
  REQUIRE_FALSE(host.abandoned);
}
TEST_CASE("Shared Uploads Abandon an Unpublished Fence Failure") {
  UploadHost host;
  host.wrong_fence = true;
  REQUIRE_THROWS_AS(upload_bytes(host.operations(), {'a', 'b', 'c'}, {}, "upload-request"), std::runtime_error);
  REQUIRE(host.abandoned);
  REQUIRE(host.identities.back() == "upload-request/abandon");
  REQUIRE(host.data == "ab");
}
TEST_CASE("Shared Uploads Refuse a Changed Published Digest Without Deleting It") {
  UploadHost host;
  host.wrong_digest = true;
  REQUIRE_THROWS_AS(upload_bytes(host.operations(), {'a'}, {}, "upload-request"), std::runtime_error);
  REQUIRE_FALSE(host.abandoned);
}
