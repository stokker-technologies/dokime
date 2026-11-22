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
from google.protobuf.struct_pb2 import Struct

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@operation("example.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: Context, request: Struct) -> Struct:
    return request


# === Private Functions === #

# === Entry Point === #
