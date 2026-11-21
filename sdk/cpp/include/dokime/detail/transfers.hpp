/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports
#include <functional>
#include <memory>

// Third Party Imports
#include "harmos/harmos.hpp"

// Local Imports
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
std::string mutation_identity(std::optional<std::string> request_id);
harmos::FileReference upload_bytes(const UploadOperations &uploads, const std::vector<std::uint8_t> &data,
                                   const harmos::FileMetadata &metadata, const std::string &identity);
Artifact artifact_value(const ArtifactMetadata &value);
} // namespace dokime::detail

/* === Templates === */

/* === Public Declarations === */
