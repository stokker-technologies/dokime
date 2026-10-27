/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Disk-Grouped WiFi Occupancy Analysis

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use rusqlite::params;

// Local Imports
use super::super::analyses::{PlotAnalysis, wifi};
use super::{
    input::{Read, failure},
    series::Series,
};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn reset(read: &Read<'_>) -> Result<()> {
    read.connection.execute_batch(
        "DROP TABLE IF EXISTS wifi_channels;
         DROP TABLE IF EXISTS wifi_scans;
         CREATE TABLE wifi_channels(channel INTEGER PRIMARY KEY,occupied INTEGER NOT NULL);
         CREATE TABLE wifi_scans(second INTEGER NOT NULL,channel INTEGER NOT NULL,time TEXT NOT NULL,count BLOB NOT NULL,PRIMARY KEY(second,channel));"
    ).map_err(failure)
}

pub(super) fn append(read: &Read<'_>, series: &Series) -> Result<()> {
    let Some(channel) = wifi::series_channel(&series.metadata) else {
        return Ok(());
    };
    read.connection.execute("INSERT OR IGNORE INTO wifi_channels VALUES (?1,0)", params![channel]).map_err(failure)?;
    series.visit(|point| {
        let Some((second, count)) = wifi::reading(&point) else {
            return Ok(());
        };
        let time = wifi::time(&point).map_err(failure)?;
        let bytes = (count as u64).to_be_bytes();
        read.connection
            .execute(
                "INSERT INTO wifi_scans VALUES (?1,?2,?3,?4) ON CONFLICT(second,channel) DO UPDATE SET count=MAX(count,excluded.count)",
                params![second, channel, time, bytes.as_slice()],
            )
            .map_err(failure)?;
        if count > 0 {
            read.connection.execute("UPDATE wifi_channels SET occupied=1 WHERE channel=?1", params![channel]).map_err(failure)?;
        }
        Ok(())
    })
}

pub(super) fn finish(read: &Read<'_>) -> Result<PlotAnalysis> {
    let mut statement = read.connection.prepare("SELECT channel,occupied FROM wifi_channels ORDER BY channel").map_err(failure)?;
    let mut rows = statement.query([]).map_err(failure)?;
    let mut channels = Vec::new();
    let mut occupied_channels = Vec::new();
    while let Some(row) = rows.next().map_err(failure)? {
        let channel = row.get(0).map_err(failure)?;
        channels.push(channel);
        if row.get::<_, bool>(1).map_err(failure)? {
            occupied_channels.push(channel);
        }
    }
    let bands = match wifi::window_bands(&occupied_channels) {
        Ok(bands) => bands,
        Err(message) => return Ok(PlotAnalysis::Error { message: message.into() }),
    };
    let mut statement = read.connection.prepare("SELECT second,channel,(SELECT first.time FROM wifi_scans AS first WHERE first.second=wifi_scans.second ORDER BY first.rowid LIMIT 1),count FROM wifi_scans ORDER BY second,channel").map_err(failure)?;
    let mut rows = statement.query([]).map_err(failure)?;
    let mut current = None;
    let mut time = String::new();
    let mut counts = BTreeMap::new();
    let mut scans = Vec::new();
    while let Some(row) = rows.next().map_err(failure)? {
        let second: i64 = row.get(0).map_err(failure)?;
        if current.is_some_and(|current| current != second) {
            let scan = wifi::scan(std::mem::take(&mut time), std::mem::take(&mut counts));
            match scan {
                Ok(scan) => scans.push(scan),
                Err(message) => return Ok(PlotAnalysis::Error { message: message.into() }),
            }
        }
        if current != Some(second) {
            time = row.get(2).map_err(failure)?;
        }
        current = Some(second);
        let bytes: Vec<u8> = row.get(3).map_err(failure)?;
        let bytes = bytes.try_into().map_err(|_| failure("invalid WiFi count"))?;
        counts.insert(row.get(1).map_err(failure)?, u64::from_be_bytes(bytes) as usize);
    }
    if current.is_some() {
        match wifi::scan(time, counts) {
            Ok(scan) => scans.push(scan),
            Err(message) => return Ok(PlotAnalysis::Error { message: message.into() }),
        }
    }
    Ok(PlotAnalysis::WifiOccupancy { channels, occupied_channels, bands, scans })
}

/* === Private Functions === */

/* === Tests === */
