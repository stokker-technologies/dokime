# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Read a Simulated Instrument"""

# === Imports === #

# Standard Library Imports
# Third Party Imports
import dokime

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dokime.operation(id="bench.read", response=float, policy=dokime.OperationPolicy(5.0, 5.0), replay="idempotent")
class Read:
    index: int

    def run(self, ctx: dokime.Context) -> float:
        return self.index * 0.25


# === Public Functions === #


# === Private Functions === #

# === Entry Point === #
