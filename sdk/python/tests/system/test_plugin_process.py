# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Announce or Register a Connectable Plugin Listener and Settle a Native Interrupt."""

# === Imports === #

# Standard Library Imports
import os
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import unittest
from concurrent.futures import ThreadPoolExecutor
from importlib.metadata import version
from pathlib import Path
from queue import Queue
from urllib.parse import urlsplit

# Third Party Imports
from dokime_tools.test import policy
from grpc import ServicerContext, StatusCode, server
from harmos import CLASSIFICATION_PERMANENT
from harmos._wire.model.v1.failure_pb2 import Failure
from harmos._wire.plugin.v1.launches_pb2 import LaunchesAnnounceRequest, LaunchesAnnounceResponse
from harmos._wire.plugin.v1.launches_pb2_grpc import LaunchesServicer, add_LaunchesServicer_to_server

# Local Imports
from starter_project.plugin import ExamplePlugin

from dokime_sdk._wire.host.v1 import plugins_pb2
from dokime_sdk._wire.host.v1.plugins_pb2_grpc import PluginsServicer, add_PluginsServicer_to_server

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Host(PluginsServicer, LaunchesServicer):
    def __init__(self) -> None:
        self.requests: list[plugins_pb2.PluginsCreateRequest] = []
        self.entries: dict[str, plugins_pb2.PluginEntry] = {}
        self.refusal: Failure | None = None
        self.ready: Queue[str] = Queue()

    def Create(self, request: plugins_pb2.PluginsCreateRequest, context: ServicerContext) -> plugins_pb2.PluginsCreateResponse:
        self.requests.append(request)
        if self.refusal is not None:
            context.set_trailing_metadata((("grpc-status-details-bin", self.refusal.SerializeToString()),))
            context.abort(StatusCode.FAILED_PRECONDITION, self.refusal.message)
        # Like the Product Host, an Existing Name Reuses Its Entry.
        entry = self.entries.setdefault(request.name, plugins_pb2.PluginEntry(name=request.name, revision=b"created"))
        entry.placement.CopyFrom(request.placement)
        entry.value.CopyFrom(request.value)
        entry.state = plugins_pb2.PLUGIN_STATE_READY
        self.ready.put(entry.placement.remote.url)
        return plugins_pb2.PluginsCreateResponse(entry=entry)

    def Announce(self, request: LaunchesAnnounceRequest, context: ServicerContext) -> LaunchesAnnounceResponse:
        if request.token != "fixture-launch":
            context.abort(StatusCode.PERMISSION_DENIED, "Use the Fixture Launch Token")
        self.ready.put("http://" + request.address)
        return LaunchesAnnounceResponse()


class PluginProcessStory(unittest.TestCase):
    def setUp(self) -> None:
        self.timeout = policy.load(Path(__file__).resolve().parents[4]).deadline("system")
        self.host = Host()
        listener = server(ThreadPoolExecutor(max_workers=2))
        add_PluginsServicer_to_server(self.host, listener)
        add_LaunchesServicer_to_server(self.host, listener)
        self.endpoint = f"http://127.0.0.1:{listener.add_insecure_port('127.0.0.1:0')}"
        listener.start()
        self.addCleanup(listener.stop, None)

    def test_launch_announces_and_interrupt_closes_without_traceback(self) -> None:
        directory = Path(tempfile.mkdtemp(prefix="dokime-managed-plugin-"))
        self.addCleanup(shutil.rmtree, directory)
        (directory / ".env").write_text('DOKIME_REMOTE="unterminated\n', encoding="utf-8")
        self._run(
            {
                "HARMOS_HOST_ENDPOINT": self.endpoint.removeprefix("http://"),
                "HARMOS_LAUNCH_TOKEN": "fixture-launch",
                "DOKIME_REMOTE": self.endpoint,
            },
            directory=directory,
        )
        self.assertEqual(self.host.requests, [])

    def test_paired_start_registers_once_and_a_second_start_reuses_the_entry(self) -> None:
        for _ in range(2):
            self._run({"DOKIME_REMOTE": self.endpoint})
        names = [request.name for request in self.host.requests]
        self.assertEqual(names, [ExamplePlugin().identifier] * 2)
        self.assertEqual({request.sdk_version for request in self.host.requests}, {version("dokime-sdk")})
        # Stopping Keeps the Entry for the Host to Show Offline.
        self.assertEqual(list(self.host.entries), names[:1])

    def test_refused_registration_exits_nonzero_with_the_host_message(self) -> None:
        self.host.refusal = Failure(
            classification=CLASSIFICATION_PERMANENT,
            code="plugin_incompatible",
            message="Fixture Host Refuses SDK",
            action="Rebuild the Fixture Plugin",
        )
        environment = dict(os.environ)
        environment.pop("HARMOS_HOST_ENDPOINT", None)
        environment.pop("HARMOS_LAUNCH_TOKEN", None)
        environment.update({"DOKIME_REMOTE": self.endpoint})
        result = subprocess.run(
            [sys.executable, "-m", "starter_project.plugin"],
            check=False,
            env=environment,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=self.timeout,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn(self.host.refusal.message, result.stderr)
        self.assertIn(self.host.refusal.action, result.stderr)

    def _run(self, overrides: dict[str, str], *, directory: Path | None = None) -> str:
        timeout = self.timeout
        # Start the Installed Plugin With Closed Standard Input.
        flags = 0
        startup = None
        command = [sys.executable, "-m", "starter_project.plugin"]
        if os.name == "nt":
            flags = subprocess.CREATE_NEW_CONSOLE
            startup = subprocess.STARTUPINFO()
            startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow = subprocess.SW_HIDE
            # Headless Launchers May Pass an Ignored Native Ctrl-C Flag.
            launch = """
import ctypes
import runpy

kernel = ctypes.WinDLL("kernel32", use_last_error=True)
if not kernel.SetConsoleCtrlHandler(None, False):
    raise ctypes.WinError(ctypes.get_last_error())
runpy.run_module("starter_project.plugin", run_name="__main__")
"""
            command = [sys.executable, "-c", launch]
        environment = dict(os.environ)
        environment.pop("HARMOS_HOST_ENDPOINT", None)
        environment.pop("HARMOS_LAUNCH_TOKEN", None)
        environment.update(overrides)
        process = subprocess.Popen(
            command,
            cwd=directory,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            creationflags=flags,
            startupinfo=startup,
        )
        try:
            # The Host Observes Launch Announcement or Registration Directly.
            served = self.host.ready.get(timeout=timeout)
            address = urlsplit(served)
            self.assertEqual(address.scheme, "http")
            assert address.hostname is not None and address.port is not None
            connection = socket.create_connection((address.hostname, address.port), timeout=timeout)
            connection.close()
            self.assertIsNone(process.poll())

            # Deliver a Native Interrupt Only to the Owned Process's Console.
            if os.name == "nt":
                interrupt = """
import ctypes
import sys

kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.FreeConsole()
if not kernel.AttachConsole(int(sys.argv[1])):
    raise ctypes.WinError(ctypes.get_last_error())
if not kernel.SetConsoleCtrlHandler(None, True):
    raise ctypes.WinError(ctypes.get_last_error())
if not kernel.GenerateConsoleCtrlEvent(0, 0):
    raise ctypes.WinError(ctypes.get_last_error())
"""
                subprocess.run([sys.executable, "-c", interrupt, str(process.pid)], check=True, timeout=timeout)
            else:
                process.send_signal(signal.SIGINT)
            remaining, errors = process.communicate(timeout=timeout)
            self.assertEqual(process.returncode, 0, errors)
            self.assertEqual(remaining, "")
            self.assertEqual(errors, "")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=timeout)
            if process.stdout is not None:
                process.stdout.close()
            if process.stderr is not None:
                process.stderr.close()
        return served


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
