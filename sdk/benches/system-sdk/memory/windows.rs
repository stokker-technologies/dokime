/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Windows Plugin Resident Memory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, mem::size_of};

// Third Party Imports
use windows_sys::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS},
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn resident(pid: u32) -> Option<f64> {
    // Include Interpreter Children Created by the Windows Virtual Environment Launcher.
    // Each Snapshot and Process Handle Is Owned Here and Closed After Inspection.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut children = BTreeMap::<u32, Vec<u32>>::new();
        if Process32FirstW(snapshot, &mut entry) == 0 {
            CloseHandle(snapshot);
            return None;
        }
        loop {
            children.entry(entry.th32ParentProcessID).or_default().push(entry.th32ProcessID);
            if Process32NextW(snapshot, &mut entry) == 0 {
                break;
            }
        }
        CloseHandle(snapshot);
        let mut pending = vec![pid];
        let mut resident = 0;
        while let Some(pid) = pending.pop() {
            pending.extend(children.remove(&pid).unwrap_or_default());
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
            if process.is_null() {
                return None;
            }
            let mut counters = PROCESS_MEMORY_COUNTERS {
                cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                ..Default::default()
            };
            let size = counters.cb;
            let success = GetProcessMemoryInfo(process, &mut counters, size);
            CloseHandle(process);
            if success == 0 {
                return None;
            }
            resident += counters.WorkingSetSize;
        }
        Some(resident as f64)
    }
}

/* === Private Functions === */

/* === Tests === */
