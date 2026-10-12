# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Read-only host row types derived from the same contracts Rust uses.

Declare ``subscribes = (host.System,)`` on a template. The backend leases the
stream for the execution at its producer's cadence; read its settled rows
with ``await ctx.samples(host.System)`` in a criterion.
"""

# === Imports === #

# Standard Library Imports

from dataclasses import dataclass, make_dataclass
from datetime import datetime  # noqa: F401 -- generated row annotations resolve this name at runtime

# Third Party Imports

# Local Imports

from dokime import _native

# Type Checking Imports

# === Public Exports === #

# === Constants === #

_CONTRACTS = _native.host_stream_contracts()

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class _Stream:
    stream: str


# === Public Functions === #

# === Private Functions === #


def _row(name: str) -> type:
    definition = _CONTRACTS[name]
    assert isinstance(definition, dict)
    contract = definition["contract"]
    assert isinstance(contract, dict)
    fields = definition["fields"]
    assert isinstance(fields, dict)
    row = make_dataclass(
        name,
        [
            (
                column["metadata"]["identifier"],
                fields[column["metadata"]["identifier"]],
                None,
            )
            for column in contract["columns"]
        ],
        frozen=True,
        module=__name__,
    )
    row.__dokime_stream__ = _Stream(contract["stream"])
    return row


# === Entry Point === #

# Construct the public row types after their factory is defined.

System = _row("System")
Network = _row("Network")
Wifi = _row("Wifi")
WifiLatency = _row("WifiLatency")
WifiChannelOccupancy = _row("WifiChannelOccupancy")
Bluetooth = _row("Bluetooth")
