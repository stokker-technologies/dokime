# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Testing Policy from `[conventions.testing]` and the Expected Lane Set.

Keys, all optional:

    [conventions.testing]
    limits = { unit = 512, integration = 256, acceptance = 128, system = 64 }
    deadlines = { unit = 30, integration = 120, acceptance = 900, system = 900 }
    bench_regression_percent = 20

    [[conventions.testing.overrides]]
    category = "acceptance"
    language = "rust"
    select = "test(=cycle::runs)"
    timeout_seconds = 300
    reason = "Streams a full capture"

Limits are story counts per layer, enforced by the source inventory. TypeScript and
JavaScript workspace packages each have their own scope; other tests share the project scope.
Exceeding one is a curation finding in the
index, never a failure. Deadlines are hang guards in seconds, read from
`tools/project.toml`. Native runners terminate hung stories; script lanes also have a
whole-lane process-tree guard. Completed story durations are measurements, never pass/fail
criteria. Rust overrides use nextest filtersets to replace selected story hang guards.
Acceptance and system lanes receive the whole Product and Tests builds of their profile
and target, so lanes declare no inputs.

The expected lanes are declared in `conventions.testing.expected_lanes` in `tools/project.toml`.
"""

# === Imports === #

# Standard Library Imports
import math
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CATEGORIES = ("unit", "integration", "acceptance", "system")
BENCH_CATEGORIES = ("unit", "integration", "system")
LANGUAGES = ("rust", "python", "typescript", "javascript")
PROFILES = ("dev", "release")
LIMITS = {"unit": 512, "integration": 256, "acceptance": 128, "system": 64}
DEADLINES = {"unit": 30, "integration": 120, "acceptance": 900, "system": 900}

# Minimum Correctness-Test Hang Guard; Performance Belongs To Benchmarks.
MINIMUM_TIMEOUT = 30
COVERAGE_FACTOR = 5
OVERRIDE_KEYS = {"category", "language", "select", "timeout_seconds", "reason"}

# === Types === #

# === Classes === #


class TestError(Exception):
    """A preflight, policy or runner failure reported as one clear message."""


@dataclass(frozen=True)
class Override:
    category: str
    language: str
    select: str
    timeout_seconds: int
    reason: str


@dataclass(frozen=True)
class Policy:
    limits: dict[str, int]
    deadlines: dict[str, int]
    overrides: tuple[Override, ...]
    bench_regression_percent: float = 20

    def deadline(self, category: str, coverage: bool = False) -> int:
        """The lane hang guard, multiplied for instrumented runs."""
        return self.deadlines[category] * (COVERAGE_FACTOR if coverage else 1)

    def selected(self, category: str, language: str) -> list[Override]:
        selected_items = []
        for selected_entry in self.overrides:
            if selected_entry.category != category:
                continue
            if selected_entry.language != language:
                continue
            selected_items.append(selected_entry)
        return selected_items


# === Public Functions === #


def root(start: Path | None = None) -> Path:
    """The project root: the nearest directory holding project policy."""
    current = (start or Path.cwd()).resolve()
    for candidate in (current, *current.parents):
        if (candidate / "tools/project.toml").is_file():
            return candidate
    raise TestError("Run Policy Tests: no project policy file found above the working directory")


def load(directory: Path) -> Policy:
    document = _toml(directory / "tools/project.toml")
    testing = document.get("conventions", {}).get("testing", {})
    if not isinstance(testing, dict):
        raise TestError("Run Policy Tests: [conventions.testing] must be a table")

    limits = _numbers(testing.get("limits", {}), "limits", LIMITS, 0)
    deadlines = _numbers(testing.get("deadlines", {}), "deadlines", DEADLINES, MINIMUM_TIMEOUT)
    retired = []
    for retired_key in ("hosts", "lanes"):
        if retired_key not in testing:
            continue
        retired.append(retired_key)

    if retired:
        raise TestError(
            f"Run Policy Tests: conventions.testing.{retired[0]} is retired; acceptance and system lanes receive the whole Product and Tests builds"
        )

    overrides = []
    for entry in testing.get("overrides", []):
        if not isinstance(entry, dict) or set(entry) != OVERRIDE_KEYS:
            raise TestError("each override declares " + ", ".join(sorted(OVERRIDE_KEYS)))
        if entry["category"] not in CATEGORIES or entry["language"] not in LANGUAGES:
            raise TestError(f"Run Policy Tests: override names an unknown lane: {entry}")
        if not _positive(entry["timeout_seconds"], MINIMUM_TIMEOUT) or not str(entry["reason"]).strip():
            raise TestError(f"Run Policy Tests: override needs a timeout of at least 30 seconds and a reason: {entry}")
        overrides.append(Override(**entry))
    threshold = testing.get("bench_regression_percent", 20)
    if isinstance(threshold, bool) or not isinstance(threshold, (int, float)) or not math.isfinite(threshold) or threshold <= 0:
        raise TestError("Run Policy Tests: bench_regression_percent must be finite and positive")
    return Policy(
        limits,
        deadlines,
        tuple(overrides),
        float(threshold),
    )


def lanes(directory: Path) -> list[tuple[str, str]]:
    """Read Expected Lanes Independently of the Public or Hidden Task Surface."""
    testing = _toml(directory / "tools/project.toml").get("conventions", {}).get("testing", {})
    declared = testing.get("expected_lanes", [])
    if not isinstance(declared, list) or not all(isinstance(name, str) for name in declared):
        raise TestError("Select Expected Lanes: Declare language:category Names in conventions.testing.expected_lanes")
    expected = []
    for name in declared:
        parts = name.split(":")
        if len(parts) != 2 or parts[0] not in LANGUAGES or parts[1] not in CATEGORIES:
            raise TestError(f"Select Expected Lanes: Replace Unknown Lane {name}")
        lane = (parts[1], parts[0])
        if lane in expected:
            raise TestError(f"Select Expected Lanes: Remove Duplicate Lane {name}")
        expected.append(lane)

    def sort_key(lane: tuple[str, str]) -> tuple:
        return (CATEGORIES.index(lane[0]), lane[1])

    return sorted(expected, key=sort_key)


# === Private Functions === #


def _toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise TestError(f"Run Policy Tests: read {path.name}: {error}") from error


def _numbers(value: object, name: str, defaults: dict[str, int], minimum: int) -> dict[str, int]:
    if not isinstance(value, dict) or not set(value) <= set(defaults):
        raise TestError(f"Run Policy Tests: conventions.testing.{name} keys are {', '.join(defaults)}")
    if not all(_positive(number, minimum) for number in value.values()):
        raise TestError(f"Run Policy Tests: conventions.testing.{name} values must be integers >= {minimum}")
    return {**defaults, **value}


def _positive(value: object, minimum: int = 1) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= minimum


# === Entry Point === #
