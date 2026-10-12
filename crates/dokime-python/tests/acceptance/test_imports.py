# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from __future__ import annotations

# Third Party Imports

# Local Imports
from dokime.records import builtin_record_contracts
from dokime.host import System, WifiChannelOccupancy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_modules_available():
    import dokime.client
    from dokime.client import DokimeClient
    import dokime.core
    import dokime.core.authoring.parameters
    import dokime.records
    import dokime.host
    import dokime.suites
    import dokime.runtime
    import dokime.telemetry

    assert (
        dokime.core.Metadata("fixture", "Fixture").to_contract()["identifier"]
        == "fixture"
    )
    assert len(builtin_record_contracts()) > 5
    assert dokime.client.DokimeClient is DokimeClient


def test_host_rows():
    assert System.__dokime_stream__.stream == "host.system"
    assert (
        WifiChannelOccupancy.__dokime_stream__.stream == "host.wifi.channel_occupancy"
    )
    assert System().cpu_load_percent is None
    assert not hasattr(System(), "__dokime_publication__")


def test_native_values():
    import importlib
    from dokime import _native
    from dokime.runtime import Role

    authoring = importlib.import_module("dokime._native.authoring")
    contracts = importlib.import_module("dokime._native.core_contracts")
    server = importlib.import_module("dokime._native.server")
    assert authoring.__name__ == "dokime._native.authoring"
    assert authoring.__package__ == "dokime._native"
    assert authoring.AuthoringContext is _native.AuthoringContext
    assert authoring.Artifacts is _native.Artifacts
    assert server.Role is Role
    assert contracts.identifier_contract("fixture") == _native.identifier_contract(
        "fixture"
    )

    assert [int(role) for role in (Role.Read, Role.Control, Role.Admin)] == [0, 1, 2]
    assert repr(Role.Admin) == "Role.Admin"
    assert Role.Control == 1
    assert Role.Admin != Role.Read


# === Private Functions === #

# === Entry Point === #
