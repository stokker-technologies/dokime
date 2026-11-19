/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/configuration.hpp"
#include <catch2/catch_test_macros.hpp>

using dokime::detail::configuration::parse;
using std::invalid_argument;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

TEST_CASE("Dotenv Preserves Literal Values and Decodes Only Supported Escapes") {
  const auto values = parse("\xef\xbb\xbf# Addresses\r\nDOKIME_REMOTE = 'host#tag.pair' # Kept\r\n"
                            "DOKIME_PLUGIN_ADDRESS=127.0.0.1:0 # Comment\n"
                            "UNKNOWN=\"one\\ntwo\\r\\t\\\"\\\\\"\nEMPTY=\nLITERAL=$HOME`command`\n",
                            ".env");
  REQUIRE(values.at("DOKIME_REMOTE") == "host#tag.pair");
  REQUIRE(values.at("DOKIME_PLUGIN_ADDRESS") == "127.0.0.1:0");
  REQUIRE(values.at("UNKNOWN") == "one\ntwo\r\t\"\\");
  REQUIRE(values.at("EMPTY").empty());
  REQUIRE(values.at("LITERAL") == "$HOME`command`");
}
TEST_CASE("Dotenv Refuses Duplicate Malformed and Non UTF-8 Assignments") {
  for (const auto &content :
       {string("A=1\nA=2"), string("export A=1"), string("A=\"unterminated"), string("A='ok' trailing"), string("A=\"\\x00\""),
        string("A=one\rtwo"), string("A=\0", 3), string("A=\xc0\xaf"), string("A=\xed\xa0\x80"), string("A=\xf4\x90\x80\x80")})
    REQUIRE_THROWS_AS(parse(content, ".env"), invalid_argument);
  try {
    parse("A=1\ninvalid", "settings.env");
    FAIL("Malformed Assignment Was Accepted");
  } catch (const invalid_argument &error) {
    REQUIRE(string(error.what()).starts_with("Read settings.env:2:"));
  }
}

/* === Private Functions === */

/* === Entry Point === */
