# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Desktop Development Commands For The Native Owner"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, outputs
from dokime_tools.build.frontend import ROOT, WEB_ROOT
from dokime_tools.prepare.resources import refresh

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def merge_config(base: object, override: object) -> object:
    if not isinstance(base, dict) or not isinstance(override, dict):
        return override
    result = dict(base)
    for key, value in override.items():
        result[key] = merge_config(base[key], value) if key in base else value
    return result


def dev_plan(environment: dict[str, str], args: list[str]) -> tuple[list[str], dict[str, str], dict]:
    host = environment.get("DOKIME_DEV_HOST", "127.0.0.1")
    port = environment.get("DOKIME_DEV_PORT", "5173")
    url = f"http://{host}:{port}"
    brand_path = ROOT / "organization/generated/tauri.organization.json"
    brand = json.loads(brand_path.read_text(encoding="utf-8")) if brand_path.exists() else {}
    overlay = {"build": {"devUrl": url}, "bundle": {"externalBin": []}}
    config = merge_config(
        json.loads(environment.get("TAURI_CONFIG") or "{}"),
        merge_config(brand, overlay),
    )
    target = environment.get("DOKIME_DEV_TARGET_DIR") or (
        environment.get("CARGO_TARGET_DIR") if port == "5173" else f"target/tauri-dev-{port}"
    )
    child_env = dict(environment)
    child_env.pop("TAURI_CONFIG", None)
    child_env.update(
        DOKIME_DEV_HOST=host,
        DOKIME_DEV_PORT=port,
        DOKIME_APPLICATION_ROOT=environment.get("DOKIME_APPLICATION_ROOT", str(ROOT / ".run/dokime-dev")),
    )
    if target:
        child_env["CARGO_TARGET_DIR"] = target
    command = [
        "node",
        "./node_modules/@tauri-apps/cli/tauri.js",
        "dev",
        "--config",
        json.dumps(config, separators=(",", ":")),
        "--features",
        "console",
        *args,
    ]
    description = {
        "devUrl": url,
        "cargoTargetDir": target,
        "tauriArgs": command[1:],
        "tauriConfig": config,
    }
    return command, child_env, description


def launch(
    commands: list[tuple[list[str], dict[str, str]]],
    *,
    cwd: Path,
    environment: dict[str, str],
    messages: Path,
) -> None:
    if not messages.is_file():
        raise ValueError(f"missing native supervisor compiler record: {messages}; run the existing Rust build task first")
    finished = [
        event["success"]
        for line in messages.read_text(encoding="utf-8").splitlines()
        if line.startswith("{") and (event := json.loads(line)).get("reason") == "build-finished"
    ]
    if finished != [True]:
        raise ValueError("native supervisor requires a successful selected Cargo build")
    executable = outputs.executables(messages, ["cargo-dokime"])["cargo-dokime"]
    if not executable.is_file():
        raise ValueError(f"selected native supervisor is missing: {executable}; run the existing Rust build task first")
    prepared = [
        {
            "arguments": command,
            "environment": {key: value for key, value in selected.items() if environment.get(key) != value},
            "remove_environment": [key for key in environment if key not in selected],
        }
        for command, selected in commands
    ]
    plan = json.dumps({"cwd": str(cwd), "commands": prepared}, separators=(",", ":"))
    os.execve(executable, [str(executable), "__desktop-development", "--plan", plan], environment)


def main() -> int:
    args = sys.argv[1:]
    report_only = bool(args and args[0] == "watch-report")
    if report_only:
        args = args[1:]
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--build-json", type=Path, default=configuration.messages("dev"))
    options, args = parser.parse_known_args(args)
    messages = options.build_json if options.build_json.is_absolute() else ROOT / options.build_json
    original = dict(os.environ)
    if report_only:
        launch(
            [
                (
                    ["node", "./node_modules/vite/bin/vite.js", "build", "--config", "vite.report.config.ts", "--watch"],
                    original,
                )
            ],
            cwd=WEB_ROOT,
            environment=original,
            messages=messages,
        )
        return 0
    print_config = "--print-config" in args
    args = [arg for arg in args if arg != "--print-config"]
    command, environment, description = dev_plan(original, args)
    if print_config:
        print(json.dumps(description, indent=2))
        return 0
    refresh()
    commands = []
    if environment.get("DOKIME_DEV_REPORT_WATCH") != "0":
        commands.append(
            (
                ["node", "./node_modules/vite/bin/vite.js", "build", "--config", "vite.report.config.ts", "--watch"],
                original,
            )
        )
    commands.append((command, environment))
    launch(commands, cwd=WEB_ROOT, environment=environment, messages=messages)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
