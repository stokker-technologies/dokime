# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from __future__ import annotations

from collections.abc import Awaitable
from os import PathLike
from typing import ClassVar, final

from . import (
    client as client,
    authoring as authoring,
    core_contracts as core_contracts,
    server as server,
)

__all__ = [
    "authoring",
    "client",
    "core_contracts",
    "server",
    "Runtime",
    "AuthoringContext",
    "Artifacts",
    "ArtifactWriter",
    "ServerConfig",
    "TlsIdentity",
    "BearerGrant",
    "Role",
    "host_stream_contracts",
    "new_template_id",
    "new_parameter_id",
    "new_step_id",
    "new_criterion_id",
    "new_case_id",
    "identifier_contract",
    "metadata_contract",
    "measurement_unit_contract",
    "value_type_name",
    "nullable_value_type",
    "parameter_definition_contract",
    "prompt_definition_contract",
    "file_definition_contract",
    "execution_phase_definition_contract",
    "criterion_definition_contract",
    "case_template_contract",
    "suite_contract",
    "materialized_case_contract",
    "materialize_template_cases_contract",
    "stable_wire_key",
    "stable_wire_hash",
    "record_field_contract",
    "record_contract",
    "telemetry_column_contract",
    "telemetry_contract",
    "document_plot_contract",
    "suite_registration_contract",
    "validate_identifier",
    "validate_metadata",
    "validate_parameter_definition",
    "validate_execution_phase_definition",
    "validate_criterion_definition",
    "validate_case_template",
    "validate_suite",
    "validate_materialized_case",
    "validate_suite_registration",
    "builtin_record_contracts",
    "write_json_file",
    "write_json_lines",
]

@final
class Role:
    Read: ClassVar[Role]
    Control: ClassVar[Role]
    Admin: ClassVar[Role]
    def __int__(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...

@final
class BearerGrant:
    def __new__(cls, subject: str, role: Role, token: str) -> BearerGrant: ...

@final
class TlsIdentity:
    @staticmethod
    def from_pem(certificate: bytes, private_key: bytes) -> TlsIdentity: ...

@final
class ServerConfig:
    @staticmethod
    def local(
        *,
        host: str = ...,
        port: int = ...,
        max_message_bytes: int = ...,
        grants: list[BearerGrant] | None = ...,
    ) -> ServerConfig: ...
    @staticmethod
    def remote(
        identity: TlsIdentity,
        grants: list[BearerGrant],
        *,
        host: str = ...,
        port: int = ...,
        max_message_bytes: int = ...,
    ) -> ServerConfig: ...

@final
class Runtime:
    def __new__(
        cls,
        storage_root: str | PathLike[str] | None = None,
        application_root: str | PathLike[str] | None = None,
        plugins_root: str | PathLike[str] | None = None,
    ) -> Runtime: ...
    def register(self, provider: object) -> Awaitable[None]: ...
    def start(self, server_config: ServerConfig | None = None) -> Awaitable[str]: ...
    def close(self) -> Awaitable[None]: ...

def new_template_id() -> str: ...
def new_parameter_id() -> str: ...
def new_step_id() -> str: ...
def new_criterion_id() -> str: ...
def new_case_id() -> str: ...
def identifier_contract(value: str) -> str: ...
def metadata_contract(
    identifier: str,
    label: str,
    description: str | None,
    tags: list[str] | None,
    attributes: dict[str, object] | None = None,
    created_at: object | None = None,
    updated_at: object | None = None,
) -> dict[str, object]: ...
def measurement_unit_contract(symbol: str) -> dict[str, object]: ...
def value_type_name(value_type: object) -> str: ...
def nullable_value_type(value_type: object) -> bool: ...
def parameter_definition_contract(
    parameter_id: str,
    metadata: dict[str, object],
    value_type: str,
    default: object,
    values: list[object],
    required: bool,
    unit: dict[str, object] | None,
    group: str | None,
    attributes: dict[str, object],
) -> dict[str, object]: ...
def prompt_definition_contract(
    prompt_type: str,
    title: str,
    message: str | None = None,
    submit: str | None = None,
    cancel: str | None = None,
    fields: list[dict[str, object]] | None = None,
) -> dict[str, object]: ...
def file_definition_contract(
    metadata: dict[str, object],
    default: str | None,
    required: bool,
    media_types: list[str],
    attributes: dict[str, object],
) -> dict[str, object]: ...
def execution_phase_definition_contract(
    step_id: str,
    metadata: dict[str, object],
    repeatable: bool,
) -> dict[str, object]: ...
def criterion_definition_contract(
    criterion_id: str,
    metadata: dict[str, object],
    expected: object,
) -> dict[str, object]: ...
def case_template_contract(
    template_id: str,
    metadata: dict[str, object],
    tags: list[str],
    parameters: list[dict[str, object]],
    files: list[dict[str, object]],
    document: dict[str, object] | None,
    phases: list[dict[str, object]],
    criteria: list[dict[str, object]],
    continue_on_failure: bool,
    prompt_definitions: list[dict[str, object]],
    subscriptions: list[dict[str, str]],
    attributes: dict[str, object],
) -> dict[str, object]: ...
def suite_contract(
    metadata: dict[str, object],
    tags: list[str],
    templates: list[dict[str, object]],
) -> dict[str, object]: ...
def materialized_case_contract(
    suite_id: str,
    case_id: str,
    template_id: str,
    metadata: dict[str, object],
    parameters: dict[str, object],
    generation_index: int | None,
) -> dict[str, object]: ...
def materialize_template_cases_contract(
    suite_id: str,
    template: dict[str, object],
    case_filter: dict[str, list[object]],
    case_overrides: dict[str, list[object]],
    file_overrides: dict[str, str | None],
    max_cases: int,
    force: bool,
) -> dict[str, object]: ...
def stable_wire_key(value: object) -> str: ...
def stable_wire_hash(value: object) -> str: ...
def record_field_contract(
    name: str,
    label: str | None,
    description: str | None,
    value_type: object,
    required: bool,
    unit: str | None,
) -> dict[str, object]: ...
def record_contract(
    record_type: str,
    label: str | None,
    description: str | None,
    default_level: str,
    allow_extra_fields: bool,
    fields: list[dict[str, object]],
    template_scope: str | None,
    icon: str | None,
    tone: str | None,
) -> dict[str, object]: ...
def telemetry_column_contract(
    name: str,
    label: str | None,
    description: str | None,
    value_type: object,
    role: str | None,
    unit: str | None,
    required: bool,
) -> dict[str, object]: ...
def telemetry_contract(
    stream: str,
    label: str | None,
    description: str | None,
    columns: list[dict[str, object]],
    default_x: str | None,
    allow_extra_fields: bool,
    scalar: bool,
    template_scope: str | None,
) -> dict[str, object]: ...
def document_plot_contract(
    label: str,
    plot_type: str,
    identifier: str | None,
    stream: str | None,
    x: str | None,
    y: list[str],
    z_stream: str | None,
    z_field: str | None,
    series_by: str | None,
    x_expression: dict[str, object] | None,
    y_expressions: list[dict[str, object]],
    z_expression: dict[str, object] | None,
    series_by_expression: dict[str, object] | None,
    filters: list[dict[str, object]],
    group: str | None,
    unit: str | None,
    style: str | None,
    x_scale: str | None,
    y_scale: str | None,
    fft: dict[str, object] | None,
    interval: dict[str, object] | None,
    histogram: dict[str, object] | None,
    marker_record_types: list[str],
    reference_lines: list[dict[str, object]],
    reference_bands: list[dict[str, object]],
    live_window_s: float | None,
    x_format: dict[str, object] | None,
    y_format: dict[str, object] | None,
    analysis: dict[str, object] | None,
) -> dict[str, object]: ...
def suite_registration_contract(
    suite_id: str,
    product: str | None,
    tags: list[str],
    workflow: str | None,
    suite: dict[str, object],
    record_contracts: list[dict[str, object]],
    telemetry_contracts: list[dict[str, object]],
) -> dict[str, object]: ...
def validate_identifier(payload: object) -> object: ...
def validate_metadata(payload: object) -> dict[str, object]: ...
def validate_parameter_definition(payload: object) -> dict[str, object]: ...
def validate_execution_phase_definition(payload: object) -> dict[str, object]: ...
def validate_criterion_definition(payload: object) -> dict[str, object]: ...
def validate_case_template(payload: object) -> dict[str, object]: ...
def validate_suite(payload: object) -> dict[str, object]: ...
def validate_materialized_case(payload: object) -> dict[str, object]: ...
def validate_suite_registration(payload: object) -> dict[str, object]: ...
def builtin_record_contracts() -> list[dict[str, object]]: ...
def write_json_file(path: str, payload: object) -> None: ...
def write_json_lines(path: str, payloads: list[object]) -> None: ...

@final
class AuthoringContext:
    @property
    def artifacts(self) -> Artifacts: ...
    @property
    def parameters(self) -> dict[str, object]: ...
    @property
    def evidence(self) -> dict[str, object]: ...
    def request(self, operation: str, payload: object) -> Awaitable[object]: ...

def host_stream_contracts() -> dict[str, dict[str, object]]: ...

@final
class Artifacts:
    def create(self, name: str) -> ArtifactWriter: ...
    def directory(self, name: str) -> PathLike[str]: ...
    def import_file(self, path: str | PathLike[str], name: str) -> None: ...

@final
class ArtifactWriter:
    def write_all(self, data: bytes) -> None: ...
    def flush(self) -> None: ...
    def finish(self) -> None: ...
    def abandon(self) -> None: ...
