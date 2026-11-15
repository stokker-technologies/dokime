# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Windows Story Process Trees"""

# === Imports === #

# Standard Library Imports
import ctypes
import os
import subprocess
import sys
from ctypes import wintypes

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

_HANDLE = None

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


# === Public Functions === #


def own_children() -> None:
    """Kill Every Descendant When This Runner Exits, Including Forced Termination."""
    global _HANDLE
    if os.name != "nt" or _HANDLE is not None:
        return
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
    kernel.CreateJobObjectW.restype = wintypes.HANDLE
    kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    kernel.SetInformationJobObject.restype = wintypes.BOOL
    kernel.GetCurrentProcess.argtypes = []
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
    kernel.AssignProcessToJobObject.restype = wintypes.BOOL
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    handle = kernel.CreateJobObjectW(None, None)
    if not handle:
        raise OSError(ctypes.get_last_error(), "Own Story Children: Create Job")
    limits = _Limits()
    limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
    configured = kernel.SetInformationJobObject(handle, 9, ctypes.byref(limits), ctypes.sizeof(limits))
    assigned = configured and kernel.AssignProcessToJobObject(handle, kernel.GetCurrentProcess())
    if not assigned:
        error = ctypes.get_last_error()
        kernel.CloseHandle(handle)
        raise OSError(error, "Own Story Children: Configure and Assign Job")
    _HANDLE = handle  # Never Inherited; the OS Closes It When the Runner Dies.


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    own_children()
    arguments = sys.argv[1:]
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    if not arguments:
        raise SystemExit("Run Owned Story: Supply Its Command")
    # Exit Closes the Job Before the Outer Runner Cleans the Story's Scratch.
    with subprocess.Popen(arguments) as child:
        raise SystemExit(child.wait())
