# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""C++ Native Build Environment Regression Tests"""

# === Imports === #

# Standard Library Imports
import json
import os
import runpy
import tempfile
import unittest
from itertools import product
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[4]
SDK = runpy.run_path(str(ROOT / "sdk/cpp/tools/sdk.py"))

# === Types === #

# === Classes === #


class CppEnvironmentTests(unittest.TestCase):
    def test_project_pin_survives_native_environment_loading(self) -> None:
        manifest = json.loads((ROOT / "sdk/cpp/vcpkg.json").read_text(encoding="utf-8"))
        revision = manifest["builtin-baseline"]
        temporary = tempfile.TemporaryDirectory()
        try:
            cache = Path(temporary.name)
            checkout = cache / "vcpkg" / revision[:12]
            checkout.mkdir(parents=True)
            (checkout / "vcpkg.exe").touch()
            (checkout / "vcpkg").touch()
            bundled = cache / "visual-studio/vcpkg"
            native = cache / "visual-studio/bin"
            tools = cache / "tools"
            inherited = cache / "inherited"
            binary_cache = cache / "archives"
            environment = SDK["environment"]
            for (platform, initialized), jobs in product((("nt", False), ("nt", True), ("posix", False)), (None, "2")):
                child = {
                    "PATH": str(inherited),
                    "PROGRAMFILES(X86)": str(cache / "programs"),
                    "SYSTEMROOT": str(cache / "windows"),
                    "VCPKG_ROOT": str(bundled),
                    "VCPKG_DEFAULT_BINARY_CACHE": str(binary_cache),
                }
                if jobs is not None:
                    child["CARGO_BUILD_JOBS"] = jobs
                if initialized:
                    child["VCINSTALLDIR"] = str(cache / "visual-studio")

                def output(command: list[str] | str, **_: object) -> str:
                    if isinstance(command, str):
                        return f"PATH={native}\nVCPKG_ROOT={bundled}\nVCINSTALLDIR={cache / 'visual-studio'}\n"
                    if command[:2] == ["mise", "which"]:
                        return str(tools / command[2])
                    if command == ["mise", "cache", "path"]:
                        return str(cache)
                    return revision

                native_os = SimpleNamespace(name=platform, environ=child, pathsep=os.pathsep)
                subprocess_module = environment.__globals__["subprocess"]
                with (
                    self.subTest(platform=platform, initialized=initialized, jobs=jobs),
                    patch.dict(environment.__globals__, {"os": native_os, "visual_studio": lambda: cache / "visual-studio"}),
                    patch.object(subprocess_module, "check_output", side_effect=output) as invoke,
                    patch.object(subprocess_module, "run") as bootstrap,
                ):
                    result = environment()
                    self.assertEqual(Path(result["VCPKG_ROOT"]), checkout)
                    self.assertEqual(result["VCPKG_DEFAULT_BINARY_CACHE"], str(binary_cache))
                    if jobs is None:
                        self.assertNotIn("VCPKG_MAX_CONCURRENCY", result)
                    else:
                        self.assertEqual(result["VCPKG_MAX_CONCURRENCY"], jobs)
                    paths = result["PATH"].split(os.pathsep)
                    self.assertEqual(paths[0], str(tools))
                    self.assertIn(str(inherited), paths)
                    developer_calls = [call for call in invoke.call_args_list if isinstance(call.args[0], str)]
                    self.assertEqual(len(developer_calls), int(platform == "nt" and not initialized))
                    bootstrap.assert_not_called()
        finally:
            temporary.cleanup()

    def test_both_builds_inherit_optional_job_limit(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            main = SDK["main"]
            bundle = root / "target/cp/tests/dokime-cpp"
            bundle.mkdir(parents=True)
            (bundle / "dokime-cpp.json").write_text(json.dumps({"build_type": "Debug"}), encoding="utf-8")
            for action, jobs in product(("build", "consumer"), (None, "2")):
                child = {"VCPKG_ROOT": str(root / "vcpkg")}
                if jobs is not None:
                    child["CARGO_BUILD_JOBS"] = jobs
                invoke = Mock()
                with (
                    self.subTest(action=action, jobs=jobs),
                    patch.dict(main.__globals__, {"ROOT": root, "environment": Mock(return_value=child), "run": invoke}),
                    patch.object(main.__globals__["sys"], "argv", ["sdk.py", action]),
                    patch.object(main.__globals__["subprocess"], "check_output", return_value="ninja: no work to do."),
                ):
                    main()
                    command = invoke.call_args.args[0]
                    self.assertEqual(command[command.index("--parallel") :], ["--parallel", *([jobs] if jobs is not None else [])])
        finally:
            temporary.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
