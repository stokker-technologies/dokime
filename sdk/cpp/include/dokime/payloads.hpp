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
#include <google/protobuf/wrappers.pb.h>
#include <limits>
#include <stdexcept>
#include <string>
#include <type_traits>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

namespace dokime::templates::detail {
template <class T> auto payload_message() {
  if constexpr (std::is_same_v<T, bool>)
    return google::protobuf::BoolValue{};
  else if constexpr (std::is_same_v<T, std::string>)
    return google::protobuf::StringValue{};
  else if constexpr (std::is_integral_v<T> && std::is_signed_v<T>)
    return google::protobuf::Int64Value{};
  else if constexpr (std::is_floating_point_v<T>)
    return google::protobuf::DoubleValue{};
  else {
    static_assert(std::is_base_of_v<google::protobuf::Message, T>, "Declare Payload: Supply a Plain Value or Protobuf Message");
    return T{};
  }
}
template <class T> auto pack(const T &value) {
  auto message = payload_message<T>();
  if constexpr (std::is_base_of_v<google::protobuf::Message, T>)
    message = value;
  else
    message.set_value(value);
  return message;
}
template <class T> T unpack(const decltype(payload_message<T>()) &message) {
  if constexpr (std::is_base_of_v<google::protobuf::Message, T>)
    return message;
  else {
    if constexpr (std::is_integral_v<T> && !std::is_same_v<T, bool>)
      if (message.value() < std::numeric_limits<T>::min() || message.value() > std::numeric_limits<T>::max())
        throw std::invalid_argument("Decode Integer: Declare an Integer Type That Holds the Returned Value");
    return static_cast<T>(message.value());
  }
}
template <class T> T decode(const std::string &bytes) {
  const auto message = harmos::plugin::decode<decltype(payload_message<T>())>(bytes);
  return unpack<T>(message);
}
} // namespace dokime::templates::detail

/* === Public Declarations === */
