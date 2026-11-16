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

// Standard Library Imports
#include <cstdint>
#include <optional>
#include <string>
#include <utility>
#include <vector>

// Third Party Imports

// Local Imports
#include "dokime/messages.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

namespace dokime::authoring {

/* === Classes === */

struct Parameter {
  dokime::Parameter message;
  static Parameter create(std::string identifier, std::string label, ValueType kind) {
    Parameter result;
    result.message.mutable_metadata()->set_identifier(std::move(identifier));
    result.message.mutable_metadata()->set_label(std::move(label));
    result.message.set_type(kind);
    return result;
  }
  static Parameter number(std::string identifier, std::string label) {
    return create(std::move(identifier), std::move(label), VALUE_TYPE_NUMBER);
  }
  static Parameter count(std::string identifier, std::string label) {
    return create(std::move(identifier), std::move(label), VALUE_TYPE_UNSIGNED_INTEGER);
  }
  static Parameter flag(std::string identifier, std::string label) {
    return create(std::move(identifier), std::move(label), VALUE_TYPE_BOOLEAN);
  }
  static Parameter sweep(std::string identifier, std::string label, const std::vector<double> &values) {
    auto result = number(std::move(identifier), std::move(label));
    result.message.mutable_expression()->mutable_array();
    for (double value : values)
      result.message.mutable_expression()->mutable_array()->add_values()->mutable_literal()->set_number(value);
    result.message.mutable_expansion()->set_cartesian(true);
    return result;
  }
  static Parameter numbers(std::string identifier, std::string label, const std::vector<double> &values) {
    auto result = create(std::move(identifier), std::move(label), VALUE_TYPE_LIST);
    result.message.set_element_type(VALUE_TYPE_NUMBER);
    result.message.mutable_expression()->mutable_literal()->mutable_list();
    for (double value : values)
      result.message.mutable_expression()->mutable_literal()->mutable_list()->add_values()->set_number(value);
    return result;
  }
  Parameter &unit(std::string value) {
    message.set_unit(std::move(value));
    return *this;
  }
  Parameter &default_value(double value) {
    message.mutable_expression()->mutable_literal()->set_number(value);
    return *this;
  }
  Parameter &default_value(std::uint64_t value) {
    message.mutable_expression()->mutable_literal()->set_unsigned_integer(value);
    return *this;
  }
  Parameter &default_value(bool value) {
    message.mutable_expression()->mutable_literal()->set_boolean(value);
    return *this;
  }
  Parameter &range(std::optional<double> minimum = {}, std::optional<double> maximum = {}) {
    message.clear_min();
    message.clear_max();
    if (minimum)
      message.set_min(*minimum);
    if (maximum)
      message.set_max(*maximum);
    return *this;
  }
  Parameter &items(std::optional<std::uint32_t> minimum = {}, std::optional<std::uint32_t> maximum = {}) {
    message.clear_min_items();
    message.clear_max_items();
    if (minimum)
      message.set_min_items(*minimum);
    if (maximum)
      message.set_max_items(*maximum);
    return *this;
  }
  dokime::Parameter build() const { return message; }
};

/* === Templates === */

/* === Public Declarations === */

} // namespace dokime::authoring
