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

#include "dokime/messages.hpp"
#include "dokime/payloads.hpp"
#include <memory>
#include <optional>
#include <string_view>
#include <unordered_map>
#include <vector>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime::templates {
Metadata metadata(std::string identifier, std::string label);
struct Declaration {
  std::string name;
  std::string label;
  std::optional<std::string> identifier;
  std::shared_ptr<const int> key = std::make_shared<const int>(0);
  Declaration(std::string name, std::string label = {}, std::optional<std::string> identifier = {})
      : name(std::move(name)), label(std::move(label)), identifier(std::move(identifier)) {}
};
template <class T> struct Record : Declaration {
  using Declaration::Declaration;
};
struct PromptDeclaration : Declaration {
  Prompt value;
  PromptDeclaration(std::string name, std::string text, std::vector<std::string> choices = {}, std::optional<std::string> identifier = {});
  PromptDeclaration(std::string name, Prompt value, std::optional<std::string> identifier = {});
};
namespace detail {
struct Bindings {
  std::unordered_map<std::shared_ptr<const int>, std::string> identifiers;
  std::unordered_map<std::shared_ptr<const int>, Prompt> prompts;
  std::string template_id;
};
// Compiler Signatures Supply Code Names Until C++ Has Static Reflection.
inline std::string code_name(std::string_view signature, std::string_view marker, std::string_view end) {
  auto start = signature.find(marker);
  if (start == std::string_view::npos)
    throw std::invalid_argument("Read Declaration Name: Use a Supported C++ Compiler");
  start += marker.size();
  auto value = signature.substr(start, signature.find(end, start) - start);
  if (value.ends_with(']'))
    value.remove_suffix(1);
  value = value.substr(0, value.find('('));
  auto scope = value.rfind("::");
  if (scope != std::string_view::npos)
    value.remove_prefix(scope + 2);
  auto space = value.rfind(' ');
  if (space != std::string_view::npos)
    value.remove_prefix(space + 1);
  return std::string(value);
}
template <class T> std::string type_name() {
#ifdef _MSC_VER
  return code_name(__FUNCSIG__, "type_name<", ">(void)");
#else
  return code_name(__PRETTY_FUNCTION__, "T = ", ";");
#endif
}
template <auto Method> std::string method_name() {
#ifdef _MSC_VER
  return code_name(__FUNCSIG__, "method_name<", ">(void)");
#else
  return code_name(__PRETTY_FUNCTION__, "Method = ", ";");
#endif
}
template <auto Method> struct Member {
  inline static const std::shared_ptr<const int> key = std::make_shared<const int>(0);
};
template <class T, class R, class... A> T member_owner(R (T::*)(A...));
} // namespace detail
} // namespace dokime::templates

/* === Templates === */

/* === Public Declarations === */

namespace dokime::templates {
std::string build_identifier(std::string name, std::optional<std::string> explicit_identifier = {}, std::string prefix = {},
                             bool class_name = false);
std::string build_label(std::string name);
} // namespace dokime::templates
