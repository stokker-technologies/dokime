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

#include "dokime/_generated/client.hpp"
#include "dokime/messages.hpp"

#include <filesystem>
#include <functional>
#include <map>

namespace dokime::client {
harmos::FileReference upload(Client &client, const std::filesystem::path &path,
                             const std::function<void(grpc::ClientContext &)> &configure = {});

class FrozenSchemaCache {
public:
  static std::string message_name(const ExecutionDetail &detail, const std::string &identity);
  std::shared_ptr<const google::protobuf::FileDescriptorSet> get(Client &client, const ExecutionDetail &detail,
                                                                 const std::function<void(grpc::ClientContext &)> &configure = {});

private:
  std::map<std::string, std::shared_ptr<const google::protobuf::FileDescriptorSet>> sets_;
};
} // namespace dokime::client
