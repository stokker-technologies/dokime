# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Authoring helpers that export suite definitions."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from typing import cast

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core._validation import stable_id
from dokime.core.authoring import AuthoredParameterDefinition
from dokime.core.authoring import TestCaseTemplate as AuthoredTestCaseTemplate
from dokime.core.authoring import TestSuite as AuthoredTestSuite
from dokime.core.suites import CaseTemplate
from dokime.core.suites import Suite
from dokime.core.suites import CriterionDefinition
from dokime.core.suites import ExecutionPhaseDefinition
from dokime.core.suites import FileDefinition
from dokime.core.suites import ParameterDefinition
from dokime.core.metadata import Metadata

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def suite_from_authored_suite(suite: type[AuthoredTestSuite]) -> Suite:
    """Convert authored template classes to one Rust-validated suite."""

    return Suite(
        metadata=Metadata(
            suite.resolved_suite_id(),
            suite.resolved_name(),
            suite.description,
        ),
        tags=list(suite.tags),
        templates=[_case_template(template) for template in suite.members],
    )


# === Private Functions === #


def _case_template(template: type[AuthoredTestCaseTemplate]) -> CaseTemplate:
    metadata_model = template.metadata_model()
    return CaseTemplate(
        metadata=Metadata(
            template.resolved_template_id(),
            template.resolved_name(),
            template.resolved_description(),
        ),
        template_id=stable_id("template", template.resolved_template_id()),
        tags=list(template.tags),
        parameters=[
            _parameter_definition(definition)
            for definition in template.parameter_definitions()
        ],
        files=[
            _file_definition(definition) for definition in template.file_definitions()
        ],
        phases=[
            ExecutionPhaseDefinition(
                repeatable=phase.repeatable,
                metadata=Metadata(
                    phase.step_id,
                    phase.label or _title(phase.step_id),
                    phase.description,
                ),
            )
            for phase in template.step_definitions()
        ],
        criteria=[
            CriterionDefinition(
                metadata=Metadata(
                    criterion.criterion_id,
                    criterion.label or _title(criterion.criterion_id),
                )
            )
            for criterion in template.criterion_definitions()
        ],
        continue_on_failure=bool(template.continue_on_failure),
        prompt_definitions=template.resolved_prompt_definitions(),
        subscriptions=template.subscription_sources(),
        attributes=_template_attributes(template),
        document=cast(dict[str, object] | None, metadata_model.document),
    )


def _parameter_definition(
    definition: AuthoredParameterDefinition,
) -> ParameterDefinition:
    return ParameterDefinition(
        metadata=Metadata(
            definition.name,
            definition.label or _title(definition.name),
            definition.description,
        ),
        value_type=definition.value_type or "text",
        min=definition.min,
        max=definition.max,
        element_type=definition.element_type,
        min_items=definition.min_items,
        max_items=definition.max_items,
        default=_parameter_default(definition),
        values=list(definition.values),
        required=False,
        unit=_unit(definition.unit),
        attributes={
            **dict(definition.metadata),
            **({"choices": list(definition.choices)} if definition.choices else {}),
            "expansion": {"kind": definition.expansion}
            if isinstance(definition.expansion, str)
            else definition.expansion,
        },
    )


def _template_attributes(template: type[AuthoredTestCaseTemplate]) -> dict[str, object]:
    attributes = dict(template.metadata)
    return attributes


def _file_definition(definition: object) -> FileDefinition:
    return FileDefinition(
        metadata=Metadata(
            getattr(definition, "name"),
            getattr(definition, "label") or _title(getattr(definition, "name")),
            getattr(definition, "description"),
        ),
        default=getattr(definition, "default"),
        required=bool(getattr(definition, "required")),
        media_types=list(getattr(definition, "media_types")),
        attributes=dict(getattr(definition, "metadata")),
    )


def _parameter_default(definition: AuthoredParameterDefinition) -> object | None:
    if not definition.values:
        return None
    return definition.values[0]


def _unit(unit: str | None) -> dict[str, object] | None:
    if unit is None:
        return None
    return _native.measurement_unit_contract(unit)


def _title(identifier: str) -> str:
    return identifier.replace(".", " ").replace("_", " ").replace("-", " ").title()


# === Entry Point === #
