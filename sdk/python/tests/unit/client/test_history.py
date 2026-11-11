# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Validate Native Saved-Evidence Cursors"""

# === Imports === #

# Standard Library Imports
import base64
from dataclasses import replace

# Third Party Imports
import pytest

# Local Imports
from dokime.client import ProtocolError
from dokime.client.history import Checkpoint, StorageCursor, StoreHistory, _page_cursor

# Type Checking Imports


# === Public Exports === #

# === Constants === #

HISTORY = StoreHistory("fixture.store", "fixture.epoch")
POINT = Checkpoint(
    "47acbd5a-7496-4947-8078-0b282cf58643", StorageCursor(HISTORY, 7), b"\x0dfixture.store\x0dfixture.epoch\x07", StorageCursor(HISTORY, 3)
)

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    ("kind", "source", "first", "second"),
    [
        ("records", "", "L585FVCqXtjDB0phQScwNv7PbO9iJ0yX950tgy4teoIAAAAAAAAAAQ", "L585FVCqXtjDB0phQScwNv7PbO9iJ0yX950tgy4teoIAAAAAAAAAAg"),
        (
            "telemetry",
            "voltage",
            "nkC8SvUk7mbfa3PIDoLVE7QNkHVbzve9UPlFWlWSJY4AAAAAAAAAAQ",
            "nkC8SvUk7mbfa3PIDoLVE7QNkHVbzve9UPlFWlWSJY4AAAAAAAAAAg",
        ),
    ],
)
def test_native_cursor_binds_checkpoint_and_source(kind: str, source: str, first: str, second: str) -> None:
    assert _page_cursor(POINT, kind, source, first, 0) == 1
    assert _page_cursor(POINT, kind, source, second, 1) == 2
    for point, selected_kind, selected_source in (
        (POINT, "other", source),
        (POINT, kind, "other"),
        (replace(POINT, execution_id="00000000-0000-0000-0000-000000000001"), kind, source),
        (replace(POINT, position=StorageCursor(HISTORY, 8)), kind, source),
        (replace(POINT, evidence_through=StorageCursor(HISTORY, 2)), kind, source),
        (replace(POINT, position=StorageCursor(StoreHistory("another.store", "fixture.epoch"), 7)), kind, source),
    ):
        with pytest.raises(ProtocolError):
            _page_cursor(point, selected_kind, selected_source, first, 0)


def test_native_cursor_refuses_corruption_and_nonadvance() -> None:
    token = "nkC8SvUk7mbfa3PIDoLVE7QNkHVbzve9UPlFWlWSJY4AAAAAAAAAAQ"
    raw = base64.urlsafe_b64decode(token + "==")
    corrupted = bytes([raw[0] ^ 1]) + raw[1:]
    beyond = raw[:32] + (POINT.evidence_through.sequence + 1).to_bytes(8, "big")
    for selected in (
        token[:-1],
        token + "==",
        "AS-NHy5qsY3VX7wdUoUc1hTUnmoBSgaqNQrN5IyZu8n1AAAAAAAAAAE",
        base64.urlsafe_b64encode(corrupted).rstrip(b"=").decode("ascii"),
        base64.urlsafe_b64encode(beyond).rstrip(b"=").decode("ascii"),
    ):
        with pytest.raises(ProtocolError):
            _page_cursor(POINT, "telemetry", "voltage", selected, 0)
    with pytest.raises(ProtocolError):
        _page_cursor(POINT, "telemetry", "voltage", token, 1)


# === Private Functions === #

# === Entry Point === #
