# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Prove Task Transport, Required Gates and Compiler Cache Compatibility."

# === Imports === #

# Standard Library Imports
import io
import json
import os
import runpy
import subprocess
import tempfile
import unittest
from collections.abc import Iterator
from pathlib import Path
from queue import Queue
from types import SimpleNamespace
from unittest.mock import MagicMock, Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.ci import bootstrap, compiler, native, signing, tasks, transport, workflow
from dokime_tools.release import guard, nightly, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WorkflowTests(unittest.TestCase):
    def test_compiler_cache_resolves_the_project_wrapper_before_entering_a_consumer(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            wrapper = root / "tools/sccache"
            environment = {"COMPILER_CACHE_BACKEND": "disk", "SCCACHE_DIR": str(root / "cache"), "RUSTC_WRAPPER": "sccache"}
            with (
                patch.dict(os.environ, environment, clear=True),
                patch.object(compiler.subprocess, "check_output", return_value=str(wrapper) + "\n") as resolve,
                patch.object(compiler, "_execute", return_value=7) as execute,
            ):
                self.assertEqual(compiler.run(["mise", "-C", str(root / "consumer"), "run", "build"]), 7)
            self.assertEqual(resolve.call_args.args[0], ["mise", "which", "sccache"])
            self.assertEqual(resolve.call_args.kwargs["cwd"], Path(compiler.__file__).resolve().parents[5])
            selected = execute.call_args.args[2]
            self.assertEqual(selected["RUSTC_WRAPPER"], str(wrapper))
            self.assertEqual(selected["SCCACHE_DIR"], str((root / "cache").resolve()))
            self.assertEqual(execute.call_args.args[3], "disk")
        finally:
            temporary.cleanup()

    def test_sdk_operations_install_pinned_tools_before_selected_tasks(self) -> None:
        for language, operation, expected in (
            ("csharp", "sdk-check", ["install", "restore", "check"]),
            ("cpp", "sdk-check", ["install", "check"]),
            ("csharp", "sdk-test", ["install", "build", "test"]),
            ("cpp", "sdk-test", ["install", "build", "test"]),
        ):
            with patch.object(workflow.subprocess, "run", return_value=Mock(returncode=0)) as command:
                self.assertEqual(workflow.main([operation, "--language", language]), 0)
            observed = [call.args[0][3] if call.args[0][3] == "install" else call.args[0][6] for call in command.call_args_list]
            self.assertEqual(observed, expected)
            for call in command.call_args_list:
                self.assertIs(call.kwargs["stdout"], workflow.sys.stdout)
                self.assertIs(call.kwargs["stderr"], workflow.sys.stderr)
                self.assertIs(call.kwargs["stdin"], workflow.sys.stdin)

    def test_cpp_workflow_selects_profile_and_reports_the_failing_step(self) -> None:
        for profile, preset in (("dev", "tests"), ("release", "release")):
            for failure_at in range(3):
                results = [Mock(returncode=0) for _ in range(failure_at)] + [Mock(returncode=7)]
                with (
                    patch.object(workflow.subprocess, "run", side_effect=results) as command,
                    patch.object(workflow.sys, "stderr", new_callable=io.StringIO) as stderr,
                ):
                    self.assertEqual(workflow.main(["sdk-test", "--language", "cpp", "--profile", profile]), 7)
                self.assertEqual(command.call_count, failure_at + 1)
                if failure_at:
                    arguments = command.call_args.args[0]
                    expected = ["--", "--preset", preset]
                    if failure_at == 2:
                        expected.append("--verbose")
                    self.assertEqual(arguments[7:], expected)
                    self.assertIn(subprocess.list2cmdline(arguments), stderr.getvalue())
                self.assertIn("7", stderr.getvalue())

    def test_cpp_commands_stream_and_preserve_output_before_waiting(self) -> None:
        sdk = runpy.run_path(str(Path(__file__).resolve().parents[4] / "sdk/cpp/tools/sdk.py"))
        runner = sdk["run"]
        for returncode in (0, 7):
            temporary = tempfile.TemporaryDirectory()
            try:
                log = Path(temporary.name) / "build.log"
                log.write_text("previous attempt\n", encoding="utf-8")
                console = io.StringIO()

                def lines(console: io.StringIO = console, log: Path = log) -> Iterator[str]:
                    yield "compiler progress\n"
                    self.assertIn("compiler progress\n", console.getvalue())
                    self.assertEqual(log.read_text(encoding="utf-8"), "compiler progress\n")
                    yield "compiler diagnostic\n"

                def wait(console: io.StringIO = console, log: Path = log, returncode: int = returncode) -> int:
                    self.assertIn("compiler diagnostic\n", console.getvalue())
                    self.assertEqual(log.read_text(encoding="utf-8"), "compiler progress\ncompiler diagnostic\n")
                    return returncode

                stream = MagicMock()
                stream.__iter__.side_effect = lines
                process = Mock(stdout=stream, wait=Mock(side_effect=wait))
                with (
                    patch.object(subprocess, "Popen", return_value=process) as start,
                    patch("sys.stdout", console),
                    patch("sys.stderr", new_callable=io.StringIO),
                ):
                    if returncode:
                        with self.assertRaises(subprocess.CalledProcessError) as failure:
                            runner(["cmake", "--build", "--preset", "tests"], action="Build C++ SDK", cwd=log.parent, env={}, log=log)
                        self.assertEqual(failure.exception.returncode, returncode)
                    else:
                        runner(["cmake", "--build", "--preset", "tests"], action="Build C++ SDK", cwd=log.parent, env={}, log=log)
                self.assertEqual(start.call_args.kwargs["stderr"], subprocess.STDOUT)
                self.assertEqual((log.parent / "build-1.log").read_text(encoding="utf-8"), "previous attempt\n")
                stream.close.assert_called_once()
            finally:
                temporary.cleanup()

    def test_cpp_environment_keeps_resolved_tools_and_msvc_with_a_long_parent_path(self) -> None:
        sdk = runpy.run_path(str(Path(__file__).resolve().parents[4] / "sdk/cpp/tools/sdk.py"))
        prepare = sdk["environment"]
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            manifest = json.loads((sdk["ROOT"] / "sdk/cpp/vcpkg.json").read_text())
            revision = manifest["builtin-baseline"]
            checkout = root / "vcpkg" / revision[:12]
            checkout.mkdir(parents=True)
            (checkout / "vcpkg.exe").touch()
            tools = {tool: root / "tools" / tool / f"{tool}.exe" for tool in ("cmake", "ninja")}
            parent = str(root / "parent")
            native_path = str(root / "msvc")
            long_path = os.pathsep.join([parent] * 1000)
            environment = {"PATH": long_path, "PROGRAMFILES(X86)": str(root), "SYSTEMROOT": str(root / "Windows")}

            def output(command: list[str] | str, **options: object) -> str:
                if command == ["mise", "cache", "path"]:
                    return str(root)
                if isinstance(command, list) and command[:2] == ["mise", "which"]:
                    self.assertEqual(options["cwd"], sdk["ROOT"] / "sdk/cpp")
                    return str(tools[command[2]])
                if isinstance(command, list):
                    self.assertEqual(command, ["git", "-C", str(checkout), "rev-parse", "HEAD"])
                    return revision
                selected = options["env"]
                if not isinstance(selected, dict):
                    self.fail("Pass the Native Setup Environment to the Developer Command")
                self.assertNotIn(parent, selected["PATH"])
                return f"PATH={native_path}\nINCLUDE=native-headers\nLIB=native-libraries\nVCINSTALLDIR={native_path}\n"

            with (
                patch.dict(
                    prepare.__globals__, os=SimpleNamespace(name="nt", environ=environment, pathsep=os.pathsep), visual_studio=lambda: root
                ),
                patch.object(subprocess, "check_output", side_effect=output),
                patch.object(subprocess, "run") as run,
            ):
                selected = prepare()
            self.assertEqual(selected["PATH"].split(os.pathsep), [str(path.parent) for path in tools.values()] + [native_path, parent])
            self.assertEqual(selected["INCLUDE"], "native-headers")
            self.assertEqual(selected["LIB"], "native-libraries")
            self.assertEqual(selected["VCPKG_ROOT"], str(checkout))
            run.assert_not_called()
        finally:
            temporary.cleanup()

    def test_cpp_plugin_output_is_drained_after_readiness_and_on_startup_failure(self) -> None:
        support = runpy.run_path(str(Path(__file__).resolve().parents[4] / "sdk/cpp/tests/support/run_plugin.py"))
        for lines, address in (
            ("plugin startup\nharmos listening http://127.0.0.1:7000\nplugin progress\n", "http://127.0.0.1:7000"),
            ("plugin startup failed\n", None),
        ):
            ready: Queue[str | None] = Queue()
            process = Mock(stdout=io.StringIO(lines))
            with patch("sys.stdout", new_callable=io.StringIO) as console:
                support["read_output"](process, ready)
            self.assertEqual(ready.get_nowait(), address)
            self.assertTrue(ready.empty())
            self.assertEqual(console.getvalue(), lines)

    def test_bootstrap_commands_run_in_order_and_stop_on_failure(self) -> None:
        commands = ["mise run ci:prerequisites -- --gui-runtime", "mise run ci:prerequisites -- --desktop"]
        environment = SimpleNamespace(name="posix", environ={"SETUP_BOOTSTRAP": json.dumps(commands)})
        with patch.object(bootstrap, "os", environment), patch.object(bootstrap.subprocess, "run") as command:
            self.assertEqual(bootstrap.main(["--commands"]), 0)
            self.assertEqual([call.args[0][-1] for call in command.call_args_list], commands)
        with (
            patch.object(bootstrap, "os", environment),
            patch.object(bootstrap.subprocess, "run", side_effect=subprocess.CalledProcessError(7, commands[0])) as command,
            self.assertRaises(subprocess.CalledProcessError),
        ):
            bootstrap.main(["--commands"])
        self.assertEqual(command.call_count, 1)

    def test_signing_preserves_provider_selection_and_package_order(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            binary = root / "target/build/dev/windows-x86_64/bin"
            binary.mkdir(parents=True)
            for name in ("dokime.exe", "dokime-gui.exe"):
                (binary / name).touch()
            output = root / "output"

            def path(value: str) -> Path:
                return root / value

            for provider in ("none", "azure", "pfx"):
                environment = {"GITHUB_OUTPUT": str(output), "DOKIME_BIN": str(binary), "DOKIME_AUTHENTICODE_PROVIDER": provider}
                with (
                    self.subTest(provider=provider),
                    patch.object(signing, "os", SimpleNamespace(name="nt", environ=environment)),
                    patch.object(signing, "Path", side_effect=path),
                    patch.object(signing.transport, "receive") as receive,
                    patch.object(signing.subprocess, "check_output", return_value=provider + "\n"),
                    patch.object(signing.subprocess, "run", return_value=Mock(returncode=0)) as command,
                ):
                    output.write_text("")
                    self.assertEqual(signing.main(["select", "--profile", "dev"]), 0)
                    receive.assert_called_once()
                    records = dict(line.split("=", 1) for line in output.read_text().splitlines())
                    self.assertEqual(records, {"provider": provider, "bin": str(binary.resolve())})
                    self.assertEqual(signing.main(["package", "--profile", "dev"]), 0)
                    calls = [call.args[0] for call in command.call_args_list]
                    self.assertEqual(calls[-1], ["mise", "run", "package", "--", "--language", "rust", "--profile", "dev"])
                    self.assertEqual(environment["CI_INPUT_ARCHIVES"], "[]")
                    if provider == "none":
                        self.assertEqual(len(calls), 1)
                        continue
                    self.assertEqual(calls[-2], ["mise", "run", "ci:distribution", "--", "--profile", "dev"])
                    self.assertEqual([call[5] for call in calls[:2]], ["Sign" if provider == "pfx" else "Verify"] * 2)
        finally:
            temporary.cleanup()

    def test_release_mode_exports_routing_and_target_revision(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            output = Path(temporary) / "output"
            for mode, sha in (("tag", "a" * 40), ("prepare", ""), ("skip", "")):
                with (
                    self.subTest(mode=mode),
                    patch.dict(os.environ, {"GITHUB_OUTPUT": str(output)}, clear=True),
                    patch("dokime_tools.ci.review.github_release_mode", return_value={"mode": mode, "sha": sha}),
                ):
                    output.write_text("", encoding="utf-8")
                    self.assertEqual(workflow.main(["release-mode"]), 0)
                    self.assertEqual(output.read_text(encoding="utf-8"), f"mode={mode}\nsha={sha}\n")
        finally:
            temporary_directory.cleanup()

    def test_nightly_report_requires_exact_publication_identity(self) -> None:
        sha = "a" * 40
        url = "https://example.test/run"
        identity = f"Built from [`{sha[:8]}`](https://example.test/commit/{sha}) · [run 7]({url})"
        notes = identity + "\n"
        release = {"body": notes + "\n<!-- nightly-results -->\n{}\n<!-- /nightly-results -->\n", "target_commitish": sha}
        with (
            patch.object(nightly.policy, "policy", return_value={"provider": "github"}),
            patch.object(nightly, "_source", return_value=(sha, "7", url)),
            patch.object(nightly.registry.provider, "document", return_value=release),
            patch.object(nightly, "publish") as publish,
        ):
            self.assertTrue(nightly.complete(Path.cwd()))
            self.assertTrue(nightly.complete(Path.cwd()))
            self.assertEqual(publish.call_count, 2)
            self.assertEqual(publish.call_args.kwargs["release_notes"], notes)
            for body, revision in (
                (identity.replace("[run 7]", "[run 70]"), sha),
                (identity.replace(url, url + "/other"), sha),
                (identity.replace(sha, "b" * 40), sha),
                (identity, "b" * 40),
            ):
                release.update(body=body, target_commitish=revision)
                self.assertFalse(nightly.complete(Path.cwd()))
            self.assertEqual(publish.call_count, 2)

    def test_nightly_channel_uses_shared_publish_and_verify_entries(self) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(nightly, "publish") as publish,
            patch.object(nightly, "verify") as verify,
            patch.object(guard, "check") as guard_call,
        ):
            self.assertEqual(tasks.main(["publish", "--channel", "nightly"]), 0)
            self.assertEqual(workflow.main(["release-verify", "--channel", "nightly"]), 0)
            publish.assert_called_once_with(Path.cwd())
            verify.assert_called_once_with(Path.cwd())
            guard_call.assert_not_called()

    def test_nightly_freshness_compares_tags_and_only_earlier_duplicates(self) -> None:
        sha = "a" * 40
        cases = (
            ("schedule", sha, 0, "", [], False),
            ("schedule", "b" * 40, 0, "", [], True),
            ("schedule", None, 1, "HTTP/2 404", [], True),
            ("schedule", None, 1, "HTTP/2 403", [], None),
            ("schedule", None, 1, "HTTP/2 503", [], None),
            ("workflow_dispatch", sha, 0, "", [], True),
            ("workflow_dispatch", sha, 0, "", [{"id": 6}], False),
            ("workflow_dispatch", sha, 0, "", [{"id": 7}, {"id": 8}], True),
        )
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            output = Path(temporary) / "output"
            for event, published, code, message, runs, expected in cases:
                environment = {
                    "GITHUB_REF": "refs/heads/dev",
                    "GITHUB_EVENT_NAME": event,
                    "GITHUB_SHA": sha,
                    "GITHUB_RUN_ID": "7",
                    "GITHUB_REPOSITORY": "example/dokime",
                    "GITHUB_OUTPUT": str(output),
                }
                with (
                    self.subTest(event=event, code=code, runs=runs),
                    patch.dict(os.environ, environment, clear=True),
                    patch.object(subprocess, "run", return_value=Mock(returncode=code, stdout=message, stderr="")),
                    patch("dokime_tools.ci.review.request", return_value={"object": {"sha": published}}) as request,
                    patch("dokime_tools.ci.review.pages", return_value=runs),
                ):
                    output.write_text("", encoding="utf-8")
                    if expected is None:
                        with self.assertRaises(RuntimeError):
                            workflow.main(["nightly-freshness"])
                        self.assertEqual(output.read_text(), "")
                        continue
                    self.assertEqual(workflow.main(["nightly-freshness"]), 0)
                    self.assertEqual(output.read_text(), f"build={str(expected).lower()}\n")
                    if event == "workflow_dispatch":
                        request.assert_not_called()
            with patch.dict(os.environ, {"GITHUB_REF": "refs/heads/topic"}, clear=True), self.assertRaises(ValueError):
                workflow.main(["nightly-freshness"])
        finally:
            temporary_directory.cleanup()

    def test_named_transport_rejects_missing_inputs_before_extracting(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            archive = Path(temporary) / "package-release-unix.tar.zst"
            archive.touch()
            missing = Path(temporary) / "missing.tar.zst"
            selection = [str(archive).replace("release", "{profile}").replace("unix", "{platform}"), str(missing)]
            environment = {"CI_INPUT_ARCHIVES": json.dumps(selection), "PROFILE": "release", "CI_PLATFORM": "unix"}
            with patch.dict(os.environ, environment, clear=True), patch.object(transport.artifacts, "extract") as extract:
                with self.assertRaises(FileNotFoundError):
                    transport.receive()
                extract.assert_not_called()
                missing.touch()
                transport.receive()
                self.assertEqual([call.args[1] for call in extract.call_args_list], [archive, missing])
        finally:
            temporary_directory.cleanup()

    def test_transport_expands_producer_identity(self) -> None:
        output = {"archive": "target/ci/package-{profile}-{platform}.tar.zst", "paths": ["target/package/{profile}/{target}"]}
        environment = {"CI_OUTPUTS": json.dumps([output]), "CI_PLATFORM": "unix"}
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(transport, "target", return_value="linux-x86_64"),
            patch.object(transport.artifacts, "pack") as pack,
        ):
            transport.send("release")
        self.assertEqual(pack.call_args.args[1], Path("target/ci/package-release-unix.tar.zst"))
        self.assertEqual(pack.call_args.args[2], output["paths"])
        self.assertEqual(pack.call_args.args[3], "release")

    def test_transport_extracts_downloads_only_when_requested(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            archive = root / "product.tar.zst"
            archive.touch()
            (root / "report.json").touch()
            with patch.dict(os.environ, {}, clear=True), patch.object(transport.artifacts, "extract") as extract:
                transport.receive()
                extract.assert_not_called()
            with (
                patch.dict(os.environ, {"CI_TRANSPORT": "true"}, clear=True),
                patch.object(transport.Path, "glob", return_value=[archive]),
                patch.object(transport.artifacts, "extract") as extract,
            ):
                transport.receive()
                extract.assert_called_once_with(Path.cwd(), archive)
        finally:
            temporary_directory.cleanup()

    def test_compiler_cache_ignores_unrelated_tools_but_tracks_rust_and_crates(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            for name in ("mise.lock", "Cargo.lock", "uv.lock", "crates/dokime-gui/pnpm-lock.yaml", "site/pnpm-lock.yaml"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("original", encoding="utf-8")
            environment = {
                "RUNNER_OS": "Linux",
                "RUNNER_ARCH": "X64",
                "COMPILER_CACHE_BACKEND": "disk",
                "CI_CACHE_PROFILE": "release",
                "RUST_TOOLCHAIN": "1.98.0",
            }
            original = native.default_caches(root, environment)
            (root / "mise.lock").write_text("other tool changed", encoding="utf-8")
            tools = native.default_caches(root, environment)
            self.assertEqual(original["sccache"], tools["sccache"])
            self.assertNotEqual(original["mise"]["key"], tools["mise"]["key"])
            (root / "Cargo.lock").write_text("crate changed", encoding="utf-8")
            crates = native.default_caches(root, environment)["sccache"]
            self.assertNotEqual(crates["key"], original["sccache"]["key"])
            self.assertEqual(crates["restore-keys"], original["sccache"]["restore-keys"])
            rust = native.default_caches(root, environment | {"RUST_TOOLCHAIN": "1.99.0"})["sccache"]
            self.assertNotEqual(rust["restore-keys"], crates["restore-keys"])
            tests = native.default_caches(root, environment | {"CI_CACHE_PROFILE": "test"})["sccache"]
            self.assertNotEqual(tests["restore-keys"], crates["restore-keys"])
        finally:
            temporary_directory.cleanup()

    def test_release_rehearsal_uses_moved_evidence_and_requires_green_jobs(self) -> None:
        head = "a" * 40
        merged = "b" * 40
        review = {"head": head, "number": 7}
        pull = {"number": 7, "head": {"sha": head}}
        run = {
            "id": 9,
            "head_sha": head,
            "pull_requests": [{"number": 7}],
            "display_title": "CI / Validation",
            "status": "completed",
            "conclusion": "success",
        }
        jobs = [
            {"name": "Check / CI / Required", "conclusion": "success"},
            {"name": registry.reviews.REHEARSAL_CHECK, "conclusion": "success"},
        ]

        def pages(settings: dict, endpoint: str, key: str | None = None) -> list[dict]:
            return [run] if "workflows/ci.yml" in endpoint else jobs

        with (
            patch.object(registry.provider, "document", return_value=pull),
            patch.object(registry.provider, "pages", side_effect=pages),
            patch.object(registry.reviews, "_same_tree") as tree,
        ):
            registry.reviews.rehearsed(Path.cwd(), {"provider": "github"}, review, merged)
            tree.assert_called_once_with(Path.cwd(), head, merged)
            jobs[1]["conclusion"] = "failure"
            with self.assertRaises(policy.ToolError):
                registry.reviews.rehearsed(Path.cwd(), {"provider": "github"}, review, merged)

    def test_publication_guard_restores_mutating_token_after_read_guard(self) -> None:
        observed = []
        environment = {"RELEASE_TARGET_SHA": "a" * 40, "CI_READ_TOKEN": "read", "GH_TOKEN": "release"}
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(guard.green, "main", side_effect=lambda: observed.append(os.environ["GH_TOKEN"])),
            patch("dokime_tools.release.run.main", return_value=0),
        ):
            guard.check()
            self.assertEqual(os.environ["GH_TOKEN"], "release")
        self.assertEqual(observed, ["read"])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
