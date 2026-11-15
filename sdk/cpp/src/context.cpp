/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/context.hpp"

using TransportContext = harmos::plugin::Context;

using std::lock_guard;
using std::make_shared;
using std::move;
using std::shared_ptr;
using std::string;
using std::unordered_map;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::plugin {
shared_ptr<detail::Resources> detail::Executions::open(const string &execution) {
  lock_guard lock(mutex);
  auto &resources = values[execution];
  if (!resources)
    resources = make_shared<Resources>();
  return resources;
}
void detail::Executions::close(const string &execution) {
  lock_guard lock(mutex);
  values.erase(execution);
}
void detail::Executions::clear() {
  unordered_map<string, shared_ptr<Resources>> released;
  {
    lock_guard lock(mutex);
    values.swap(released);
  }
}
Context::Context(TransportContext &transport, TestCase test_case, string item, shared_ptr<detail::Resources> resources)
    : transport_(transport), test_case_(move(test_case)), item_(move(item)), resources_(move(resources)) {}

} // namespace dokime::plugin

/* === Private Functions === */

/* === Entry Point === */
