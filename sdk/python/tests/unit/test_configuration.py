# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Connection Configuration Grammar and Precedence"""

# === Imports === #

# Standard Library Imports
import os
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_sdk._configuration import load, parse, resolve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConfigurationStory(TestCase):
    def test_quoting_comments_and_literal_shell_text(self) -> None:
        content = "\ufeff# Settings\r\n A = plain#literal # comment\r\nB='  # literal $HOME `pwd`  '\nC=\"one\\ntwo\\r\\t\\\\\\\"#\" # comment\nEMPTY=\nQUOTED=''\n_UNKNOWN = value\n"
        self.assertEqual(
            parse(content, Path(".env")),
            {
                "A": "plain#literal",
                "B": "  # literal $HOME `pwd`  ",
                "C": 'one\ntwo\r\t\\"#',
                "EMPTY": "",
                "QUOTED": "",
                "_UNKNOWN": "value",
            },
        )
        self.assertEqual(parse("A= # comment\nB=#comment\n", Path(".env")), {"A": "", "B": ""})

    def test_malformed_lines_identify_the_path_and_line(self) -> None:
        for line in (
            "A=1\nA=2",
            "export A=1",
            "no assignment",
            "1KEY=x",
            "A='unclosed",
            'A="bad\\q"',
            'A="unclosed',
            "A='closed' trailing",
            "A=one\x00two",
            "A='multi\nline'",
            "A=x\ry",
        ):
            with self.subTest(line=line), self.assertRaises(ValueError) as failed:
                parse("# Header\n" + line, Path("settings.env"))
            self.assertRegex(str(failed.exception), r"settings\.env:[23]:")

    def test_precedence_empty_values_and_no_environment_mutation(self) -> None:
        values = {"DOKIME_REMOTE": "http://file:1", "DOKIME_PLUGIN_ADDRESS": "file:2"}
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(resolve("DOKIME_REMOTE", None, values), "http://file:1")
            self.assertEqual(resolve("DOKIME_PLUGIN_ADDRESS", None, {}), "127.0.0.1:0")
            os.environ["DOKIME_REMOTE"] = "http://process:3"
            self.assertEqual(resolve("DOKIME_REMOTE", None, values), "http://process:3")
            self.assertEqual(resolve("DOKIME_REMOTE", "http://explicit:4", values), "http://explicit:4")
            os.environ["DOKIME_REMOTE"] = ""
            with self.assertRaises(ValueError):
                resolve("DOKIME_REMOTE", None, values)
            with self.assertRaises(ValueError):
                resolve("DOKIME_REMOTE", "", values)
            with self.assertRaises(ValueError):
                resolve("DOKIME_PLUGIN_ADDRESS", "", values)
        self.assertEqual(values["DOKIME_REMOTE"], "http://file:1")

    def test_load_reads_only_the_working_directory_and_refuses_invalid_utf8(self) -> None:
        directory = TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        child = root / "child"
        child.mkdir()
        (root / ".env").write_text("DOKIME_REMOTE=http://parent:1", encoding="utf-8")
        with patch("dokime_sdk._configuration.Path.cwd", return_value=child):
            self.assertEqual(load(), {})
            (child / ".env").write_text("DOKIME_REMOTE=http://child:2", encoding="utf-8-sig")
            self.assertEqual(load(), {"DOKIME_REMOTE": "http://child:2"})
            (child / ".env").write_bytes(b"KEY=ok\nKEY2=\xff")
            with self.assertRaises(ValueError):
                load()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
