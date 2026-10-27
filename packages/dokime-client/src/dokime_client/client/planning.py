# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Materialize Template Cases Without Creating Executions"""

# === Imports === #

# Standard Library Imports
import math
import re
from collections.abc import Mapping
from dataclasses import dataclass

# Third Party Imports
# Local Imports
from . import _wire
from ._transport import Transport, _cursor
from .errors import ProtocolError
from .models import Backend, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

VALUE_DEPTH = 64

# === Types === #

# === Classes === #


@dataclass(frozen=True, slots=True)
class TestCase:
    """A Registered Immutable Case; Reading Parameters Returns An Owned Copy."""

    id: str
    suite_id: str
    template_id: str
    label: str
    definition_revision: str
    provider: str
    _parameters: bytes
    _transport: Transport
    _backend: Backend

    @property
    def parameters(self) -> dict[str, Value]:
        message = _wire.nested("Planning", "MaterializeTemplate", "cases.parameters", direction="response")
        message.ParseFromString(self._parameters)
        parameters = _wire.decode(message, VALUE_DEPTH)
        if not isinstance(parameters, dict):
            raise ProtocolError("materialized parameters must be an object")
        return parameters


@dataclass(frozen=True, slots=True)
class Template:
    """One Registered Template Bound To The Backend That Supplies Its Definition."""

    id: str
    label: str
    _suite_id: str
    _template_id: str
    _transport: Transport
    _backend: Backend

    async def materialize(self, *, parameters: Mapping[str, Value] | None = None) -> tuple[TestCase, ...]:
        """Resolve And Register Cases Without Constructing Or Starting Executions."""
        from .execution.request import _identity

        if parameters is not None and not isinstance(parameters, Mapping):
            raise ValueError("materialization parameters must be an object")
        supplied = dict(parameters) if parameters is not None else {}
        if any(not isinstance(key, str) for key in supplied):
            raise ValueError("materialization parameter names must be strings")

        entries = []
        for key, value in sorted(supplied.items()):
            encoded = _wire.encode(value, VALUE_DEPTH)
            numbers = [node["kind"]["Number"]["field_0"] for node in encoded["nodes"] if "Number" in node["kind"]]
            if any(not math.isfinite(number) for number in numbers):
                raise ValueError("materialization parameters must contain finite numbers")
            entries.append({"key": key, "value": {"values": [encoded]}})

        selection = {
            "suite_id": self._suite_id,
            "template_id": self._template_id,
            "parameters": entries,
        }
        request = _wire.request("Planning", "MaterializeTemplate", selection=selection)
        await self._transport.verify(self._backend, live=True)
        reply = await self._transport.unary("Planning", "MaterializeTemplate", request)
        _cursor(self._backend, reply)

        cases = []
        seen = set()
        for ordinal, case in enumerate(reply.cases):
            try:
                _identity(case.case_id)
            except ValueError as error:
                raise ProtocolError("materialized case identity is invalid") from error
            same_template = case.suite_id == self._suite_id and case.template_id == self._template_id
            if not same_template or case.ordinal != ordinal or case.case_id in seen:
                raise ProtocolError("materialized case identity, template or order differs")
            if not case.HasField("metadata") or not case.HasField("parameters"):
                raise ProtocolError("materialized case has no definition or parameters")
            if re.fullmatch(r"[a-f0-9]{64}", case.definition_revision) is None or not case.provider:
                raise ProtocolError("materialized case has no authoritative definition or provider reference")
            seen.add(case.case_id)
            cases.append(
                TestCase(
                    case.case_id,
                    case.suite_id,
                    case.template_id,
                    case.metadata.label,
                    case.definition_revision,
                    case.provider,
                    case.parameters.SerializeToString(deterministic=True),
                    self._transport,
                    self._backend,
                )
            )
        if not cases:
            raise ProtocolError("template materialization returned no cases")
        return tuple(cases)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
