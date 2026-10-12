# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compile Python suites into callbacks for the shared Rust execution engine."""

# === Imports === #

# Standard Library Imports
import inspect
import asyncio
from contextlib import AsyncExitStack
from dataclasses import dataclass
from typing import Any

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core.authoring.context import TestContext
from dokime.core.authoring.traceability import TraceabilitySection
from dokime.core.authoring.templates import TestSuite
from dokime.suites import suite_from_authored_suite

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Tick:
    index: int
    elapsed: float


class Provider:
    """One implicit complete registration. It owns resources, never scheduling."""

    def __init__(self, suites: tuple[type[TestSuite], ...]) -> None:
        self.bindings: dict[str, Binding] = {}
        self.streams: list[dict[str, object]] = []
        self._suites = suites
        self._resources: dict[type[TestSuite], object] = {}
        self._stack = AsyncExitStack()
        contracts = []
        streams = {}
        identifiers = set()
        for suite in suites:
            if not isinstance(suite, type) or not issubclass(suite, TestSuite):
                raise TypeError("register expects TestSuite classes")
            identifier = suite.resolved_suite_id()
            if identifier in identifiers:
                raise ValueError(f"duplicate suite {identifier!r}")
            identifiers.add(identifier)
            if hasattr(suite, "resources"):
                _resource_method(getattr(suite, "resources"))
            records = []
            telemetry = []
            for template in suite.members:
                binding = Binding(self, suite, template)
                template_id = template.resolved_template_id()
                if template_id in self.bindings:
                    raise ValueError(f"duplicate template {template_id!r}")
                self.bindings[template_id] = binding
                records.extend(template.record_contract_definitions())
                telemetry.extend(template.telemetry_contract_definitions())
                for row_type in template.stream_types():
                    stream = row_type.__dokime_wire__
                    previous = streams.setdefault(stream["id"], stream)
                    if previous != stream:
                        raise ValueError(f"conflicting stream {stream['id']!r}")
            contracts.append(
                _native.suite_registration_contract(
                    identifier,
                    None,
                    list(suite.tags),
                    identifier,
                    dict(suite_from_authored_suite(suite).to_contract()),
                    records,
                    telemetry,
                )
            )
        self.suites = {"suites": contracts}
        self.streams = list(streams.values())

    async def start(self) -> None:
        try:
            for suite in self._suites:
                instance = suite()
                if hasattr(instance, "resources"):
                    self._resources[suite] = await self._stack.enter_async_context(
                        getattr(instance, "resources")()
                    )
                else:
                    self._resources[suite] = None
        except BaseException:
            await _close_resources(self._stack)
            raise

    async def close(self) -> None:
        await _close_resources(self._stack)
        self._resources.clear()


class Binding:
    def __init__(self, provider: Provider, suite: type, template: type) -> None:
        self.provider = provider
        self.suite = suite
        self.template = template
        self.steps = {
            item.step_id: item.method_name for item in template.step_definitions()
        }
        self.criteria = {
            item.criterion_id: item.method_name
            for item in template.criterion_definitions()
        }
        self.initialize_hook = _hook(template, template.initialize_definitions())
        self.cleanup_hook = _hook(template, template.cleanup_definitions())
        self.sampling = {
            item.stream: item.method_name for item in template.sampler_definitions()
        }
        self.samplers = [
            (item.stream, item.stream, item.interval_ms)
            for item in template.sampler_definitions()
        ]
        if len(self.sampling) != len(self.samplers):
            raise ValueError("a stream has one sampler per template")
        if any(interval <= 0 for _, _, interval in self.samplers):
            raise ValueError("sampler interval must be positive")
        for name in [
            *self.steps.values(),
            *self.criteria.values(),
            *self.sampling.values(),
        ]:
            _async_method(getattr(template, name))
        for name in self.criteria.values():
            if not isinstance(
                inspect.getattr_static(template, name), staticmethod | classmethod
            ):
                raise TypeError(
                    "criteria must be static or class async methods; resources are released before evaluation"
                )
        if hasattr(template, "resources"):
            raise TypeError(
                "construct template resources in initialize using ctx.acquire"
            )
        if hasattr(template, "traceability"):
            _async_method(template.traceability)

    def context(
        self,
        native: Any,
        *,
        resources: object = None,
        reason: str | None = None,
        acquisitions: AsyncExitStack | None = None,
    ) -> TestContext:
        return TestContext(
            native,
            plugin=self.provider._resources[self.suite],
            resources=resources,
            reason=reason,
            acquisitions=acquisitions,
        )

    async def initialize(self, native: Any) -> "Instance":
        context = self.context(native)
        instance = Instance(self, self.template(case_parameters=context.parameters))
        context = self.context(native, acquisitions=instance._stack)
        try:
            if self.initialize_hook is not None:
                instance._resources = await getattr(
                    instance.template, self.initialize_hook
                )(context)
        except BaseException:
            await _close_resources(instance._stack)
            raise
        return instance

    async def criterion(self, identifier: str, native: Any) -> bool:
        return await getattr(self.template, self.criteria[identifier])(
            self.context(native)
        )


class Instance:
    def __init__(self, binding: Binding, template: object) -> None:
        self.binding = binding
        self.template = template
        self._stack = AsyncExitStack()
        self._resources: object = None

    async def traceability(self, native: Any) -> list[TraceabilitySection]:
        if hasattr(self.template, "traceability"):
            return await getattr(self.template, "traceability")(
                self.binding.context(native, resources=self._resources)
            )
        return []

    async def step(self, identifier: str, native: Any) -> None:
        return await getattr(self.template, self.binding.steps[identifier])(
            self.binding.context(native, resources=self._resources)
        )

    async def cleanup(self, reason: str, native: Any) -> None:
        try:
            if self.binding.cleanup_hook is not None:
                result = await getattr(self.template, self.binding.cleanup_hook)(
                    self.binding.context(
                        native, resources=self._resources, reason=reason
                    )
                )
                _none(result)
        finally:
            await _close_resources(self._stack)
            self._resources = None

    async def sample(self, stream: str, index: int, elapsed: float) -> dict:
        row = await getattr(self.template, self.binding.sampling[stream])(
            self._resources, Tick(index, elapsed)
        )
        kind, publication = row.__dokime_publication__()
        if kind != "row" or publication["stream"] != stream:
            raise TypeError("a sampler must return its declared stream row")
        return publication["values"]


# === Public Functions === #

# === Private Functions === #


async def _close_resources(stack: AsyncExitStack) -> None:
    closing = asyncio.create_task(stack.aclose())
    try:
        await asyncio.shield(closing)
    except asyncio.CancelledError:
        # The Rust callback owner cancels once and joins us. Preserve that
        # ownership until asynchronous resource release has really finished.
        await closing
        raise


def _async_method(method: object) -> None:
    if not inspect.iscoroutinefunction(method):
        raise TypeError(f"{method!r} must be async")


def _resource_method(method: object) -> None:
    if not callable(method) or not inspect.isasyncgenfunction(inspect.unwrap(method)):
        raise TypeError(
            "resources must be an async generator decorated with @asynccontextmanager"
        )


def _hook(template: type, definitions: list) -> str | None:
    if len(definitions) > 1:
        raise ValueError("a template declares one callback per lifecycle phase")
    if not definitions:
        return None
    name = definitions[0].method_name
    _async_method(getattr(template, name))
    return name


def _none(result: object) -> None:
    if result is not None:
        raise TypeError("a lifecycle callback must return None")


# === Entry Point === #
