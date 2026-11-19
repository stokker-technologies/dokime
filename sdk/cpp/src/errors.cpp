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
#include "dokime/errors.hpp"
#include "harmos/messages.hpp"

using std::make_shared;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

namespace dokime::detail {
Failure project_failure(const harmos::FailureValue &value) {
  Failure result;
  switch (static_cast<int>(value.classification())) {
  case 1:
    result.classification = "Permanent";
    break;
  case 2:
    result.classification = "Transient";
    break;
  case 3:
    result.classification = "Cancelled";
    break;
  case 4:
    result.classification = "Timed Out";
    break;
  case 5:
    result.classification = "Refused";
    break;
  default:
    result.classification = "Unspecified";
    break;
  }
  result.code = value.code();
  result.message = value.message();
  result.action = value.action();
  result.operation = value.operation();
  result.plugin = value.plugin();
  if (value.has_cause())
    result.cause = make_shared<Failure>(project_failure(value.cause()));
  result.details.insert(value.details().begin(), value.details().end());
  result.diagnostics = value.diagnostics();
  return result;
}
std::shared_ptr<const Failure> append_failure(Failure failure, std::shared_ptr<const Failure> tail) {
  failure.cause = failure.cause ? append_failure(*failure.cause, std::move(tail)) : std::move(tail);
  return make_shared<Failure>(std::move(failure));
}
Error rpc_error(const string &operation, const grpc::Status &status, const grpc::ClientContext &context) {
  for (const auto &[key, bytes] : context.GetServerTrailingMetadata()) {
    if (string(key.data(), key.length()) != "grpc-status-details-bin")
      continue;
    harmos::FailureValue value;
    if (value.ParseFromArray(bytes.data(), static_cast<int>(bytes.length())) && value.classification()) {
      auto failure = project_failure(value);
      failure.details["grpc_status"] = std::to_string(static_cast<int>(status.error_code()));
      return Error(std::move(failure));
    }
  }
  Failure failure;
  switch (status.error_code()) {
  case grpc::StatusCode::CANCELLED:
    failure.classification = "Cancelled";
    break;
  case grpc::StatusCode::DEADLINE_EXCEEDED:
    failure.classification = "Timed Out";
    break;
  case grpc::StatusCode::UNAVAILABLE:
    failure.classification = "Transient";
    break;
  case grpc::StatusCode::ABORTED:
  case grpc::StatusCode::FAILED_PRECONDITION:
  case grpc::StatusCode::PERMISSION_DENIED:
    failure.classification = "Refused";
    break;
  default:
    failure.classification = "Permanent";
    break;
  }
  failure.code = std::to_string(static_cast<int>(status.error_code()));
  failure.operation = operation;
  failure.message = status.error_message();
  failure.action = "Check the Host Failure and Retry the Unchanged Request When Appropriate";
  failure.diagnostics = context.debug_error_string();
  failure.details["grpc_status"] = std::to_string(static_cast<int>(status.error_code()));
  return Error(std::move(failure));
}
} // namespace dokime::detail

/* === Entry Point === */
