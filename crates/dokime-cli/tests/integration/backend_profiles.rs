/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Offline Backend Profile and Credential Admission Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports
use dokime_cli::Cli;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn pairing_is_the_only_cli_connection_input() {
    assert!(Cli::try_parse_from(["dokime", "status", "--remote", "lab.pair"]).is_ok());
    for option in ["--endpoint", "--backend", "--profiles-file", "--ca", "--expected-backend"] {
        assert!(Cli::try_parse_from(["dokime", "status", option, "value"]).is_err());
    }
    assert!(Cli::try_parse_from(["dokime", "backend", "list"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "status", "--timeout-seconds", "0"]).is_err());
}
