# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compose the Same Test Using an Explicit Template."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue
from suite import Capture

# Local Imports
from dokime import Prompt, PromptDefinitions
from dokime.authoring import Record, Suite, Template
from dokime.plugin import HostOperation, Plugin

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def plugin() -> Plugin:
    confirmation = Prompt(
        metadata={"identifier": "example.confirmation", "label": "Confirmation"},
        message="Continue?",
        actions=[{"identifier": "yes", "label": "Yes"}, {"identifier": "no", "label": "No"}],
    )
    template = Template("example.capture", Capture, "Capture")
    return Plugin(
        "example",
        "0.1.0",
        suites=[Suite("example", [template], "Example")],
        records=[Record("example.confirmation", BoolValue, "Confirmation")],
        sections=[PromptDefinitions(prompts=[confirmation])],
        requires=[
            HostOperation.PROMPTS_CREATE,
            HostOperation.EVIDENCE_RECORDS_APPEND,
            HostOperation.EVIDENCE_RECORDS_LIST,
            HostOperation.TIME_GET,
        ],
    )


# === Private Functions === #

# === Entry Point === #
