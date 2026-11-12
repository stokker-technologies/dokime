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

#include "journey.grpc.pb.h"
#include <grpcpp/grpcpp.h>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Backend final : public dokime::sdk::test::BackendJourney::Service {
  grpc::Status Identity(grpc::ServerContext *context, const dokime::v1::GetBackendIdentity *,
                        dokime::v1::BackendIdentity *response) override {
    if (!context->client_metadata().contains("authorization"))
      return {grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"};
    response->set_backend_id("bench");
    response->set_epoch("boot");
    return grpc::Status::OK;
  }
  grpc::Status Logs(grpc::ServerContext *, const dokime::v1::WatchBackendLogs *,
                    grpc::ServerWriter<dokime::v1::LogFrame> *output) override {
    dokime::v1::LogFrame frame;
    frame.set_level(dokime::v1::LOG_LEVEL_INFO);
    frame.set_target("execution");
    output->Write(frame);
    return grpc::Status::OK;
  }
};

/* === Templates === */

/* === Public Declarations === */
