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
from dokime import HOST_OPERATIONS
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

__all__ = ["Capture", "plugin"]

# === Constants === #

# === Types === #

# === Classes === #


class Capture:
    @dokime.step("Confirm", identifier="confirm")
    async def confirm(self, ctx):
        question = next(operation for operation in HOST_OPERATIONS if operation.id == "dokime.plugin.host.v1.Prompts/Create")
        answer = await ctx.operation(question)(
            message("dokime.plugin.host.v1.PromptsCreateRequest", prompt="confirmation", prompt_type="example.confirmation")
        )
        await ctx.record("example.confirmation", value=answer.accepted and answer.action == "yes")

    @dokime.criterion("Confirmed", identifier="confirmed")
    async def confirmed(self, ctx):
        confirmation = await ctx.records("example.confirmation", step="confirm").last()
        return confirmation is not None and confirmation.value


# === Public Functions === #


def plugin():
    needed = {
        "dokime.plugin.host.v1.Prompts/Create",
        "dokime.plugin.host.v1.evidence.Records/Append",
        "dokime.plugin.host.v1.evidence.Records/List",
    }
    definition = message(
        "PromptDefinitions",
        prompts=[
            message(
                "Prompt",
                metadata={"identifier": "example.confirmation", "label": "Confirmation"},
                message="Continue?",
                actions=[{"identifier": "yes", "label": "Yes"}, {"identifier": "no", "label": "No"}],
            )
        ],
    )
    return dokime.Plugin(
        "example",
        "0.1.0",
        suites=[dokime.Suite("example", [dokime.Template("example.capture", Capture, "Capture")], "Example")],
        records=[dokime.Record("example.confirmation", BoolValue, "Confirmation")],
        sections=[definition],
        requires=[operation for operation in HOST_OPERATIONS if operation.id in needed],
    )


# === Private Functions === #

# === Entry Point === #
