# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Typed prompt instances whose declared fields become the operator's form.

A template declares the prompt class. A step passes an instance to ``await
ctx.ask(Checkpoint(note="observed"))`` and receives an instance of the same
class; a ``Confirmation`` instance answers with ``bool``. Constructor defaults
exist only for fields with explicit declared defaults.
"""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import enum
import inspect
import types
from collections.abc import Callable, Mapping, Sequence
from copy import deepcopy
from dataclasses import dataclass
from typing import (
    Any,
    Literal,
    TypeVar,
    Union,
    dataclass_transform,
    get_args,
    get_origin,
)

# Third Party Imports

# Local Imports
from dokime import _native

# Type Checking Imports

# === Public Exports === #

__all__ = [
    "Confirmation",
    "DeclaredPrompt",
    "PromptCancelled",
    "PromptFieldDeclaration",
    "PromptRefused",
    "declared_prompt",
    "prompt",
    "prompt_field",
]

# === Constants === #

_MISSING: Any = object()
_SCALAR_KINDS: dict[type, str] = {
    str: "text",
    bool: "boolean",
    int: "int",
    float: "float",
}

# === Types === #

T = TypeVar("T")

# === Classes === #


class PromptCancelled(RuntimeError):
    """The operator cancelled the question instead of answering it."""


class PromptRefused(ValueError):
    """The answer does not fit the declaration it was given for."""


class Confirmation:
    """A yes/no question.

    Derive it and decorate with ``@dokime.prompt(id=..., label=...)``; the
    class declares no fields and ``ctx.ask`` answers with ``bool``.
    """


@dataclass(frozen=True)
class PromptFieldDeclaration:
    """One field of a declared form, as the host reads it and the answer
    decodes it."""

    name: str
    label: str
    kind: str
    required: bool
    multiple: bool
    description: str | None
    default: object
    has_default: bool
    choices: tuple[str, ...]
    validation: dict[str, object]
    element: object

    def contract(self) -> dict[str, object]:
        written: dict[str, object] = {
            "name": self.name,
            "label": self.label,
            "kind": self.kind,
            "required": self.required,
            "multiple": self.multiple,
            "description": self.description,
            "choices": list(self.choices),
            "validation": dict(self.validation),
        }
        if self.has_default:
            written["default"] = _wire(self.default)
        return written

    def decode(self, value: object) -> object:
        if value is None:
            if self.required:
                raise PromptRefused(f"field `{self.name}` was not answered")
            return None
        if self.multiple:
            if not isinstance(value, Sequence) or isinstance(value, str | bytes):
                raise PromptRefused(f"field `{self.name}` is a list")
            return [self._decode_one(item) for item in value]
        return self._decode_one(value)

    def _decode_one(self, value: object) -> object:
        element = self.element
        if self.kind == "boolean":
            if not isinstance(value, bool):
                raise PromptRefused(f"field `{self.name}` is a boolean")
            return value
        if self.kind == "int":
            if isinstance(value, bool) or not isinstance(value, int):
                raise PromptRefused(f"field `{self.name}` is an integer")
            return value
        if self.kind == "float":
            if isinstance(value, bool) or not isinstance(value, int | float):
                raise PromptRefused(f"field `{self.name}` is a number")
            return float(value)
        if not isinstance(value, str):
            raise PromptRefused(f"field `{self.name}` is text")
        if self.kind == "choice":
            if value not in self.choices:
                raise PromptRefused(
                    f"field `{self.name}` admits one of {', '.join(self.choices)}"
                )
            if isinstance(element, type) and issubclass(element, enum.Enum):
                return _enum_member(element, value)
        return value


@dataclass(frozen=True)
class DeclaredPrompt:
    """What ``@dokime.prompt`` wrote for one class: the definition the template
    registers and the decode an asking answers through."""

    prompt_type: str
    label: str
    description: str | None
    submit: str | None
    cancel: str | None
    fields: tuple[PromptFieldDeclaration, ...]
    owner: type[object]
    confirmation: bool

    def definition(self) -> dict[str, object]:
        """The definition, as the template's contract carries it."""
        return _native.prompt_definition_contract(
            self.prompt_type,
            self.label,
            self.description,
            self.submit,
            self.cancel,
            None if self.confirmation else [field.contract() for field in self.fields],
        )

    def values(self) -> dict[str, object]:
        """The declared defaults, which every asking carries unless it says
        otherwise."""
        return {
            field.name: _wire(field.default)
            for field in self.fields
            if field.has_default
        }

    def decode(self, response: Mapping[str, object]) -> object:
        """The answer, out of the host's response."""
        action = str(response.get("action") or "")
        if response.get("accepted") is False or action == "cancel":
            raise PromptCancelled(f"`{self.prompt_type}` was cancelled")
        if self.confirmation:
            if action == "yes":
                return True
            if action == "no":
                return False
            raise PromptCancelled(f"`{self.prompt_type}` answered `{action}`")
        values = response.get("values")
        if not isinstance(values, Mapping):
            values = {}
        return self.instance(
            {field.name: field.decode(values.get(field.name)) for field in self.fields}
        )

    def instance(self, answered: Mapping[str, object]) -> object:
        """One decoded answer, as the declaring class."""
        answer = object.__new__(self.owner)
        for field in self.fields:
            object.__setattr__(answer, field.name, answered.get(field.name))
        return answer


class _PromptField:
    """What `prompt_field(...)` wrote for one field: its label and default,
    and the bounds an answer must meet. The field's type is its annotation."""

    def __init__(
        self,
        *,
        label: str | None = None,
        description: str | None = None,
        default: object = _MISSING,
        min: object = None,  # noqa: A002 - mirrors the Rust argument name
        max: object = None,  # noqa: A002 - mirrors the Rust argument name
        min_length: int | None = None,
        max_length: int | None = None,
        pattern: str | None = None,
        min_items: int | None = None,
        max_items: int | None = None,
    ) -> None:
        self.name = ""
        self.annotation: object = None
        self.label = label
        self.description = description
        self.default = default
        self.validation: dict[str, object] = {
            key: value
            for key, value in {
                "minimum": min,
                "maximum": max,
                "min_length": min_length,
                "max_length": max_length,
                "pattern": pattern,
                "min_items": min_items,
                "max_items": max_items,
            }.items()
            if value is not None
        }

    def __set_name__(self, owner: type[object], name: str) -> None:
        self.name = name
        self.annotation = _annotations(owner).get(name)

    def __get__(self, instance: object, owner: type[object] | None = None) -> Any:
        if instance is None:
            return self
        raise AttributeError(self.name)

    def declaration(self) -> PromptFieldDeclaration:
        kind, required, multiple, choices, element = _resolve(
            self.name, self.annotation
        )
        if not multiple and any(
            key in self.validation for key in ("min_items", "max_items")
        ):
            raise TypeError(
                f"min_items and max_items bound a multiple choice: declare "
                f"`{self.name}` as `list[...]`"
            )
        default = self.default
        if default is not _MISSING and isinstance(default, enum.Enum):
            default = _choice_id(default)
        if default is not _MISSING and multiple and isinstance(default, Sequence):
            default = [
                _choice_id(item) if isinstance(item, enum.Enum) else item
                for item in default
            ]
        return PromptFieldDeclaration(
            name=self.name,
            label=self.label or _title(self.name),
            kind=kind,
            required=required,
            multiple=multiple,
            description=self.description,
            default=None if default is _MISSING else default,
            has_default=default is not _MISSING,
            choices=choices,
            validation=dict(self.validation),
            element=element,
        )


# === Public Functions === #


def prompt_field(
    *,
    label: str | None = None,
    description: str | None = None,
    default: Any = _MISSING,
    min: object = None,  # noqa: A002 - mirrors the Rust argument name
    max: object = None,  # noqa: A002 - mirrors the Rust argument name
    min_length: int | None = None,
    max_length: int | None = None,
    pattern: str | None = None,
    min_items: int | None = None,
    max_items: int | None = None,
) -> Any:
    """The rest of one field of a declared prompt: its label and default, and
    the bounds an answer must meet. The field's type is its annotation."""
    return _PromptField(
        label=label,
        description=description,
        default=default,
        min=min,
        max=max,
        min_length=min_length,
        max_length=max_length,
        pattern=pattern,
        min_items=min_items,
        max_items=max_items,
    )


@dataclass_transform(kw_only_default=True, field_specifiers=(prompt_field,))
def prompt(
    *,
    id: str,  # noqa: A002 - mirrors the Rust attribute argument
    label: str,
    description: str | None = None,
    submit: str | None = None,
    cancel: str | None = None,
) -> Callable[[type[T]], type[T]]:
    """Declares one operator question as the decorated class.

    The class's annotated attributes are the form's fields, each optionally
    detailed with ``prompt_field(...)``, and the class is what the answer
    decodes into. A class deriving ``Confirmation`` declares no fields and
    answers with ``bool``. Name the class in the template's ``prompts`` and ask
    it with ``await ctx.ask(Class(...))``. Instance fields supply this asking's
    values, including explicit ``None`` for optional fields.
    """

    def declare(cls: type[T]) -> type[T]:
        confirmation = issubclass(cls, Confirmation)
        fields = _fields(cls)
        if confirmation and fields:
            raise TypeError(
                f"`{cls.__name__}` is a confirmation and declares no fields"
            )
        if confirmation and (submit is not None or cancel is not None):
            raise TypeError(
                "a confirmation answers yes or no; `submit` and `cancel` label "
                "a form's actions"
            )
        declared = DeclaredPrompt(
            prompt_type=id,
            label=label,
            description=description,
            submit=submit,
            cancel=cancel,
            fields=tuple(fields),
            owner=cls,
            confirmation=confirmation,
        )
        # Both languages carry the same declaration and construct prompt values
        # directly from the authored type.
        written: dict[str, object] = {
            "__dokime_prompt__": declared,
            "PROMPT_TYPE": id,
        }
        for name, value in written.items():
            setattr(cls, name, value)
        if not confirmation:
            _decorate_answer(cls, declared)
        return cls

    return declare


def declared_prompt(cls: object) -> DeclaredPrompt:
    """The declaration behind a ``@dokime.prompt`` class, refusing any other."""
    declared = getattr(cls, "__dokime_prompt__", None)
    if not isinstance(declared, DeclaredPrompt):
        raise TypeError(
            f"{cls!r} is not a declared prompt; decorate it with `@dokime.prompt`"
        )
    return declared


# === Private Functions === #


def _fields(cls: type[object]) -> list[PromptFieldDeclaration]:
    declared: list[PromptFieldDeclaration] = []
    annotations = _annotations(cls)
    # A field marker without an annotation has no type to be a field of, and
    # skipping it silently would drop a question the author wrote.
    for name, written in vars(cls).items():
        if isinstance(written, _PromptField) and name not in annotations:
            raise TypeError(f"prompt field `{name}` needs a type annotation")
    for name in annotations:
        if name.startswith("_"):
            continue
        written = cls.__dict__.get(name, _MISSING)
        if isinstance(written, _PromptField):
            declared.append(written.declaration())
            continue
        # An annotation alone is a field with no default and no bounds, the way
        # a bare struct field is in Rust; a plain class attribute is its default.
        implicit = _PromptField()
        if written is not _MISSING:
            implicit.default = written
        implicit.__set_name__(cls, name)
        declared.append(implicit.declaration())
    return declared


def _annotations(cls: type[object]) -> dict[str, object]:
    try:
        return dict(inspect.get_annotations(cls, eval_str=True))
    except (NameError, TypeError):
        return dict(getattr(cls, "__annotations__", {}))


def _resolve(
    name: str, annotation: object
) -> tuple[str, bool, bool, tuple[str, ...], object]:
    if annotation is None:
        raise TypeError(f"prompt field `{name}` needs a type annotation")
    required = True
    origin = get_origin(annotation)
    if origin is Union or origin is types.UnionType:
        members = [
            member for member in get_args(annotation) if member is not type(None)
        ]
        if len(members) != 1 or len(get_args(annotation)) != 2:
            raise TypeError(f"prompt field `{name}` is one type, or that type or None")
        annotation = members[0]
        required = False
        origin = get_origin(annotation)
    multiple = False
    if origin is list:
        (annotation,) = get_args(annotation) or (None,)
        multiple = True
        origin = get_origin(annotation)
    if origin is Literal:
        choices = get_args(annotation)
        if not choices or not all(isinstance(choice, str) for choice in choices):
            raise TypeError(f"prompt field `{name}` is a choice over text literals")
        return "choice", required, multiple, tuple(choices), annotation
    if isinstance(annotation, type) and issubclass(annotation, enum.Enum):
        return (
            "choice",
            required,
            multiple,
            tuple(_choice_id(member) for member in annotation),
            annotation,
        )
    if isinstance(annotation, type) and annotation in _SCALAR_KINDS:
        return _SCALAR_KINDS[annotation], required, multiple, (), annotation
    raise TypeError(
        f"prompt field `{name}` is str, bool, int, float, a Literal of text, an "
        f"Enum, a list of one of those, or one of those or None"
    )


def _choice_id(member: enum.Enum) -> str:
    value = member.value
    return value if isinstance(value, str) else str(member.name)


def _enum_member(enumeration: type[enum.Enum], value: str) -> enum.Enum:
    for member in enumeration:
        if _choice_id(member) == value:
            return member
    raise PromptRefused(f"`{value}` is not one of {enumeration.__name__}")


def _wire(value: object) -> object:
    if isinstance(value, enum.Enum):
        return _choice_id(value)
    if isinstance(value, Sequence) and not isinstance(value, str | bytes):
        return [_wire(item) for item in value]
    return value


def _title(name: str) -> str:
    return " ".join(part.capitalize() for part in name.split("_") if part)


def _decorate_answer(cls: type[object], declared: DeclaredPrompt) -> None:
    names = [field.name for field in declared.fields]

    def __init__(self: object, **answered: object) -> None:
        unknown = set(answered) - set(names)
        if unknown:
            raise TypeError(f"{cls.__name__} has no field {sorted(unknown)}")
        for field in declared.fields:
            if field.name in answered:
                value = answered[field.name]
            elif field.has_default:
                value = deepcopy(field.default)
            else:
                raise TypeError(f"{cls.__name__} requires field {field.name!r}")
            object.__setattr__(self, field.name, field.decode(value))

    def __repr__(self: object) -> str:
        fields = ", ".join(f"{name}={getattr(self, name)!r}" for name in names)
        return f"{cls.__name__}({fields})"

    def __eq__(self: object, other: object) -> bool:
        if type(other) is not type(self):
            return NotImplemented
        return all(getattr(self, name) == getattr(other, name) for name in names)

    generated: dict[str, object] = {"__init__": __init__, "__repr__": __repr__}
    if "__eq__" not in cls.__dict__:
        generated["__eq__"] = __eq__
        # An answer compares by its fields, so like a dataclass it is unhashable.
        generated["__hash__"] = None
    for name, member in generated.items():
        if name not in cls.__dict__:
            setattr(cls, name, member)


# === Entry Point === #
