/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One ICMP Echo Through The IP Helper API, Without Starting ping.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    ffi::c_void,
    mem::size_of,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs},
    time::Duration,
};

// Third Party Imports
use windows::Win32::{
    Foundation::HANDLE,
    NetworkManagement::IpHelper::{
        ICMP_ECHO_REPLY, ICMPV6_ECHO_REPLY_LH, IP_REQ_TIMED_OUT, IP_SUCCESS, Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle,
        IcmpCreateFile, IcmpSendEcho2,
    },
    Networking::WinSock::{AF_INET6, IN6_ADDR, IN6_ADDR_0, SOCKADDR_IN6},
};

// Local Imports

/* === Constants === */

// The Request Payload Matches ping's Default 32 Bytes.
const PAYLOAD: [u8; 32] = [b'a'; 32];

// Room For The ICMP Error Message And The IO_STATUS_BLOCK The API Appends.
const REPLY_SLACK_BYTES: usize = 64;

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

// The Round Trip In Milliseconds, Or None When No Reply Arrived Within The Wait.
pub fn round_trip_ms(host: &str, timeout: Duration) -> Result<Option<f64>, String> {
    let address = (host, 0)
        .to_socket_addrs()
        .map_err(|error| format!("{host} could not be resolved: {error}"))?
        .next()
        .ok_or_else(|| format!("{host} resolved to no address"))?
        .ip();
    let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    let (status, round_trip) = match address {
        IpAddr::V4(address) => echo_v4(address, timeout_ms),
        IpAddr::V6(address) => echo_v6(address, timeout_ms),
    }
    .map_err(|error| format!("echo to {host} failed: {error}"))?;
    // Any Status Other Than Success (Timeout, Unreachable, TTL Expired) Is A Lost Echo.
    if status != IP_SUCCESS {
        return Ok(None);
    }
    // A Sub-Millisecond Reply Reports 1 ms, As ping's "time<1ms" Did.
    Ok(Some(f64::from(round_trip.max(1))))
}

/* === Private Functions === */

fn echo_v4(address: Ipv4Addr, timeout_ms: u32) -> windows::core::Result<(u32, u32)> {
    // SAFETY: Opening an ICMP handle has no preconditions; the guard closes it.
    let handle = IcmpHandle(unsafe { IcmpCreateFile()? });
    let destination = u32::from_ne_bytes(address.octets());
    let request = PAYLOAD.as_ptr().cast::<c_void>();
    let mut reply = vec![0u8; size_of::<ICMP_ECHO_REPLY>() + PAYLOAD.len() + REPLY_SLACK_BYTES];
    let output = reply.as_mut_ptr().cast::<c_void>();
    // SAFETY: The request and reply buffers outlive this synchronous call and their sizes are exact.
    let replies = unsafe {
        IcmpSendEcho2(handle.0, None, None, None, destination, request, PAYLOAD.len() as u16, None, output, reply.len() as u32, timeout_ms)
    };
    if replies == 0 {
        return lost_or_failed();
    }
    // SAFETY: A successful call wrote one ICMP_ECHO_REPLY at the start of the buffer.
    let reply = unsafe { reply.as_ptr().cast::<ICMP_ECHO_REPLY>().read_unaligned() };
    Ok((reply.Status, reply.RoundTripTime))
}

fn echo_v6(address: Ipv6Addr, timeout_ms: u32) -> windows::core::Result<(u32, u32)> {
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
    let request = PAYLOAD.as_ptr().cast::<c_void>();
    let mut reply = vec![0u8; size_of::<ICMPV6_ECHO_REPLY_LH>() + PAYLOAD.len() + REPLY_SLACK_BYTES];
    let output = reply.as_mut_ptr().cast::<c_void>();
    let (request_size, reply_size) = (PAYLOAD.len() as u16, reply.len() as u32);
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

#[cfg(test)]
mod tests {
    #[test]
    fn loopback_echo_answers_without_a_process() {
        let latency = super::round_trip_ms("127.0.0.1", std::time::Duration::from_secs(1)).expect("loopback echo runs");
        assert!(latency.expect("loopback answers") >= 1.0);
    }
}
