/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include <dokime/plugin.hpp>

using dokime::Plugin;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class SamplePlugin : public Plugin {
public:
  SamplePlugin() : Plugin(Plugin::from<SamplePlugin>("0.1.0")) {}
};

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main() {
  SamplePlugin plugin;
  plugin.serve();
}
