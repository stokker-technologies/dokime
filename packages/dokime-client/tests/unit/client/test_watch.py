# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Preserve Watch Selection, Optional Waiting And Empty Armed Baselines On The Wire"""

# === Imports === #

# Standard Library Imports

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import _contracts

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("selection", ["All", "Latest", "New"])
@pytest.mark.parametrize("waiting", [None, 3])
def test_watch_selection_and_waiting(selection: str, waiting: int | None) -> None:
    operation = _contracts.operation("Files", "FileWatch")
    request = _contracts.request(
        "Files",
        "FileWatch",
        output="capture",
        policy={"Stream": {}},
        selection={selection: {}},
        wait_for_source_seconds=waiting,
        maximum_seconds=7,
    )
    received = operation.request.FromString(request.SerializeToString())

    assert _contracts.variant(received.selection) == selection
    assert received.HasField("wait_for_source_seconds") == (waiting is not None)
    if waiting is not None:
        assert received.wait_for_source_seconds == waiting
    assert received.maximum_seconds == 7


@pytest.mark.parametrize("baseline", [[], ["device.log"]])
def test_armed_baseline_is_an_explicit_frame(baseline: list[str]) -> None:
    operation = _contracts.operation("Files", "FileWatch")
    frame = _contracts.construct(operation.output.DESCRIPTOR, {"payload": {"Armed": {"field_0": {"baseline": baseline}}}})
    encoded = frame.SerializeToString()
    received = operation.output.FromString(encoded)

    assert encoded != operation.output().SerializeToString()
    assert _contracts.variant(received.payload) == "Armed"
    assert list(received.payload.Armed.field_0.baseline) == baseline


# === Private Functions === #

# === Entry Point === #
