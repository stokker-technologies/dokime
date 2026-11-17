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

#include "dokime/host/v1/suites.grpc.pb.h"
#include "harmos/host/v1/plugins.grpc.pb.h"
#include "harmos/host/v1/status.grpc.pb.h"
#include "harmos/host/v1/status/logs.grpc.pb.h"
#include <grpcpp/grpcpp.h>
#include <string>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Backend final : public harmos::host::v1::Status::CallbackService {
  grpc::ServerUnaryReactor *Get(grpc::CallbackServerContext *context, const harmos::host::v1::StatusGetRequest *,
                                harmos::host::v1::StatusGetResponse *response) override {
    auto *reactor = context->DefaultReactor();
    if (!context->client_metadata().contains("authorization")) {
      reactor->Finish({grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"});
      return reactor;
    }
    response->mutable_status()->mutable_identity()->set_host_id("bench");
    response->mutable_status()->mutable_identity()->set_application_name("Dokime");
    reactor->Finish(grpc::Status::OK);
    return reactor;
  }
};

class LogRead final : public grpc::ServerWriteReactor<harmos::host::v1::status::LogsWatchResponse> {
  harmos::host::v1::status::LogsWatchResponse frame_;

public:
  explicit LogRead(bool authenticated) {
    if (!authenticated) {
      Finish({grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"});
      return;
    }
    frame_.add_events()->set_target("execution");
    StartWrite(&frame_);
  }
  void OnWriteDone(bool success) override {
    Finish(success ? grpc::Status::OK : grpc::Status(grpc::StatusCode::CANCELLED, "Read Logs: Client Closed"));
  }
  void OnDone() override {
    delete this;
  }
};
class Logs final : public harmos::host::v1::status::Logs::CallbackService {
  grpc::ServerWriteReactor<harmos::host::v1::status::LogsWatchResponse> *
  Watch(grpc::CallbackServerContext *context, const harmos::host::v1::status::LogsWatchRequest *) override {
    return new LogRead(context->client_metadata().contains("authorization"));
  }
};

class Suites final : public dokime::host::v1::Suites::Service {
  grpc::Status Get(grpc::ServerContext *, const dokime::host::v1::SuitesGetRequest *request,
                   dokime::host::v1::SuitesGetResponse *response) override {
    if (request->name() != "sample")
      return {grpc::StatusCode::NOT_FOUND, "Get Suite: Unknown Name"};
    response->mutable_value()->mutable_metadata()->set_identifier(request->name());
    return grpc::Status::OK;
  }
};

class Plugins final : public harmos::host::v1::Plugins::CallbackService {
  harmos::host::v1::PluginsListResponse entries_;

  grpc::ServerUnaryReactor *Create(grpc::CallbackServerContext *context, const harmos::host::v1::PluginsCreateRequest *request,
                                   harmos::host::v1::PluginsCreateResponse *response) override {
    auto *reactor = context->DefaultReactor();
    if (!request->placement().remote().url().starts_with("http://")) {
      reactor->Finish({grpc::StatusCode::INVALID_ARGUMENT, "Create Plugin: URL Is Required"});
      return reactor;
    }
    auto *entry = response->mutable_entry();
    entry->set_name(request->name());
    *entry->mutable_placement() = request->placement();
    *entry->mutable_value() = request->value();
    entry->set_state(harmos::host::v1::PLUGIN_STATE_CONNECTING);
    *entries_.add_entries() = *entry;
    reactor->Finish(grpc::Status::OK);
    return reactor;
  }

  grpc::ServerUnaryReactor *List(grpc::CallbackServerContext *context, const harmos::host::v1::PluginsListRequest *,
                                 harmos::host::v1::PluginsListResponse *response) override {
    auto *reactor = context->DefaultReactor();
    *response = entries_;
    reactor->Finish(grpc::Status::OK);
    return reactor;
  }
};

/* === Templates === */

/* === Public Declarations === */
