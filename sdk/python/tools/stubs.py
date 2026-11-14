# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Strictly Compare SDK APIs and Resource Stubs with the Installed Wheel."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from mypy import stubtest
from mypy.options import Options

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

MODULES = [
    "dokime",
    "dokime._client",
    "dokime._context",
    "dokime._plugin",
    "dokime.authoring",
    "dokime.client",
    "dokime.documents",
    "dokime.plot",
    "dokime.plugin",
    "dokime.schemas",
    "dokime.values",
    "dokime.v1",
    "dokime.generated.resources",
]

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    # Protocol metaclasses are verified against descriptors by generated.py.
    # Stubtest checks every authored SDK module and the generated resource API.
    options = Options()
    options.incremental = False
    modules = stubtest.build_stubs(MODULES, options)
    errors = [error for module in modules for error in stubtest.test_module(module)]
    for error in errors:
        print(error.get_description())
    print(f"Strict SDK Stubtest: {len(modules)} Modules, {len(errors)} Errors")
    return int(bool(errors))


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
