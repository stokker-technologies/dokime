# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Test Coverage Operations."""

# === Imports === #

# Standard Library Imports
import os
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..build import paths
from ..ci.matrices import SDK_LANGUAGES, SDKS
from . import archive, execution, lanes, nextest, policy, pytest, report

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def run(
    directory: Path,
    language: str,
    category: str | None,
    project: str,
    profile: str,
    extra: Sequence[str],
) -> int:
    """Instrumented lane over one layer, or every layer when no category is given."""
    rules = policy.load(directory)
    key = f"coverage-{category}-{language}" if category else f"coverage-{language}"
    lanes.preflight(directory, category, language, profile)
    scratch_directory = lanes.prepare_scratch(directory, key, profile)
    scratch = scratch_directory.path
    try:
        categories = [category] if category else list(policy.CATEGORIES)
        deadline = max(rules.deadline(name, coverage=True) for name in categories)
        cobertura = scratch / "coverage.xml"
        environment = dict(os.environ, **{lanes.COVERAGE_VARIABLE: "1"})

        # Instrumented Run
        if language == "rust":
            binaries = archive.binaries(directory, profile)
            names = sorted(set(binaries.values()))
            # Equivalence Stories of Unshipped SDKs Need Fixtures CI No Longer Builds.
            unshipped = [f"test({sdk}_plugin_matches_rust_thermal_evidence)" for sdk in SDKS if sdk not in SDK_LANGUAGES]
            exclusion = ["-E", f"not ({' or '.join(unshipped)})"] if unshipped else []
            expression, rest = nextest.selection(category, [*extra, *exclusion], names)
            measured = nextest.overrides(rules, category, coverage=True, binaries=binaries)
            configuration = scratch / "nextest.toml"
            store = str(scratch / "store")
            configuration.write_text(nextest.config(store, deadline, measured), encoding="utf-8")
            junit = scratch / "store" / nextest.PROFILE / "junit.xml"
            command = nextest.coverage(profile, configuration, expression, rest)
            cwd = directory
        elif language == "python":
            cwd = directory / project
            junit = scratch / "junit.xml"
            suite = f"tests/{category}" if category else "tests"
            command = pytest.run(cwd, suite, deadline, junit, [*extra, *pytest.coverage(cwd, cobertura)])
            environment = {**pytest.environment(directory), lanes.COVERAGE_VARIABLE: "1"}
        else:
            raise policy.TestError("Run Coverage Tests: coverage is measured for rust and python")

        environment["TEST_PROFILE"] = profile
        if language == "python" and category in (None, "system"):
            environment["HARMOS_CONFORMANCE_FIXTURE"] = str(paths.output(directory, "build", profile) / "fixtures/host/python/conformance")
            environment["HARMOS_CONFORMANCE_HOST"] = str(
                paths.output(directory, "build", profile) / "fixtures/host" / f"harmos-conformance{'.exe' if os.name == 'nt' else ''}"
            )
        environment.pop("TEST_REPORT_DIR", None)
        lane = lanes.record(
            directory,
            rules,
            key,
            category or "coverage",
            language,
            project,
            profile,
            command,
            cwd,
            environment,
            junit,
            scratch,
        )
        if language == "rust" and execution.execute(nextest.report(profile, cobertura), directory, environment) != 0:
            lane["outcome"] = "failed"
        if not cobertura.is_file():
            raise policy.TestError(f"Run Coverage Tests: {key}: no coverage report was written")
        lane["kind"] = "coverage"
        lane["arguments"] = list(extra)
        lane["coverage"] = (report.location(directory, profile, key) / "coverage.xml").relative_to(directory).as_posix()
        report.write(
            report.location(directory, profile, key) / "coverage.xml",
            cobertura.read_text(encoding="utf-8"),
        )
        return 0 if lane["outcome"] == "passed" else 1
    finally:
        scratch_directory.close()


# === Private Functions === #

# === Entry Point === #
