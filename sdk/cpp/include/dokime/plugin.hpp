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

#include "dokime/configuration.hpp"
#include "dokime/templates.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime {
using harmos::LaneDeclaration;

struct DeclarationDescription {
  std::string identifier;
  std::string label;
};
struct TemplateDescription : DeclarationDescription {
  std::vector<DeclarationDescription> steps;
};
struct SuiteDescription : DeclarationDescription {
  std::vector<TemplateDescription> templates;
};
struct RecordDescription : DeclarationDescription {
  std::string message;
};
struct OperationDescription {
  std::string identifier;
  std::string request;
  std::string response;
  operations::Visibility visibility;
  std::string lane;
};
struct PluginDescription {
  std::string identifier;
  std::string version;
  std::vector<SuiteDescription> suites;
  std::vector<RecordDescription> records;
  std::vector<Prompt> prompts;
  std::vector<OperationDescription> operations;
  std::vector<LaneDeclaration> lanes;
};
struct Suite : templates::Declaration {
  std::vector<templates::Template> templates;
  Suite(std::string name, std::vector<templates::Template> templates, std::string label = {}, std::optional<std::string> identifier = {})
      : templates::Declaration(std::move(name), std::move(label), std::move(identifier)), templates(std::move(templates)) {}
  Suite &template_value(templates::Template value) {
    templates.push_back(std::move(value));
    return *this;
  }
  template <class T> static Suite from(std::string label = {}, std::optional<std::string> identifier = {}) {
    static_assert(std::is_base_of_v<Suite, T>, "Declare Suite: Derive the Author Class From dokime::Suite");
    Suite suite(templates::detail::type_name<T>(), {}, std::move(label), std::move(identifier));
    return suite;
  }
};

class Plugin {
public:
  Plugin(std::string identifier, std::string version, std::vector<Suite> suites = {});
  Plugin &suite(Suite suite);
  Plugin &lane(const Lane &lane) {
    native_.lane(lane);
    return *this;
  }
  virtual void initialize(operations::Context &) {}
  virtual void finalize(operations::Context &) {}
  template <class Q, class R> Plugin &operation(const operations::OperationValue<Q, R> &operation) {
    Registrar registrar(native_);
    registrar.add(operation.native(plugin_));
    return *this;
  }
  Plugin &secrets(SecretsDeclarations secrets);
  PluginDescription describe() const;
  virtual ~Plugin() {
    if (plugin_ && plugin_->plugin == this)
      plugin_->plugin = nullptr;
  }
  Plugin(Plugin &&other) noexcept;
  Plugin(const Plugin &) = delete;
  Plugin &operator=(const Plugin &) = delete;
  template <class T> static Plugin from(std::string version, std::optional<std::string> identifier = {}) {
    static_assert(std::is_base_of_v<Plugin, T>, "Declare Plugin: Derive the Author Class From dokime::Plugin");
    return Plugin(templates::build_identifier(templates::detail::type_name<T>(), std::move(identifier), {}, true), std::move(version));
  }
  const std::string &identifier() const {
    return identifier_;
  }
  void serve();

private:
  friend class operations::Context;
  harmos::Declarations declarations() const;
  friend class Plugins;
  friend class Cases;
  Plugin &context_operations();
  std::string identifier_;
  std::string version_;
  std::shared_ptr<detail::PluginState> plugin_;
  std::unordered_map<std::type_index, std::vector<std::pair<std::string, std::string>>> bindings_;
  template <class Q>
  static templates::Context execution_context(harmos::plugin::Context &transport, const Q &request, const std::string &template_id,
                                              const std::shared_ptr<templates::detail::Executions> &executions,
                                              std::shared_ptr<const templates::detail::Bindings> bindings, const std::string &action,
                                              std::string item, std::shared_ptr<detail::PluginState> plugin);
  harmos::plugin::Plugin compose() const;
  harmos::plugin::Plugin native_;
  dokime::Suites suites_;
  PromptDefinitions prompts_;
  RecordContracts records_;
  SecretsDeclarations secrets_;
  std::shared_ptr<templates::detail::Executions> executions_;
};
} // namespace dokime
