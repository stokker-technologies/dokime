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

class Backend final : public harmos::v1::Status::Service {
  grpc::Status Get(grpc::ServerContext *context, const harmos::v1::StatusGetRequest *, harmos::v1::StatusGetResponse *response) override {
    if (!context->client_metadata().contains("authorization"))
      return {grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"};
    response->mutable_status()->mutable_identity()->set_host_id("bench");
    response->mutable_status()->mutable_identity()->set_application_name("Dokime");
    return grpc::Status::OK;
  }
};

class Logs final : public harmos::v1::status::Logs::Service {
  grpc::Status Watch(grpc::ServerContext *context, const harmos::v1::status::LogsWatchRequest *,
                     grpc::ServerWriter<harmos::v1::status::LogsWatchResponse> *output) override {
    if (!context->client_metadata().contains("authorization"))
      return {grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"};
    harmos::v1::status::LogsWatchResponse frame;
    frame.add_events()->set_target("execution");
    output->Write(frame);
    return grpc::Status::OK;
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
