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

#include "harmos/harmos.hpp"
#include <grpcpp/grpcpp.h>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime::detail {
harmos::OperationError rpc_error(const std::string &operation, const grpc::Status &status, const grpc::ClientContext &context);
} // namespace dokime::detail
