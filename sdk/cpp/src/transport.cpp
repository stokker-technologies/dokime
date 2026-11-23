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

// Local Imports
#include "dokime/detail/transport.hpp"
#include "harmos/messages.hpp"

using harmos::OperationError;
using harmos::model::Failure;
using std::move;
using std::string;
using std::to_string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail {
OperationError rpc_error(const string &operation, const grpc::Status &status, const grpc::ClientContext &context) {
  for (const auto &[key, bytes] : context.GetServerTrailingMetadata()) {
    if (string(key.data(), key.length()) != "grpc-status-details-bin")
      continue;
    Failure value;
    if (value.ParseFromArray(bytes.data(), static_cast<int>(bytes.length())) && value.classification()) {
      auto failure = move(value);
      (*failure.mutable_details())["grpc_status"] = to_string(static_cast<int>(status.error_code()));
      return OperationError(move(failure));
    }
  }
  Failure failure;
  switch (status.error_code()) {
  case grpc::StatusCode::CANCELLED:
    failure.set_classification(harmos::CLASSIFICATION_CANCELLED);
    break;
  case grpc::StatusCode::DEADLINE_EXCEEDED:
    failure.set_classification(harmos::CLASSIFICATION_TIMED_OUT);
    break;
  case grpc::StatusCode::UNAVAILABLE:
    failure.set_classification(harmos::CLASSIFICATION_TRANSIENT);
    break;
  case grpc::StatusCode::ABORTED:
  case grpc::StatusCode::FAILED_PRECONDITION:
  case grpc::StatusCode::PERMISSION_DENIED:
    failure.set_classification(harmos::CLASSIFICATION_REFUSED);
    break;
  default:
    failure.set_classification(harmos::CLASSIFICATION_PERMANENT);
    break;
  }
  failure.set_code(to_string(static_cast<int>(status.error_code())));
  failure.set_operation(operation);
  failure.set_message(status.error_message());
  failure.set_action("Check the Host Failure and Retry the Unchanged Request When Appropriate");
  failure.set_diagnostics(context.debug_error_string());
  (*failure.mutable_details())["grpc_status"] = to_string(static_cast<int>(status.error_code()));
  return OperationError(move(failure));
}
} // namespace dokime::detail

/* === Private Functions === */

/* === Entry Point === */
