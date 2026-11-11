# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run Strict Stubtest With Semantic Python Type Alias Comparison."""

# === Imports === #

# Standard Library Imports
from collections.abc import Iterator
from functools import reduce
from operator import or_
from typing import Any, Literal, TypeAliasType, cast, get_args, get_origin

# Third Party Imports
from mypy import nodes, stubtest

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    original = stubtest.verify.dispatch(nodes.TypeAlias)

    @stubtest.verify.register(nodes.TypeAlias)
    def verify_alias(stub: nodes.TypeAlias, runtime: Any, path: list[str]) -> Iterator[stubtest.Error]:
        # Mypy 2.4.1 Compares Aliases As Their Target Types But Does Not Unwrap Runtime Aliases.
        if isinstance(runtime, TypeAliasType):
            runtime = runtime.__value__
        if get_origin(runtime) is Literal:
            # Literal[A, B] And Literal[A] | Literal[B] Have Identical Typing Meaning.
            runtime = reduce(or_, (cast(Any, Literal)[value] for value in get_args(runtime)))
        yield from original(stub, runtime, path)

    return stubtest.main()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
