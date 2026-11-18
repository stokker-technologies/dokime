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
from collections.abc import Iterator
from typing import TypeAliasType

# Third Party Imports
from mypy import nodes, stubtest
from mypy.options import Options

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

MODULES = [
    "dokime_sdk",
    "dokime_sdk._context",
    "dokime_sdk._plugin",
    "dokime_sdk.authoring",
    "dokime_sdk.client",
    "dokime_sdk.connection",
    "dokime_sdk.evidence",
    "dokime_sdk.transfers",
    "dokime_sdk.documents",
    "dokime_sdk.operations",
    "dokime_sdk.plot",
    "dokime_sdk.plugin",
    "dokime_sdk.schemas",
    "dokime_sdk.values",
    "dokime_sdk._generated.client",
]

# === Types === #

# === Classes === #

# === Public Functions === #


@stubtest.verify.register(nodes.TypeAlias)
def verify_typealias(stub: nodes.TypeAlias, runtime: object, object_path: list[str]) -> Iterator[stubtest.Error]:
    """Verify PEP 695 Alias Values Through the Same Strict Alias Checker."""
    if isinstance(runtime, TypeAliasType):
        runtime = runtime.__value__
    yield from stubtest.verify_typealias(stub, runtime, object_path)


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
