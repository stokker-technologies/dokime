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
#include <map>
#include <memory>
#include <stdexcept>
#include <string>

// Third Party Imports
#include <grpcpp/grpcpp.h>

// Local Imports

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
struct Failure {
  std::string classification;
  std::string code;
  std::string message;
  std::string action;
  std::string operation;
  std::string plugin;
  std::shared_ptr<const Failure> cause;
  std::map<std::string, std::string> details;
  std::string diagnostics;
};
class Error : public std::runtime_error {
public:
  explicit Error(Failure failure) : std::runtime_error(failure.message + "; " + failure.action), failure(std::move(failure)) {}
  Failure failure;
};
namespace detail {
std::shared_ptr<const Failure> append_failure(Failure failure, std::shared_ptr<const Failure> tail);
Error rpc_error(const std::string &operation, const grpc::Status &status, const grpc::ClientContext &context);
} // namespace detail
} // namespace dokime

/* === Templates === */

/* === Public Declarations === */
