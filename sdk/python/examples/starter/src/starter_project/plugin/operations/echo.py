# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Echo Operations."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from dokime_sdk.operations import Context, Visibility, operation

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@operation("example.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: Context, request: str) -> str:
    return request


# === Private Functions === #

# === Entry Point === #
