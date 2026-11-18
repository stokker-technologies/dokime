# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compile the Generated Public Client and Plugin Call Shapes."""

# === Imports === #

# Standard Library Imports
from typing import Self, assert_type

# Third Party Imports
from google.protobuf.wrappers_pb2 import Int64Value

# Local Imports
from dokime import (
    Case,
    Context,
    FileReference,
    FileSource,
    Operation,
    Output,
    Visibility,
    Window,
    criterion,
    materialize,
    model,
    observer,
    operation,
    prompt,
    record,
    step,
    stream,
    suite,
    window_observer,
)
from dokime.client import ClientRoot

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@suite()
class TypedAuthor:
    accepted = record(bool)
    count = record(int)
    reading = record(float)
    label = record(str)
    message = record(Int64Value)
    question = prompt("Continue?")

    @materialize
    def case(self, case: Case, parameters: dict[str, bool | str | int | float | None]) -> None:
        case.step("Measure", self.measure)
        case.criterion("Accept", self.accept)

    @step("Measure")
    async def measure(self, context: Context) -> None:
        await context.record(self.accepted, True)
        await context.record(self.count, 1)
        await context.record(self.reading, 1.5)
        await context.record(self.label, "ready")
        await context.record(self.message, Int64Value(value=1))
        assert_type(await context.records(self.accepted).last(), bool)
        assert_type(await context.records(self.count).last(), int)
        assert_type(await context.records(self.reading).last(), float)
        assert_type(await context.records(self.label).last(), str)
        assert_type(await context.records(self.message).last(), Int64Value)
        assert_type(await context.prompt(self.question), str)
        assert_type(await context.operation(Operation("read", request=str, response=int, visibility=Visibility.PUBLIC))("count"), int)

    @criterion("Accept")
    def accept(self, context: Context) -> bool:
        return True

    @stream("Reading")
    @observer()
    def sample(self, context: Context) -> float:
        return 1.5

    @stream("Window", row=int)
    @window_observer(columns=Int64Value)
    def samples(self, context: Context, window: Window) -> list[int]:
        return [1] * window.count

    @operation("echo", request=str, response=str, visibility=Visibility.PUBLIC)
    async def echo(self, context: Context, value: str) -> str:
        return value

    @classmethod
    def instance(cls) -> Self:
        return cls()


# === Public Functions === #


async def generated_surface(client: ClientRoot, context: Context) -> None:
    """Type-Check Content Calls Without Starting a Product Host."""
    await client.plugins.list()
    await context.host.files.capture(FileSource(), "output")
    await context.host.content.artifacts.create(FileReference(), "capture.bin", "application/octet-stream")
    await context.host.evidence.statistics("stream", [], model.Window(), list[Output](), "field", "step")


# === Private Functions === #

# === Entry Point === #
