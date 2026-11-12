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

#include "dokime/v1/client_backend.grpc.pb.h"
#include "dokime/v1/client_plugins.grpc.pb.h"
#include <grpcpp/grpcpp.h>
#include <string>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Backend final : public dokime::v1::Backend::Service {
  grpc::Status GetIdentity(grpc::ServerContext *context, const dokime::v1::GetBackendIdentity *,
                           dokime::v1::BackendIdentity *response) override {
    if (!context->client_metadata().contains("authorization"))
      return {grpc::StatusCode::UNAUTHENTICATED, "Pair This Client"};
    response->set_backend_id("bench");
    response->set_epoch("boot");
    return grpc::Status::OK;
  }
  grpc::Status WatchLogs(grpc::ServerContext *, const dokime::v1::WatchBackendLogs *,
                         grpc::ServerWriter<dokime::v1::BackendWatchLogsOutput> *output) override {
    dokime::v1::BackendWatchLogsOutput frame;
    frame.mutable_item()->set_level(dokime::v1::LOG_LEVEL_INFO);
    frame.mutable_item()->set_target("execution");
    output->Write(frame);
    frame.mutable_response();
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
