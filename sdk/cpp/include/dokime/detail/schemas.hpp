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
#include <map>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */
namespace dokime::detail {
class FrozenSchemaCache {
public:
  static std::string message_name(const client::ExecutionDetail &detail, const std::string &identity);
  std::shared_ptr<const google::protobuf::FileDescriptorSet>
  get(Transport &client, const client::ExecutionDetail &detail,
      const std::function<std::shared_ptr<grpc::ClientContext>()> &configure = {});

private:
  std::map<std::string, std::shared_ptr<const google::protobuf::FileDescriptorSet>> sets_;
};
} // namespace dokime::detail

/* === Templates === */

/* === Public Declarations === */
