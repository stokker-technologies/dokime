# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Mechanical Release Downloads Fixtures."""

# === Imports === #

# Standard Library Imports

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #


# === Constants === #

SETTINGS = {"product": "dokime", "provider": "github", "repository": "example/dokime"}


EXECUTABLES = {"dokime-cli": "cli", "companion-tool": "companion-tool"}


EXPECTED = {
    "CHANGELOG.md": ("changelog", "any", "any", "md"),
    "SHA256SUMS": ("checksums", "any", "any", "txt"),
    "companion-tool-0.1.3-x86_64-pc-windows-msvc.zip": ("companion-tool", "windows", "x86_64", "zip"),
    "companion-tool-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("companion-tool", "linux", "x86_64", "tar.gz"),
    "dokime-0.1.3-1.x86_64.rpm": ("installer", "linux", "x86_64", "rpm"),
    "dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe": ("installer", "windows", "x86_64", "exe"),
    "dokime-cli-0.1.3-x86_64-pc-windows-msvc.zip": ("cli", "windows", "x86_64", "zip"),
    "dokime-cli-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("cli", "linux", "x86_64", "tar.gz"),
    "dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip": ("payload", "windows", "x86_64", "zip"),
    "dokime-distribution-0.1.3-x86_64-unknown-linux-gnu.zip": ("payload", "linux", "x86_64", "zip"),
    "dokime-guests-0.1.3.zip": ("plugin", "any", "any", "zip"),
    "dokime-plugins-0.1.3-x86_64-pc-windows-msvc.zip": ("plugin", "windows", "x86_64", "zip"),
    "dokime-plugins-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("plugin", "linux", "x86_64", "tar.gz"),
    "dokime_0.1.3_amd64.deb": ("installer", "linux", "x86_64", "deb"),
    "dokime_sdk-0.1.3-py3-none-any.whl": ("sdk-python", "any", "any", "whl"),
    "dokime_sdk-0.1.3.tar.gz": ("sdk-python", "any", "any", "tar.gz"),
    "dokime-compliance-0.1.3.zip": ("compliance", "any", "any", "zip"),
    "dokime-proto-0.1.3.zip": ("proto", "any", "any", "zip"),
    "dokime-reports-0.1.3.zip": ("reports", "any", "any", "zip"),
    "unexpected.bin": ("other", "any", "any", "bin"),
}

# === Types === #


# === Classes === #


# === Public Functions === #


# === Private Functions === #


# === Entry Point === #
