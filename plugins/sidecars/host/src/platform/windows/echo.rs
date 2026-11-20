/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// ICMP Echo Through the IP Helper API, Without Starting ping

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::ffi::c_void;
use std::mem::size_of;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::time::Duration;

// Third Party Imports
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::IpHelper::{
    ICMP_ECHO_REPLY, ICMPV6_ECHO_REPLY_LH, IP_REQ_TIMED_OUT, IP_SUCCESS, Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle, IcmpCreateFile,
    IcmpSendEcho2,
};
use windows::Win32::Networking::WinSock::{AF_INET6, IN6_ADDR, IN6_ADDR_0, SOCKADDR_IN6};

// Local Imports

/* === Constants === */

/// Room for the ICMP Error Message and the IO_STATUS_BLOCK the API Appends.
const REPLY_SLACK_BYTES: usize = 8 + size_of::<windows::Win32::System::IO::IO_STATUS_BLOCK>();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct IcmpHandle(HANDLE);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for IcmpHandle {
    fn drop(&mut self) {
        // SAFETY: The handle came from IcmpCreateFile or Icmp6CreateFile and is closed once.
        let _ = unsafe { IcmpCloseHandle(self.0) };
    }
}

/* === Public Functions === */

/// The Round Trip in Milliseconds, or None When No Reply Arrived Within the Wait.
pub fn round_trip_ms(host: &str, timeout: Duration, payload_bytes: u32) -> Result<Option<f64>, String> {
    let address = (host, 0)
        .to_socket_addrs()
        .map_err(|error| format!("Resolve Latency Host {host}: {error}; Check the Latency Host Setting"))?
        .next()
        .ok_or_else(|| format!("Resolve Latency Host {host}: No Address; Check the Latency Host Setting"))?
        .ip();
    let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    let payload_bytes = u16::try_from(payload_bytes).map_err(|_| "Send ICMP Echo: Lower Echo Payload Below 65536 Bytes".to_owned())?;
    let payload = vec![b'a'; usize::from(payload_bytes)];
    let (status, round_trip) = match address {
        IpAddr::V4(address) => echo_v4(address, timeout_ms, &payload),
        IpAddr::V6(address) => echo_v6(address, timeout_ms, &payload),
    }
    .map_err(|error| format!("Send ICMP Echo to {host}: {error}"))?;
    // Any Status Other Than Success (Timeout, Unreachable, TTL Expired) Is a Lost Echo.
    if status != IP_SUCCESS {
        return Ok(None);
    }
    // A Sub-Millisecond Reply Reports 1 ms, as ping's "time<1ms" Did.
    Ok(Some(f64::from(round_trip.max(1))))
}

/* === Private Functions === */

fn echo_v4(address: Ipv4Addr, timeout_ms: u32, payload: &[u8]) -> windows::core::Result<(u32, u32)> {
    // SAFETY: Opening an ICMP handle has no preconditions; the guard closes it.
    let handle = IcmpHandle(unsafe { IcmpCreateFile()? });
    let destination = u32::from_ne_bytes(address.octets());
    let request = payload.as_ptr().cast::<c_void>();
    let mut reply = vec![0u8; size_of::<ICMP_ECHO_REPLY>() + payload.len() + REPLY_SLACK_BYTES];
    let output = reply.as_mut_ptr().cast::<c_void>();
    // SAFETY: The request and reply buffers outlive this synchronous call and their sizes are exact.
    let replies = unsafe {
        IcmpSendEcho2(handle.0, None, None, None, destination, request, payload.len() as u16, None, output, reply.len() as u32, timeout_ms)
    };
    if replies == 0 {
        return lost_or_failed();
    }
    // SAFETY: A successful call wrote one ICMP_ECHO_REPLY at the start of the buffer.
    let reply = unsafe { reply.as_ptr().cast::<ICMP_ECHO_REPLY>().read_unaligned() };
    Ok((reply.Status, reply.RoundTripTime))
}

fn echo_v6(address: Ipv6Addr, timeout_ms: u32, payload: &[u8]) -> windows::core::Result<(u32, u32)> {
    // SAFETY: Opening an ICMP handle has no preconditions; the guard closes it.
    let handle = IcmpHandle(unsafe { Icmp6CreateFile()? });
    let source = SOCKADDR_IN6 {
        sin6_family: AF_INET6,
        ..Default::default()
    };
    let destination = SOCKADDR_IN6 {
        sin6_family: AF_INET6,
        sin6_addr: IN6_ADDR { u: IN6_ADDR_0 { Byte: address.octets() } },
        ..Default::default()
    };
    let request = payload.as_ptr().cast::<c_void>();
    let mut reply = vec![0u8; size_of::<ICMPV6_ECHO_REPLY_LH>() + payload.len() + REPLY_SLACK_BYTES];
    let output = reply.as_mut_ptr().cast::<c_void>();
    let (request_size, reply_size) = (payload.len() as u16, reply.len() as u32);
    // SAFETY: The addresses and buffers outlive this synchronous call and their sizes are exact.
    let replies = unsafe {
        Icmp6SendEcho2(handle.0, None, None, None, &source, &destination, request, request_size, None, output, reply_size, timeout_ms)
    };
    if replies == 0 {
        return lost_or_failed();
    }
    // SAFETY: A successful call wrote one ICMPV6_ECHO_REPLY at the start of the buffer.
    let reply = unsafe { reply.as_ptr().cast::<ICMPV6_ECHO_REPLY_LH>().read_unaligned() };
    Ok((reply.Status, reply.RoundTripTime))
}

fn lost_or_failed() -> windows::core::Result<(u32, u32)> {
    let error = windows::core::Error::from_thread();
    if error.code() == windows::core::HRESULT::from_win32(IP_REQ_TIMED_OUT) {
        return Ok((IP_REQ_TIMED_OUT, 0));
    }
    Err(error)
}

/* === Tests === */
