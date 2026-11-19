/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
#include "dokime/_generated/client.hpp"
#include "dokime/values.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime::detail {
struct UploadOperations {
  std::function<UploadsCreateResponse(const harmos::FileMetadata &, const std::string &)> create;
  std::function<UploadsAppendResponse(const harmos::Upload &, std::uint64_t, const std::string &, const std::string &)> append;
  std::function<UploadsPublishResponse(const harmos::Upload &, std::uint64_t, const std::string &, const std::string &)> publish;
  std::function<void(const harmos::Upload &, const std::string &)> abandon;
};
struct ClientState;
std::string mutation_identity(std::optional<std::string> request_id);
UploadOperations uploads(const std::shared_ptr<ClientState> &client);
harmos::FileReference upload_bytes(const UploadOperations &uploads, const std::vector<std::uint8_t> &data,
                                   const harmos::FileMetadata &metadata, const std::string &identity);
std::vector<std::uint8_t> download(const std::shared_ptr<ClientState> &client, const harmos::FileReference &file);
Artifact artifact_value(const ArtifactMetadata &value);
Artifact publish_artifact(const std::shared_ptr<ClientState> &client, const ArtifactScope &scope, const std::string &name,
                          const std::vector<std::uint8_t> &data, const std::string &mime, std::optional<std::string> request_id);
} // namespace dokime::detail

/* === Templates === */

/* === Public Declarations === */
