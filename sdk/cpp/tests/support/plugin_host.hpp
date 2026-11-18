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
#include "harmos/plugin/v1/instance.grpc.pb.h"
#include <grpcpp/grpcpp.h>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class PluginHost {
public:
  explicit PluginHost(const std::string &address)
      : stub_(harmos::plugin::v1::Instance::NewStub(grpc::CreateChannel(address, grpc::InsecureChannelCredentials()))) {}
  harmos::plugin::v1::InstanceGetResponse describe() {
    grpc::ClientContext context;
    harmos::plugin::v1::InstanceGetRequest request;
    request.set_revision(1);
    harmos::plugin::v1::InstanceGetResponse response;
    auto status = stub_->Get(&context, request, &response);
    if (!status.ok())
      throw harmos::TransportError(status.error_code(), status.error_message());
    return response;
  }
  std::function<std::string(const std::string &, const std::string &)> child;
  template <class R, class Q> R invoke(std::string operation, Q request, std::string scope = {}) {
    grpc::ClientContext context;
    auto call = stub_->Invoke(&context);
    harmos::plugin::v1::InstanceInvokeRequest frame;
    frame.mutable_request()->set_operation(std::move(operation));
    frame.mutable_request()->set_invocation(std::to_string(++invocation_));
    frame.mutable_request()->set_attempt(1);
    frame.mutable_request()->set_scope(std::move(scope));
    frame.mutable_request()->set_payload(harmos::plugin::detail::encode(request));
    call->Write(frame);
    harmos::plugin::v1::InstanceInvokeResponse output;
    std::string payload;
    while (call->Read(&output)) {
      if (output.has_child()) {
        if (!child) {
          context.TryCancel();
          call->WritesDone();
          call->Finish();
          throw std::invalid_argument("Answer Child Call: Register the Test Host Handler");
        }
        try {
          harmos::plugin::v1::InstanceInvokeRequest reply;
          reply.mutable_child()->set_call(output.child().call());
          reply.mutable_child()->set_payload(child(output.child().operation(), output.child().payload()));
          call->Write(reply);
        } catch (...) {
          context.TryCancel();
          call->WritesDone();
          call->Finish();
          throw;
        }
      }
      if (output.has_response()) {
        call->WritesDone();
        if (output.response().has_failure()) {
          const auto failure = output.response().failure();
          call->Finish();
          throw harmos::Failure(failure);
        }
        payload = output.response().payload();
      }
    }
    auto status = call->Finish();
    if (!status.ok())
      throw harmos::TransportError(status.error_code(), status.error_message());
    return harmos::plugin::decode<R>(payload);
  }
  void stop() {
    grpc::ClientContext context;
    harmos::plugin::v1::InstanceStopResponse response;
    auto status = stub_->Stop(&context, {}, &response);
    if (!status.ok())
      throw harmos::TransportError(status.error_code(), status.error_message());
  }

private:
  std::unique_ptr<harmos::plugin::v1::Instance::Stub> stub_;
  unsigned invocation_ = 0;
};

/* === Templates === */

/* === Public Declarations === */
