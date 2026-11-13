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

#include "dokime/v1/client_plugins.grpc.pb.h"
#include "harmos/v1/status.grpc.pb.h"
#include "harmos/v1/status/logs.grpc.pb.h"
#include <grpcpp/grpcpp.h>
#include <string>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Backend final : public harmos::v1::Status::CallbackService {
  grpc::ServerUnaryReactor *Get(grpc::CallbackServerContext *context, const harmos::v1::StatusGetRequest *,
                                harmos::v1::StatusGetResponse *response) override {
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

class LogRead final : public grpc::ServerWriteReactor<harmos::v1::status::LogsWatchResponse> {
  harmos::v1::status::LogsWatchResponse frame_;

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
  void OnDone() override { delete this; }
};
class Logs final : public harmos::v1::status::Logs::CallbackService {
  grpc::ServerWriteReactor<harmos::v1::status::LogsWatchResponse> *Watch(grpc::CallbackServerContext *context,
                                                                         const harmos::v1::status::LogsWatchRequest *) override {
    return new LogRead(context->client_metadata().contains("authorization"));
  }
};

class Plugins final : public dokime::v1::Plugins::Service {
  dokime::v1::PluginEntries entries_;

  grpc::Status Add(grpc::ServerContext *, const dokime::v1::AddRemotePlugin *request, dokime::v1::Plugin *response) override {
    if (!request->url().starts_with("http://"))
      return {grpc::StatusCode::INVALID_ARGUMENT, "Plugin URL Is Required"};
    response->set_entry("entry-" + std::to_string(entries_.plugins_size()));
    response->mutable_remote()->set_url(request->url());
    response->set_state(dokime::v1::PLUGIN_STATE_CONNECTING);
    *entries_.add_plugins() = *response;
    return grpc::Status::OK;
  }

  grpc::Status List(grpc::ServerContext *, const dokime::v1::ListPlugins *, dokime::v1::PluginEntries *response) override {
    *response = entries_;
    return grpc::Status::OK;
  }
};

/* === Templates === */

/* === Public Declarations === */
