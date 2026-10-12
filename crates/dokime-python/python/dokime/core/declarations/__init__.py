# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Python declaration objects adapted to Rust-owned contracts."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from collections.abc import Mapping
from copy import deepcopy
from dataclasses import dataclass, field as dataclass_field
from typing import Literal, Protocol, Self, TypeGuard, cast

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core._validation import stable_id

# Type Checking Imports

# === Public Exports === #

# === Constants === #

NETWORK_WIFI_SECTION = "Network / WiFi"
NETWORK_BLUETOOTH_SECTION = "Network / Bluetooth"
_MAIN_SECTION_KEY = "main"
_WIFI_SECTION_KEY = "network_wifi"
_BLUETOOTH_SECTION_KEY = "network_bluetooth"
_SECTION_METADATA = {
    _MAIN_SECTION_KEY: ("main", "Main"),
    _WIFI_SECTION_KEY: ("network.wifi", NETWORK_WIFI_SECTION),
    _BLUETOOTH_SECTION_KEY: ("network.bluetooth", NETWORK_BLUETOOTH_SECTION),
}
_WIFI_GROUP_ALIASES = {
    "wifi",
    "wifi link",
    "network / wifi",
    "network/wifi",
    "network wifi",
}
_BLUETOOTH_GROUP_ALIASES = {
    "bluetooth",
    "bluetooth link",
    "network / bluetooth",
    "network/bluetooth",
    "network bluetooth",
}

# === Types === #

type JsonObject = dict[str, object]
type DocumentCardBodyKind = Literal[
    "text",
    "telemetry_summary",
    "plot",
    "table",
    "steps",
    "criteria",
    "traceability",
    "notes",
    "records",
    "artifacts",
    "media",
    "health",
    "telemetry_context",
]

# === Classes === #


class ContractSerializable(Protocol):
    def to_contract(self) -> object: ...


@dataclass(frozen=True)
class ViewDefinition:
    items: tuple[object, ...]
    title: str | None = None
    metadata: JsonObject = dataclass_field(default_factory=dict)

    def to_contract(self) -> JsonObject:
        return {
            "title": self.title,
            "items": [_declaration_payload(item) for item in self.items],
            "metadata": dict(self.metadata),
        }

    def to_document(
        self,
        *,
        template_id: str | None = None,
        template_name: str | None = None,
    ) -> JsonObject:
        identifier = template_id or "python.template.view"
        sections = _document_sections(self.items)
        cards, layout = _document_cards_and_layout(sections)
        return {
            "document_id": stable_id("document", identifier),
            "metadata": {
                "identifier": identifier,
                "label": template_name or self.title or "Template View",
                "description": None,
            },
            "cards": cards,
            "layout": layout,
            "sections": sections,
        }


@dataclass(frozen=True)
class MetricDeclaration:
    label: str
    source: object | None = None
    stat: str = "last"
    unit: str | None = None
    group: str | None = None
    metadata: JsonObject = dataclass_field(default_factory=dict)


@dataclass(frozen=True)
class DataExpression:
    source: JsonObject
    transforms: tuple[JsonObject, ...] = ()

    def to_contract(self) -> JsonObject:
        return {
            "source": dict(self.source),
            "transforms": [dict(transform) for transform in self.transforms],
        }

    def transform(self, transform_type: str, **fields: object) -> Self:
        return type(self)(
            dict(self.source),
            (*self.transforms, {"type": transform_type, **fields}),
        )

    def count(self) -> Self:
        return self.transform("count")

    def mean(self) -> Self:
        return self.transform("mean")

    def median(self) -> Self:
        return self.transform("median")

    def min(self) -> Self:
        return self.transform("min")

    def max(self) -> Self:
        return self.transform("max")

    def time_between(self, other_record_type: str) -> Self:
        return self.transform("time_between", previous_record_type=other_record_type)


@dataclass(frozen=True)
class Document:
    identifier: str
    label: str
    sections: tuple["Section", ...] = ()
    description: str | None = None

    def section(self, section: "Section") -> Self:
        return type(self)(
            self.identifier,
            self.label,
            (*self.sections, section),
            self.description,
        )

    def with_section(self, section: "Section") -> Self:
        return self.section(section)

    def to_contract(self) -> JsonObject:
        sections = [section.to_contract() for section in self.sections]
        cards, layout = _document_cards_and_layout(sections)
        return {
            "document_id": stable_id("document", self.identifier),
            "metadata": _metadata_contract(
                identifier=self.identifier,
                label=self.label,
                description=self.description,
                tags=(),
            ),
            "cards": cards,
            "layout": layout,
            "sections": sections,
        }

    def to_document(self, **_: object) -> JsonObject:
        return self.to_contract()


@dataclass(frozen=True)
class Section:
    identifier: str
    label: str
    cards: tuple[object, ...] = ()
    description: str | None = None

    def card(self, card: object) -> Self:
        return type(self)(
            self.identifier,
            self.label,
            (*self.cards, card),
            self.description,
        )

    def plot(self, plot: PlotDeclaration) -> Self:
        return self.card(plot)

    def to_contract(self) -> JsonObject:
        return {
            "metadata": _metadata_contract(
                identifier=self.identifier,
                label=self.label,
                description=self.description,
                tags=(),
            ),
            "cards": [_card_contract(card) for card in self.cards],
        }


# === Public Functions === #


def telemetry(stream: str, field_name: str) -> DataExpression:
    return DataExpression(
        {"type": "telemetry", "stream": str(stream), "field": str(field_name)}
    )


def dimension(name: str) -> DataExpression:
    return DataExpression({"type": "dimension", "name": str(name)})


def parameter(name: str) -> DataExpression:
    return DataExpression({"type": "parameter", "name": str(name)})


def artifact(name: str) -> DataExpression:
    return DataExpression({"type": "artifact", "artifact": str(name)})


def metric(label: str, **kwargs: object) -> MetricDeclaration:
    known = {
        "source": kwargs.pop("source", None),
        "stat": str(kwargs.pop("stat", "last")),
        "unit": kwargs.pop("unit", None),
        "group": kwargs.pop("group", None),
    }
    return MetricDeclaration(
        label=label,
        source=known["source"],
        stat=cast(str, known["stat"]),
        unit=None if known["unit"] is None else str(known["unit"]),
        group=None if known["group"] is None else str(known["group"]),
        metadata=dict(kwargs),
    )


def view(
    *items: object, title: str | None = None, **metadata: object
) -> ViewDefinition:
    return ViewDefinition(items=tuple(items), title=title, metadata=dict(metadata))


def artifacts(*args: object, **kwargs: object) -> JsonObject:
    return _card("artifacts", args, kwargs)


def criteria(*args: object, **kwargs: object) -> JsonObject:
    return _card("criteria", args, kwargs)


def records(*args: object, **kwargs: object) -> JsonObject:
    return _card("records", args, kwargs)


def media(*args: object, **kwargs: object) -> JsonObject:
    return _card("media", args, kwargs)


def progress(*args: object, **kwargs: object) -> JsonObject:
    return _card("progress", args, kwargs)


def steps(*args: object, **kwargs: object) -> JsonObject:
    return _card("steps", args, kwargs)


def button(label: str, **kwargs: object) -> JsonObject:
    return {"kind": "button", "label": label, **kwargs}


def box_summary(label: str, **kwargs: object) -> BoxSummaryDeclaration:
    return BoxSummaryDeclaration(label, **kwargs)


def value_type_name(value_type: object) -> str:
    return _native.value_type_name(value_type)


# === Private Functions === #


def _coerce_field(item: FieldDeclaration | tuple[object, ...]) -> FieldDeclaration:
    if isinstance(item, FieldDeclaration):
        return item
    if not item:
        raise ValueError("Telemetry field tuples must not be empty.")
    name = str(item[0])
    value_type = item[1] if len(item) >= 2 else str
    unit = str(item[2]) if len(item) >= 3 and item[2] is not None else None
    return FieldDeclaration(name=name, value_type=value_type, unit=unit)


def _record_field_contract(field: FieldDeclaration) -> JsonObject:
    return _native.record_field_contract(
        field.name,
        field.label,
        field.description,
        field.value_type,
        field.required,
        field.unit,
    )


def _telemetry_column_contract(field: FieldDeclaration) -> JsonObject:
    return _native.telemetry_column_contract(
        field.name,
        field.label,
        field.description,
        field.value_type,
        field.role,
        field.unit,
        field.required,
    )


def _string_list(value: object) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    return [str(item) for item in cast(list[object] | tuple[object, ...], value)]


def _object_list(value: object) -> list[dict[str, object]]:
    if value is None:
        return []
    items = value if isinstance(value, list | tuple) else [value]
    return [dict(item) for item in items if isinstance(item, Mapping)]


def _expression(value: object) -> DataExpression | None:
    if value is None:
        return None
    if isinstance(value, DataExpression):
        return value
    if isinstance(value, RecordDeclaration):
        return value.to_expression()
    if isinstance(value, TelemetryFieldSource) and value.field_name is not None:
        return telemetry(value.stream, value.field_name)
    if isinstance(value, Mapping) and "source" in value:
        return DataExpression(
            dict(cast(Mapping[str, object], value["source"])),
            tuple(
                dict(cast(Mapping[str, object], transform))
                for transform in cast(list[object], value.get("transforms", []))
            ),
        )
    if isinstance(value, str) and "." in value:
        stream, field_name = value.rsplit(".", 1)
        return telemetry(stream, field_name)
    return None


def _expression_contract(value: object) -> JsonObject:
    expression = _expression(value)
    if expression is None:
        raise ValueError(f"Value {value!r} cannot be used as a data expression.")
    return expression.to_contract()


def _optional_expression_contract(value: object) -> JsonObject | None:
    expression = _expression(value)
    return None if expression is None else expression.to_contract()


def _optional_format(value: object) -> JsonObject | None:
    """A declared `x_format`/`y_format`/field `format`, or `None`.

    Written as a plain mapping — `{"kind": "fixed", "decimals": 1}` — the same
    tagged shape `ValueFormat` serializes as on the Rust side, so there is no
    second vocabulary to keep in agreement. An object with its own
    `to_contract()` (a `dokime.core.format` constructor, once one exists) is
    honoured the same way `_expression` honours a declared expression object.
    """
    if value is None:
        return None
    to_contract = getattr(value, "to_contract", None)
    if callable(to_contract):
        return cast(JsonObject, to_contract())
    if isinstance(value, Mapping):
        return dict(cast(Mapping[str, object], value))
    raise ValueError(f"Value {value!r} cannot be used as a value format.")


def _expression_contracts(value: object) -> list[JsonObject]:
    if value is None:
        return []
    if isinstance(value, list | tuple):
        values = cast(list[object] | tuple[object, ...], value)
    else:
        values = (value,)
    return [
        expression.to_contract()
        for item in values
        if (expression := _expression(item)) is not None
    ]


def _filter_contracts(value: object) -> list[JsonObject]:
    if value is None:
        return []
    values = value if isinstance(value, list | tuple) else [value]
    return [dict(item) for item in values if isinstance(item, Mapping)]


def _source_stream(source: object) -> str | None:
    expression = _expression(source)
    if expression is not None and expression.source.get("type") == "telemetry":
        stream = expression.source.get("stream")
        return str(stream) if stream is not None else None
    if source is None:
        return None
    value = getattr(source, "stream", None)
    if isinstance(value, str):
        return value
    value = getattr(source, "stream_id", None)
    if isinstance(value, str):
        return value
    value = getattr(source, "id", None)
    if isinstance(value, str):
        return value
    if isinstance(source, str) and "." not in source:
        return source
    return None


def _source_field(source: object) -> str | None:
    expression = _expression(source)
    if expression is not None:
        return _expression_field(expression)
    if source is None:
        return None
    value = getattr(source, "field_name", None)
    if isinstance(value, str):
        return value
    if isinstance(source, str) and "." in source:
        return source.rsplit(".", 1)[1]
    return None


def _dotted_source_stream(source: object) -> str | None:
    if isinstance(source, str) and "." in source:
        return source.rsplit(".", 1)[0]
    return None


def _field_name(value: object) -> str | None:
    expression = _expression(value)
    if expression is not None:
        return _expression_field(expression)
    if value is None:
        return None
    field_name = getattr(value, "field_name", None)
    if isinstance(field_name, str):
        return field_name
    if isinstance(value, str) and "." in value:
        return value.rsplit(".", 1)[1]
    return str(value)


def _expression_field(expression: DataExpression) -> str | None:
    source = expression.source
    source_type = source.get("type")
    if source_type == "telemetry":
        field_name = source.get("field")
    elif source_type in {"dimension", "parameter"}:
        field_name = source.get("name")
    elif source_type == "record":
        field_name = source.get("record_type")
    elif source_type == "artifact":
        field_name = source.get("artifact")
    else:
        field_name = None
    return None if field_name is None else str(field_name)


def _field_list(value: object, *, fallback: str | None = None) -> list[str]:
    if value is None:
        return [] if fallback is None else [fallback]
    if not isinstance(value, list | tuple):
        if field := _field_name(value):
            return [field]
    return [
        field
        for item in cast(list[object] | tuple[object, ...], value)
        if (field := _field_name(item)) is not None
    ]


def _mapping_like(value: object) -> JsonObject:
    if value is None:
        return {}
    if isinstance(value, SigmaInterval):
        return dict(value.options)
    if isinstance(value, dict):
        return dict(value)
    options = getattr(value, "options", None)
    if isinstance(options, dict):
        return dict(options)
    return {}


def _plot_options(options: Mapping[str, object]) -> JsonObject:
    nested = options.get("options")
    merged = (
        _plot_options(cast(Mapping[str, object], nested))
        if isinstance(nested, Mapping)
        else {}
    )
    for key, value in options.items():
        if key == "options":
            continue
        if value is not None or key not in merged:
            merged[key] = value
    return merged


def _interval_options(value: object) -> JsonObject:
    return _mapping_like(value)


def _histogram_options(options: JsonObject, unit: object) -> JsonObject:
    histogram = _mapping_like(options.pop("histogram", None))
    if "bins" in options and "bins" not in histogram:
        histogram["bins"] = options.pop("bins")
    if unit is not None and "unit" not in histogram:
        histogram["unit"] = str(unit)
    return histogram


def _optional_float(value: object) -> float | None:
    if value is None:
        return None
    return float(cast(float | int | str, value))


def _is_nullable(value_type: object) -> bool:
    return _native.nullable_value_type(value_type)


def _title(value: str) -> str:
    return value.replace(".", " ").replace("_", " ").replace("-", " ").title()


def _declaration_payload(value: object) -> JsonObject:
    if _has_contract(value):
        payload = value.to_contract()
        return payload if isinstance(payload, dict) else {"value": payload}
    if hasattr(value, "__dict__"):
        return {"kind": type(value).__name__, **dict(value.__dict__)}
    if isinstance(value, dict):
        return dict(value)
    return {"value": str(value)}


def _document_sections(items: tuple[object, ...]) -> list[JsonObject]:
    grouped: dict[str, list[JsonObject]] = {
        _MAIN_SECTION_KEY: [],
        _WIFI_SECTION_KEY: [],
        _BLUETOOTH_SECTION_KEY: [],
    }

    for item in items:
        grouped[_document_section_key(item)].append(_declaration_payload(item))

    sections: list[JsonObject] = []
    for key in (_MAIN_SECTION_KEY, _WIFI_SECTION_KEY, _BLUETOOTH_SECTION_KEY):
        cards = grouped[key]
        if not cards:
            continue
        identifier, label = _SECTION_METADATA[key]
        sections.append(
            {
                "metadata": {
                    "identifier": identifier,
                    "label": label,
                    "description": None,
                },
                "cards": cards,
            }
        )
    return sections


def _document_section_key(value: object) -> str:
    group = getattr(value, "group", None)
    options = getattr(value, "options", None)
    if group is None and isinstance(options, Mapping):
        group = options.get("group")
    if group is None and isinstance(value, dict):
        group = value.get("group")
    if not isinstance(group, str):
        return _MAIN_SECTION_KEY

    normalized = group.strip().casefold()
    if normalized in _WIFI_GROUP_ALIASES:
        return _WIFI_SECTION_KEY
    if normalized in _BLUETOOTH_GROUP_ALIASES:
        return _BLUETOOTH_SECTION_KEY
    return _MAIN_SECTION_KEY


def _document_cards_and_layout(
    sections: list[JsonObject],
) -> tuple[list[JsonObject], JsonObject]:
    cards: list[JsonObject] = []
    card_indices: dict[str, int] = {}
    nodes = [
        node
        for section in sections
        if (node := _document_section_node(section, cards, card_indices)) is not None
    ]
    return cards, {"nodes": nodes}


def _document_section_node(
    section: JsonObject,
    cards: list[JsonObject],
    card_indices: dict[str, int],
) -> JsonObject | None:
    metadata = section.get("metadata")
    if not isinstance(metadata, Mapping):
        return None
    section_metadata = dict(metadata)
    section_id = str(section_metadata.get("identifier") or "section")
    children = [
        _document_card_placement(card, cards, card_indices, section_id, index)
        for index, card in enumerate(cast(list[object], section.get("cards", [])))
        if isinstance(card, Mapping)
    ]
    return {
        "type": "section",
        "data": {
            "node_id": section_id,
            "metadata": section_metadata,
            "children": children,
        },
    }


def _document_card_placement(
    card: Mapping[str, object],
    cards: list[JsonObject],
    card_indices: dict[str, int],
    section_id: str,
    index: int,
) -> JsonObject:
    definition = dict(card)
    layout = cast(
        JsonObject, deepcopy(definition.pop("layout", _default_card_layout()))
    )
    fallback_id = f"{section_id}.card.{index + 1}"
    card_id = _ensure_document_card_metadata(definition, fallback_id)
    definition.setdefault("filters", [])
    if card_id in card_indices:
        cards[card_indices[card_id]] = definition
    else:
        card_indices[card_id] = len(cards)
        cards.append(definition)
    return {
        "type": "card",
        "data": {
            "card_id": card_id,
            "layout": layout,
        },
    }


def _ensure_document_card_metadata(card: JsonObject, fallback_id: str) -> str:
    metadata = card.get("metadata")
    if isinstance(metadata, Mapping):
        normalized = dict(metadata)
        identifier = str(normalized.get("identifier") or fallback_id)
        normalized["identifier"] = identifier
        normalized.setdefault("label", _title(identifier))
        normalized.setdefault("description", None)
        card["metadata"] = normalized
        return identifier

    card["metadata"] = _metadata_contract(
        identifier=fallback_id,
        label=_title(fallback_id),
        description=None,
        tags=(),
    )
    return fallback_id


def _card_contract(value: object) -> JsonObject:
    if isinstance(value, PlotDeclaration):
        return value.to_contract()
    if isinstance(value, Mapping):
        payload = dict(value)
        if "metadata" in payload and "type" in payload:
            return payload
        kind = payload.get("kind")
        if isinstance(kind, str):
            label = str(payload.get("label") or _title(kind))
            identifier = str(payload.get("identifier") or f"card.{kind}")
            if kind == "text":
                body = {
                    "type": "text",
                    "data": {"markdown": str(payload.get("markdown") or "")},
                }
            elif kind in {"artifacts", "criteria", "records", "steps"}:
                body = {"type": kind}
            else:
                body = {"type": "text", "data": {"markdown": label}}
            return {
                "metadata": _metadata_contract(
                    identifier=identifier,
                    label=label,
                    description=None,
                    tags=(),
                ),
                "layout": _default_card_layout(),
                "filters": [],
                **body,
            }
        return payload
    if _has_contract(value):
        contract = value.to_contract()
        if isinstance(contract, dict):
            return contract
    return {
        "metadata": _metadata_contract(
            identifier=_identifier_from_label("card", str(value)),
            label=str(value),
            description=None,
            tags=(),
        ),
        "layout": _default_card_layout(),
        "filters": [],
        "type": "text",
        "data": {"markdown": str(value)},
    }


def _card(kind: str, args: tuple[object, ...], kwargs: dict[str, object]) -> JsonObject:
    payload = {"kind": kind, **kwargs}
    if args:
        payload["label"] = str(args[0])
    if len(args) > 1:
        payload["args"] = [str(item) for item in args[1:]]
    return payload


def _metadata_contract(
    *,
    identifier: str,
    label: str,
    description: str | None,
    tags: tuple[str, ...],
) -> JsonObject:
    return cast(
        JsonObject,
        _native.metadata_contract(identifier, label, description, list(tags)),
    )


def _default_card_layout() -> JsonObject:
    return {
        "col_span": {
            "base": 12,
            "sm": None,
            "md": None,
            "lg": None,
            "xl": None,
        },
        "row_span": 1,
        "order": 0,
        "density": "normal",
        "rail": "main",
    }


def _identifier_from_label(prefix: str, label: str) -> str:
    suffix = (
        "".join(
            character.lower() if character.isalnum() else "."
            for character in label.strip()
        )
        .strip(".")
        .replace("..", ".")
    )
    return f"{prefix}.{suffix or 'item'}"


def _has_contract(value: object) -> TypeGuard[ContractSerializable]:
    return callable(getattr(value, "to_contract", None))


# === Entry Point === #

# Compose the facade after its shared declarations exist: these modules import
# those declarations while they initialize.
# === Plot declarations (split into ./plots.py) === #
from dokime.core.declarations.plots import (  # noqa: E402
    PlotDeclaration as PlotDeclaration,
    SigmaInterval as SigmaInterval,
    TimeSeriesPlot as TimeSeriesPlot,
    XyScatterPlot as XyScatterPlot,
    XyPlot as XyPlot,
    XyLinePlot as XyLinePlot,
    XyIntervalPlot as XyIntervalPlot,
    TrajectoryPlot as TrajectoryPlot,
    HeatmapPlot as HeatmapPlot,
    WifiChannelOccupancyPlot as WifiChannelOccupancyPlot,
    HistogramPlot as HistogramPlot,
    FrequencySeriesPlot as FrequencySeriesPlot,
    BarPlot as BarPlot,
    GroupedBarPlot as GroupedBarPlot,
    StackedBarPlot as StackedBarPlot,
    PiePlot as PiePlot,
    DonutPlot as DonutPlot,
    ParetoPlot as ParetoPlot,
    XyValueHeatmapPlot as XyValueHeatmapPlot,
    BoxSummaryDeclaration as BoxSummaryDeclaration,
    AggregateTablePlot as AggregateTablePlot,
)

# === Record schema declarations (split into ./record_schemas.py) === #
from dokime.core.declarations.record_schemas import (  # noqa: E402
    FieldDeclaration as FieldDeclaration,
    RecordDeclaration as RecordDeclaration,
    HubRecord as HubRecord,
    field as field,
)

# === Telemetry schema declarations (split into ./telemetry_schemas.py) === #
from dokime.core.declarations.telemetry_schemas import (  # noqa: E402
    TelemetryFieldSource as TelemetryFieldSource,
    TelemetryStreamDeclaration as TelemetryStreamDeclaration,
    telemetry_stream as telemetry_stream,
)
