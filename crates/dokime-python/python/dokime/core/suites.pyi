# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from __future__ import annotations

from typing import TypedDict, type_check_only

from dokime.core.metadata import Metadata

@type_check_only
class MetadataContract(TypedDict):
    uuid: str
    created_at: str
    id: str
    identifier: str
    label: str
    description: str | None
    tags: list[str]

@type_check_only
class ParameterListContract(TypedDict):
    element_type: str
    min_items: int | None
    max_items: int | None

@type_check_only
class ParameterDefinitionContract(TypedDict):
    expansion: dict[str, str]
    parameter_id: str
    metadata: MetadataContract
    group_id: str | None
    value_type: str
    unit: dict[str, object] | None
    required: bool
    min: float | None
    max: float | None
    expression: dict[str, object]
    runtime_writable: bool
    selection: dict[str, object] | None
    list: ParameterListContract | None
    extensions: dict[str, object]

@type_check_only
class FileDefinitionContract(TypedDict):
    metadata: MetadataContract
    default: str | None
    required: bool
    media_types: list[str]
    attributes: dict[str, object]

@type_check_only
class ExecutionPhaseDefinitionContract(TypedDict):
    repeatable: bool
    step_id: str
    metadata: MetadataContract

@type_check_only
class CriterionDefinitionContract(TypedDict):
    criterion_id: str
    metadata: MetadataContract
    expected: object

@type_check_only
class CaseTemplateContract(TypedDict):
    test_case_template_id: str
    metadata: MetadataContract
    tags: list[str]
    parameter_groups: list[dict[str, object]]
    parameters: list[ParameterDefinitionContract]
    files: list[FileDefinitionContract]
    document: dict[str, object] | None
    phases: list[ExecutionPhaseDefinitionContract]
    criteria: list[CriterionDefinitionContract]
    continue_on_failure: bool
    prompt_definitions: list[dict[str, object]]
    subscriptions: list[dict[str, str]]
    attributes: dict[str, object]

@type_check_only
class SuiteContract(TypedDict):
    metadata: MetadataContract
    tags: list[str]
    parameter_groups: list[dict[str, object]]
    test_case_templates: list[CaseTemplateContract]

@type_check_only
class MaterializedCaseContract(TypedDict):
    suite_id: str
    test_case_id: str
    test_case_template_id: str
    metadata: MetadataContract
    parameters: dict[str, object]
    generation_index: int

class ParameterDefinition:
    metadata: Metadata
    value_type: str
    default: object | None
    values: list[object]
    required: bool
    unit: dict[str, object] | None
    group: str | None
    attributes: dict[str, object]
    expansion: str | dict[str, str]
    expression: object | None
    parameter_id: str
    min: float | None
    max: float | None
    element_type: str | None
    min_items: int | None
    max_items: int | None
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        value_type: str,
        default: object | None = ...,
        values: list[object] = ...,
        required: bool = ...,
        unit: dict[str, object] | None = ...,
        group: str | None = ...,
        attributes: dict[str, object] = ...,
        expansion: str | dict[str, str] = ...,
        expression: object | None = ...,
        parameter_id: str = ...,
        min: float | None = ...,
        max: float | None = ...,
        element_type: str | None = ...,
        min_items: int | None = ...,
        max_items: int | None = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> ParameterDefinition: ...
    def to_contract(self) -> ParameterDefinitionContract: ...

class FileDefinition:
    metadata: Metadata
    default: str | None
    required: bool
    media_types: list[str]
    attributes: dict[str, object]
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        default: str | None = ...,
        required: bool = ...,
        media_types: list[str] = ...,
        attributes: dict[str, object] = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> FileDefinition: ...
    def to_contract(self) -> FileDefinitionContract: ...

class ExecutionPhaseDefinition:
    metadata: Metadata
    repeatable: bool
    step_id: str
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        step_id: str = ...,
        repeatable: bool = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> ExecutionPhaseDefinition: ...
    def to_contract(self) -> ExecutionPhaseDefinitionContract: ...

class CriterionDefinition:
    metadata: Metadata
    expected: object | None
    criterion_id: str
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        expected: object | None = ...,
        criterion_id: str = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> CriterionDefinition: ...
    def to_contract(self) -> CriterionDefinitionContract: ...

class CaseTemplate:
    metadata: Metadata
    parameters: list[ParameterDefinition]
    files: list[FileDefinition]
    phases: list[ExecutionPhaseDefinition]
    criteria: list[CriterionDefinition]
    tags: list[str]
    continue_on_failure: bool
    prompt_definitions: list[dict[str, object]]
    subscriptions: list[dict[str, str]]
    attributes: dict[str, object]
    document: dict[str, object] | None
    template_id: str
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        parameters: list[ParameterDefinition] = ...,
        files: list[FileDefinition] = ...,
        phases: list[ExecutionPhaseDefinition] = ...,
        criteria: list[CriterionDefinition] = ...,
        tags: list[str] = ...,
        continue_on_failure: bool = ...,
        prompt_definitions: list[dict[str, object]] = ...,
        subscriptions: list[dict[str, str]] = ...,
        attributes: dict[str, object] = ...,
        document: dict[str, object] | None = ...,
        template_id: str = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> CaseTemplate: ...
    def to_contract(self) -> CaseTemplateContract: ...

class Suite:
    metadata: Metadata
    tags: list[str]
    templates: list[CaseTemplate]
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        metadata: Metadata,
        tags: list[str] = ...,
        templates: list[CaseTemplate] = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> Suite: ...
    def to_contract(self) -> SuiteContract: ...

class MaterializedCase:
    suite_id: str
    template_id: str
    metadata: Metadata
    parameters: dict[str, object]
    generation_index: int
    case_id: str
    __match_args__: tuple[str, ...]
    def __init__(
        self,
        suite_id: str,
        template_id: str,
        metadata: Metadata,
        parameters: dict[str, object] = ...,
        generation_index: int = ...,
        case_id: str = ...,
    ) -> None: ...
    def __replace__(self, /, **changes: object) -> MaterializedCase: ...
    def to_contract(self) -> MaterializedCaseContract: ...
