/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports
#include <optional>
#include <stdexcept>
#include <string>

// Third Party Imports

// Local Imports
#include "dokime/detail/client.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
template <class T> class EvidencePages {
public:
  explicit EvidencePages(std::function<std::pair<std::vector<T>, std::string>(const std::string &)> fetch) : fetch_(std::move(fetch)) {}
  EvidencePages(EvidencePages &&) = default;
  EvidencePages(const EvidencePages &) = delete;
  void close() {
    finished_ = true;
    fetch_ = {};
  }
  std::optional<std::vector<T>> next() {
    if (finished_)
      return {};
    try {
      auto result = fetch_(token_);
      token_ = std::move(result.second);
      if (token_.empty())
        close();
      else if (!tokens_.insert(token_).second)
        throw std::runtime_error("Read Evidence Pages: Continue With a New Page Token");
      return std::move(result.first);
    } catch (...) {
      close();
      throw;
    }
  }

private:
  std::function<std::pair<std::vector<T>, std::string>(const std::string &)> fetch_;
  std::string token_;
  std::set<std::string> tokens_;
  bool finished_ = false;
};
template <class T> class RecordsQuery {
public:
  std::uint64_t count() {
    return page(1, "").count();
  }
  T last() {
    const auto result = page(1, "");
    if (!result.has_last())
      throw std::invalid_argument("Read Last Record: Append a Matching Record Before Reading It");
    return decode(result.last()).value;
  }
  std::vector<RecordRow<T>> read(int limit = 0) {
    if (limit < 0)
      throw std::invalid_argument("Read Records: Supply a Nonnegative Limit");
    auto stream = pages(limit);
    std::vector<RecordRow<T>> rows;
    while (auto page = stream.next()) {
      for (auto &row : *page) {
        rows.push_back(std::move(row));
        if (limit && rows.size() == static_cast<std::size_t>(limit))
          return rows;
      }
      if (!limit)
        return rows;
    }
    return rows;
  }
  EvidencePages<RecordRow<T>> pages(int page_size = 0) {
    if (page_size < 0)
      throw std::invalid_argument("Read Record Pages: Supply a Nonnegative Page Size");
    return EvidencePages<RecordRow<T>>([query = *this, page_size](const std::string &token) mutable {
      const auto result = query.page(page_size, token);
      std::vector<RecordRow<T>> rows;
      for (const auto &row : result.records())
        rows.push_back(query.decode(row));
      return std::make_pair(std::move(rows), result.next_page_token());
    });
  }

private:
  friend class Execution;
  RecordsQuery(std::shared_ptr<detail::ExecutionState> execution, std::string name, std::string step)
      : execution_(std::move(execution)), name_(std::move(name)), step_(std::move(step)) {
    if (name_.empty())
      throw std::invalid_argument("Read Records: Supply a Nonempty Record Type");
  }
  RecordsListResponse page(int size, const std::string &token) {
    const auto message = templates::detail::payload_message<T>();
    detail::validate_schema(execution_, name_, message.GetDescriptor()->full_name());
    client::EvidenceRecordsListOptions options;
    options.scope.emplace().set_execution(execution_->detail.summary().execution_id());
    options.at = at_;
    options.page.emplace().set_page_size(size);
    options.page->set_page_token(token);
    options.context = execution_->client->context();
    const auto result = execution_->client->transport->evidence.records.list(name_, {}, {}, step_, {}, options);
    if (at_ && at_->SerializeAsString() != result.resolved_at().SerializeAsString())
      throw std::runtime_error("Read Records: Retain the Original Evidence Cut");
    at_ = result.resolved_at();
    return result;
  }
  RecordRow<T> decode(const CapturedRecord &row) const {
    if (row.record_type() != name_ || row.scope().execution_id() != execution_->detail.summary().execution_id())
      throw std::runtime_error("Decode Record: Select the Requested Execution and Record Type");
    return {row.record_id(),
            row.sequence(),
            detail::instant(row.timestamp()),
            detail::instant(row.received_at()),
            row.record_type(),
            row.schema_version(),
            row.source(),
            templates::detail::decode<T>(row.payload())};
  }
  std::shared_ptr<detail::ExecutionState> execution_;
  std::string name_;
  std::string step_;
  std::optional<harmos::ReadCut> at_;
};
template <class T> class SamplesQuery {
public:
  EvidencePages<SampleRow<T>> pages(int page_size = 0) {
    if (page_size < 0)
      throw std::invalid_argument("Read Sample Pages: Supply a Nonnegative Page Size");
    return EvidencePages<SampleRow<T>>([query = *this, page_size](const std::string &token) mutable {
      const auto result = query.page(page_size, token);
      std::vector<SampleRow<T>> rows;
      for (const auto &sample : result.samples())
        rows.push_back(query.decode(sample));
      return std::make_pair(std::move(rows), result.next_page_token());
    });
  }
  std::vector<SampleRow<T>> collect(std::size_t max_rows = 100000) {
    if (!max_rows)
      throw std::invalid_argument("Collect Samples: Supply a Positive Row Bound");
    std::vector<SampleRow<T>> rows;
    auto stream = pages();
    while (auto page = stream.next()) {
      if (page->size() > max_rows - rows.size())
        throw std::invalid_argument("Collect Samples: Narrow the Selection or Increase max_rows");
      rows.insert(rows.end(), std::make_move_iterator(page->begin()), std::make_move_iterator(page->end()));
    }
    return rows;
  }

private:
  friend class Execution;
  SamplesQuery(std::shared_ptr<detail::ExecutionState> execution, std::string name, std::vector<std::string> fields,
               std::pair<std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>,
                         std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>>
                   between)
      : execution_(std::move(execution)), name_(std::move(name)), fields_(std::move(fields)) {
    if (name_.empty())
      throw std::invalid_argument("Read Samples: Supply a Nonempty Stream");
    if (between.first)
      *between_.mutable_start() = detail::timestamp(*between.first);
    if (between.second)
      *between_.mutable_end() = detail::timestamp(*between.second);
  }
  client::SamplesListResponse page(int size, const std::string &token) {
    const auto message = templates::detail::payload_message<T>();
    detail::validate_schema(execution_, name_, message.GetDescriptor()->full_name());
    client::EvidenceSamplesListOptions options;
    options.page.emplace().set_page_size(size);
    options.page->set_page_token(token);
    options.at = at_;
    options.context = execution_->client->context();
    EvidenceScope scope;
    scope.set_execution_id(execution_->detail.summary().execution_id());
    const auto result =
        execution_->client->transport->evidence.samples.list(scope, {name_}, {}, fields_, {}, {}, between_, {}, false, options);
    if (at_ && at_->SerializeAsString() != result.resolved_at().SerializeAsString())
      throw std::runtime_error("Read Samples: Retain the Original Evidence Cut");
    at_ = result.resolved_at();
    return result;
  }
  SampleRow<T> decode(const client::SampleRow &sample) const {
    if (!sample.has_raw())
      throw std::runtime_error("Read Samples: Request Raw Samples for Typed Decoding");
    const auto &row = sample.raw();
    if (row.stream() != name_ || row.scope().execution_id() != execution_->detail.summary().execution_id())
      throw std::runtime_error("Decode Sample: Select the Requested Execution and Stream");
    SampleRow<T> result{row.sample_id(),
                        row.sequence(),
                        detail::instant(row.timestamp()),
                        detail::instant(row.received_at()),
                        row.stream(),
                        row.schema_version(),
                        row.source(),
                        templates::detail::decode<T>(row.payload()),
                        {}};
    for (const auto &[name, value] : row.values().values())
      result.values.emplace(name, detail::decode_scalar(value));
    return result;
  }
  std::shared_ptr<detail::ExecutionState> execution_;
  std::string name_;
  std::vector<std::string> fields_;
  model::Window between_;
  std::optional<harmos::ReadCut> at_;
};
} // namespace dokime

/* === Templates === */

namespace dokime {
template <class T> RecordsQuery<T> Execution::records(const templates::Record<T> &declaration, const std::string &step) {
  const auto identity =
      templates::build_identifier(declaration.name, declaration.identifier, state_->detail.admitted_template().identifier());
  return records<T>(identity, step);
}
template <class T> RecordsQuery<T> Execution::records(const std::string &record_type, const std::string &step) {
  return RecordsQuery<T>(state_, record_type, step);
}
template <class T>
SamplesQuery<T> Execution::samples(const templates::Stream<T> &stream, const std::vector<std::string> &fields,
                                   std::pair<std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>,
                                             std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>>
                                       between) {
  // Streams Belong to the Plugin: Select the One Frozen Case Stream Named by the Declaration.
  const auto local = templates::build_identifier(stream.name, stream.identifier);
  std::optional<std::string> identity;
  for (const auto &candidate : state_->detail.origin().plugin_run().test_case().streams()) {
    if (candidate != local && !candidate.ends_with("." + local))
      continue;
    if (identity)
      throw std::invalid_argument("Read Samples: Name the Qualified Stream of " + local);
    identity = candidate;
  }
  if (!identity)
    throw std::invalid_argument("Read Samples: Select a Stream of This Execution's Frozen Case: " + local);
  return samples<T>(*identity, fields, between);
}
template <class T>
SamplesQuery<T> Execution::samples(const std::string &stream, const std::vector<std::string> &fields,
                                   std::pair<std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>,
                                             std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>>
                                       between) {
  return SamplesQuery<T>(state_, stream, fields, between);
}
} // namespace dokime

/* === Public Declarations === */
