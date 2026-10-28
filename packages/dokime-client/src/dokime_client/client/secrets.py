# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Validate Remote Credential Bytes Without Exposing Their Contents"""

_MIN_REMOTE_KEY_BYTES = 32
_MAX_REMOTE_KEY_BYTES = 4096


def _validate_key(key: str | None) -> None:
    if key is None:
        return
    if not isinstance(key, str) or not _MIN_REMOTE_KEY_BYTES <= len(key) <= _MAX_REMOTE_KEY_BYTES:
        raise ValueError("remote key does not match its credential format")
    if not all(33 <= ord(character) <= 126 for character in key):
        raise ValueError("remote key does not match its credential format")
