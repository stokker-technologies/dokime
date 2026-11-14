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

#include <catch2/catch_test_macros.hpp>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

inline void check(bool condition, const char *message) {
  INFO(message);
  REQUIRE(condition);
}

/* === Templates === */

/* === Public Declarations === */
