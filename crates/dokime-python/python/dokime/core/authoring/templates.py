# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Template and suite base classes for authored tests."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import inspect
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any, ClassVar, Self, cast

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core._validation import stable_id
from dokime.core.authoring.decorators import (
    CRITERION_ATTRIBUTE,
    CLEANUP_ATTRIBUTE,
    INITIALIZE_ATTRIBUTE,
    SAMPLER_ATTRIBUTE,
    STEP_ATTRIBUTE,
    _class_members,
    _decorated_methods,
    _phase_definitions,
    snake_case_identifier,
    title_from_identifier,
)
from dokime.core.authoring.definitions import (
    CleanupDefinition,
    InitializeDefinition,
    SamplerDefinition,
    StepDefinition,
    ValidationCriterionDefinition,
)
from dokime.core.authoring.parameters import (
    AuthoredParameterDefinition,
    FileDefinition,
    FileDescriptor,
    ParameterDescriptor,
    TestCaseMaterializationError,
    _contract_value,
    _default_value,
    _value_from_contract,
)
from dokime.core.authoring.prompts import declared_prompt
from dokime.core.declarations import JsonObject

# Type Checking Imports

# === Public Exports === #

# === Constants === #

DEFAULT_MAX_MATERIALIZED_CASES = 1000

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class TestCase:
    suite_id: str
    case_id: str
    template_id: str
    parameters: JsonObject
    generation_index: int
    label: str
    metadata: JsonObject = field(default_factory=dict)


@dataclass(frozen=True)
class TestTemplateMetadata:
    template_id: str
    name: str
    description: str | None
    parameters: list[AuthoredParameterDefinition] = field(default_factory=list)
    files: list[FileDefinition] = field(default_factory=list)
    steps: list[StepDefinition] = field(default_factory=list)
    samplers: list[SamplerDefinition] = field(default_factory=list)
    criteria: list[ValidationCriterionDefinition] = field(default_factory=list)
    initialize_hooks: list[InitializeDefinition] = field(default_factory=list)
    cleanup_hooks: list[CleanupDefinition] = field(default_factory=list)
    record_contracts: list[JsonObject] = field(default_factory=list)
    telemetry_contracts: list[JsonObject] = field(default_factory=list)
    document: object | None = None
    metadata: JsonObject = field(default_factory=dict)
    source_file: str | None = None

    def model_copy(self, *, update: Mapping[str, object] | None = None) -> Self:
        return cast(Self, replace(self, **dict(update or {})))


class TestCaseTemplate:
    """Base class for reusable test case definitions."""

    __test__: ClassVar[bool] = False
    template_id: ClassVar[str | None] = None
    name: ClassVar[str | None] = None
    description: ClassVar[str | None] = None
    path: ClassVar[Sequence[str]] = ()
    tags: ClassVar[Sequence[str]] = ()
    metadata: ClassVar[JsonObject] = {}
    source_file: ClassVar[str | None] = None
    document: ClassVar[object | None] = None
    records: ClassVar[Sequence[type]] = ()
    streams: ClassVar[Sequence[type]] = ()
    subscribes: ClassVar[Sequence[type]] = ()
    continue_on_failure: ClassVar[bool] = False
    # The questions this template asks, named by their `@dokime.prompt` class
    # the way records are named by type. `prompt_definitions` remains the
    # programmatic path for a definition built by hand.
    prompts: ClassVar[Sequence[type[object]]] = ()
    prompt_definitions: ClassVar[list[dict[str, object]]] = []

    def __init__(
        self,
        case_parameters: Mapping[str, object] | None = None,
    ) -> None:
        self._case_parameters = dict(case_parameters or {})

    @classmethod
    def resolved_template_id(cls) -> str:
        return cls.template_id or snake_case_identifier(cls.__name__)

    @classmethod
    def resolved_name(cls) -> str:
        return cls.name or title_from_identifier(cls.__name__)

    @classmethod
    def resolved_description(cls) -> str | None:
        return cls.description or inspect.getdoc(cls)

    @classmethod
    def parameter_definitions(cls) -> list[AuthoredParameterDefinition]:
        return [
            descriptor.resolved_definition()
            for _, descriptor in _class_members(cls, ParameterDescriptor)
        ]

    @classmethod
    def file_definitions(cls) -> list[FileDefinition]:
        return [
            descriptor.definition
            for _, descriptor in _class_members(cls, FileDescriptor)
        ]

    @classmethod
    def resolved_prompt_definitions(cls) -> list[dict[str, object]]:
        """Every question this template asks: the declared classes named in
        `prompts`, then any definition built by hand."""
        return [declared_prompt(declared).definition() for declared in cls.prompts] + [
            dict(definition) for definition in cls.prompt_definitions
        ]

    @classmethod
    def subscription_sources(cls) -> list[dict[str, str]]:
        sources = []
        for declared in cls.subscribes:
            if hasattr(declared, "__dokime_stream__"):
                sources.append(
                    {"kind": "stream", "identifier": declared.__dokime_stream__.stream}
                )
            elif hasattr(declared, "__dokime_record__"):
                sources.append(
                    {
                        "kind": "record",
                        "identifier": declared.__dokime_record__.identifier,
                    }
                )
            else:
                raise TypeError("A subscription must name a declared stream or record")
        return sources

    @classmethod
    def step_definitions(cls) -> list[StepDefinition]:
        return _phase_definitions(cls, STEP_ATTRIBUTE, StepDefinition, "step_id")

    @classmethod
    def sampler_definitions(cls) -> list[SamplerDefinition]:
        return [
            SamplerDefinition(
                method_name=method_name, **getattr(method, SAMPLER_ATTRIBUTE)
            )
            for method_name, method in _decorated_methods(cls, SAMPLER_ATTRIBUTE)
        ]

    @classmethod
    def stream_types(cls) -> tuple[type, ...]:
        return tuple(
            dict.fromkeys(
                [*cls.streams, *(item.row_type for item in cls.sampler_definitions())]
            )
        )

    @classmethod
    def criterion_definitions(cls) -> list[ValidationCriterionDefinition]:
        return _phase_definitions(
            cls, CRITERION_ATTRIBUTE, ValidationCriterionDefinition, "criterion_id"
        )

    @classmethod
    def initialize_definitions(cls) -> list[InitializeDefinition]:
        return _phase_definitions(
            cls, INITIALIZE_ATTRIBUTE, InitializeDefinition, "initialize_id"
        )

    @classmethod
    def cleanup_definitions(cls) -> list[CleanupDefinition]:
        return _phase_definitions(
            cls, CLEANUP_ATTRIBUTE, CleanupDefinition, "cleanup_id"
        )

    @classmethod
    def record_contract_definitions(cls) -> list[JsonObject]:
        return [
            item.__dokime_record__.to_contract(template_id=cls.resolved_template_id())
            for item in cls.records
        ]

    @classmethod
    def telemetry_contract_definitions(cls) -> list[JsonObject]:
        return [
            item.__dokime_stream__.to_contract(template_id=cls.resolved_template_id())
            for item in cls.stream_types()
        ]

    @classmethod
    def metadata_model(cls) -> TestTemplateMetadata:
        document = _document_contract(
            cls.document,
            template_id=cls.resolved_template_id(),
            template_name=cls.resolved_name(),
        )
        return TestTemplateMetadata(
            template_id=cls.resolved_template_id(),
            name=cls.resolved_name(),
            description=cls.resolved_description(),
            source_file=cls.source_file,
            parameters=cls.parameter_definitions(),
            files=cls.file_definitions(),
            steps=cls.step_definitions(),
            samplers=cls.sampler_definitions(),
            criteria=cls.criterion_definitions(),
            initialize_hooks=cls.initialize_definitions(),
            cleanup_hooks=cls.cleanup_definitions(),
            record_contracts=cls.record_contract_definitions(),
            telemetry_contracts=cls.telemetry_contract_definitions(),
            document=document,
            metadata=dict(cls.metadata),
        )

    @classmethod
    def expand_case_parameters(cls, parameters: Mapping[str, Any]) -> list[JsonObject]:
        return [dict(parameters)]

    @classmethod
    def materialize(
        cls,
        *,
        suite_id: str,
        case_filter: Mapping[str, Iterable[Any]] | None = None,
        case_overrides: Mapping[str, Iterable[Any]] | None = None,
        file_overrides: Mapping[str, str | Path | None] | None = None,
        max_cases: int = DEFAULT_MAX_MATERIALIZED_CASES,
        force: bool = False,
    ) -> list[TestCase]:
        try:
            output = _native.materialize_template_cases_contract(
                suite_id,
                _template_contract_for_materialization(cls),
                _case_values(case_filter),
                _case_values(case_overrides),
                _file_values(file_overrides),
                max_cases,
                force,
            )
        except ValueError as error:
            raise TestCaseMaterializationError(str(error)) from error

        materialized_cases = cast(
            list[dict[str, object]],
            output.get("test_cases", output.get("cases", [])),
        )
        cases: list[TestCase] = []
        for materialized in materialized_cases:
            parameters = {
                key: _value_from_contract(value)
                for key, value in cast(JsonObject, materialized["parameters"]).items()
            }
            expansions = cls.expand_case_parameters(parameters)
            for expanded in expansions:
                _validate_case_parameters(cls, expanded)
                cases.append(
                    TestCase(
                        suite_id=suite_id,
                        case_id=(
                            cast(str, materialized["test_case_id"])
                            if len(expansions) == 1 and expanded == parameters
                            else _native.new_case_id()
                        ),
                        template_id=cls.resolved_template_id(),
                        parameters=dict(expanded),
                        generation_index=len(cases),
                        label=_case_label(cls.resolved_name(), expanded),
                    )
                )
        return cases

    @classmethod
    def case(cls, *, suite_id: str, **parameters: Any) -> TestCase:
        if not suite_id.strip():
            raise TestCaseMaterializationError("Suite registration ID is required.")

        expected = {definition.name for definition in cls.parameter_definitions()}
        file_names = {definition.name for definition in cls.file_definitions()}
        provided = set(parameters)
        missing = expected - provided
        unknown = provided - expected - file_names
        if missing:
            raise TestCaseMaterializationError(
                f"Missing parameters: {', '.join(sorted(missing))}."
            )
        if unknown:
            raise TestCaseMaterializationError(
                f"Unknown parameters: {', '.join(sorted(unknown))}."
            )
        _validate_case_parameters(cls, parameters)

        return TestCase(
            suite_id=suite_id,
            case_id=_native.new_case_id(),
            template_id=cls.resolved_template_id(),
            parameters=dict(parameters),
            generation_index=0,
            label=_case_label(cls.resolved_name(), parameters),
        )


class TestSuite:
    __test__: ClassVar[bool] = False
    suite_id: ClassVar[str | None] = None
    name: ClassVar[str | None] = None
    version: ClassVar[str | None] = None
    description: ClassVar[str | None] = None
    path: ClassVar[Sequence[str]] = ()
    tags: ClassVar[Sequence[str]] = ()
    continue_on_failure: ClassVar[bool] = True
    members: ClassVar[Sequence[type[TestCaseTemplate]]] = ()
    metadata: ClassVar[JsonObject] = {}

    @classmethod
    def resolved_suite_id(cls) -> str:
        return cls.suite_id or snake_case_identifier(cls.__name__)

    @classmethod
    def resolved_name(cls) -> str:
        return cls.name or title_from_identifier(cls.__name__)

    @classmethod
    def parameters(cls) -> Mapping[str, Any]:
        return {}


# === Public Functions === #

# === Private Functions === #


def _clean(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def _document_contract(
    value: object | None,
    *,
    template_id: str,
    template_name: str,
) -> object | None:
    if value is None:
        return None
    if callable(to_contract := getattr(value, "to_contract", None)):
        return _json_compatible(to_contract())
    if callable(to_document := getattr(value, "to_document", None)):
        return _json_compatible(
            to_document(template_id=template_id, template_name=template_name)
        )
    return _json_compatible(value)


def _json_compatible(value: object) -> object:
    if value is None or isinstance(value, bool | int | float | str):
        return value
    if isinstance(value, Mapping):
        return {str(key): _json_compatible(item) for key, item in value.items()}
    if isinstance(value, Sequence) and not isinstance(value, str | bytes | bytearray):
        return [_json_compatible(item) for item in value]
    return str(value)


def _template_contract_for_materialization(
    cls: type[TestCaseTemplate],
) -> dict[str, object]:
    return _native.case_template_contract(
        stable_id("template", cls.resolved_template_id()),
        _native.metadata_contract(
            cls.resolved_template_id(),
            cls.resolved_name(),
            cls.resolved_description(),
            list(cls.tags),
        ),
        list(cls.tags),
        [
            _parameter_contract_for_materialization(definition)
            for definition in cls.parameter_definitions()
        ],
        [
            _file_contract_for_materialization(definition)
            for definition in cls.file_definitions()
        ],
        None,
        [],
        [],
        bool(cls.continue_on_failure),
        cls.resolved_prompt_definitions(),
        cls.subscription_sources(),
        dict(cls.metadata),
    )


def _parameter_contract_for_materialization(
    definition: AuthoredParameterDefinition,
) -> dict[str, object]:
    return _native.parameter_definition_contract(
        _native.new_parameter_id(),
        _native.metadata_contract(
            definition.name,
            definition.label or title_from_identifier(definition.name),
            definition.description,
            [],
        ),
        definition.value_type or "text",
        _contract_value(_default_value(definition)),
        [_contract_value(value) for value in definition.values],
        False,
        None
        if definition.unit is None
        else _native.measurement_unit_contract(definition.unit),
        None,
        {
            **dict(definition.metadata),
            **{
                name: getattr(definition, name)
                for name in ("min", "max", "element_type", "min_items", "max_items")
                if getattr(definition, name) is not None
            },
            **(
                {"choices": [_contract_value(choice) for choice in definition.choices]}
                if definition.choices
                else {}
            ),
            "expansion": {"kind": definition.expansion}
            if isinstance(definition.expansion, str)
            else definition.expansion,
        },
    )


def _file_contract_for_materialization(definition: FileDefinition) -> dict[str, object]:
    return _native.file_definition_contract(
        _native.metadata_contract(
            definition.name,
            definition.label or title_from_identifier(definition.name),
            definition.description,
            [],
        ),
        definition.default,
        definition.required,
        list(definition.media_types),
        dict(definition.metadata),
    )


def _case_values(
    values: Mapping[str, Iterable[Any]] | None,
) -> dict[str, list[object]]:
    return (
        {}
        if values is None
        else {
            name: [_contract_value(item) for item in items]
            for name, items in values.items()
        }
    )


def _file_values(
    values: Mapping[str, str | Path | None] | None,
) -> dict[str, str | None]:
    if values is None:
        return {}
    return {
        name: None if value is None else str(value) for name, value in values.items()
    }


def _validate_case_parameters(
    cls: type[TestCaseTemplate],
    parameters: Mapping[str, Any],
) -> None:
    descriptors = dict(_class_members(cls, ParameterDescriptor))
    for name, value in parameters.items():
        descriptor = descriptors.get(name)
        if descriptor is not None:
            descriptor.validate_value(value)


def _case_label(template_name: str, parameters: Mapping[str, object]) -> str:
    if not parameters:
        return template_name
    parts = [f"{name}={value}" for name, value in parameters.items()]
    return f"{template_name} ({', '.join(parts)})"


# === Entry Point === #
