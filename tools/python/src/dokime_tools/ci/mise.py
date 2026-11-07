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
from .simulate import PLAN

# Type Checking Imports


# === Public Exports === #

# === Constants === #

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
