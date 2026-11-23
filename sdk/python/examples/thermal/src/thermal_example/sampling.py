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

# isort: split

# Third Party Imports
from dokime_sdk.operations import Context
from dokime_sdk.templates import Observer, Window
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


def channel(identifier: str) -> type[Observer[Counter]]:
    class CounterObserver(Observer[Counter]):
        row = Counter
        columns = CounterColumns
        interval_ms = 1

        def sample(self, ctx: Context, window: Window) -> Message:
            return counter(window)

    CounterObserver.identifier = identifier
    CounterObserver.label = identifier.replace("_", " ").title()
    return CounterObserver


# === Private Functions === #

# === Entry Point === #
