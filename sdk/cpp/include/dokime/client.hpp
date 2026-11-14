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

#include "dokime/client/v1/plugins.grpc.pb.h"
#include "dokime/resources.hpp"
#include "harmos/client.hpp"
#include <filesystem>
#include <functional>
#include <grpcpp/grpcpp.h>
#include <map>
#include <stdexcept>
#include <string>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime {
class RpcError : public std::runtime_error {
public:
  explicit RpcError(grpc::Status status) : std::runtime_error(status.error_message()), status_(std::move(status)) {}
  const grpc::Status &status() const { return status_; }

private:
  grpc::Status status_;
};

class Files : public resources::FilesResource {
public:
  explicit Files(std::shared_ptr<grpc::Channel> channel) : resources::FilesResource(std::move(channel)) {}
  ::harmos::storage::v1::FileReference upload(const std::filesystem::path &path,
                                              const std::function<void(grpc::ClientContext &)> &configure = {});
};

class Client : public resources::Client {
public:
  explicit Client(std::shared_ptr<grpc::Channel> channel) : resources::Client(channel), files(channel), channel_(std::move(channel)) {
    if (!channel_)
      throw std::invalid_argument("Client Channel Is Required");
  }
  Files files;
  static Client connect(std::string address, std::shared_ptr<grpc::ChannelCredentials> credentials = grpc::InsecureChannelCredentials());
  template <class Service> auto stub() const { return Service::NewStub(channel_); }
  const std::shared_ptr<grpc::Channel> &channel() const { return channel_; }
  // Explicit Context Preserves Caller Metadata, Cancellation and Deadline Overrides.
  template <class Q, class R, class Stub>
  R unary(Stub &stub, grpc::Status (Stub::*method)(grpc::ClientContext *, const Q &, R *), Q request, grpc::ClientContext &context) const {
    R response;
    auto status = (stub.*method)(&context, request, &response);
    if (!status.ok())
      throw RpcError(std::move(status));
    return response;
  }

private:
  std::shared_ptr<grpc::Channel> channel_;
};
// Caller-Owned Cache; Confine It to One Thread or Synchronize Access.
class FrozenSchemaCache {
public:
  static std::string message_name(const ::dokime::client::v1::ExecutionDetail &detail, const std::string &identity);
  std::shared_ptr<const google::protobuf::FileDescriptorSet> get(const Client &client, const ::dokime::client::v1::ExecutionDetail &detail,
                                                                 const std::function<void(grpc::ClientContext &)> &configure = {});

private:
  std::map<std::string, std::shared_ptr<const google::protobuf::FileDescriptorSet>> sets_;
};
} // namespace dokime
