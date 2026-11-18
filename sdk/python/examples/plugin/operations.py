# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""An Independent Plain String Echo Operation."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from dokime_sdk import Context, Plugin, Visibility, operation

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Operations:
    @operation("example.echo", request=str, response=str, visibility=Visibility.PUBLIC)
    async def echo(self, ctx: Context, request: str) -> str:
        return request


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

plugin = Plugin("example.echo", "1.0.0", operations=Operations())
