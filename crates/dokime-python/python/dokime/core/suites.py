# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Suite authoring wrappers backed by Rust contract builders."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from dataclasses import dataclass, field

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core._validation import new_id
from dokime.core.metadata import Metadata

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass
class ParameterDefinition:
    """A named test input with optional numeric bounds and a typed list shape.

    ``element_type`` makes the value one list belonging to one case; ``min_items``
    and ``max_items`` constrain its length. Numeric ``min`` and ``max`` apply to
    the scalar value or every numeric list element.
    """

    metadata: Metadata
    value_type: str
    default: object | None = None
    values: list[object] = field(default_factory=list)
    required: bool = False
    unit: dict[str, object] | None = None
    group: str | None = None
    attributes: dict[str, object] = field(default_factory=dict)
    expansion: str | dict[str, str] = "single"
    expression: object | None = None
    parameter_id: str = field(default_factory=lambda: new_id("new_parameter_id"))
    min: float | None = None
    max: float | None = None
    element_type: str | None = None
    min_items: int | None = None
    max_items: int | None = None

    def to_contract(self) -> dict[str, object]:
        attributes = dict(self.attributes)
        attributes["expansion"] = (
            {"kind": self.expansion}
            if isinstance(self.expansion, str)
            else self.expansion
        )
        for name in ("min", "max", "element_type", "min_items", "max_items"):
            value = getattr(self, name)
            if value is not None:
                attributes[name] = value
        if self.expression is not None:
            attributes["expression"] = _expression_contract(self.expression)
        return _native.parameter_definition_contract(
            self.parameter_id,
            self.metadata.to_contract(),
            self.value_type,
            self.default,
            list(self.values),
            self.required,
            self.unit,
            self.group,
            attributes,
        )


@dataclass
class FileDefinition:
    metadata: Metadata
    default: str | None = None
    required: bool = False
    media_types: list[str] = field(default_factory=list)
    attributes: dict[str, object] = field(default_factory=dict)

    def to_contract(self) -> dict[str, object]:
        return _native.file_definition_contract(
            self.metadata.to_contract(),
            self.default,
            self.required,
            list(self.media_types),
            dict(self.attributes),
        )


@dataclass
class ExecutionPhaseDefinition:
    metadata: Metadata
    step_id: str = field(default_factory=lambda: new_id("new_step_id"))
    repeatable: bool = False

    def to_contract(self) -> dict[str, object]:
        return _native.execution_phase_definition_contract(
            self.step_id,
            self.metadata.to_contract(),
            self.repeatable,
        )


@dataclass
class CriterionDefinition:
    metadata: Metadata
    expected: object | None = None
    criterion_id: str = field(default_factory=lambda: new_id("new_criterion_id"))

    def to_contract(self) -> dict[str, object]:
        return _native.criterion_definition_contract(
            self.criterion_id,
            self.metadata.to_contract(),
            self.expected,
        )


@dataclass
class CaseTemplate:
    metadata: Metadata
    parameters: list[ParameterDefinition] = field(default_factory=list)
    files: list[FileDefinition] = field(default_factory=list)
    phases: list[ExecutionPhaseDefinition] = field(default_factory=list)
    criteria: list[CriterionDefinition] = field(default_factory=list)
    tags: list[str] = field(default_factory=list)
    continue_on_failure: bool = False
    #: Explicit questions admitted by the host for this template.
    prompt_definitions: list[dict[str, object]] = field(default_factory=list)
    #: Stream and record inputs captured for the duration of an execution.
    subscriptions: list[dict[str, str]] = field(default_factory=list)
    attributes: dict[str, object] = field(default_factory=dict)
    document: dict[str, object] | None = None
    template_id: str = field(default_factory=lambda: new_id("new_template_id"))

    def to_contract(self) -> dict[str, object]:
        return _native.case_template_contract(
            self.template_id,
            self.metadata.to_contract(),
            list(self.tags),
            [parameter.to_contract() for parameter in self.parameters],
            [file_definition.to_contract() for file_definition in self.files],
            self.document,
            [phase.to_contract() for phase in self.phases],
            [criterion.to_contract() for criterion in self.criteria],
            self.continue_on_failure,
            list(self.prompt_definitions),
            list(self.subscriptions),
            dict(self.attributes),
        )


@dataclass
class Suite:
    metadata: Metadata
    tags: list[str] = field(default_factory=list)
    templates: list[CaseTemplate] = field(default_factory=list)

    def to_contract(self) -> dict[str, object]:
        return _native.suite_contract(
            self.metadata.to_contract(),
            list(self.tags),
            [template.to_contract() for template in self.templates],
        )


@dataclass
class MaterializedCase:
    suite_id: str
    template_id: str
    metadata: Metadata
    parameters: dict[str, object] = field(default_factory=dict)
    generation_index: int = 0
    case_id: str = field(default_factory=lambda: new_id("new_case_id"))

    def to_contract(self) -> dict[str, object]:
        return _native.materialized_case_contract(
            self.suite_id,
            self.case_id,
            self.template_id,
            self.metadata.to_contract(),
            dict(self.parameters),
            self.generation_index,
        )


# === Public Functions === #

# === Private Functions === #


def _expression_contract(value: object) -> object:
    to_contract = getattr(value, "to_contract", None)
    if callable(to_contract):
        return to_contract()
    return value


# === Entry Point === #
