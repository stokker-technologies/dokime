# SPDX-License-Identifier: Apache-2.0

"""Tracking Distance at Connections and Disconnections from One Status Record."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from dokime_sdk.templates import Document, FilterOperator, Section, field, histogram

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def tracking_document(stream: str, distance: str, status_record: str) -> Document:
    sampled = field(stream, distance).at(status_record)
    event = sampled.detail("event")
    disconnected = histogram(sampled, label="Distance at Disconnection").filter(event, FilterOperator.EQ, "disconnected")
    connected = histogram(sampled, label="Distance at Connection").filter(event, FilterOperator.EQ, "connected")
    return Document("Tracking").section(Section("Connection Events").plot(disconnected).plot(connected))


# === Private Functions === #

# === Entry Point === #
