# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compose the Thermal Counter Observers"""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable

# isort: split

# Third Party Imports
from dokime_sdk.templates import Context, Window, stream, window_observer
from google.protobuf.message import Message

# isort: split

# Local Imports
from thermal_example.models import Counter, CounterColumns

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def counter(window: Window) -> Message:
    return CounterColumns(value=[1_152_921_504_606_847_975] * window.count)


def channel(identifier: str) -> Callable[[object, Context, Window], Message]:
    @stream(identifier.replace("_", " ").title(), identifier=identifier, row=Counter)
    @window_observer(columns=CounterColumns, interval_ms=1)
    def read(self: object, ctx: Context, window: Window) -> Message:
        return counter(window)

    return read


# === Private Functions === #

# === Entry Point === #
