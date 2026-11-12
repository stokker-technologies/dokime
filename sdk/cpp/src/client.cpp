/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/client.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
Client Client::connect(std::string address, std::shared_ptr<grpc::ChannelCredentials> credentials) {
  return Client(grpc::CreateChannel(std::move(address), std::move(credentials)));
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
