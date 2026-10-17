# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Supervise Owned Desktop Development Processes"""

# === Imports === #

# Standard Library Imports

import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build.frontend import ROOT as ROOT
from dokime_tools.build.frontend import WEB_ROOT as WEB_ROOT
from dokime_tools.build.frontend import ensure_report as ensure_report

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


def dev_plan(
    environment: dict[str, str], args: list[str]
) -> tuple[list[str], dict[str, str], dict]:
    host = environment.get("DOKIME_DEV_HOST", "127.0.0.1")
    port = environment.get("DOKIME_DEV_PORT", "5173")
    url = f"http://{host}:{port}"
    brand_path = ROOT / "branding/generated/tauri.brand.json"
    brand = (
        json.loads(brand_path.read_text(encoding="utf-8"))
        if brand_path.exists()
        else {}
    )
    overlay = {"build": {"devUrl": url}, "bundle": {"externalBin": []}}
    config = merge_config(
        json.loads(environment.get("TAURI_CONFIG") or "{}"),
        merge_config(brand, overlay),
    )
    target = environment.get("DOKIME_DEV_TARGET_DIR") or (
        environment.get("CARGO_TARGET_DIR")
        if port == "5173"
        else f"target/tauri-dev-{port}"
    )
    child_env = dict(environment)
    child_env.pop("TAURI_CONFIG", None)
    child_env.update(
        DOKIME_DEV_HOST=host,
        DOKIME_DEV_PORT=port,
        DOKIME_APPLICATION_ROOT=environment.get(
            "DOKIME_APPLICATION_ROOT", str(ROOT / ".run/dokime-dev")
        ),
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


def supervise(
    commands: list[list[str]], *, cwd: Path, environment: dict[str, str]
) -> int:
    children: list[subprocess.Popen] = []
    stopped = 0

    def stop(signum: int, _frame: object) -> None:
        nonlocal stopped
        stopped = signum

    signals = [signal.SIGINT, signal.SIGTERM]
    if hasattr(signal, "SIGHUP"):
        signals.append(signal.SIGHUP)
    previous = {signum: signal.signal(signum, stop) for signum in signals}
    try:
        for command in commands:
            children.append(
                subprocess.Popen(
                    command, cwd=cwd, env=environment, start_new_session=os.name != "nt"
                )
            )
        while not stopped:
            for index, child in enumerate(children):
                code = child.poll()
                if code is not None:
                    return code if index == len(children) - 1 else code or 1
            time.sleep(0.02)
        return 128 + stopped
    finally:
        # Owned Process Groups Survive Their Leader
        for child in children:
            if os.name == "nt":
                subprocess.run(
                    ["taskkill.exe", "/pid", str(child.pid), "/t", "/f"],
                    check=False,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            else:
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
        for child in children:
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                if os.name != "nt":
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    child.kill()
                child.wait()
        if os.name != "nt":
            # A reaped leader does not imply that its owned process group has
            # exited. Give descendants time to handle TERM, then bound cleanup
            # even when a descendant ignores it.
            time.sleep(0.1)
            for child in children:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        for signum, handler in previous.items():
            signal.signal(signum, handler)


def main() -> int:
    args = sys.argv[1:]
    if args and args[0] == "watch-report":
        return supervise(
            [
                [
                    "node",
                    "./node_modules/vite/bin/vite.js",
                    "build",
                    "--config",
                    "vite.report.config.ts",
                    "--watch",
                ]
            ],
            cwd=WEB_ROOT,
            environment=dict(os.environ),
        )
    print_config = "--print-config" in args
    args = [arg for arg in args if arg != "--print-config"]
    command, environment, description = dev_plan(dict(os.environ), args)
    if print_config:
        print(json.dumps(description, indent=2))
        return 0
    ensure_report()
    commands = []
    if environment.get("DOKIME_DEV_REPORT_WATCH") != "0":
        commands.append(
            [
                "node",
                "./node_modules/vite/bin/vite.js",
                "build",
                "--config",
                "vite.report.config.ts",
                "--watch",
            ]
        )
    commands.append(command)
    return supervise(commands, cwd=WEB_ROOT, environment=environment)


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
