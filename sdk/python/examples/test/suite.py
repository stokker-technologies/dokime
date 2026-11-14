# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Deterministic Suite Shared With The Cross-Language Host Story."""

# === Imports === #

# Standard Library Imports
# Third Party Imports
# Local Imports
from google.protobuf.wrappers_pb2 import BoolValue

import dokime
from dokime import Prompt, PromptDefinitions
from dokime.authoring import Record, criterion, step, suite
from dokime.plugin import Context, HostOperation, Plugin

# Type Checking Imports

# === Public Exports === #

__all__ = ["Capture", "plugin"]

# === Constants === #

# === Types === #

# === Classes === #


@suite("example.capture", label="Capture")
class Capture:
    @step("Confirm", identifier="confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.host.prompts.create("confirmation", "example.confirmation", {})
        await ctx.record("example.confirmation", value=answer.accepted and answer.action == "yes")

    @criterion("Confirmed", identifier="confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        confirmation = await ctx.records("example.confirmation", step="confirm").last()
        return isinstance(confirmation, BoolValue) and confirmation.value


# === Public Functions === #


def plugin() -> Plugin:
    needed = [
        HostOperation.PROMPTS_CREATE,
        HostOperation.EVIDENCE_RECORDS_APPEND,
        HostOperation.EVIDENCE_RECORDS_LIST,
        HostOperation.TIME_GET,
    ]
    definition = PromptDefinitions(
        prompts=[
            Prompt(
                metadata={"identifier": "example.confirmation", "label": "Confirmation"},
                message="Continue?",
                actions=[{"identifier": "yes", "label": "Yes"}, {"identifier": "no", "label": "No"}],
            )
        ],
    )
    return dokime.Plugin(
        "example",
        "0.1.0",
        suites=[Capture],
        records=[Record("example.confirmation", BoolValue, "Confirmation")],
        sections=[definition],
        requires=needed,
    )


# === Private Functions === #

# === Entry Point === #
