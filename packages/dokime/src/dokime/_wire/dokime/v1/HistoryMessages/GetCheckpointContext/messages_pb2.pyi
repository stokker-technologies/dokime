from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("checkpoint",)
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("checkpoint", "session_id", "execution_id", "execution_scope", "cycle", "execution_documents", "prompts", "health", "notes", "traceability", "appearance")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_SCOPE_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_DOCUMENTS_FIELD_NUMBER: _ClassVar[int]
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    session_id: str
    execution_id: str
    execution_scope: Row4
    cycle: Row5
    execution_documents: _containers.RepeatedCompositeFieldContainer[Row6]
    prompts: _containers.RepeatedCompositeFieldContainer[Row7]
    health: _containers.RepeatedCompositeFieldContainer[Row8]
    notes: _containers.RepeatedCompositeFieldContainer[Row9]
    traceability: _containers.RepeatedCompositeFieldContainer[Row10]
    appearance: Row11
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., session_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., execution_scope: _Optional[_Union[Row4, _Mapping]] = ..., cycle: _Optional[_Union[Row5, _Mapping]] = ..., execution_documents: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., prompts: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., health: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., notes: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., traceability: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., appearance: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row12
    version: bytes
    evidence_through: Row12
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row12, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row13
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("cycle", "execution_order")
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ORDER_FIELD_NUMBER: _ClassVar[int]
    cycle: Row14
    execution_order: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, cycle: _Optional[_Union[Row14, _Mapping]] = ..., execution_order: _Optional[_Iterable[str]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("presentation", "layout_nodes", "cards", "table_rows")
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_NODES_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    TABLE_ROWS_FIELD_NUMBER: _ClassVar[int]
    presentation: Row15
    layout_nodes: _containers.RepeatedCompositeFieldContainer[Row16]
    cards: _containers.RepeatedCompositeFieldContainer[Row17]
    table_rows: _containers.RepeatedCompositeFieldContainer[Row18]
    def __init__(self, presentation: _Optional[_Union[Row15, _Mapping]] = ..., layout_nodes: _Optional[_Iterable[_Union[Row16, _Mapping]]] = ..., cards: _Optional[_Iterable[_Union[Row17, _Mapping]]] = ..., table_rows: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("prompt_id", "definition", "kind", "values", "scope", "created_at")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    definition: Row19
    kind: Row20
    values: Row21
    scope: Row4
    created_at: Row22
    def __init__(self, prompt_id: _Optional[str] = ..., definition: _Optional[_Union[Row19, _Mapping]] = ..., kind: _Optional[_Union[Row20, _Mapping]] = ..., values: _Optional[_Union[Row21, _Mapping]] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., created_at: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("health_item_id", "scope", "severity", "message", "detail", "observed_at")
    HEALTH_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    health_item_id: str
    scope: Row4
    severity: Row23
    message: str
    detail: str
    observed_at: Row22
    def __init__(self, health_item_id: _Optional[str] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., severity: _Optional[_Union[Row23, _Mapping]] = ..., message: _Optional[str] = ..., detail: _Optional[str] = ..., observed_at: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("note_id", "scope", "author", "message", "created_at")
    NOTE_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AUTHOR_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    note_id: str
    scope: Row4
    author: str
    message: str
    created_at: Row22
    def __init__(self, note_id: _Optional[str] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., author: _Optional[str] = ..., message: _Optional[str] = ..., created_at: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("traceability_id", "scope", "created_at", "entries", "metadata")
    TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    traceability_id: str
    scope: Row4
    created_at: Row22
    entries: _containers.RepeatedCompositeFieldContainer[Row24]
    metadata: Row25
    def __init__(self, traceability_id: _Optional[str] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., created_at: _Optional[_Union[Row22, _Mapping]] = ..., entries: _Optional[_Iterable[_Union[Row24, _Mapping]]] = ..., metadata: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("app_name", "organization", "title_template", "logo", "themes", "active_theme", "theme_family_locked", "mode_policy", "mode_locked", "source_plugin_id")
    APP_NAME_FIELD_NUMBER: _ClassVar[int]
    ORGANIZATION_FIELD_NUMBER: _ClassVar[int]
    TITLE_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    LOGO_FIELD_NUMBER: _ClassVar[int]
    THEMES_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_THEME_FIELD_NUMBER: _ClassVar[int]
    THEME_FAMILY_LOCKED_FIELD_NUMBER: _ClassVar[int]
    MODE_POLICY_FIELD_NUMBER: _ClassVar[int]
    MODE_LOCKED_FIELD_NUMBER: _ClassVar[int]
    SOURCE_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    app_name: str
    organization: Row26
    title_template: str
    logo: Row27
    themes: _containers.RepeatedCompositeFieldContainer[Row28]
    active_theme: str
    theme_family_locked: bool
    mode_policy: Row29
    mode_locked: bool
    source_plugin_id: str
    def __init__(self, app_name: _Optional[str] = ..., organization: _Optional[_Union[Row26, _Mapping]] = ..., title_template: _Optional[str] = ..., logo: _Optional[_Union[Row27, _Mapping]] = ..., themes: _Optional[_Iterable[_Union[Row28, _Mapping]]] = ..., active_theme: _Optional[str] = ..., theme_family_locked: _Optional[bool] = ..., mode_policy: _Optional[_Union[Row29, _Mapping]] = ..., mode_locked: _Optional[bool] = ..., source_plugin_id: _Optional[str] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row30
    sequence: int
    def __init__(self, history: _Optional[_Union[Row30, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row31
    def __init__(self, kind: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("cycle_id", "metadata", "lifecycle", "created_at", "started_at", "updated_at", "cases", "result")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    LIFECYCLE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    CASES_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: Row25
    lifecycle: Row32
    created_at: Row22
    started_at: Row22
    updated_at: Row22
    cases: _containers.RepeatedCompositeFieldContainer[Row33]
    result: Row34
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[Row25, _Mapping]] = ..., lifecycle: _Optional[_Union[Row32, _Mapping]] = ..., created_at: _Optional[_Union[Row22, _Mapping]] = ..., started_at: _Optional[_Union[Row22, _Mapping]] = ..., updated_at: _Optional[_Union[Row22, _Mapping]] = ..., cases: _Optional[_Iterable[_Union[Row33, _Mapping]]] = ..., result: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("document_id", "metadata", "scope", "frontiers")
    DOCUMENT_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FRONTIERS_FIELD_NUMBER: _ClassVar[int]
    document_id: str
    metadata: Row25
    scope: Row4
    frontiers: Row35
    def __init__(self, document_id: _Optional[str] = ..., metadata: _Optional[_Union[Row25, _Mapping]] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., frontiers: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("ordinal", "parent_ordinal", "node")
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    PARENT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    NODE_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    parent_ordinal: int
    node: Row36
    def __init__(self, ordinal: _Optional[int] = ..., parent_ordinal: _Optional[int] = ..., node: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("metadata", "filters", "body")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    BODY_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    filters: _containers.RepeatedCompositeFieldContainer[Row37]
    body: Row38
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row37, _Mapping]]] = ..., body: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("card_id", "ordinal", "values")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    ordinal: int
    values: Row21
    def __init__(self, card_id: _Optional[str] = ..., ordinal: _Optional[int] = ..., values: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("prompt_type", "schema_version", "archetype", "title", "message", "fields", "actions", "rules", "condition", "owner_suite_id", "template_identifier", "examples", "level")
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    ARCHETYPE_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    CONDITION_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    EXAMPLES_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    schema_version: int
    archetype: Row39
    title: str
    message: str
    fields: _containers.RepeatedCompositeFieldContainer[Row40]
    actions: _containers.RepeatedCompositeFieldContainer[Row41]
    rules: _containers.RepeatedCompositeFieldContainer[Row42]
    condition: Row43
    owner_suite_id: str
    template_identifier: str
    examples: _containers.RepeatedCompositeFieldContainer[Row21]
    level: Row23
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., archetype: _Optional[_Union[Row39, _Mapping]] = ..., title: _Optional[str] = ..., message: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[Row40, _Mapping]]] = ..., actions: _Optional[_Iterable[_Union[Row41, _Mapping]]] = ..., rules: _Optional[_Iterable[_Union[Row42, _Mapping]]] = ..., condition: _Optional[_Union[Row43, _Mapping]] = ..., owner_suite_id: _Optional[str] = ..., template_identifier: _Optional[str] = ..., examples: _Optional[_Iterable[_Union[Row21, _Mapping]]] = ..., level: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("Unspecified", "Confirmation", "Form", "Choice", "Status", "Verification")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CONFIRMATION_FIELD_NUMBER: _ClassVar[int]
    FORM_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    VERIFICATION_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row44
    Confirmation: Row45
    Form: Row46
    Choice: Row47
    Status: Row48
    Verification: Row49
    def __init__(self, Unspecified: _Optional[_Union[Row44, _Mapping]] = ..., Confirmation: _Optional[_Union[Row45, _Mapping]] = ..., Form: _Optional[_Union[Row46, _Mapping]] = ..., Choice: _Optional[_Union[Row47, _Mapping]] = ..., Status: _Optional[_Union[Row48, _Mapping]] = ..., Verification: _Optional[_Union[Row49, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row50]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row50, _Mapping]]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row51
    Trace: Row52
    Debug: Row53
    Info: Row54
    Warn: Row55
    Error: Row56
    def __init__(self, Unspecified: _Optional[_Union[Row51, _Mapping]] = ..., Trace: _Optional[_Union[Row52, _Mapping]] = ..., Debug: _Optional[_Union[Row53, _Mapping]] = ..., Info: _Optional[_Union[Row54, _Mapping]] = ..., Warn: _Optional[_Union[Row55, _Mapping]] = ..., Error: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("metadata", "value", "source", "unit")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    value: Row21
    source: str
    unit: str
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., value: _Optional[_Union[Row21, _Mapping]] = ..., source: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("identifier", "label", "description", "tags", "attributes", "created_at", "updated_at")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    tags: _containers.RepeatedScalarFieldContainer[str]
    attributes: _containers.RepeatedCompositeFieldContainer[Row57]
    created_at: Row22
    updated_at: Row22
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row57, _Mapping]]] = ..., created_at: _Optional[_Union[Row22, _Mapping]] = ..., updated_at: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("name", "unit", "website")
    NAME_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    WEBSITE_FIELD_NUMBER: _ClassVar[int]
    name: str
    unit: str
    website: str
    def __init__(self, name: _Optional[str] = ..., unit: _Optional[str] = ..., website: _Optional[str] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("asset", "alt")
    ASSET_FIELD_NUMBER: _ClassVar[int]
    ALT_FIELD_NUMBER: _ClassVar[int]
    asset: Row58
    alt: str
    def __init__(self, asset: _Optional[_Union[Row58, _Mapping]] = ..., alt: _Optional[str] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("plugin_id", "theme_id", "label", "description", "stylesheet", "variables", "corner_radii", "light", "dark")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    THEME_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    STYLESHEET_FIELD_NUMBER: _ClassVar[int]
    VARIABLES_FIELD_NUMBER: _ClassVar[int]
    CORNER_RADII_FIELD_NUMBER: _ClassVar[int]
    LIGHT_FIELD_NUMBER: _ClassVar[int]
    DARK_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    theme_id: str
    label: str
    description: str
    stylesheet: str
    variables: _containers.RepeatedCompositeFieldContainer[Row59]
    corner_radii: Row60
    light: Row61
    dark: Row61
    def __init__(self, plugin_id: _Optional[str] = ..., theme_id: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., stylesheet: _Optional[str] = ..., variables: _Optional[_Iterable[_Union[Row59, _Mapping]]] = ..., corner_radii: _Optional[_Union[Row60, _Mapping]] = ..., light: _Optional[_Union[Row61, _Mapping]] = ..., dark: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("Unspecified", "UserOrSystem", "ForceSystem", "ForceLight", "ForceDark")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    USERORSYSTEM_FIELD_NUMBER: _ClassVar[int]
    FORCESYSTEM_FIELD_NUMBER: _ClassVar[int]
    FORCELIGHT_FIELD_NUMBER: _ClassVar[int]
    FORCEDARK_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row62
    UserOrSystem: Row63
    ForceSystem: Row64
    ForceLight: Row65
    ForceDark: Row66
    def __init__(self, Unspecified: _Optional[_Union[Row62, _Mapping]] = ..., UserOrSystem: _Optional[_Union[Row63, _Mapping]] = ..., ForceSystem: _Optional[_Union[Row64, _Mapping]] = ..., ForceLight: _Optional[_Union[Row65, _Mapping]] = ..., ForceDark: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row67
    Initialization: Row68
    StepAttemptId: Row69
    Cleanup: Row70
    def __init__(self, Default: _Optional[_Union[Row67, _Mapping]] = ..., Initialization: _Optional[_Union[Row68, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row69, _Mapping]] = ..., Cleanup: _Optional[_Union[Row70, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("Unspecified", "Previewed", "Ready", "Running", "Paused", "Stopping", "Completed", "Failed", "Blocked", "Stopped", "Cleared")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PREVIEWED_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    STOPPING_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    STOPPED_FIELD_NUMBER: _ClassVar[int]
    CLEARED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row71
    Previewed: Row72
    Ready: Row73
    Running: Row74
    Paused: Row75
    Stopping: Row76
    Completed: Row77
    Failed: Row78
    Blocked: Row79
    Stopped: Row80
    Cleared: Row81
    def __init__(self, Unspecified: _Optional[_Union[Row71, _Mapping]] = ..., Previewed: _Optional[_Union[Row72, _Mapping]] = ..., Ready: _Optional[_Union[Row73, _Mapping]] = ..., Running: _Optional[_Union[Row74, _Mapping]] = ..., Paused: _Optional[_Union[Row75, _Mapping]] = ..., Stopping: _Optional[_Union[Row76, _Mapping]] = ..., Completed: _Optional[_Union[Row77, _Mapping]] = ..., Failed: _Optional[_Union[Row78, _Mapping]] = ..., Blocked: _Optional[_Union[Row79, _Mapping]] = ..., Stopped: _Optional[_Union[Row80, _Mapping]] = ..., Cleared: _Optional[_Union[Row81, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("case_id", "template_id", "metadata", "ordinal", "status", "latest_execution_id", "latest_execution_status", "attempt_count", "rerun_scheduled", "actions", "suite_id")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_STATUS_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_COUNT_FIELD_NUMBER: _ClassVar[int]
    RERUN_SCHEDULED_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    template_id: str
    metadata: Row25
    ordinal: int
    status: Row82
    latest_execution_id: str
    latest_execution_status: Row83
    attempt_count: int
    rerun_scheduled: bool
    actions: Row84
    suite_id: str
    def __init__(self, case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[Row25, _Mapping]] = ..., ordinal: _Optional[int] = ..., status: _Optional[_Union[Row82, _Mapping]] = ..., latest_execution_id: _Optional[str] = ..., latest_execution_status: _Optional[_Union[Row83, _Mapping]] = ..., attempt_count: _Optional[int] = ..., rerun_scheduled: _Optional[bool] = ..., actions: _Optional[_Union[Row84, _Mapping]] = ..., suite_id: _Optional[str] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row85
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row85, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("process_epoch", "state_revision", "telemetry", "artifacts_sha256")
    PROCESS_EPOCH_FIELD_NUMBER: _ClassVar[int]
    STATE_REVISION_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_SHA256_FIELD_NUMBER: _ClassVar[int]
    process_epoch: str
    state_revision: int
    telemetry: Row86
    artifacts_sha256: str
    def __init__(self, process_epoch: _Optional[str] = ..., state_revision: _Optional[int] = ..., telemetry: _Optional[_Union[Row86, _Mapping]] = ..., artifacts_sha256: _Optional[str] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("Section", "Placement")
    SECTION_FIELD_NUMBER: _ClassVar[int]
    PLACEMENT_FIELD_NUMBER: _ClassVar[int]
    Section: Row87
    Placement: Row88
    def __init__(self, Section: _Optional[_Union[Row87, _Mapping]] = ..., Placement: _Optional[_Union[Row88, _Mapping]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: Row89
    operator: Row90
    value: Row21
    def __init__(self, expression: _Optional[_Union[Row89, _Mapping]] = ..., operator: _Optional[_Union[Row90, _Mapping]] = ..., value: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("Markdown", "Table", "Summary", "Plot", "Owner")
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    Markdown: Row91
    Table: Row92
    Summary: Row93
    Plot: Row94
    Owner: Row95
    def __init__(self, Markdown: _Optional[_Union[Row91, _Mapping]] = ..., Table: _Optional[_Union[Row92, _Mapping]] = ..., Summary: _Optional[_Union[Row93, _Mapping]] = ..., Plot: _Optional[_Union[Row94, _Mapping]] = ..., Owner: _Optional[_Union[Row95, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("Unspecified", "Confirmation", "Form", "Choice", "Status")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CONFIRMATION_FIELD_NUMBER: _ClassVar[int]
    FORM_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row96
    Confirmation: Row97
    Form: Row98
    Choice: Row99
    Status: Row100
    def __init__(self, Unspecified: _Optional[_Union[Row96, _Mapping]] = ..., Confirmation: _Optional[_Union[Row97, _Mapping]] = ..., Form: _Optional[_Union[Row98, _Mapping]] = ..., Choice: _Optional[_Union[Row99, _Mapping]] = ..., Status: _Optional[_Union[Row100, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("metadata", "value_type", "kind", "multiple", "required", "default_value", "choices", "validation")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    value_type: Row101
    kind: Row102
    multiple: bool
    required: bool
    default_value: Row21
    choices: _containers.RepeatedScalarFieldContainer[str]
    validation: Row103
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., value_type: _Optional[_Union[Row101, _Mapping]] = ..., kind: _Optional[_Union[Row102, _Mapping]] = ..., multiple: _Optional[bool] = ..., required: _Optional[bool] = ..., default_value: _Optional[_Union[Row21, _Mapping]] = ..., choices: _Optional[_Iterable[str]] = ..., validation: _Optional[_Union[Row103, _Mapping]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("id", "label", "emphasis", "requires_condition")
    ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    EMPHASIS_FIELD_NUMBER: _ClassVar[int]
    REQUIRES_CONDITION_FIELD_NUMBER: _ClassVar[int]
    id: str
    label: str
    emphasis: Row104
    requires_condition: bool
    def __init__(self, id: _Optional[str] = ..., label: _Optional[str] = ..., emphasis: _Optional[_Union[Row104, _Mapping]] = ..., requires_condition: _Optional[bool] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("predicate", "message")
    PREDICATE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    predicate: Row105
    message: str
    def __init__(self, predicate: _Optional[_Union[Row105, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Row43(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row106]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row106, _Mapping]]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row49(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row50(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row107
    def __init__(self, kind: _Optional[_Union[Row107, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row53(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row54(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row55(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row56(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row57(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row21
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row58(_message.Message):
    __slots__ = ("reference", "content_type", "size_bytes")
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    reference: Row108
    content_type: str
    size_bytes: int
    def __init__(self, reference: _Optional[_Union[Row108, _Mapping]] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("base", "sm", "md", "lg", "xl", "two_xl", "three_xl", "four_xl")
    BASE_FIELD_NUMBER: _ClassVar[int]
    SM_FIELD_NUMBER: _ClassVar[int]
    MD_FIELD_NUMBER: _ClassVar[int]
    LG_FIELD_NUMBER: _ClassVar[int]
    XL_FIELD_NUMBER: _ClassVar[int]
    TWO_XL_FIELD_NUMBER: _ClassVar[int]
    THREE_XL_FIELD_NUMBER: _ClassVar[int]
    FOUR_XL_FIELD_NUMBER: _ClassVar[int]
    base: str
    sm: str
    md: str
    lg: str
    xl: str
    two_xl: str
    three_xl: str
    four_xl: str
    def __init__(self, base: _Optional[str] = ..., sm: _Optional[str] = ..., md: _Optional[str] = ..., lg: _Optional[str] = ..., xl: _Optional[str] = ..., two_xl: _Optional[str] = ..., three_xl: _Optional[str] = ..., four_xl: _Optional[str] = ...) -> None: ...

class Row61(_message.Message):
    __slots__ = ("variables", "plot_palette")
    VARIABLES_FIELD_NUMBER: _ClassVar[int]
    PLOT_PALETTE_FIELD_NUMBER: _ClassVar[int]
    variables: _containers.RepeatedCompositeFieldContainer[Row109]
    plot_palette: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, variables: _Optional[_Iterable[_Union[Row109, _Mapping]]] = ..., plot_palette: _Optional[_Iterable[str]] = ...) -> None: ...

class Row62(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row63(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row64(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row65(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row66(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row67(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row72(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row73(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row74(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row75(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row76(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row77(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row78(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row79(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row80(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row81(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row82(_message.Message):
    __slots__ = ("Unspecified", "Skipped", "Invalidated", "Planned", "Queued", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PLANNED_FIELD_NUMBER: _ClassVar[int]
    QUEUED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row110
    Skipped: Row111
    Invalidated: Row112
    Planned: Row113
    Queued: Row114
    Running: Row115
    Passed: Row116
    Failed: Row117
    Blocked: Row118
    Cancelled: Row119
    Interrupted: Row120
    def __init__(self, Unspecified: _Optional[_Union[Row110, _Mapping]] = ..., Skipped: _Optional[_Union[Row111, _Mapping]] = ..., Invalidated: _Optional[_Union[Row112, _Mapping]] = ..., Planned: _Optional[_Union[Row113, _Mapping]] = ..., Queued: _Optional[_Union[Row114, _Mapping]] = ..., Running: _Optional[_Union[Row115, _Mapping]] = ..., Passed: _Optional[_Union[Row116, _Mapping]] = ..., Failed: _Optional[_Union[Row117, _Mapping]] = ..., Blocked: _Optional[_Union[Row118, _Mapping]] = ..., Cancelled: _Optional[_Union[Row119, _Mapping]] = ..., Interrupted: _Optional[_Union[Row120, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("Unspecified", "Invalidated", "Pending", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row121
    Invalidated: Row122
    Pending: Row123
    Running: Row124
    Passed: Row125
    Failed: Row126
    Blocked: Row127
    Cancelled: Row128
    Interrupted: Row129
    Skipped: Row130
    def __init__(self, Unspecified: _Optional[_Union[Row121, _Mapping]] = ..., Invalidated: _Optional[_Union[Row122, _Mapping]] = ..., Pending: _Optional[_Union[Row123, _Mapping]] = ..., Running: _Optional[_Union[Row124, _Mapping]] = ..., Passed: _Optional[_Union[Row125, _Mapping]] = ..., Failed: _Optional[_Union[Row126, _Mapping]] = ..., Blocked: _Optional[_Union[Row127, _Mapping]] = ..., Cancelled: _Optional[_Union[Row128, _Mapping]] = ..., Interrupted: _Optional[_Union[Row129, _Mapping]] = ..., Skipped: _Optional[_Union[Row130, _Mapping]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("run", "schedule_rerun", "delete", "unavailable_reasons")
    RUN_FIELD_NUMBER: _ClassVar[int]
    SCHEDULE_RERUN_FIELD_NUMBER: _ClassVar[int]
    DELETE_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_REASONS_FIELD_NUMBER: _ClassVar[int]
    run: bool
    schedule_rerun: bool
    delete: bool
    unavailable_reasons: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, run: _Optional[bool] = ..., schedule_rerun: _Optional[bool] = ..., delete: _Optional[bool] = ..., unavailable_reasons: _Optional[_Iterable[str]] = ...) -> None: ...

class Row85(_message.Message):
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row131
    Pending: Row132
    Passed: Row133
    Failed: Row134
    Blocked: Row135
    Cancelled: Row136
    def __init__(self, Unspecified: _Optional[_Union[Row131, _Mapping]] = ..., Pending: _Optional[_Union[Row132, _Mapping]] = ..., Passed: _Optional[_Union[Row133, _Mapping]] = ..., Failed: _Optional[_Union[Row134, _Mapping]] = ..., Blocked: _Optional[_Union[Row135, _Mapping]] = ..., Cancelled: _Optional[_Union[Row136, _Mapping]] = ...) -> None: ...

class Row86(_message.Message):
    __slots__ = ("delivered_cursor", "source", "series_ordinal")
    DELIVERED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SERIES_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    delivered_cursor: int
    source: Row137
    series_ordinal: int
    def __init__(self, delivered_cursor: _Optional[int] = ..., source: _Optional[_Union[Row137, _Mapping]] = ..., series_ordinal: _Optional[int] = ...) -> None: ...

class Row87(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row138
    def __init__(self, field_0: _Optional[_Union[Row138, _Mapping]] = ...) -> None: ...

class Row88(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row139
    def __init__(self, field_0: _Optional[_Union[Row139, _Mapping]] = ...) -> None: ...

class Row89(_message.Message):
    __slots__ = ("source", "transforms", "label")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    source: Row140
    transforms: _containers.RepeatedCompositeFieldContainer[Row141]
    label: str
    def __init__(self, source: _Optional[_Union[Row140, _Mapping]] = ..., transforms: _Optional[_Iterable[_Union[Row141, _Mapping]]] = ..., label: _Optional[str] = ...) -> None: ...

class Row90(_message.Message):
    __slots__ = ("Unspecified", "Eq", "Ne", "Gt", "Gte", "Lt", "Lte", "Contains", "Exists")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    NE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    CONTAINS_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row142
    Eq: Row143
    Ne: Row144
    Gt: Row145
    Gte: Row146
    Lt: Row147
    Lte: Row148
    Contains: Row149
    Exists: Row150
    def __init__(self, Unspecified: _Optional[_Union[Row142, _Mapping]] = ..., Eq: _Optional[_Union[Row143, _Mapping]] = ..., Ne: _Optional[_Union[Row144, _Mapping]] = ..., Gt: _Optional[_Union[Row145, _Mapping]] = ..., Gte: _Optional[_Union[Row146, _Mapping]] = ..., Lt: _Optional[_Union[Row147, _Mapping]] = ..., Lte: _Optional[_Union[Row148, _Mapping]] = ..., Contains: _Optional[_Union[Row149, _Mapping]] = ..., Exists: _Optional[_Union[Row150, _Mapping]] = ...) -> None: ...

class Row91(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row92(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row151
    def __init__(self, field_0: _Optional[_Union[Row151, _Mapping]] = ...) -> None: ...

class Row93(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row152
    def __init__(self, field_0: _Optional[_Union[Row152, _Mapping]] = ...) -> None: ...

class Row94(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row153
    def __init__(self, field_0: _Optional[_Union[Row153, _Mapping]] = ...) -> None: ...

class Row95(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row154
    def __init__(self, field_0: _Optional[_Union[Row154, _Mapping]] = ...) -> None: ...

class Row96(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row97(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row98(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row99(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row100(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row101(_message.Message):
    __slots__ = ("Unspecified", "Boolean", "Integer", "Number", "Text", "File", "Object", "Array", "Datetime")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row155
    Boolean: Row156
    Integer: Row157
    Number: Row158
    Text: Row159
    File: Row160
    Object: Row161
    Array: Row162
    Datetime: Row163
    def __init__(self, Unspecified: _Optional[_Union[Row155, _Mapping]] = ..., Boolean: _Optional[_Union[Row156, _Mapping]] = ..., Integer: _Optional[_Union[Row157, _Mapping]] = ..., Number: _Optional[_Union[Row158, _Mapping]] = ..., Text: _Optional[_Union[Row159, _Mapping]] = ..., File: _Optional[_Union[Row160, _Mapping]] = ..., Object: _Optional[_Union[Row161, _Mapping]] = ..., Array: _Optional[_Union[Row162, _Mapping]] = ..., Datetime: _Optional[_Union[Row163, _Mapping]] = ...) -> None: ...

class Row102(_message.Message):
    __slots__ = ("Unspecified", "Text", "Float", "Int", "Choice", "Boolean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FLOAT_FIELD_NUMBER: _ClassVar[int]
    INT_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row164
    Text: Row165
    Float: Row166
    Int: Row167
    Choice: Row168
    Boolean: Row169
    def __init__(self, Unspecified: _Optional[_Union[Row164, _Mapping]] = ..., Text: _Optional[_Union[Row165, _Mapping]] = ..., Float: _Optional[_Union[Row166, _Mapping]] = ..., Int: _Optional[_Union[Row167, _Mapping]] = ..., Choice: _Optional[_Union[Row168, _Mapping]] = ..., Boolean: _Optional[_Union[Row169, _Mapping]] = ...) -> None: ...

class Row103(_message.Message):
    __slots__ = ("min_length", "max_length", "pattern", "minimum", "maximum", "min_items", "max_items")
    MIN_LENGTH_FIELD_NUMBER: _ClassVar[int]
    MAX_LENGTH_FIELD_NUMBER: _ClassVar[int]
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    MINIMUM_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    min_length: int
    max_length: int
    pattern: str
    minimum: Row21
    maximum: Row21
    min_items: int
    max_items: int
    def __init__(self, min_length: _Optional[int] = ..., max_length: _Optional[int] = ..., pattern: _Optional[str] = ..., minimum: _Optional[_Union[Row21, _Mapping]] = ..., maximum: _Optional[_Union[Row21, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class Row104(_message.Message):
    __slots__ = ("Unspecified", "Primary", "Secondary", "Danger")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PRIMARY_FIELD_NUMBER: _ClassVar[int]
    SECONDARY_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row170
    Primary: Row171
    Secondary: Row172
    Danger: Row173
    def __init__(self, Unspecified: _Optional[_Union[Row170, _Mapping]] = ..., Primary: _Optional[_Union[Row171, _Mapping]] = ..., Secondary: _Optional[_Union[Row172, _Mapping]] = ..., Danger: _Optional[_Union[Row173, _Mapping]] = ...) -> None: ...

class Row105(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row174]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row174, _Mapping]]] = ...) -> None: ...

class Row106(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row175
    def __init__(self, kind: _Optional[_Union[Row175, _Mapping]] = ...) -> None: ...

class Row107(_message.Message):
    __slots__ = ("Null", "Bool", "Integer", "Unsigned", "Number", "Text", "Datetime", "Array", "Object")
    NULL_FIELD_NUMBER: _ClassVar[int]
    BOOL_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    Null: Row176
    Bool: Row177
    Integer: Row178
    Unsigned: Row179
    Number: Row180
    Text: Row181
    Datetime: Row182
    Array: Row183
    Object: Row184
    def __init__(self, Null: _Optional[_Union[Row176, _Mapping]] = ..., Bool: _Optional[_Union[Row177, _Mapping]] = ..., Integer: _Optional[_Union[Row178, _Mapping]] = ..., Unsigned: _Optional[_Union[Row179, _Mapping]] = ..., Number: _Optional[_Union[Row180, _Mapping]] = ..., Text: _Optional[_Union[Row181, _Mapping]] = ..., Datetime: _Optional[_Union[Row182, _Mapping]] = ..., Array: _Optional[_Union[Row183, _Mapping]] = ..., Object: _Optional[_Union[Row184, _Mapping]] = ...) -> None: ...

class Row108(_message.Message):
    __slots__ = ("plugin_id", "asset_id", "sha256")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    ASSET_ID_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    asset_id: str
    sha256: str
    def __init__(self, plugin_id: _Optional[str] = ..., asset_id: _Optional[str] = ..., sha256: _Optional[str] = ...) -> None: ...

class Row109(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Row110(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row111(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row112(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row113(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row114(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row115(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row116(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row117(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row118(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row119(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row120(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row121(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row122(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row123(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row124(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row125(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row126(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row127(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row128(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row129(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row130(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row131(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row132(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row133(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row134(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row135(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row136(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row137(_message.Message):
    __slots__ = ("SeriesEpoch", "RetainedWindow")
    SERIESEPOCH_FIELD_NUMBER: _ClassVar[int]
    RETAINEDWINDOW_FIELD_NUMBER: _ClassVar[int]
    SeriesEpoch: Row185
    RetainedWindow: Row186
    def __init__(self, SeriesEpoch: _Optional[_Union[Row185, _Mapping]] = ..., RetainedWindow: _Optional[_Union[Row186, _Mapping]] = ...) -> None: ...

class Row138(_message.Message):
    __slots__ = ("node_id", "metadata", "repeat")
    NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEAT_FIELD_NUMBER: _ClassVar[int]
    node_id: str
    metadata: Row25
    repeat: Row187
    def __init__(self, node_id: _Optional[str] = ..., metadata: _Optional[_Union[Row25, _Mapping]] = ..., repeat: _Optional[_Union[Row187, _Mapping]] = ...) -> None: ...

class Row139(_message.Message):
    __slots__ = ("card_id", "column_span", "row_span", "order", "density", "rail")
    CARD_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMN_SPAN_FIELD_NUMBER: _ClassVar[int]
    ROW_SPAN_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    DENSITY_FIELD_NUMBER: _ClassVar[int]
    RAIL_FIELD_NUMBER: _ClassVar[int]
    card_id: str
    column_span: Row188
    row_span: int
    order: int
    density: Row189
    rail: Row190
    def __init__(self, card_id: _Optional[str] = ..., column_span: _Optional[_Union[Row188, _Mapping]] = ..., row_span: _Optional[int] = ..., order: _Optional[int] = ..., density: _Optional[_Union[Row189, _Mapping]] = ..., rail: _Optional[_Union[Row190, _Mapping]] = ...) -> None: ...

class Row140(_message.Message):
    __slots__ = ("Telemetry", "Parameter", "Dimension", "Record", "Artifact")
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    Telemetry: Row191
    Parameter: Row192
    Dimension: Row193
    Record: Row194
    Artifact: Row195
    def __init__(self, Telemetry: _Optional[_Union[Row191, _Mapping]] = ..., Parameter: _Optional[_Union[Row192, _Mapping]] = ..., Dimension: _Optional[_Union[Row193, _Mapping]] = ..., Record: _Optional[_Union[Row194, _Mapping]] = ..., Artifact: _Optional[_Union[Row195, _Mapping]] = ...) -> None: ...

class Row141(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row196
    def __init__(self, kind: _Optional[_Union[Row196, _Mapping]] = ...) -> None: ...

class Row142(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row143(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row144(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row145(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row146(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row147(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row148(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row149(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row150(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row151(_message.Message):
    __slots__ = ("row_count",)
    ROW_COUNT_FIELD_NUMBER: _ClassVar[int]
    row_count: int
    def __init__(self, row_count: _Optional[int] = ...) -> None: ...

class Row152(_message.Message):
    __slots__ = ("groups",)
    GROUPS_FIELD_NUMBER: _ClassVar[int]
    groups: _containers.RepeatedCompositeFieldContainer[Row197]
    def __init__(self, groups: _Optional[_Iterable[_Union[Row197, _Mapping]]] = ...) -> None: ...

class Row153(_message.Message):
    __slots__ = ("metadata", "request", "presentation", "z_expression")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    request: Row198
    presentation: Row199
    z_expression: Row89
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., request: _Optional[_Union[Row198, _Mapping]] = ..., presentation: _Optional[_Union[Row199, _Mapping]] = ..., z_expression: _Optional[_Union[Row89, _Mapping]] = ...) -> None: ...

class Row154(_message.Message):
    __slots__ = ("kind", "plot_id", "plot")
    KIND_FIELD_NUMBER: _ClassVar[int]
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    kind: Row200
    plot_id: str
    plot: Row153
    def __init__(self, kind: _Optional[_Union[Row200, _Mapping]] = ..., plot_id: _Optional[str] = ..., plot: _Optional[_Union[Row153, _Mapping]] = ...) -> None: ...

class Row155(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row156(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row157(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row158(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row159(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row160(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row161(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row162(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row163(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row164(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row165(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row166(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row167(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row168(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row169(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row170(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row171(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row172(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row173(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row174(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row201
    def __init__(self, kind: _Optional[_Union[Row201, _Mapping]] = ...) -> None: ...

class Row175(_message.Message):
    __slots__ = ("Record", "Telemetry", "All", "Any")
    RECORD_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    ANY_FIELD_NUMBER: _ClassVar[int]
    Record: Row202
    Telemetry: Row203
    All: Row204
    Any: Row205
    def __init__(self, Record: _Optional[_Union[Row202, _Mapping]] = ..., Telemetry: _Optional[_Union[Row203, _Mapping]] = ..., All: _Optional[_Union[Row204, _Mapping]] = ..., Any: _Optional[_Union[Row205, _Mapping]] = ...) -> None: ...

class Row176(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row177(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row178(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row179(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row180(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row181(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row182(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row22
    def __init__(self, field_0: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row183(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row206
    def __init__(self, field_0: _Optional[_Union[Row206, _Mapping]] = ...) -> None: ...

class Row184(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row207
    def __init__(self, field_0: _Optional[_Union[Row207, _Mapping]] = ...) -> None: ...

class Row185(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row186(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row187(_message.Message):
    __slots__ = ("value", "label_template", "id_template", "order_by", "filters")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    LABEL_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ID_TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    ORDER_BY_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    value: Row89
    label_template: str
    id_template: str
    order_by: Row89
    filters: _containers.RepeatedCompositeFieldContainer[Row208]
    def __init__(self, value: _Optional[_Union[Row89, _Mapping]] = ..., label_template: _Optional[str] = ..., id_template: _Optional[str] = ..., order_by: _Optional[_Union[Row89, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row208, _Mapping]]] = ...) -> None: ...

class Row188(_message.Message):
    __slots__ = ("base", "sm", "md", "lg", "xl")
    BASE_FIELD_NUMBER: _ClassVar[int]
    SM_FIELD_NUMBER: _ClassVar[int]
    MD_FIELD_NUMBER: _ClassVar[int]
    LG_FIELD_NUMBER: _ClassVar[int]
    XL_FIELD_NUMBER: _ClassVar[int]
    base: int
    sm: int
    md: int
    lg: int
    xl: int
    def __init__(self, base: _Optional[int] = ..., sm: _Optional[int] = ..., md: _Optional[int] = ..., lg: _Optional[int] = ..., xl: _Optional[int] = ...) -> None: ...

class Row189(_message.Message):
    __slots__ = ("Unspecified", "Compact", "Normal", "Spacious")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPACT_FIELD_NUMBER: _ClassVar[int]
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    SPACIOUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row209
    Compact: Row210
    Normal: Row211
    Spacious: Row212
    def __init__(self, Unspecified: _Optional[_Union[Row209, _Mapping]] = ..., Compact: _Optional[_Union[Row210, _Mapping]] = ..., Normal: _Optional[_Union[Row211, _Mapping]] = ..., Spacious: _Optional[_Union[Row212, _Mapping]] = ...) -> None: ...

class Row190(_message.Message):
    __slots__ = ("Unspecified", "Main", "Right", "Left")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    MAIN_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row213
    Main: Row214
    Right: Row215
    Left: Row216
    def __init__(self, Unspecified: _Optional[_Union[Row213, _Mapping]] = ..., Main: _Optional[_Union[Row214, _Mapping]] = ..., Right: _Optional[_Union[Row215, _Mapping]] = ..., Left: _Optional[_Union[Row216, _Mapping]] = ...) -> None: ...

class Row191(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row217
    def __init__(self, field_0: _Optional[_Union[Row217, _Mapping]] = ...) -> None: ...

class Row192(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row218
    def __init__(self, field_0: _Optional[_Union[Row218, _Mapping]] = ...) -> None: ...

class Row193(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row219
    def __init__(self, field_0: _Optional[_Union[Row219, _Mapping]] = ...) -> None: ...

class Row194(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row220
    def __init__(self, field_0: _Optional[_Union[Row220, _Mapping]] = ...) -> None: ...

class Row195(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row221
    def __init__(self, field_0: _Optional[_Union[Row221, _Mapping]] = ...) -> None: ...

class Row196(_message.Message):
    __slots__ = ("Aggregate", "TimeBetween", "TelemetryAtOrBefore", "Detail", "PerCase")
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    TIMEBETWEEN_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYATORBEFORE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    PERCASE_FIELD_NUMBER: _ClassVar[int]
    Aggregate: Row222
    TimeBetween: Row223
    TelemetryAtOrBefore: Row224
    Detail: Row225
    PerCase: Row226
    def __init__(self, Aggregate: _Optional[_Union[Row222, _Mapping]] = ..., TimeBetween: _Optional[_Union[Row223, _Mapping]] = ..., TelemetryAtOrBefore: _Optional[_Union[Row224, _Mapping]] = ..., Detail: _Optional[_Union[Row225, _Mapping]] = ..., PerCase: _Optional[_Union[Row226, _Mapping]] = ...) -> None: ...

class Row197(_message.Message):
    __slots__ = ("metadata", "tiles")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILES_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    tiles: _containers.RepeatedCompositeFieldContainer[Row227]
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., tiles: _Optional[_Iterable[_Union[Row227, _Mapping]]] = ...) -> None: ...

class Row198(_message.Message):
    __slots__ = ("plot_id", "plot_type", "x_scale", "y_scale", "stream", "y_keys", "marker_record_types", "filters", "x_key", "series_by", "x_expression", "y_expressions", "series_by_expression", "z_stream", "z_field", "unit", "max_points", "window_s", "scope", "cursor", "technique", "z_expression", "fft", "interval", "histogram", "analysis")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    PLOT_TYPE_FIELD_NUMBER: _ClassVar[int]
    X_SCALE_FIELD_NUMBER: _ClassVar[int]
    Y_SCALE_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    Y_KEYS_FIELD_NUMBER: _ClassVar[int]
    MARKER_RECORD_TYPES_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    X_KEY_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_FIELD_NUMBER: _ClassVar[int]
    X_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    Y_EXPRESSIONS_FIELD_NUMBER: _ClassVar[int]
    SERIES_BY_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    Z_STREAM_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    MAX_POINTS_FIELD_NUMBER: _ClassVar[int]
    WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    TECHNIQUE_FIELD_NUMBER: _ClassVar[int]
    Z_EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    plot_type: Row228
    x_scale: Row229
    y_scale: Row229
    stream: str
    y_keys: _containers.RepeatedScalarFieldContainer[str]
    marker_record_types: _containers.RepeatedScalarFieldContainer[str]
    filters: _containers.RepeatedCompositeFieldContainer[Row208]
    x_key: str
    series_by: str
    x_expression: Row89
    y_expressions: _containers.RepeatedCompositeFieldContainer[Row89]
    series_by_expression: Row89
    z_stream: str
    z_field: str
    unit: str
    max_points: int
    window_s: float
    scope: Row4
    cursor: Row230
    technique: Row231
    z_expression: Row89
    fft: Row232
    interval: Row233
    histogram: Row234
    analysis: Row235
    def __init__(self, plot_id: _Optional[str] = ..., plot_type: _Optional[_Union[Row228, _Mapping]] = ..., x_scale: _Optional[_Union[Row229, _Mapping]] = ..., y_scale: _Optional[_Union[Row229, _Mapping]] = ..., stream: _Optional[str] = ..., y_keys: _Optional[_Iterable[str]] = ..., marker_record_types: _Optional[_Iterable[str]] = ..., filters: _Optional[_Iterable[_Union[Row208, _Mapping]]] = ..., x_key: _Optional[str] = ..., series_by: _Optional[str] = ..., x_expression: _Optional[_Union[Row89, _Mapping]] = ..., y_expressions: _Optional[_Iterable[_Union[Row89, _Mapping]]] = ..., series_by_expression: _Optional[_Union[Row89, _Mapping]] = ..., z_stream: _Optional[str] = ..., z_field: _Optional[str] = ..., unit: _Optional[str] = ..., max_points: _Optional[int] = ..., window_s: _Optional[float] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., cursor: _Optional[_Union[Row230, _Mapping]] = ..., technique: _Optional[_Union[Row231, _Mapping]] = ..., z_expression: _Optional[_Union[Row89, _Mapping]] = ..., fft: _Optional[_Union[Row232, _Mapping]] = ..., interval: _Optional[_Union[Row233, _Mapping]] = ..., histogram: _Optional[_Union[Row234, _Mapping]] = ..., analysis: _Optional[_Union[Row235, _Mapping]] = ...) -> None: ...

class Row199(_message.Message):
    __slots__ = ("style", "fft", "interval", "histogram", "reference_lines", "reference_bands", "live_window_s", "x_format", "y_format", "x_axis", "x_axis_unit")
    STYLE_FIELD_NUMBER: _ClassVar[int]
    FFT_FIELD_NUMBER: _ClassVar[int]
    INTERVAL_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_LINES_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_BANDS_FIELD_NUMBER: _ClassVar[int]
    LIVE_WINDOW_S_FIELD_NUMBER: _ClassVar[int]
    X_FORMAT_FIELD_NUMBER: _ClassVar[int]
    Y_FORMAT_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_FIELD_NUMBER: _ClassVar[int]
    X_AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    style: Row236
    fft: Row232
    interval: Row233
    histogram: Row234
    reference_lines: _containers.RepeatedCompositeFieldContainer[Row237]
    reference_bands: _containers.RepeatedCompositeFieldContainer[Row238]
    live_window_s: float
    x_format: Row239
    y_format: Row239
    x_axis: Row240
    x_axis_unit: str
    def __init__(self, style: _Optional[_Union[Row236, _Mapping]] = ..., fft: _Optional[_Union[Row232, _Mapping]] = ..., interval: _Optional[_Union[Row233, _Mapping]] = ..., histogram: _Optional[_Union[Row234, _Mapping]] = ..., reference_lines: _Optional[_Iterable[_Union[Row237, _Mapping]]] = ..., reference_bands: _Optional[_Iterable[_Union[Row238, _Mapping]]] = ..., live_window_s: _Optional[float] = ..., x_format: _Optional[_Union[Row239, _Mapping]] = ..., y_format: _Optional[_Union[Row239, _Mapping]] = ..., x_axis: _Optional[_Union[Row240, _Mapping]] = ..., x_axis_unit: _Optional[str] = ...) -> None: ...

class Row200(_message.Message):
    __slots__ = ("Unspecified", "Text", "TelemetrySummary", "Plot", "Table", "Steps", "Criteria", "Traceability", "Notes", "Records", "Artifacts", "Media", "Health", "TelemetryContext")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYSUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    MEDIA_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    TELEMETRYCONTEXT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row241
    Text: Row242
    TelemetrySummary: Row243
    Plot: Row244
    Table: Row245
    Steps: Row246
    Criteria: Row247
    Traceability: Row248
    Notes: Row249
    Records: Row250
    Artifacts: Row251
    Media: Row252
    Health: Row253
    TelemetryContext: Row254
    def __init__(self, Unspecified: _Optional[_Union[Row241, _Mapping]] = ..., Text: _Optional[_Union[Row242, _Mapping]] = ..., TelemetrySummary: _Optional[_Union[Row243, _Mapping]] = ..., Plot: _Optional[_Union[Row244, _Mapping]] = ..., Table: _Optional[_Union[Row245, _Mapping]] = ..., Steps: _Optional[_Union[Row246, _Mapping]] = ..., Criteria: _Optional[_Union[Row247, _Mapping]] = ..., Traceability: _Optional[_Union[Row248, _Mapping]] = ..., Notes: _Optional[_Union[Row249, _Mapping]] = ..., Records: _Optional[_Union[Row250, _Mapping]] = ..., Artifacts: _Optional[_Union[Row251, _Mapping]] = ..., Media: _Optional[_Union[Row252, _Mapping]] = ..., Health: _Optional[_Union[Row253, _Mapping]] = ..., TelemetryContext: _Optional[_Union[Row254, _Mapping]] = ...) -> None: ...

class Row201(_message.Message):
    __slots__ = ("Literal", "Field", "Not", "Binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    NOT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    Literal: Row255
    Field: Row256
    Not: Row257
    Binary: Row258
    def __init__(self, Literal: _Optional[_Union[Row255, _Mapping]] = ..., Field: _Optional[_Union[Row256, _Mapping]] = ..., Not: _Optional[_Union[Row257, _Mapping]] = ..., Binary: _Optional[_Union[Row258, _Mapping]] = ...) -> None: ...

class Row202(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row259
    def __init__(self, field_0: _Optional[_Union[Row259, _Mapping]] = ...) -> None: ...

class Row203(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row260
    def __init__(self, field_0: _Optional[_Union[Row260, _Mapping]] = ...) -> None: ...

class Row204(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row261
    def __init__(self, field_0: _Optional[_Union[Row261, _Mapping]] = ...) -> None: ...

class Row205(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row261
    def __init__(self, field_0: _Optional[_Union[Row261, _Mapping]] = ...) -> None: ...

class Row206(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row207(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row262]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row262, _Mapping]]] = ...) -> None: ...

class Row208(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: Row89
    operator: Row90
    value: Row21
    def __init__(self, expression: _Optional[_Union[Row89, _Mapping]] = ..., operator: _Optional[_Union[Row90, _Mapping]] = ..., value: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row209(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row210(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row211(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row212(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row213(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row214(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row215(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row216(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row217(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row218(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row219(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row220(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class Row221(_message.Message):
    __slots__ = ("artifact",)
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    def __init__(self, artifact: _Optional[str] = ...) -> None: ...

class Row222(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row263
    def __init__(self, field_0: _Optional[_Union[Row263, _Mapping]] = ...) -> None: ...

class Row223(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row264
    def __init__(self, field_0: _Optional[_Union[Row264, _Mapping]] = ...) -> None: ...

class Row224(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row265
    def __init__(self, field_0: _Optional[_Union[Row265, _Mapping]] = ...) -> None: ...

class Row225(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row266
    def __init__(self, field_0: _Optional[_Union[Row266, _Mapping]] = ...) -> None: ...

class Row226(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row267
    def __init__(self, field_0: _Optional[_Union[Row267, _Mapping]] = ...) -> None: ...

class Row227(_message.Message):
    __slots__ = ("metadata", "value", "filters", "stat", "format", "presentation", "unit", "conditions")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    STAT_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    metadata: Row25
    value: Row89
    filters: _containers.RepeatedCompositeFieldContainer[Row37]
    stat: Row268
    format: Row269
    presentation: Row270
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[Row271]
    def __init__(self, metadata: _Optional[_Union[Row25, _Mapping]] = ..., value: _Optional[_Union[Row89, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row37, _Mapping]]] = ..., stat: _Optional[_Union[Row268, _Mapping]] = ..., format: _Optional[_Union[Row269, _Mapping]] = ..., presentation: _Optional[_Union[Row270, _Mapping]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[Row271, _Mapping]]] = ...) -> None: ...

class Row228(_message.Message):
    __slots__ = ("Unspecified", "TimeSeries", "Bar", "GroupedBar", "StackedBar", "Pie", "Donut", "Pareto", "XyScatter", "XyLine", "XyInterval", "Trajectory", "Heatmap", "XyValueHeatmap", "WifiChannelOccupancy", "FrequencySeries", "Histogram", "BoxSummary", "AggregateTable")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TIMESERIES_FIELD_NUMBER: _ClassVar[int]
    BAR_FIELD_NUMBER: _ClassVar[int]
    GROUPEDBAR_FIELD_NUMBER: _ClassVar[int]
    STACKEDBAR_FIELD_NUMBER: _ClassVar[int]
    PIE_FIELD_NUMBER: _ClassVar[int]
    DONUT_FIELD_NUMBER: _ClassVar[int]
    PARETO_FIELD_NUMBER: _ClassVar[int]
    XYSCATTER_FIELD_NUMBER: _ClassVar[int]
    XYLINE_FIELD_NUMBER: _ClassVar[int]
    XYINTERVAL_FIELD_NUMBER: _ClassVar[int]
    TRAJECTORY_FIELD_NUMBER: _ClassVar[int]
    HEATMAP_FIELD_NUMBER: _ClassVar[int]
    XYVALUEHEATMAP_FIELD_NUMBER: _ClassVar[int]
    WIFICHANNELOCCUPANCY_FIELD_NUMBER: _ClassVar[int]
    FREQUENCYSERIES_FIELD_NUMBER: _ClassVar[int]
    HISTOGRAM_FIELD_NUMBER: _ClassVar[int]
    BOXSUMMARY_FIELD_NUMBER: _ClassVar[int]
    AGGREGATETABLE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row272
    TimeSeries: Row273
    Bar: Row274
    GroupedBar: Row275
    StackedBar: Row276
    Pie: Row277
    Donut: Row278
    Pareto: Row279
    XyScatter: Row280
    XyLine: Row281
    XyInterval: Row282
    Trajectory: Row283
    Heatmap: Row284
    XyValueHeatmap: Row285
    WifiChannelOccupancy: Row286
    FrequencySeries: Row287
    Histogram: Row288
    BoxSummary: Row289
    AggregateTable: Row290
    def __init__(self, Unspecified: _Optional[_Union[Row272, _Mapping]] = ..., TimeSeries: _Optional[_Union[Row273, _Mapping]] = ..., Bar: _Optional[_Union[Row274, _Mapping]] = ..., GroupedBar: _Optional[_Union[Row275, _Mapping]] = ..., StackedBar: _Optional[_Union[Row276, _Mapping]] = ..., Pie: _Optional[_Union[Row277, _Mapping]] = ..., Donut: _Optional[_Union[Row278, _Mapping]] = ..., Pareto: _Optional[_Union[Row279, _Mapping]] = ..., XyScatter: _Optional[_Union[Row280, _Mapping]] = ..., XyLine: _Optional[_Union[Row281, _Mapping]] = ..., XyInterval: _Optional[_Union[Row282, _Mapping]] = ..., Trajectory: _Optional[_Union[Row283, _Mapping]] = ..., Heatmap: _Optional[_Union[Row284, _Mapping]] = ..., XyValueHeatmap: _Optional[_Union[Row285, _Mapping]] = ..., WifiChannelOccupancy: _Optional[_Union[Row286, _Mapping]] = ..., FrequencySeries: _Optional[_Union[Row287, _Mapping]] = ..., Histogram: _Optional[_Union[Row288, _Mapping]] = ..., BoxSummary: _Optional[_Union[Row289, _Mapping]] = ..., AggregateTable: _Optional[_Union[Row290, _Mapping]] = ...) -> None: ...

class Row229(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Log")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    LOG_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row291
    Linear: Row292
    Log: Row293
    def __init__(self, Unspecified: _Optional[_Union[Row291, _Mapping]] = ..., Linear: _Optional[_Union[Row292, _Mapping]] = ..., Log: _Optional[_Union[Row293, _Mapping]] = ...) -> None: ...

class Row230(_message.Message):
    __slots__ = ("telemetry_streams", "record_sequence", "epoch", "selection")
    TELEMETRY_STREAMS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    telemetry_streams: _containers.RepeatedCompositeFieldContainer[Row294]
    record_sequence: int
    epoch: str
    selection: str
    def __init__(self, telemetry_streams: _Optional[_Iterable[_Union[Row294, _Mapping]]] = ..., record_sequence: _Optional[int] = ..., epoch: _Optional[str] = ..., selection: _Optional[str] = ...) -> None: ...

class Row231(_message.Message):
    __slots__ = ("Unspecified", "Raw", "MinMax", "Lttb", "Auto")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RAW_FIELD_NUMBER: _ClassVar[int]
    MINMAX_FIELD_NUMBER: _ClassVar[int]
    LTTB_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row295
    Raw: Row296
    MinMax: Row297
    Lttb: Row298
    Auto: Row299
    def __init__(self, Unspecified: _Optional[_Union[Row295, _Mapping]] = ..., Raw: _Optional[_Union[Row296, _Mapping]] = ..., MinMax: _Optional[_Union[Row297, _Mapping]] = ..., Lttb: _Optional[_Union[Row298, _Mapping]] = ..., Auto: _Optional[_Union[Row299, _Mapping]] = ...) -> None: ...

class Row232(_message.Message):
    __slots__ = ("sample_rate_hz", "window", "detrend", "level_unit", "include_dc", "min_frequency_hz", "max_frequency_hz")
    SAMPLE_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    WINDOW_FIELD_NUMBER: _ClassVar[int]
    DETREND_FIELD_NUMBER: _ClassVar[int]
    LEVEL_UNIT_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_DC_FIELD_NUMBER: _ClassVar[int]
    MIN_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    MAX_FREQUENCY_HZ_FIELD_NUMBER: _ClassVar[int]
    sample_rate_hz: float
    window: Row300
    detrend: Row301
    level_unit: Row302
    include_dc: bool
    min_frequency_hz: float
    max_frequency_hz: float
    def __init__(self, sample_rate_hz: _Optional[float] = ..., window: _Optional[_Union[Row300, _Mapping]] = ..., detrend: _Optional[_Union[Row301, _Mapping]] = ..., level_unit: _Optional[_Union[Row302, _Mapping]] = ..., include_dc: _Optional[bool] = ..., min_frequency_hz: _Optional[float] = ..., max_frequency_hz: _Optional[float] = ...) -> None: ...

class Row233(_message.Message):
    __slots__ = ("bin_width", "sigma", "clip_to_observed")
    BIN_WIDTH_FIELD_NUMBER: _ClassVar[int]
    SIGMA_FIELD_NUMBER: _ClassVar[int]
    CLIP_TO_OBSERVED_FIELD_NUMBER: _ClassVar[int]
    bin_width: float
    sigma: float
    clip_to_observed: bool
    def __init__(self, bin_width: _Optional[float] = ..., sigma: _Optional[float] = ..., clip_to_observed: _Optional[bool] = ...) -> None: ...

class Row234(_message.Message):
    __slots__ = ("bins", "unit")
    BINS_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    bins: int
    unit: str
    def __init__(self, bins: _Optional[int] = ..., unit: _Optional[str] = ...) -> None: ...

class Row235(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: Row21
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row236(_message.Message):
    __slots__ = ("Unspecified", "Line", "Points", "LinePoints", "Step")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINE_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    LINEPOINTS_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row303
    Line: Row304
    Points: Row305
    LinePoints: Row306
    Step: Row307
    def __init__(self, Unspecified: _Optional[_Union[Row303, _Mapping]] = ..., Line: _Optional[_Union[Row304, _Mapping]] = ..., Points: _Optional[_Union[Row305, _Mapping]] = ..., LinePoints: _Optional[_Union[Row306, _Mapping]] = ..., Step: _Optional[_Union[Row307, _Mapping]] = ...) -> None: ...

class Row237(_message.Message):
    __slots__ = ("axis", "value", "source", "source_scale", "label", "color")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_SCALE_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    axis: Row308
    value: float
    source: Row89
    source_scale: float
    label: str
    color: str
    def __init__(self, axis: _Optional[_Union[Row308, _Mapping]] = ..., value: _Optional[float] = ..., source: _Optional[_Union[Row89, _Mapping]] = ..., source_scale: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ...) -> None: ...

class Row238(_message.Message):
    __slots__ = ("axis", "min", "max", "label", "color", "center", "tolerance")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    axis: Row308
    min: float
    max: float
    label: str
    color: str
    center: Row89
    tolerance: Row89
    def __init__(self, axis: _Optional[_Union[Row308, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., label: _Optional[str] = ..., color: _Optional[str] = ..., center: _Optional[_Union[Row89, _Mapping]] = ..., tolerance: _Optional[_Union[Row89, _Mapping]] = ...) -> None: ...

class Row239(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row309
    def __init__(self, kind: _Optional[_Union[Row309, _Mapping]] = ...) -> None: ...

class Row240(_message.Message):
    __slots__ = ("Unspecified", "WallTime", "Elapsed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    WALLTIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row310
    WallTime: Row311
    Elapsed: Row312
    def __init__(self, Unspecified: _Optional[_Union[Row310, _Mapping]] = ..., WallTime: _Optional[_Union[Row311, _Mapping]] = ..., Elapsed: _Optional[_Union[Row312, _Mapping]] = ...) -> None: ...

class Row241(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row242(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row243(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row244(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row245(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row246(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row247(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row248(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row249(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row250(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row251(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row252(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row253(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row254(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row255(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row21
    def __init__(self, field_0: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row256(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row257(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row258(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row313
    def __init__(self, field_0: _Optional[_Union[Row313, _Mapping]] = ...) -> None: ...

class Row259(_message.Message):
    __slots__ = ("record_type", "match_fields")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    MATCH_FIELDS_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    match_fields: Row21
    def __init__(self, record_type: _Optional[str] = ..., match_fields: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row260(_message.Message):
    __slots__ = ("stream", "field", "op", "target", "tolerance")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    op: Row314
    target: Row21
    tolerance: float
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ..., op: _Optional[_Union[Row314, _Mapping]] = ..., target: _Optional[_Union[Row21, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class Row261(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, conditions: _Optional[_Iterable[int]] = ...) -> None: ...

class Row262(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row263(_message.Message):
    __slots__ = ("aggregate",)
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    aggregate: str
    def __init__(self, aggregate: _Optional[str] = ...) -> None: ...

class Row264(_message.Message):
    __slots__ = ("previous_record_type",)
    PREVIOUS_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    previous_record_type: str
    def __init__(self, previous_record_type: _Optional[str] = ...) -> None: ...

class Row265(_message.Message):
    __slots__ = ("stream", "field")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

class Row266(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class Row267(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row268(_message.Message):
    __slots__ = ("Unspecified", "Latest")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row315
    Latest: Row316
    def __init__(self, Unspecified: _Optional[_Union[Row315, _Mapping]] = ..., Latest: _Optional[_Union[Row316, _Mapping]] = ...) -> None: ...

class Row269(_message.Message):
    __slots__ = ("Unspecified", "Auto", "Number", "Percent", "Duration", "Distance", "Status", "Text")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    AUTO_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    PERCENT_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row317
    Auto: Row318
    Number: Row319
    Percent: Row320
    Duration: Row321
    Distance: Row322
    Status: Row323
    Text: Row324
    def __init__(self, Unspecified: _Optional[_Union[Row317, _Mapping]] = ..., Auto: _Optional[_Union[Row318, _Mapping]] = ..., Number: _Optional[_Union[Row319, _Mapping]] = ..., Percent: _Optional[_Union[Row320, _Mapping]] = ..., Duration: _Optional[_Union[Row321, _Mapping]] = ..., Distance: _Optional[_Union[Row322, _Mapping]] = ..., Status: _Optional[_Union[Row323, _Mapping]] = ..., Text: _Optional[_Union[Row324, _Mapping]] = ...) -> None: ...

class Row270(_message.Message):
    __slots__ = ("Unspecified", "Value", "Progress", "Badge")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    BADGE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row325
    Value: Row326
    Progress: Row327
    Badge: Row328
    def __init__(self, Unspecified: _Optional[_Union[Row325, _Mapping]] = ..., Value: _Optional[_Union[Row326, _Mapping]] = ..., Progress: _Optional[_Union[Row327, _Mapping]] = ..., Badge: _Optional[_Union[Row328, _Mapping]] = ...) -> None: ...

class Row271(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: Row90
    value: Row21
    state: Row329
    def __init__(self, operator: _Optional[_Union[Row90, _Mapping]] = ..., value: _Optional[_Union[Row21, _Mapping]] = ..., state: _Optional[_Union[Row329, _Mapping]] = ...) -> None: ...

class Row272(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row273(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row274(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row275(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row276(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row277(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row278(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row279(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row280(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row281(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row282(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row283(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row284(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row285(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row286(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row287(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row288(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row289(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row290(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row291(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row292(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row293(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row294(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row295(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row296(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row297(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row298(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row299(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row300(_message.Message):
    __slots__ = ("Unspecified", "Hann", "Rectangular")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    HANN_FIELD_NUMBER: _ClassVar[int]
    RECTANGULAR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row330
    Hann: Row331
    Rectangular: Row332
    def __init__(self, Unspecified: _Optional[_Union[Row330, _Mapping]] = ..., Hann: _Optional[_Union[Row331, _Mapping]] = ..., Rectangular: _Optional[_Union[Row332, _Mapping]] = ...) -> None: ...

class Row301(_message.Message):
    __slots__ = ("Unspecified", "Mean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NONE_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row333
    Mean: Row335
    def __init__(self, Unspecified: _Optional[_Union[Row333, _Mapping]] = ..., Mean: _Optional[_Union[Row335, _Mapping]] = ..., **kwargs) -> None: ...

class Row302(_message.Message):
    __slots__ = ("Unspecified", "Linear", "Db", "Dbu", "Dbm")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    DB_FIELD_NUMBER: _ClassVar[int]
    DBU_FIELD_NUMBER: _ClassVar[int]
    DBM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row336
    Linear: Row337
    Db: Row338
    Dbu: Row339
    Dbm: Row340
    def __init__(self, Unspecified: _Optional[_Union[Row336, _Mapping]] = ..., Linear: _Optional[_Union[Row337, _Mapping]] = ..., Db: _Optional[_Union[Row338, _Mapping]] = ..., Dbu: _Optional[_Union[Row339, _Mapping]] = ..., Dbm: _Optional[_Union[Row340, _Mapping]] = ...) -> None: ...

class Row303(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row304(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row305(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row306(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row307(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row308(_message.Message):
    __slots__ = ("Unspecified", "X", "Y")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row341
    X: Row342
    Y: Row343
    def __init__(self, Unspecified: _Optional[_Union[Row341, _Mapping]] = ..., X: _Optional[_Union[Row342, _Mapping]] = ..., Y: _Optional[_Union[Row343, _Mapping]] = ...) -> None: ...

class Row309(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row344
    Fixed: Row345
    Scientific: Row346
    Engineering: Row347
    SiPrefix: Row348
    AbsoluteTime: Row349
    ElapsedSinceRun: Row350
    ElapsedSinceCycle: Row351
    def __init__(self, Normal: _Optional[_Union[Row344, _Mapping]] = ..., Fixed: _Optional[_Union[Row345, _Mapping]] = ..., Scientific: _Optional[_Union[Row346, _Mapping]] = ..., Engineering: _Optional[_Union[Row347, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row348, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row349, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row350, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row351, _Mapping]] = ...) -> None: ...

class Row310(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row311(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row312(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row313(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: Row352
    left: int
    right: int
    def __init__(self, op: _Optional[_Union[Row352, _Mapping]] = ..., left: _Optional[int] = ..., right: _Optional[int] = ...) -> None: ...

class Row314(_message.Message):
    __slots__ = ("Unspecified", "Exists", "Eq", "Gt", "Gte", "Lt", "Lte", "Within")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    WITHIN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row353
    Exists: Row354
    Eq: Row355
    Gt: Row356
    Gte: Row357
    Lt: Row358
    Lte: Row359
    Within: Row360
    def __init__(self, Unspecified: _Optional[_Union[Row353, _Mapping]] = ..., Exists: _Optional[_Union[Row354, _Mapping]] = ..., Eq: _Optional[_Union[Row355, _Mapping]] = ..., Gt: _Optional[_Union[Row356, _Mapping]] = ..., Gte: _Optional[_Union[Row357, _Mapping]] = ..., Lt: _Optional[_Union[Row358, _Mapping]] = ..., Lte: _Optional[_Union[Row359, _Mapping]] = ..., Within: _Optional[_Union[Row360, _Mapping]] = ...) -> None: ...

class Row315(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row316(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row317(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row318(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row319(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row320(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row321(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row322(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row323(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row324(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row325(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row326(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row327(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row328(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row329(_message.Message):
    __slots__ = ("Unspecified", "Ok", "Warning", "Critical", "Muted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OK_FIELD_NUMBER: _ClassVar[int]
    WARNING_FIELD_NUMBER: _ClassVar[int]
    CRITICAL_FIELD_NUMBER: _ClassVar[int]
    MUTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row361
    Ok: Row362
    Warning: Row363
    Critical: Row364
    Muted: Row365
    def __init__(self, Unspecified: _Optional[_Union[Row361, _Mapping]] = ..., Ok: _Optional[_Union[Row362, _Mapping]] = ..., Warning: _Optional[_Union[Row363, _Mapping]] = ..., Critical: _Optional[_Union[Row364, _Mapping]] = ..., Muted: _Optional[_Union[Row365, _Mapping]] = ...) -> None: ...

class Row330(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row331(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row332(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row333(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row334(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row335(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row336(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row337(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row338(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row339(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row340(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row341(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row342(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row343(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row344(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row366
    def __init__(self, field_0: _Optional[_Union[Row366, _Mapping]] = ...) -> None: ...

class Row345(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row367
    def __init__(self, field_0: _Optional[_Union[Row367, _Mapping]] = ...) -> None: ...

class Row346(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row368
    def __init__(self, field_0: _Optional[_Union[Row368, _Mapping]] = ...) -> None: ...

class Row347(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row369
    def __init__(self, field_0: _Optional[_Union[Row369, _Mapping]] = ...) -> None: ...

class Row348(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row370
    def __init__(self, field_0: _Optional[_Union[Row370, _Mapping]] = ...) -> None: ...

class Row349(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row371
    def __init__(self, field_0: _Optional[_Union[Row371, _Mapping]] = ...) -> None: ...

class Row350(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row372
    def __init__(self, field_0: _Optional[_Union[Row372, _Mapping]] = ...) -> None: ...

class Row351(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row372
    def __init__(self, field_0: _Optional[_Union[Row372, _Mapping]] = ...) -> None: ...

class Row352(_message.Message):
    __slots__ = ("Unspecified", "And", "Or", "Eq", "Ne", "Gt", "Gte", "Lt", "Lte", "Contains", "StartsWith", "EndsWith")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    AND_FIELD_NUMBER: _ClassVar[int]
    OR_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    NE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    CONTAINS_FIELD_NUMBER: _ClassVar[int]
    STARTSWITH_FIELD_NUMBER: _ClassVar[int]
    ENDSWITH_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row373
    And: Row374
    Or: Row375
    Eq: Row376
    Ne: Row377
    Gt: Row378
    Gte: Row379
    Lt: Row380
    Lte: Row381
    Contains: Row382
    StartsWith: Row383
    EndsWith: Row384
    def __init__(self, Unspecified: _Optional[_Union[Row373, _Mapping]] = ..., And: _Optional[_Union[Row374, _Mapping]] = ..., Or: _Optional[_Union[Row375, _Mapping]] = ..., Eq: _Optional[_Union[Row376, _Mapping]] = ..., Ne: _Optional[_Union[Row377, _Mapping]] = ..., Gt: _Optional[_Union[Row378, _Mapping]] = ..., Gte: _Optional[_Union[Row379, _Mapping]] = ..., Lt: _Optional[_Union[Row380, _Mapping]] = ..., Lte: _Optional[_Union[Row381, _Mapping]] = ..., Contains: _Optional[_Union[Row382, _Mapping]] = ..., StartsWith: _Optional[_Union[Row383, _Mapping]] = ..., EndsWith: _Optional[_Union[Row384, _Mapping]] = ...) -> None: ...

class Row353(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row354(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row355(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row356(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row357(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row358(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row359(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row360(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row361(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row362(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row363(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row364(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row365(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row366(_message.Message):
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row367(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row368(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row369(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row370(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row371(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row372(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row385
    def __init__(self, unit: _Optional[_Union[Row385, _Mapping]] = ...) -> None: ...

class Row373(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row374(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row375(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row376(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row377(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row378(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row379(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row380(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row381(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row382(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row383(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row384(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row385(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row386
    Seconds: Row387
    Milliseconds: Row388
    Minutes: Row389
    def __init__(self, Unspecified: _Optional[_Union[Row386, _Mapping]] = ..., Seconds: _Optional[_Union[Row387, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row388, _Mapping]] = ..., Minutes: _Optional[_Union[Row389, _Mapping]] = ...) -> None: ...

class Row386(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row387(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row388(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row389(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
