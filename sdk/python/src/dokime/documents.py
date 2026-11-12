# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Value Composition Of Canonical Document Messages."""

# === Imports === #

# Standard Library Imports
import re
from typing import Any, Self

# Third Party Imports
# Local Imports
from .plot import Expression, Plot, _expression
from .v1 import message

# Type Checking Imports

# === Public Exports === #

__all__ = ["Document", "Section", "Summary"]

# === Constants === #

# === Types === #

# === Classes === #


class Document:
    def __init__(self, label: str) -> None:
        self.message = message("Document", metadata=metadata(label))

    @property
    def label(self) -> str:
        return self.message.metadata.label

    def id(self, identifier: str) -> Self:
        self.message.metadata.identifier = identifier
        return self

    def section(self, section: "Section") -> Self:
        if not isinstance(section, Section):
            raise TypeError("Document.section requires a Section value")
        self.message.sections.add().CopyFrom(section.message)
        return self

    def _definition(self) -> dict[str, Any]:
        return {"message": list(self.message.SerializeToString())}


class Section:
    def __init__(self, label: str) -> None:
        self.message = message("Section", metadata=metadata(label))

    def id(self, identifier: str) -> Self:
        self.message.metadata.identifier = identifier
        return self

    def plot(self, plot: Plot) -> Self:
        if not isinstance(plot, Plot):
            raise TypeError("Section.plot requires a Plot value")
        value = message("Plot")
        value.CopyFrom(plot.message)
        if not value.metadata.label:
            expression = value.z if value.HasField("z") else value.y[0] if value.y else value.x
            label = expression_label(expression)
            identifier = value.metadata.identifier
            value.metadata.CopyFrom(metadata(label))
            if identifier:
                value.metadata.identifier = identifier
        self.message.cards.add(metadata=value.metadata, plot=value)
        return self

    def section(self, section: "Section") -> Self:
        if not isinstance(section, Section):
            raise TypeError("Section.section requires a Section value")
        self.message.sections.add().CopyFrom(section.message)
        return self

    def summary(self, summary: "Summary", *, label: str = "Summary") -> Self:
        if not isinstance(summary, Summary):
            raise TypeError("Section.summary requires a Summary value")
        self.message.cards.add(metadata=metadata(label), summary=summary.message)
        return self

    def text(self, label: str, markdown: str) -> Self:
        self.message.cards.add(metadata=metadata(label), markdown=markdown)
        return self

    def card(self, card: Any) -> Self:
        if card.DESCRIPTOR.full_name != "dokime.v1.Card":
            raise TypeError("Section.card requires a dokime.v1.Card message")
        self.message.cards.add().CopyFrom(card)
        return self

    def _definition(self) -> dict[str, Any]:
        return {"message": list(self.message.SerializeToString())}


class Summary:
    def __init__(self) -> None:
        self.message = message("Summary")

    def tile(self, label: str, value: Expression, *, unit: str = "") -> Self:
        self.message.tiles.add(metadata=metadata(label), value=_expression(value), unit=unit)
        return self


# === Public Functions === #


def metadata(label: str) -> Any:
    return message("Metadata", identifier=".".join(part.lower() for part in re.split(r"[\W_]+", label) if part), label=label)


# === Private Functions === #


def expression_label(value: Any) -> str:
    if value.label:
        return value.label
    kind = value.WhichOneof("kind")
    if kind in ("stream", "record"):
        return getattr(value, kind).field
    if kind in ("aggregate", "transform"):
        return expression_label(getattr(value, kind).input)
    return "Value"


# === Entry Point === #
