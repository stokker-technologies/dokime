# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exercise declared prompt instances across Python, Rust, and gRPC."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import asyncio
import enum
from pathlib import Path
from typing import Literal

# Third Party Imports
import pytest

# Local Imports
import dokime
from dokime.core.authoring import declared_prompt

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Sweep(enum.Enum):
    FAST = "fast"
    SLOW = "slow"


@dokime.prompt(id="fixture.signoff", label="Sign Off", submit="Sign Off")
class SignOff:
    note: str = dokime.prompt_field(default="steady", max_length=200)
    accepted: bool = dokime.prompt_field(default=True)
    sweep: Sweep = dokime.prompt_field(default=Sweep.FAST)
    mode: Literal["auto", "manual"] = dokime.prompt_field(default="auto")
    retries: int | None = dokime.prompt_field(default=2)
    repeats: list[Sweep] = dokime.prompt_field(default=[Sweep.SLOW], min_items=1)


@dokime.prompt(id="fixture.go", label="Continue?")
class Go(dokime.Confirmation):
    pass


# === Public Functions === #


@pytest.mark.parametrize(
    "scenario",
    ["form", "clear_optional", "yes", "no", "cancel", "cancel_handled", "undeclared"],
)
def test_declared_backend(tmp_path: Path, scenario: str) -> None:
    async def run() -> None:
        answers = []

        class Ask(dokime.TestCaseTemplate):
            template_id = "fixture.ask"
            prompts = () if scenario == "undeclared" else (SignOff, Go)

            @dokime.step
            async def ask_operator(self, ctx):
                if scenario in {"yes", "no", "cancel", "cancel_handled"}:
                    try:
                        answers.append(await ctx.ask(Go()))
                    except dokime.PromptCancelled:
                        if scenario != "cancel_handled":
                            raise
                        answers.append("cancelled")
                else:
                    question = (
                        SignOff(retries=None)
                        if scenario == "clear_optional"
                        else SignOff()
                    )
                    answers.append(await ctx.ask(question))

        class Suite(dokime.TestSuite):
            suite_id = "fixture.questions"
            members = (Ask,)

        runtime = dokime.Runtime(application_root=tmp_path)
        try:
            await runtime.register(Suite)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="fixture.questions", template="fixture.ask"
            )
            execution = await execution_cycle.run(request_key=scenario)
            if scenario != "undeclared":
                while not (
                    pending := [
                        question async for question in execution_cycle.pending_prompts()
                    ]
                ):
                    assert not (await execution.status()).terminal
                    await asyncio.sleep(0.01)
                assert len(pending) == 1
                question = pending[0]
                if scenario in {"yes", "no", "cancel", "cancel_handled"}:
                    assert question.actions == ("yes", "no", "cancel")
                    action = "cancel" if scenario == "cancel_handled" else scenario
                    await execution_cycle.respond(
                        question.id, action=action, accepted=action != "cancel"
                    )
                else:
                    assert question.values == {
                        "note": "steady",
                        "accepted": True,
                        "sweep": "fast",
                        "mode": "auto",
                        "retries": None if scenario == "clear_optional" else 2,
                        "repeats": ["slow"],
                    }
                    # Invalid answers must be refused without consuming the pending question.
                    with pytest.raises(ValueError, match="prompt response validation"):
                        await execution_cycle.respond(
                            question.id,
                            action="submit",
                            values={**question.values, "accepted": "yes"},
                        )
                    assert [
                        item.id async for item in execution_cycle.pending_prompts()
                    ] == [question.id]
                    await execution_cycle.respond(
                        question.id,
                        action="submit",
                        values={
                            **question.values,
                            "note": "observed",
                            "sweep": "slow",
                            "repeats": ["fast", "slow"],
                        },
                    )
            await execution.wait(timeout=10)
            result = await execution_cycle.result()
            expected = (
                "blocked"
                if scenario == "undeclared"
                else "failed"
                if scenario == "cancel"
                else "passed"
            )
            if result.status != expected:
                for event in [event async for event in execution_cycle.records()]:
                    print(event.type, event.message, event.error)
            assert result.status == expected
            if scenario in {"form", "clear_optional"}:
                (answer,) = answers
                assert isinstance(answer, SignOff)
                assert answer.note == "observed"
                assert answer.sweep is Sweep.SLOW
                assert answer.repeats == [Sweep.FAST, Sweep.SLOW]
                assert answer.retries == (None if scenario == "clear_optional" else 2)
            elif scenario in {"yes", "no"}:
                assert answers == [scenario == "yes"]
            elif scenario == "cancel_handled":
                assert answers == ["cancelled"]
            else:
                assert not answers
            if scenario == "undeclared":
                assert any(
                    "not declared by this template" in (event.message or "")
                    for event in [event async for event in execution_cycle.records()]
                )
        finally:
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=30))


def case_prompt_defaults() -> None:
    @dokime.prompt(id="fixture.required", label="Required")
    class Required:
        note: str
        optional: str | None

    with pytest.raises(TypeError, match="note"):
        Required()  # pyright: ignore[reportCallIssue] - exercise required-field rejection
    with pytest.raises(TypeError, match="optional"):
        Required(note="present")  # pyright: ignore[reportCallIssue] - exercise missing optional-field default
    assert Required(note="present", optional=None).optional is None
    first, second = SignOff(), SignOff()
    first.repeats.append(Sweep.FAST)
    assert second.repeats == [Sweep.SLOW]
    assert not hasattr(SignOff, "prompt")


def case_invalid_refused() -> None:
    with pytest.raises(TypeError, match="confirmation"):

        @dokime.prompt(id="fixture.bad", label="Bad", submit="Go")
        class BadConfirmation(dokime.Confirmation):
            pass

    with pytest.raises(TypeError, match="type annotation"):

        @dokime.prompt(id="fixture.bad", label="Bad")
        class Unannotated:
            note = dokime.prompt_field()  # pyright: ignore[reportGeneralTypeIssues] - exercise missing annotation

    with pytest.raises(TypeError, match="min_items"):

        @dokime.prompt(id="fixture.bad", label="Bad")
        class Bounded:
            note: str = dokime.prompt_field(min_items=1)

    with pytest.raises(TypeError, match="not a declared prompt"):
        declared_prompt(Sweep)


# === Private Functions === #

# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_prompt_defaults, id="prompt_defaults"),
        pytest.param(case_invalid_refused, id="invalid_refused"),
    ],
)
def test_contract(scenario: Callable[..., None]) -> None:
    scenario()
