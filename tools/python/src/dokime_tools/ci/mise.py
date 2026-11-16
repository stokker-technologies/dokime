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

# isort: split

# Third Party Imports

# isort: split

# Local Imports

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
        runs = []
        for runs_name in ("run", "run_windows"):
            if runs_name not in table:
                continue
            runs.append((runs_name, table[runs_name]))

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
        raise TypeError("Check Task Configuration: task_config must be a table")
    includes = configuration.get("includes", [])
    if not isinstance(includes, list):
        raise TypeError("Check Task Configuration: task_config.includes must be an array of paths")
    for relative in includes:
        if not isinstance(relative, str) or not relative.strip():
            raise ValueError("Check Task Configuration: task_config.includes must contain paths")
        path = (root / relative).resolve()
        if not path.is_relative_to(root):
            raise ValueError("Check Task Configuration: included Mise tasks must stay inside the repository")
        if not path.is_file():
            raise ValueError("Check Task Configuration: task_config.includes must name TOML files")
        tasks.update(tomllib.loads(path.read_text(encoding="utf-8")))
    project = document.get("tasks", {})
    if not isinstance(project, dict):
        raise TypeError("Check Task Configuration: tasks must be a table")
    tasks.update(project)
    findings = []
    for findings_task, findings_definition in tasks.items():
        for findings_finding in forms(findings_definition):
            findings.append(f"mise.toml: task {findings_task}: {findings_finding}")

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
