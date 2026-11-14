# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Retain Desktop Stage And Resource Observations In The Existing Nightly Inventory."""

# === Imports === #

# Standard Library Imports
import json
import math
import os
import shutil
import subprocess
import sys
import time
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path
from selectors import DefaultSelector, EVENT_READ
from statistics import median

# Third Party Imports
# Local Imports
from .. import policy, temporary
from ..ci import bootstrap
from . import policy as test_policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@contextmanager
def credentials(environment: dict[str, str] | None) -> Generator[dict[str, str]]:
    """Own An Unlocked Secret Service For In-Process Linux Host Stories."""
    selected = dict(os.environ if environment is None else environment)
    if sys.platform != "linux":
        yield selected
        return
    timeout = test_policy.load(test_policy.root()).deadline("integration")
    with temporary.directory() as scratch:
        home = scratch / "credentials"
        home.mkdir(mode=0o700)
        selected.update({name: str(home) for name in ("XDG_DATA_HOME", "XDG_RUNTIME_DIR")})
        with subprocess.Popen(
            ["dbus-daemon", "--session", "--nofork", "--print-address=1"],
            env=selected,
            stdout=subprocess.PIPE,
            text=True,
        ) as bus:
            try:
                with DefaultSelector() as ready:
                    ready.register(bus.stdout, EVENT_READ)
                    if not ready.select(timeout):
                        raise RuntimeError("Start Story Credential Store: Private D-Bus Did Not Publish Its Address")
                address = bus.stdout.readline().strip()
                if not address:
                    raise RuntimeError(f"Start Story Credential Store: Private D-Bus Exited ({bus.wait()})")
                selected["DBUS_SESSION_BUS_ADDRESS"] = address
                with subprocess.Popen(
                    ["gnome-keyring-daemon", "--foreground", "--components=secrets", "--unlock", "--control-directory", str(home)],
                    env=selected,
                    stdin=subprocess.PIPE,
                ) as keyring:
                    try:
                        keyring.stdin.write(b"dokime-story-keyring-password")
                        keyring.stdin.close()
                        subprocess.run(["gdbus", "wait", "--session", "org.freedesktop.secrets"], env=selected, check=True, timeout=timeout)
                        yield selected
                    finally:
                        if keyring.poll() is None:
                            keyring.terminate()
                        try:
                            keyring.wait(timeout=timeout)
                        except subprocess.TimeoutExpired:
                            keyring.kill()
                            keyring.wait()
            finally:
                if bus.poll() is None:
                    bus.terminate()
                try:
                    bus.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    bus.kill()
                    bus.wait()


@contextmanager
def scratch(environment: dict[str, str] | None, reports: Path) -> Generator[dict[str, str]]:
    """Reclaim Native Child Scratch Even When Process Termination Skips Rust Drops."""
    with temporary.directory() as root:
        child_environment = dict(os.environ if environment is None else environment)
        child_environment.update({name: str(root) for name in ("TEMP", "TMP", "TMPDIR")})
        try:
            yield child_environment
        finally:
            if child_environment.get("DOKIME_KEEP_EVIDENCE") == "1":
                reports.mkdir(parents=True, exist_ok=True)
                shutil.copytree(root, reports / root.parent.name)


@contextmanager
def native(directory: Path, environment: dict[str, str]) -> Generator[None]:
    """Reuse Desktop Prerequisites And Own Only A Missing Headless Display."""
    display = None
    try:
        if sys.platform == "win32":
            bootstrap.prepare_windows_desktop(directory)
        else:
            bootstrap.install_gui_packages(runtime=True, desktop=True)
            bootstrap.prepare_linux_browser(directory)
            if not environment.get("DISPLAY"):
                number = next(number for number in range(90, 120) if not Path(f"/tmp/.X11-unix/X{number}").exists())
                display = subprocess.Popen(
                    ["Xvfb", f":{number}", "-screen", "0", "1920x1080x24"],
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.DEVNULL,
                    stderr=sys.stderr,
                )
                deadline = time.monotonic() + 30
                while not Path(f"/tmp/.X11-unix/X{number}").exists():
                    if display.poll() is not None or time.monotonic() > deadline:
                        raise ValueError("desktop benchmark display did not start")
                    time.sleep(0.05)
                environment["DISPLAY"] = f":{number}"
        yield
    finally:
        if display is not None:
            display.terminate()
            try:
                display.wait(timeout=5)
            except subprocess.TimeoutExpired:
                display.kill()
                display.wait()


def collect(directory: Path, home: Path, identity: dict, threshold: float, *, refresh: bool, success: bool) -> None:
    """Publish Explicit Missing Stages And Refusals Alongside Numeric Observations."""
    source = home / "observations.json"
    if not source.is_file():
        raise ValueError("desktop benchmark wrote no observations")
    current = json.loads(source.read_text(encoding="utf-8"))
    stored = home / "dev.json"
    previous = json.loads(stored.read_text(encoding="utf-8")) if stored.is_file() else {}
    baselines = {(row["name"], row["unit"]): row["value"] for row in previous.get("benchmarks", [])}
    groups = {}
    for result in current["results"]:
        history_rows = result.get("rows", current["fixture"]["rows"])
        groups.setdefault((history_rows, result["count"], result["condition"]), []).append(result)
    revision = policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    rows = []
    for (history_rows, count, condition), samples in groups.items():
        prefix = f"desktop/{history_rows}/{count}/{condition}"
        metrics = {}
        for sample in samples:
            for stage, values in sample["backendStages"].items():
                metrics.setdefault((stage, "ms"), []).append(sum(values))
            for stage, values in sample["stages"].items():
                metrics.setdefault((stage, "ms"), []).append(sum(values))
                metrics.setdefault((f"{stage}_call", "ms"), []).extend(values)
            if sample["queryMs"]:
                metrics.setdefault(("backend_query", "ms"), []).append(sum(sample["queryMs"]))
                metrics.setdefault(("backend_query_call", "ms"), []).extend(sample["queryMs"])
            for field, unit in (
                ("firstVisibleMs", "ms"),
                ("allVisibleMs", "ms"),
                ("durationMs", "ms"),
                ("resourceWindowMs", "ms"),
                ("cpuSeconds", "s"),
                ("peakMemoryBytes", "bytes"),
                ("bytes", "bytes"),
                ("queueDepth", "frames"),
                ("frameMisses", "frames"),
            ):
                value = sample.get(field)
                if value is not None:
                    metrics.setdefault((field, unit), []).append(value)
        refusal_count = sum(sample["outcome"] == "refused" for sample in samples)
        for (metric, unit), values in metrics.items():
            ordered = sorted(values)
            if not ordered or not all(math.isfinite(value) for value in ordered):
                raise ValueError(f"invalid desktop samples: {prefix}/{metric}")
            for quantile, value in (("p50", median(ordered)), ("p95", ordered[max(0, math.ceil(len(ordered) * 0.95) - 1)])):
                name = f"{prefix}/{metric}/{quantile}"
                rows.append(
                    {
                        "name": name,
                        "suite": "rust/desktop",
                        "unit": unit,
                        "value": value,
                        "baseline_value": baselines.get((name, unit)),
                        "revision": revision,
                        "threshold_percent": threshold,
                        "refused_samples": refusal_count,
                        "samples": len(ordered),
                    }
                )
        rows.append(
            {
                "name": f"{prefix}/refusals",
                "suite": "rust/desktop",
                "unit": "cases",
                "value": refusal_count,
                "baseline_value": None,
                "revision": revision,
                "threshold_percent": threshold,
                "failures": sorted({error for sample in samples for error in sample["errors"]}),
                "partial_samples": sum(sample["outcome"] == "partial" for sample in samples),
                "missing_measurements": [
                    metric
                    for metric in (
                        "backend_query",
                        "plot.watch.frames",
                        "plot.watch.send",
                        "transport_bridge_wait",
                        "protobuf_decode",
                        "native_frame_adaptation",
                        "retained_update",
                        "canvas_draw",
                        "presentation_opportunity",
                        "firstVisibleMs",
                        "allVisibleMs",
                        "cpuSeconds",
                        "peakMemoryBytes",
                    )
                    if not any(name == metric for name, _unit in metrics)
                ],
                "unavailable": [
                    "physical_display_presentation",
                    "exclusive_encoding_native_bridge",
                    "os_cache_cold",
                    "native_server_queue_depth",
                ],
                "fixture": samples[0].get("fixture", current["fixture"]),
            }
        )
    summary = {
        "benchmarks": rows,
        "identity": identity,
        "fixture": current["fixture"],
        "unavailable": [
            "Physical display presentation (double requestAnimationFrame measures only a presentation opportunity)",
            "Exclusive encoding and native bridge time (transport_bridge_wait includes backend work and idle wait)",
            "OS disk-cache-cold reads (cold reopens a closed history with a fresh backend and desktop)",
            "Native bridge/server queue depth (queueDepth observes the actual WebView receive queue)",
        ],
    }
    policy.write_json(home / "new.json", summary)
    policy.write_json(directory / "target/benchmarks/benchmarks-rust-desktop.json", summary)
    if success and refresh:
        shutil.copyfile(home / "new.json", stored)


# === Private Functions === #

# === Entry Point === #
