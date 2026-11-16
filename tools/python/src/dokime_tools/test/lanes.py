# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Test Lanes Operations."""

# === Imports === #

# Standard Library Imports
import os
import shutil
import time
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.credentials
import dokime_tools.test.scratch

from .. import temporary
from ..build import paths
from . import archive, execution, nextest, policy, pytest, report, traces, vitest

# Type Checking Imports

# === Public Exports === #

# === Constants === #

COVERAGE_VARIABLE = "COVERAGE"
SCRIPTS = ("typescript", "javascript")

# === Types === #

# === Classes === #

# === Public Functions === #


def run(
    directory: Path,
    category: str,
    language: str,
    project: str,
    profile: str,
    extra: Sequence[str],
    *,
    prepared: bool = False,
    archive_directory: Path | None = None,
) -> int:
    """Run One Lane and Retain Its Native Report; Nonzero When Any Story Failed."""
    rules = policy.load(directory)
    key = f"{category}-{language}"
    preflight(directory, category, language, profile)
    if language == "python" and not prepared:
        archive.install_wheels(directory, profile)
    scratch_directory = prepare_scratch(directory, key, profile)
    scratch = scratch_directory.path
    fixture_directory: temporary.Directory | None = None
    try:
        if language == "python":
            fixture_directory = temporary.Directory()
        fixtures = fixture_directory.path if fixture_directory is not None else scratch
        deadline = rules.deadline(category)

        # Native Command
        environment = dict(os.environ)
        cwd = directory / project
        if language == "rust":
            archive.require(directory, profile)
            extracted = archive_directory or scratch / "extract"
            if archive_directory is None:
                archive.extract(directory, profile, extracted, category=category, arguments=extra)
            binaries = archive.binaries(directory, profile, extracted)
            names = sorted(set(binaries.values()))
            expression, rest = nextest.selection(category, extra, names)
            measured = nextest.overrides(rules, category, coverage=False, binaries=binaries)
            configuration = scratch / "nextest.toml"
            store = str(scratch / "store")
            configuration.write_text(nextest.config(store, deadline, measured), encoding="utf-8")
            junit = scratch / "store" / nextest.PROFILE / "junit.xml"
            command = nextest.run(directory, configuration, extracted, expression, rest)
            cwd = directory
        elif language == "python":
            junit = scratch / "junit.xml"
            arguments = list(extra)
            if not any(argument.startswith("--basetemp") for argument in arguments):
                arguments.append(f"--basetemp={fixtures / 'fixtures'}")
            arguments += ["-o", f"cache_dir={fixtures / 'pytest-cache'}"]
            command = pytest.run(cwd, f"tests/{category}", deadline, junit, arguments)
            environment = pytest.environment(directory)
        else:
            junit = scratch / "junit.xml"
            command = vitest.run(extra, junit, deadline)

        environment["TEST_PROFILE"] = profile
        if language == "python" and category == "system":
            environment["HARMOS_CONFORMANCE_FIXTURE"] = str(paths.output(directory, "build", profile) / "fixtures/host/python/conformance")
            environment["HARMOS_CONFORMANCE_HOST"] = str(
                paths.output(directory, "build", profile) / "fixtures/host" / f"harmos-conformance{'.exe' if os.name == 'nt' else ''}"
            )
        environment["DOKIME_TEST_LANE"] = key
        environment.pop("TEST_REPORT_DIR", None)
        if category == "system":
            story_reports = report.native_directory(directory, report.TRACES / profile / paths.target() / key)
            environment["TEST_REPORT_DIR"] = str(story_reports)
        trace_report = report.native_directory(directory, Path("target/test-reports") / key)
        environment.pop("DOKIME_TEST_TRACE_DIR", None)
        if category in ("integration", "acceptance", "system"):
            trace_directory = report.native_directory(directory, trace_report.relative_to(directory) / "traces")
            shutil.rmtree(trace_directory)
            trace_directory.mkdir()
            environment["DOKIME_TEST_TRACE_DIR"] = str(trace_directory)
        try:
            lane = record(
                directory,
                rules,
                key,
                category,
                language,
                project,
                profile,
                command,
                cwd,
                environment,
                junit,
                scratch,
            )
        finally:
            traces.summarize(trace_report, junit)
        lane["arguments"] = list(extra)
        return 0 if lane["outcome"] == "passed" else 1
    finally:
        if fixture_directory is not None:
            fixture_directory.close()
        scratch_directory.close()


def preflight(
    directory: Path,
    category: str | None,
    language: str,
    profile: str,
) -> None:
    """Refuse a stale fixture stamp before any Rust test runs."""
    if language == "rust" and (category != "unit" or profile == "bench"):
        output = paths.output(directory, "build", profile).relative_to(directory)
        archive.fixtures(directory, profile, roots=[(output / part).as_posix() for part in ("tests", "fixtures")])


def prepare_scratch(directory: Path, key: str, profile: str) -> temporary.Directory:
    if not key.startswith("bench-"):
        output = report.location(directory, profile, key)
        if output.is_symlink() or not output.resolve().is_relative_to(directory.resolve()):
            raise policy.TestError("Run Lanes Tests: lane output escapes the repository")
        shutil.rmtree(output, ignore_errors=True)
    return temporary.Directory()


def record(
    directory: Path,
    rules: policy.Policy,
    key: str,
    category: str,
    language: str,
    project: str,
    profile: str,
    command: Sequence[str],
    cwd: Path,
    environment: dict[str, str] | None,
    junit: Path,
    scratch: Path,
) -> dict:
    """Execute a lane command and turn its JUnit report into a lane record."""
    started = time.time()
    destination = report.location(directory, profile, key)
    scratch_root = dokime_tools.test.scratch.Root(environment, destination)
    try:
        scratch_environment = scratch_root.environment
        credential_store = dokime_tools.test.credentials.Store(scratch_environment) if language == "rust" and category != "unit" else None
        try:
            child_environment = credential_store.environment if credential_store is not None else scratch_environment
            if language in SCRIPTS:
                status = execution.execute(command, cwd, child_environment, timeout_s=rules.deadline(category))
            else:
                status = execution.execute(command, cwd, child_environment)
        finally:
            if credential_store is not None:
                credential_store.close()
    finally:
        scratch_root.close()
    seconds = time.time() - started
    if (
        not junit.resolve().is_relative_to(scratch.resolve())
        or any(path.is_symlink() for path in (junit, *junit.parents) if path.is_relative_to(scratch))
        or destination.is_symlink()
        or not destination.resolve().is_relative_to(directory.resolve())
    ):
        raise policy.TestError(f"Run Lanes Tests: {key}: native evidence escapes its owned report paths")
    for evidence in junit.parent.rglob("*"):
        if evidence.is_symlink() or not evidence.resolve().is_relative_to(junit.parent.resolve()):
            raise policy.TestError(f"Run Lanes Tests: {key}: native evidence escapes its report directory")
        if evidence == junit:
            continue
        if not evidence.is_file():
            continue
        target = destination / evidence.relative_to(junit.parent)
        if target.is_symlink() or not target.resolve().is_relative_to(destination.resolve()):
            raise policy.TestError(f"Run Lanes Tests: {key}: native evidence destination escapes its lane")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(evidence, target)
    if not junit.is_file():
        raise policy.TestError(f"Run Lanes Tests: {key}: the runner exited {status} without a JUnit report")

    # Stories And Measurements
    text = junit.read_text(encoding="utf-8")
    package = Path(project).name if project != "." else directory.name
    found = report.cases(text, category, language, package)
    failed = sum(case.outcome == "failed" for case in found)
    passed = sum(case.outcome == "passed" for case in found)

    report.write(destination / "junit.xml", text)
    durations = {}
    for case in found:
        if case.outcome == "passed":
            durations.setdefault(case.suite, {})[case.name] = {"seconds": case.seconds}
    report.dump(destination / "durations.json", durations)
    print(f"{key}: {passed} passed, {failed} failed, {len(found) - passed - failed} skipped")
    ran = passed > 0 or failed > 0
    return {
        "category": category,
        "language": language,
        "kind": "test",
        "outcome": "passed" if status == 0 and ran and not failed else "failed",
        "project": project,
        "profile": profile,
        "tests": len(found),
        "passed": passed,
        "failed": failed,
        "skipped": sum(case.outcome == "skipped" for case in found),
        "seconds": round(seconds, 3),
        "started_at": int(started),
        "junit": (destination / "junit.xml").relative_to(directory).as_posix(),
        "coverage": None,
        "durations": [(case.suite, case.name, case.package, case.seconds) for case in found if case.outcome == "passed"],
    }


# === Private Functions === #

# === Entry Point === #
