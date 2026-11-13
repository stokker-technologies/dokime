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

#include "dokime/context/v1/backend.pb.h"
#include "dokime/context/v1/documents.pb.h"
#include "dokime/context/v1/evidence.pb.h"
#include "dokime/context/v1/evidence/logs.pb.h"
#include "dokime/context/v1/evidence/records.pb.h"
#include "dokime/context/v1/executions/progress.pb.h"
#include "dokime/context/v1/files.pb.h"
#include "dokime/context/v1/files/captures.pb.h"
#include "dokime/context/v1/prompts.pb.h"
#include "dokime/v1/catalog.pb.h"
#include "dokime/v1/host.pb.h"
#include "harmos/harmos.hpp"
#include <mutex>
#include <typeindex>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime {
class Context;
using Parameters = google::protobuf::Map<std::string, v1::Value>;
using Prompt = v1::Prompt;
using Record = v1::RecordContract;
template <class T> using Stream = harmos::Stream<T>;
template <class T> struct Observer {
  Stream<T> stream;
  std::function<T(Context &)> sample;
  google::protobuf::Duration interval;
  std::string interval_setting;
};

namespace detail {
struct Resources {
  std::mutex mutex;
  v1::TestCase test_case;
  std::unordered_map<std::type_index, std::shared_ptr<void>> values;
};
struct Executions {
  std::mutex mutex;
  std::unordered_map<std::string, std::shared_ptr<Resources>> values;
  std::shared_ptr<Resources> open(const std::string &execution);
  void close(const std::string &execution);
  void clear();
};
} // namespace detail

class Context {
public:
  Context(harmos::Context &transport, v1::TestCase test_case, std::string item, std::shared_ptr<detail::Resources> resources);
  const Parameters &parameters() const { return test_case_.parameters(); }
  const std::string &execution() const { return transport_.scope(); }
  const std::string &item() const { return item_; }
  std::uint32_t attempt() const { return transport_.attempt; }
  template <class Q, class R> R operation(const harmos::Operation<Q, R> &operation, Q request) {
    return transport_.call(operation, request).get();
  }
  template <class T> void set(std::shared_ptr<T> value) {
    std::lock_guard lock(resources_->mutex);
    resources_->values[typeid(T)] = std::move(value);
  }
  template <class T> std::shared_ptr<T> get() const {
    std::lock_guard lock(resources_->mutex);
    auto found = resources_->values.find(typeid(T));
    if (found == resources_->values.end())
      throw std::invalid_argument("Execution Resource Is Missing");
    return std::static_pointer_cast<T>(found->second);
  }
  void sleep(google::protobuf::Duration duration) { transport_.sleep(duration); }
  google::protobuf::Timestamp now() { return transport_.now(); }
  ::dokime::context::v1::PromptsCreateResponse prompt(::dokime::context::v1::PromptsCreateRequest request);
  v1::SavedSettings settings();
  v1::Secret secret(std::string identifier);
  ::dokime::context::v1::evidence::RecordsAppendResponse record(std::string kind, const google::protobuf::Message &value);
  ::dokime::context::v1::evidence::RecordsListResponse records(std::string kind, std::string step = {}, std::uint32_t limit = 0);
  ::dokime::context::v1::EvidenceStatisticsResponse statistics(std::string stream, std::string field = {}, std::string step = {});
  void progress(std::uint64_t completed, std::uint64_t total, std::string label = {});
  void log(v1::Severity severity, std::string message);
  v1::ArtifactReceipt artifact(v1::PublishArtifact request);
  void traceability(v1::TraceabilityEntry value);
  void document(::dokime::context::v1::DocumentsAppendPartsRequest request);
  static void declare_requirements(harmos::Plugin &plugin);

private:
  harmos::Context &transport_;
  v1::TestCase test_case_;
  std::string item_;
  std::shared_ptr<detail::Resources> resources_;
};
} // namespace dokime
