# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Start the Desktop with Live Reload on a Disposable Root and Credential Namespace."""

# === Imports === #

# Standard Library Imports
import argparse
import ctypes
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid
from collections.abc import Mapping, Sequence
from ctypes import wintypes
from pathlib import Path
from typing import Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.build import cargo, configuration
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT
GUI = ROOT / "crates/dokime-gui"
# Inherited Variables That Would Point the Desktop at Real Data or Another Host.
REDIRECTS = ("DOKIME_OUTPUT_ROOT", "DOKIME_REMOTE")
# Windows Process Creation Flag; Started Processes Run Only Once They Belong to the Job.
CREATE_SUSPENDED = 0x00000004
# How Often Stop Checks That Every Owned Process Has Exited.
EXIT_POLL_S = 0.05

# === Types === #

# === Classes === #


class _BasicLimits(ctypes.Structure):
    _fields_ = [
        ("process_time", ctypes.c_int64),
        ("job_time", ctypes.c_int64),
        ("flags", wintypes.DWORD),
        ("minimum_working_set", ctypes.c_size_t),
        ("maximum_working_set", ctypes.c_size_t),
        ("active_processes", wintypes.DWORD),
        ("affinity", ctypes.c_size_t),
        ("priority", wintypes.DWORD),
        ("scheduling", wintypes.DWORD),
    ]


class _Limits(ctypes.Structure):
    _fields_ = [
        ("basic", _BasicLimits),
        ("io", ctypes.c_uint64 * 6),
        ("process_memory", ctypes.c_size_t),
        ("job_memory", ctypes.c_size_t),
        ("peak_process_memory", ctypes.c_size_t),
        ("peak_job_memory", ctypes.c_size_t),
    ]


class _Accounting(ctypes.Structure):
    _fields_ = [
        ("user_time", ctypes.c_int64),
        ("kernel_time", ctypes.c_int64),
        ("period_user_time", ctypes.c_int64),
        ("period_kernel_time", ctypes.c_int64),
        ("page_faults", wintypes.DWORD),
        ("total_processes", wintypes.DWORD),
        ("active_processes", wintypes.DWORD),
        ("terminated_processes", wintypes.DWORD),
    ]


class Owner:
    """Own Every Started Process Tree: a Kill-on-Close Job on Windows, One Session per Process Elsewhere."""

    def __init__(self) -> None:
        self.processes: list[subprocess.Popen] = []
        self.job = None
        if os.name != "nt":
            return
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.kernel.CreateJobObjectW.restype = wintypes.HANDLE
        self.kernel.OpenProcess.restype = wintypes.HANDLE
        self.job = self.kernel.CreateJobObjectW(None, None)
        if not self.job:
            raise OSError(ctypes.get_last_error(), "Own the Desktop Processes: Create Job")
        # Closing the Job, Including When This Tool Is Killed, Kills Every Process Still in It.
        limits = _Limits()
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not self.kernel.SetInformationJobObject(wintypes.HANDLE(self.job), 9, ctypes.byref(limits), ctypes.sizeof(limits)):
            raise OSError(ctypes.get_last_error(), "Own the Desktop Processes: Configure Job")

    def start(self, command: Sequence[str | Path], **options: Any) -> subprocess.Popen:
        """Start a Process Whose Descendants Stay Owned from Their First Instruction."""
        if os.name != "nt":
            process = subprocess.Popen([str(part) for part in command], start_new_session=True, **options)
            self.processes.append(process)
            return process

        # Join the Job While Suspended, So No Child Escapes Before Assignment.
        process = subprocess.Popen([str(part) for part in command], creationflags=spawn_flags(CREATE_SUSPENDED), **options)
        self.processes.append(process)
        handle = self.kernel.OpenProcess(0x0001 | 0x0100 | 0x0800, False, process.pid)  # Terminate, Set Quota, Resume
        if not handle:
            process.kill()
            raise OSError(ctypes.get_last_error(), "Own the Desktop Processes: Open Started Process")
        try:
            if not self.kernel.AssignProcessToJobObject(wintypes.HANDLE(self.job), wintypes.HANDLE(handle)):
                process.kill()
                raise OSError(ctypes.get_last_error(), "Own the Desktop Processes: Assign Started Process")
            ctypes.WinDLL("ntdll").NtResumeProcess(wintypes.HANDLE(handle))
        finally:
            self.kernel.CloseHandle(wintypes.HANDLE(handle))
        return process

    def stop(self) -> None:
        """Kill Every Owned Process and Return Once All Have Exited."""
        if os.name == "nt":
            self.kernel.TerminateJobObject(wintypes.HANDLE(self.job), 1)
            accounting = _Accounting()
            while True:
                queried = self.kernel.QueryInformationJobObject(
                    wintypes.HANDLE(self.job), 1, ctypes.byref(accounting), ctypes.sizeof(accounting), None
                )
                if not queried:
                    raise OSError(ctypes.get_last_error(), "Stop the Desktop Processes: Read Job")
                if accounting.active_processes == 0:
                    break
                time.sleep(EXIT_POLL_S)
            for process in self.processes:
                process.wait()
            return

        for process in self.processes:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            while True:
                try:
                    os.killpg(process.pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(EXIT_POLL_S)


# === Public Functions === #


def isolated(source: Mapping[str, str], root: Path, namespace: str) -> dict[str, str]:
    """The Environment That Confines the Desktop, Its Host and Tools to One Root and Namespace."""
    environment = {name: value for name, value in source.items() if name not in REDIRECTS}
    environment["DOKIME_APPLICATION_ROOT"] = str(root)
    environment["DOKIME_CREDENTIAL_NAMESPACE"] = namespace
    return environment


def discard(root: Path, namespace: str, host: Path, environment: Mapping[str, str]) -> None:
    """Remove Every Credential Each Root Under This Folder Holds, Then the Folder."""
    failed = []
    for marker in sorted(root.rglob("backend-id")):
        removed = subprocess.run(
            [str(host), "--credential-namespace", namespace, "--storage", str(marker.parent), "--remove-credentials"],
            env=dict(environment),
            check=False,
            creationflags=spawn_flags(),
            stdin=subprocess.DEVNULL,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        if removed.returncode != 0:
            failed.append(str(marker.parent))
    if failed:
        # Deleting the Folder Now Would Orphan Its Credentials; Keep It for a Manual Removal.
        raise policy.ToolError(
            f"Remove Credentials of {', '.join(failed)} in Namespace {namespace}; Kept {root}. Rerun "
            f"{host.name} --credential-namespace {namespace} --storage <root> --remove-credentials, Then Delete It"
        )
    shutil.rmtree(root)


def plugin_paths(arguments: list[str] | None = None) -> list[Path]:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plugin", action="append", default=[], type=Path, help="Install a Plugin Bundle or Plugin Folder")
    args = parser.parse_args(arguments)
    plugins = [plugin.resolve() for plugin in args.plugin]
    for plugin in plugins:
        if not plugin.is_file() and not (plugin.is_dir() and (plugin / "dokime.toml").is_file()):
            parser.error(f"Install Development Plugin {plugin}: Supply a Bundle File or a Plugin Folder With dokime.toml")
    return plugins


def command(environment: Mapping[str, str]) -> list[str]:
    """Let Tauri Watch Report Sources and Regenerate Their Template Before Each Rust Run."""
    build: dict[str, object] = {
        "additionalWatchFolders": [str(ROOT / "crates/dokime-gui/src")],
        # Tauri Prepends `run`; Uv Consumes It Before Invoking the Development Runner.
        "runner": {
            "cmd": "uv",
            "args": ["--no-sync", "--project", str(ROOT / "tools/python"), "python", "-m", "dokime_tools.dev.runtime"],
        },
    }
    if port := environment.get("DOKIME_DEV_PORT"):
        build["devUrl"] = f"http://127.0.0.1:{int(port)}"
    return ["node", "node_modules/@tauri-apps/cli/tauri.js", "dev", "--features", "console", "--config", json.dumps({"build": build})]


def main(arguments: list[str] | None = None) -> int:
    plugins = plugin_paths(arguments)
    extension = ".exe" if os.name == "nt" else ""

    # Generate the Desktop Messages and Settings the Frontend Imports
    subprocess.run(
        ["mise", "run", "generate:typescript"],
        cwd=ROOT,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )

    # Build the Shipped Plugins and the Tools That Install Them and Remove Credentials
    shipped = [plugin["package"] for plugin in policy.policy(ROOT, "release").get("plugins", [])]
    subprocess.run(
        [
            "mise",
            "run",
            "build",
            "--",
            "--cargo",
            "build",
            "--locked",
            *[argument for package in shipped for argument in ("--package", package, "--bin", package)],
            "--package",
            "dokime-cli",
            "--bin",
            "dokime",
            "--package",
            "dokime-runtime",
            "--bin",
            "dokime-host",
        ],
        cwd=ROOT,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    cli = ROOT / "target/debug" / f"dokime{extension}"
    host = ROOT / "target/debug" / f"dokime-host{extension}"

    # Create the Disposable Root and Credential Namespace; Never the User's Own
    root = Path(tempfile.mkdtemp(prefix="dokime-dev-"))
    namespace = f"dokime-dev-{uuid.uuid4().hex}"
    environment = isolated(os.environ, root, namespace)
    data = root / "data"
    print(f"[dev] Application Root: {root}", flush=True)
    print(f"[dev] Credential Namespace: {namespace}", flush=True)

    # Turn Termination Requests Into an Interrupt So Cleanup Always Runs
    def interrupt(_number: int, _frame: object) -> None:
        raise KeyboardInterrupt

    for name in ("SIGTERM", "SIGHUP", "SIGBREAK"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), interrupt)

    owner = Owner()
    status = 1
    try:
        # Bundle Each Shipped Plugin Exactly as the Product Build Packages It
        bundles = []
        for package in shipped:
            plugin = ROOT / "target/debug" / f"{package}{extension}"
            project = root / package
            project.mkdir()
            shutil.copy2(plugin, project / plugin.name)
            (project / "dokime.toml").write_text(f'[plugin]\n[scripts]\nexecutable = "{package}"\n', encoding="utf-8")
            bundle = root / f"{package}.dokime"
            subprocess.run(
                [str(cli), "plugin", "bundle", str(project), "--out", str(bundle)],
                cwd=ROOT,
                env=environment,
                check=True,
                creationflags=spawn_flags(),
                stdin=subprocess.DEVNULL,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            bundles.append(bundle)

        # Bundle Requested Folders and Append Requested Bundles After the Shipped Plugins.
        for index, plugin in enumerate(plugins):
            if plugin.is_dir():
                bundle = root / f"requested-{index}.dokime"
                subprocess.run(
                    [str(cli), "plugin", "bundle", str(plugin), "--out", str(bundle)],
                    cwd=ROOT,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=subprocess.DEVNULL,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                bundles.append(bundle)
                continue
            bundles.append(plugin)

        # Install Them Into the Desktop's Data Folder Through a Short-Lived Host; the Desktop Host Starts Them Again
        server = owner.start(
            [cli, "--credential-namespace", namespace, "server", "--endpoint", "127.0.0.1:0", "--storage", data],
            cwd=root,
            env=environment,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=sys.stderr,
            text=True,
        )
        assert server.stdin is not None and server.stdout is not None
        endpoint = server.stdout.readline().strip()
        if not endpoint:
            raise policy.ToolError(f"Install the Shipped Plugins: the Seeding Host Exited With {server.wait()}")
        for bundle in bundles:
            subprocess.run(
                [str(cli), "--credential-namespace", namespace, "--endpoint", endpoint, "plugin", "install", str(bundle)],
                cwd=root,
                env=environment,
                check=True,
                creationflags=spawn_flags(),
                stdin=subprocess.DEVNULL,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
        server.stdin.close()
        server.stdout.read()
        if server.wait() != 0:
            raise policy.ToolError(f"Install the Shipped Plugins: the Seeding Host Exited With {server.returncode}")
        # The Reaped Host's Identifier May Be Reused; Stop Never Signals It Again.
        owner.processes.remove(server)

        # Start Tauri Dev With Vite Live Reload; Closing the Window Ends It
        desktop = owner.start(
            command(environment),
            cwd=GUI,
            env=cargo.compiler_environment("dev", source=environment),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        status = desktop.wait()
    except KeyboardInterrupt:
        status = 130
    except (policy.ToolError, OSError, subprocess.CalledProcessError) as error:
        print(f"dev: {error}", file=sys.stderr)
    finally:
        # Clean Up Once, Even When Interrupted or Terminated Again
        for name in ("SIGINT", "SIGTERM", "SIGHUP", "SIGBREAK"):
            if hasattr(signal, name):
                signal.signal(getattr(signal, name), signal.SIG_IGN)
        owner.stop()
        discard(root, namespace, host, environment)
        print(f"[dev] Removed {root} and the Credentials of {namespace}", flush=True)
    return status


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (policy.ToolError, OSError, subprocess.CalledProcessError) as error:
        print(f"dev: {error}", file=sys.stderr)
        raise SystemExit(1) from error
