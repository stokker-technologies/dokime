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

#include <stdexcept>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

inline void check(bool condition, const char *message) {
  if (!condition)
    throw std::runtime_error(message);
}

/* === Templates === */

/* === Public Declarations === */
