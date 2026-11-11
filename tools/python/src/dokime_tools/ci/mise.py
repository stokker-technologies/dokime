# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check Supported Mise Task Forms Without Evaluating Authored Commands."""

# === Imports === #

# Standard Library Imports
import sys
import tomllib
from pathlib import Path

# Third Party Imports
# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# Hosted Task And Artifact Dependencies Checked By Check / Repository.
PLAN = {
    "jobs": [
        {"key": "source", "id": "check:repository", "platform": "unix", "needs": [], "inputs": [], "outputs": {}},
        {"key": "check_python", "id": "check:python", "platform": "unix", "needs": [], "inputs": [], "outputs": {}},
        {"key": "check_typescript", "id": "check:typescript", "platform": "unix", "needs": [], "inputs": [], "outputs": {}},
        {
            "key": "package_rust_windows",
            "id": "package:rust",
            "platform": "windows",
            "needs": [
                "build_product_unix",
                "build_product_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
            ],
            "inputs": ["build#cli", "build#generator", "build#gui", "build#packaging", "build#plugins"],
            "outputs": {"package": ["target/package/{profile}/{target}/rust"]},
        },
        {"key": "check_rust_windows", "id": "check:rust", "platform": "windows", "needs": [], "inputs": [], "outputs": {}},
        {
            "key": "build_product_windows",
            "id": "build",
            "platform": "windows",
            "needs": [],
            "inputs": [],
            "outputs": {
                "cli": ["target/build/{profile}/{target}/bin/dokime.exe"],
                "clients": ["target/build/{profile}/{target}/generated/typescript"],
                "generator": ["target/build/{profile}/{target}/bin/cargo-dokime.exe"],
                "gui": ["target/build/{profile}/{target}/bin/dokime-gui.exe"],
                "packaging": ["target/build/{profile}/{target}/product", "target/build/{profile}/{target}/web"],
                "plugins": ["target/build/{profile}/{target}/plugins"],
            },
        },
        {
            "key": "check_generated_python",
            "id": "check:python:bindings",
            "platform": "unix",
            "needs": ["build_python_unix"],
            "inputs": ["build:python#python"],
            "outputs": {},
        },
        {
            "key": "check_generated_typescript",
            "id": "check:typescript:generated",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows"],
            "inputs": ["build#clients"],
            "outputs": {},
        },
        {
            "key": "validate_test_python_unit",
            "id": "test:python:unit",
            "platform": "unix",
            "needs": ["build_python_unix"],
            "inputs": ["build:python#python"],
            "outputs": {},
        },
        {
            "key": "build_tests_windows",
            "id": "build:rust:tests",
            "platform": "windows",
            "needs": [],
            "inputs": [],
            "outputs": {"tests": ["target/build/{profile}/{target}/tests", "target/build/{profile}/{target}/fixtures"]},
        },
        {
            "key": "validate_test_typescript_unit",
            "id": "test:typescript:unit",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows"],
            "inputs": ["build#clients"],
            "outputs": {},
        },
        {
            "key": "validate_test_python_integration",
            "id": "test:python:integration",
            "platform": "unix",
            "needs": ["build_python_unix"],
            "inputs": ["build:python#python"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_unit_windows",
            "id": "test:rust:unit",
            "platform": "windows",
            "needs": ["build_tests_unix", "build_tests_windows"],
            "inputs": ["build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_typescript_integration",
            "id": "test:typescript:integration",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows"],
            "inputs": ["build#clients"],
            "outputs": {},
        },
        {
            "key": "validate_test_python_acceptance",
            "id": "test:python:acceptance",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_python_unix", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_integration_windows",
            "id": "test:rust:integration",
            "platform": "windows",
            "needs": ["build_tests_unix", "build_tests_windows"],
            "inputs": ["build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_acceptance_windows",
            "id": "test:rust:acceptance",
            "platform": "windows",
            "needs": ["build_product_unix", "build_product_windows", "build_python_windows", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#clients", "build#generator", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_system_windows",
            "id": "test:rust:system",
            "platform": "windows",
            "needs": ["build_product_unix", "build_product_windows", "build_python_windows", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_typescript_system_windows",
            "id": "test:typescript:system",
            "platform": "windows",
            "needs": ["build_product_unix", "build_product_windows", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#clients", "build#gui", "build#plugins", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "package_python_unix",
            "id": "package:python",
            "platform": "unix",
            "needs": [
                "build_python_unix",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
            ],
            "inputs": ["build:python#python"],
            "outputs": {"package": ["target/package/{profile}/{target}/python"]},
        },
        {
            "key": "validate_test_rust_starter_windows",
            "id": "test:rust:starter",
            "platform": "windows",
            "needs": ["build_product_unix", "build_product_windows"],
            "inputs": ["build#generator"],
            "outputs": {},
        },
        {
            "key": "validate_coverage_python",
            "id": "test:python:coverage",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_python_unix", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_coverage_rust",
            "id": "test:rust:coverage",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_python_unix", "build_tests_unix", "build_tests_windows"],
            "inputs": [
                "build#cli",
                "build#clients",
                "build#generator",
                "build#gui",
                "build#plugins",
                "build:python#python",
                "build:rust:tests#tests",
            ],
            "outputs": {},
        },
        {
            "key": "build_bench",
            "id": "build:rust:bench",
            "platform": "unix",
            "needs": [],
            "inputs": [],
            "outputs": {"bench": ["target/build/bench/{target}"]},
        },
        {
            "key": "bench_rust_integration",
            "id": "bench:rust:integration",
            "platform": "unix",
            "needs": [
                "build_bench",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
                "validate_coverage_python",
                "validate_coverage_rust",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_starter_unix",
                "validate_test_rust_starter_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
            ],
            "inputs": ["build:rust:bench#bench"],
            "outputs": {},
        },
        {
            "key": "bench_rust_system",
            "id": "bench:rust:system",
            "platform": "unix",
            "needs": [
                "build_bench",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
                "validate_coverage_python",
                "validate_coverage_rust",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_starter_unix",
                "validate_test_rust_starter_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
            ],
            "inputs": ["build:rust:bench#bench"],
            "outputs": {},
        },
        {
            "key": "bench_rust_unit",
            "id": "bench:rust:unit",
            "platform": "unix",
            "needs": [
                "build_bench",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
                "validate_coverage_python",
                "validate_coverage_rust",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_starter_unix",
                "validate_test_rust_starter_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
            ],
            "inputs": ["build:rust:bench#bench"],
            "outputs": {},
        },
        {"key": "check_rust_unix", "id": "check:rust", "platform": "unix", "needs": [], "inputs": [], "outputs": {}},
        {
            "key": "build_product_unix",
            "id": "build",
            "platform": "unix",
            "needs": [],
            "inputs": [],
            "outputs": {
                "cli": ["target/build/{profile}/{target}/bin/dokime"],
                "clients": ["target/build/{profile}/{target}/generated/typescript"],
                "generator": ["target/build/{profile}/{target}/bin/cargo-dokime"],
                "gui": ["target/build/{profile}/{target}/bin/dokime-gui"],
                "packaging": ["target/build/{profile}/{target}/product", "target/build/{profile}/{target}/web"],
                "plugins": ["target/build/{profile}/{target}/plugins"],
            },
        },
        {
            "key": "build_tests_unix",
            "id": "build:rust:tests",
            "platform": "unix",
            "needs": [],
            "inputs": [],
            "outputs": {"tests": ["target/build/{profile}/{target}/tests", "target/build/{profile}/{target}/fixtures"]},
        },
        {
            "key": "validate_test_rust_unit_unix",
            "id": "test:rust:unit",
            "platform": "unix",
            "needs": ["build_tests_unix", "build_tests_windows"],
            "inputs": ["build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_integration_unix",
            "id": "test:rust:integration",
            "platform": "unix",
            "needs": ["build_tests_unix", "build_tests_windows"],
            "inputs": ["build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_acceptance_unix",
            "id": "test:rust:acceptance",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_python_unix", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#clients", "build#generator", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_system_unix",
            "id": "test:rust:system",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_python_unix", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#plugins", "build:python#python", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_typescript_system_unix",
            "id": "test:typescript:system",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows", "build_tests_unix", "build_tests_windows"],
            "inputs": ["build#cli", "build#clients", "build#gui", "build#plugins", "build:rust:tests#tests"],
            "outputs": {},
        },
        {
            "key": "validate_test_rust_starter_unix",
            "id": "test:rust:starter",
            "platform": "unix",
            "needs": ["build_product_unix", "build_product_windows"],
            "inputs": ["build#generator"],
            "outputs": {},
        },
        {
            "key": "package_rust_unix",
            "id": "package:rust",
            "platform": "unix",
            "needs": [
                "build_product_unix",
                "build_product_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "source",
            ],
            "inputs": ["build#cli", "build#generator", "build#gui", "build#packaging", "build#plugins"],
            "outputs": {"package": ["target/package/{profile}/{target}/rust"]},
        },
        {
            "key": "release_rehearsal",
            "id": "release:rehearse",
            "platform": "unix",
            "needs": [
                "build_product_unix",
                "build_product_windows",
                "build_python_unix",
                "build_tests_unix",
                "build_tests_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "package_rust_unix",
                "package_rust_windows",
                "source",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
                "package_python_unix",
                "package_python_windows",
            ],
            "inputs": ["package:python#package", "package:rust#package"],
            "outputs": {},
        },
        {
            "key": "release_dry",
            "id": "release:publish:dry",
            "platform": "unix",
            "needs": [
                "build_product_unix",
                "build_product_windows",
                "build_python_unix",
                "build_tests_unix",
                "build_tests_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "package_rust_unix",
                "package_rust_windows",
                "source",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
                "package_python_unix",
                "package_python_windows",
            ],
            "inputs": ["package:python#package", "package:rust#package"],
            "outputs": {},
        },
        {
            "key": "release_dry_nightly",
            "id": "release:publish:dry",
            "platform": "unix",
            "needs": [
                "build_product_unix",
                "build_product_windows",
                "build_python_unix",
                "build_tests_unix",
                "build_tests_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_unix",
                "check_rust_windows",
                "check_typescript",
                "package_rust_unix",
                "package_rust_windows",
                "source",
                "validate_test_python_acceptance",
                "validate_test_python_integration",
                "validate_test_python_unit",
                "validate_test_rust_acceptance_unix",
                "validate_test_rust_acceptance_windows",
                "validate_test_rust_integration_unix",
                "validate_test_rust_integration_windows",
                "validate_test_rust_system_unix",
                "validate_test_rust_system_windows",
                "validate_test_rust_unit_unix",
                "validate_test_rust_unit_windows",
                "validate_test_typescript_integration",
                "validate_test_typescript_system_unix",
                "validate_test_typescript_system_windows",
                "validate_test_typescript_unit",
                "package_python_unix",
                "package_python_windows",
            ],
            "inputs": ["package:python#package", "package:rust#package"],
            "outputs": {},
        },
        {
            "key": "build_python_windows",
            "id": "build:python",
            "platform": "windows",
            "needs": ["build_product_windows"],
            "inputs": ["build#cli", "build#packaging"],
            "outputs": {"python": ["target/build/{profile}/{target}/python"]},
        },
        {
            "key": "build_python_unix",
            "id": "build:python",
            "platform": "unix",
            "needs": ["build_product_unix"],
            "inputs": ["build#cli", "build#packaging"],
            "outputs": {"python": ["target/build/{profile}/{target}/python"]},
        },
        {
            "key": "package_python_windows",
            "id": "package:python",
            "platform": "windows",
            "needs": [
                "build_python_windows",
                "check_generated_python",
                "check_generated_typescript",
                "check_python",
                "check_rust_windows",
                "check_rust_windows",
                "check_typescript",
                "source",
            ],
            "inputs": ["build:python#python"],
            "outputs": {"package": ["target/package/{profile}/{target}/python"]},
        },
        {
            "key": "validate_test_python_system_unix",
            "id": "test:python:system",
            "platform": "unix",
            "needs": ["build_python_unix", "build_product_unix"],
            "inputs": ["build:python#python", "build#cli"],
            "outputs": {},
        },
        {
            "key": "validate_test_python_system_windows",
            "id": "test:python:system",
            "platform": "windows",
            "needs": ["build_python_windows", "build_product_windows"],
            "inputs": ["build:python#python", "build#cli"],
            "outputs": {},
        },
    ]
}

# === Types === #

# === Classes === #

# === Public Functions === #


def forms(definition: object) -> list[str]:
    table = definition if isinstance(definition, dict) else None
    if table is None:
        runs = [("run", definition)]
    else:
        runs = [(name, table[name]) for name in ("run", "run_windows") if name in table]
    findings = []
    commands = []
    for name, run in runs:
        if isinstance(run, str):
            steps = [run]
        elif isinstance(run, list):
            steps = run
        else:
            findings.append(f"{name} must be a command string or an array of commands/task references")
            continue
        for step in steps:
            if isinstance(step, str):
                commands.append(step)
                if not step.strip():
                    findings.append(f"{name} commands must not be empty")
                continue
            reference = step if isinstance(step, dict) else {}
            task = reference.get("task")
            tasks = reference.get("tasks")
            single = isinstance(task, str) and bool(task.strip()) and "tasks" not in reference
            parallel = (
                isinstance(tasks, list)
                and bool(tasks)
                and all(isinstance(task, str) and bool(task.strip()) for task in tasks)
                and "task" not in reference
            )
            if not single and not parallel:
                findings.append(f"{name} steps must be command strings or references with a nonempty task name")
    if table is not None and "shell" in table:
        shell = table["shell"]
        if isinstance(shell, str):
            valid = bool(shell.strip())
        else:
            valid = (
                isinstance(shell, list) and bool(shell) and all(isinstance(argument, str) and bool(argument.strip()) for argument in shell)
            )
        if not valid:
            findings.append("shell must be a nonempty shell command or an array of nonempty shell arguments")
    for command in commands:
        if command.count("\n") + int(not command.endswith("\n")) > 10:
            findings.append("move scripts over 10 lines into the tooling package")
    return findings


def check(root: Path) -> list[str]:
    root = root.resolve()
    document = tomllib.loads((root / "mise.toml").read_text(encoding="utf-8"))
    tasks = {}
    configuration = document.get("task_config", {})
    if not isinstance(configuration, dict):
        raise TypeError("task_config must be a table")
    includes = configuration.get("includes", [])
    if not isinstance(includes, list):
        raise TypeError("task_config.includes must be an array of paths")
    for relative in includes:
        if not isinstance(relative, str) or not relative.strip():
            raise ValueError("task_config.includes must contain paths")
        path = (root / relative).resolve()
        if not path.is_relative_to(root):
            raise ValueError("included Mise tasks must stay inside the repository")
        if not path.is_file():
            raise ValueError("task_config.includes must name TOML files")
        tasks.update(tomllib.loads(path.read_text(encoding="utf-8")))
    project = document.get("tasks", {})
    if not isinstance(project, dict):
        raise TypeError("tasks must be a table")
    tasks.update(project)
    findings = [f"mise.toml: task {task}: {finding}" for task, definition in tasks.items() for finding in forms(definition)]
    findings.extend(graph(PLAN))
    return findings


def graph(plan: dict) -> list[str]:
    """Refuse Dangling Dependencies, Cycles And Missing Native Artifact Producers."""
    jobs = {job["key"]: job for job in plan["jobs"]}
    findings = []
    if len(jobs) != len(plan["jobs"]):
        findings.append("CI job keys must be unique")
    visited = set()
    visiting = set()

    def visit(key: str) -> None:
        if key in visiting:
            findings.append(f"CI dependency cycle at {key}")
            return
        if key in visited:
            return
        visiting.add(key)
        for need in jobs[key]["needs"]:
            if need not in jobs:
                findings.append(f"CI {key}: missing dependency {need}")
                continue
            visit(need)
        visiting.remove(key)
        visited.add(key)

    for key, job in jobs.items():
        visit(key)
        native = job.get("execution_platform", "unix" if job["platform"] == "any" else job["platform"])
        for reference in job["inputs"]:
            producer, output = reference.split("#", 1)
            sources = [
                jobs[need]
                for need in job["needs"]
                if need in jobs
                and jobs[need]["id"] == producer
                and output in jobs[need]["outputs"]
                and (jobs[need]["platform"] in ("any", native) or job["id"].startswith("release:"))
            ]
            if not sources:
                findings.append(f"CI {key}: missing native producer for {reference}")
    return findings


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        directory = next(path for path in (Path.cwd(), *Path.cwd().parents) if (path / "mise.toml").is_file())
        errors = check(directory)
    except (OSError, TypeError, ValueError, StopIteration) as error:
        print(f"Mise task check failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(bool(errors))
