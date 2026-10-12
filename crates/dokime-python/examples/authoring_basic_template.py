# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Author a Python suite and control its local backend through gRPC."""

# === Imports === #

# Standard Library Imports
import asyncio
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports

# Local Imports
import dokime

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Declarations === #


@dokime.record(id="tracking.completed")
class Completed:
    sample_count: int


@dokime.stream(id="tracking.position")
class Position:
    sample_index: int
    distance_m: float


class Tracking(dokime.TestCaseTemplate):
    template_id = "examples.tracking"
    records = (Completed,)
    streams = (Position,)

    @dokime.step
    async def collect(self, ctx):
        for index in range(3):
            await ctx.publish(Position(index, 10.0 + index))
            await asyncio.sleep(0.01)
        await ctx.publish(Completed(sample_count=3))

    @staticmethod
    @dokime.criterion
    async def enough_samples(ctx):
        return len(await ctx.samples(Position)) == 3


class Lab(dokime.TestSuite):
    suite_id = "examples.lab"
    members = (Tracking,)


# === Public Functions === #


async def run(root: Path) -> None:
    runtime = dokime.Runtime(application_root=root)
    try:
        await runtime.register(Lab)
        client = await runtime.start()
        execution_cycle = await client.new_cycle(
            suite="examples.lab", template="examples.tracking"
        )
        (execution,) = execution_cycle.executions
        await execution.start(request_key="example-1")

        async def collect_events():
            return [event async for event in execution.records(follow=True)]

        async def collect_samples():
            return [
                sample
                async for sample in execution.samples(
                    streams=["tracking.position"], follow=True
                )
            ]

        # Collectors Replay Early Publications And Drain After The Saved Seal.
        # Waiting Establishes Durability; Gather Also Waits For Both Consumers.
        events, samples, status = await asyncio.gather(
            collect_events(),
            collect_samples(),
            execution.wait(timeout=15),
        )
        completed = next(
            event for event in events if event.type == "tracking.completed"
        )
        selected = [
            event async for event in execution.records(record_ids=[completed.id])
        ]
        summary = status.result
        assert status.status == "passed" and summary.status == "passed"
        assert len(samples) == 3 and len(selected) == 1
        print(f"{summary.status}: {len(samples)} samples, {len(events)} events")
    finally:
        # Closes the attached client and shuts down the backend this runtime owns.
        await runtime.close()


def main() -> None:
    with TemporaryDirectory(prefix="dokime-example-") as root:
        asyncio.run(run(Path(root)))


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
