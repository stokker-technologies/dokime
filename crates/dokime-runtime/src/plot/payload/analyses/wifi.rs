/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, btree_map::Entry};

// Third Party Imports

// Local Imports
use super::{PlotAnalysis, PlotPoint, PlotSeries, WifiBandScan, WifiBandWindow, WifiChannelCount, WifiScan, point_x};

/* === Constants === */

const BANDS: [&str; 3] = ["2.4 GHz", "5 GHz", "6 GHz"];
const ALLOCATION: &str = "WiFi occupancy allocation failed";
const OVERFLOW: &str = "WiFi occupancy count or size overflows";
const ENCODING: &str = "WiFi occupancy time encoding failed";

/* === Macros === */

/* === Type Aliases === */

type Channel = (i64, bool);
type Counts = BTreeMap<i64, usize>;
type Scans = BTreeMap<i64, (String, Counts)>;

/* === Structs === */

struct Count(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::io::Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.checked_add(bytes.len()).ok_or_else(|| std::io::Error::other(OVERFLOW))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn analyze(series: &[PlotSeries]) -> Result<PlotAnalysis, &'static str> {
    let mut channels = channel_slots(series)?;
    let mut scans = Scans::new();
    for entry in series {
        let Some(channel) = series_channel(entry) else {
            continue;
        };
        let slot = channels.binary_search_by_key(&channel, |&(id, _)| id).map_err(|_| OVERFLOW)?;
        for point in &entry.points {
            let Some((second, count)) = reading(point) else {
                continue;
            };
            let (_, counts) = match scans.entry(second) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert((time(point)?, Counts::new())),
            };
            let retained = counts.entry(channel).or_default();
            *retained = (*retained).max(count);
            channels[slot].1 |= count > 0;
        }
    }

    let mut all_channels = slots(channels.len())?;
    let mut occupied_channels = slots(channels.iter().filter(|(_, occupied)| *occupied).count())?;
    for &(channel, occupied) in &channels {
        all_channels.push(channel);
        if occupied {
            occupied_channels.push(channel);
        }
    }
    let bands = window_bands(&occupied_channels)?;
    let mut output = slots(scans.len())?;
    for (_, (time, counts)) in scans {
        output.push(scan(time, counts)?);
    }
    Ok(PlotAnalysis::WifiOccupancy { channels: all_channels, occupied_channels, bands, scans: output })
}

pub(in crate::plot::payload) fn reading(point: &PlotPoint) -> Option<(i64, usize)> {
    let second = point_x(point)?.floor() as i64;
    let count = point.y.filter(|value| value.is_finite()).map_or(0, |value| value.round().max(0.0) as usize);
    Some((second, count))
}

pub(in crate::plot::payload) fn window_bands(occupied: &[i64]) -> Result<Vec<WifiBandWindow>, &'static str> {
    let count = (0..BANDS.len()).filter(|&band| occupied.iter().any(|&channel| band_index(channel) == Some(band))).count();
    let mut output = slots(count)?;
    for (band, name) in BANDS.iter().enumerate() {
        let occupied_channels = occupied.iter().filter(|&&channel| band_index(channel) == Some(band)).count();
        if occupied_channels > 0 {
            output.push(WifiBandWindow { band: text(name)?, occupied_channels });
        }
    }
    Ok(output)
}

pub(in crate::plot::payload) fn scan(time: String, counts: Counts) -> Result<WifiScan, &'static str> {
    let (occupied_channels, total_networks, peak_count) = occupancy(counts.values())?;
    let present = (0..BANDS.len()).filter(|&band| counts.keys().any(|&channel| band_index(channel) == Some(band))).count();
    let mut bands = slots(present)?;
    for (band, name) in BANDS.iter().enumerate() {
        let mut in_band = counts.iter().filter(|(channel, _)| band_index(**channel) == Some(band)).map(|(_, count)| count).peekable();
        if in_band.peek().is_none() {
            continue;
        }
        let (occupied_channels, total_networks, peak_count) = occupancy(in_band)?;
        bands.push(WifiBandScan { band: text(name)?, occupied_channels, total_networks, peak_count });
    }
    let mut channels = slots(counts.len())?;
    for (channel, count) in counts {
        channels.push(WifiChannelCount { channel, count });
    }
    Ok(WifiScan { time, channels, bands, occupied_channels, total_networks, peak_count })
}

pub(in crate::plot::payload) fn time(point: &PlotPoint) -> Result<String, &'static str> {
    if let Some(value) = point.x.as_str() {
        return text(value);
    }
    let mut output = slots(time_length(point)?)?;
    serde_json::to_writer(&mut output, &point.x).map_err(|_| ENCODING)?;
    String::from_utf8(output).map_err(|_| ENCODING)
}

pub(in crate::plot::payload) fn series_channel(entry: &PlotSeries) -> Option<i64> {
    let channel =
        |text: &str| text.trim().parse::<f64>().ok().filter(|value| value.is_finite() && value.fract() == 0.0).map(|value| value as i64);
    channel(&entry.label).or_else(|| entry.key.rsplit(':').next().and_then(channel))
}

/* === Private Functions === */

fn channel_slots(series: &[PlotSeries]) -> Result<Vec<Channel>, &'static str> {
    let count = series.iter().filter(|entry| series_channel(entry).is_some()).count();
    let mut channels = slots(count)?;
    for entry in series {
        if let Some(channel) = series_channel(entry) {
            channels.push((channel, false));
        }
    }
    channels.sort_unstable_by_key(|&(channel, _)| channel);
    channels.dedup_by_key(|entry| entry.0);
    Ok(channels)
}

fn occupancy<'a>(mut counts: impl Iterator<Item = &'a usize>) -> Result<(usize, usize, usize), &'static str> {
    counts.try_fold((0usize, 0usize, 0usize), |(occupied, total, peak), &count| {
        let occupied = occupied.checked_add(usize::from(count > 0)).ok_or(OVERFLOW)?;
        let total = total.checked_add(count).ok_or(OVERFLOW)?;
        Ok((occupied, total, peak.max(count)))
    })
}

fn time_length(point: &PlotPoint) -> Result<usize, &'static str> {
    if let Some(text) = point.x.as_str() {
        return Ok(text.len());
    }
    let mut output = Count(0);
    serde_json::to_writer(&mut output, &point.x).map_err(|_| ENCODING)?;
    Ok(output.0)
}

fn text(value: &str) -> Result<String, &'static str> {
    let mut output = String::new();
    output.try_reserve_exact(value.len()).map_err(|_| ALLOCATION)?;
    output.push_str(value);
    Ok(output)
}

fn slots<T>(count: usize) -> Result<Vec<T>, &'static str> {
    let mut output = Vec::new();
    output.try_reserve_exact(count).map_err(|_| ALLOCATION)?;
    Ok(output)
}

fn band_index(channel: i64) -> Option<usize> {
    match channel {
        1..=14 => Some(0),
        32..=177 => Some(1),
        1_001..=1_233 if (channel - 1_000) % 4 == 1 => Some(2),
        _ => None,
    }
}

/* === Tests === */

#[cfg(feature = "storage")]
#[cfg(test)]
mod tests {
    use super::*;

    fn series(label: &str, points: Vec<PlotPoint>) -> PlotSeries {
        PlotSeries {
            key: format!("signal:{label}"),
            label: label.into(),
            unit: None,
            raw_points: points.len(),
            points,
            downsampled: false,
            format: None,
        }
    }

    fn point(x: &str, y: Option<f64>) -> PlotPoint {
        PlotPoint { x: x.into(), y, z: None }
    }

    #[test]
    fn first_time_maximum_and_zero_bands() {
        let input = [
            series("6", vec![point("1.9", Some(1.0)), point("1.1", Some(5.0)), point("2", Some(0.0))]),
            series("36", vec![point("1.0", Some(0.0))]),
            series("1005", Vec::new()),
        ];
        let PlotAnalysis::WifiOccupancy { channels, occupied_channels, bands, scans } = analyze(&input).unwrap() else {
            panic!("wifi");
        };
        assert_eq!(channels, [6, 36, 1005]);
        assert_eq!(occupied_channels, [6]);
        assert_eq!(bands.len(), 1);
        assert_eq!(scans[0].time, "1.9");
        assert_eq!(scans[0].channels, [WifiChannelCount { channel: 6, count: 5 }, WifiChannelCount { channel: 36, count: 0 }]);
        assert_eq!((scans[0].occupied_channels, scans[0].total_networks, scans[0].peak_count), (1, 5, 5));
        assert_eq!(scans[0].bands.len(), 2);
        assert_eq!(scans[0].bands[1].total_networks, 0);
        assert_eq!(scans[1].occupied_channels, 0);
    }

    #[test]
    fn empty_and_nonfinite_rows() {
        let empty = analyze(&[]).unwrap();
        assert!(matches!(empty, PlotAnalysis::WifiOccupancy { scans, channels, .. } if scans.is_empty() && channels.is_empty()));
        let input = [series("1", vec![point("-0.1", None), point("-0.9", Some(f64::INFINITY)), point("invalid", Some(7.0))])];
        let PlotAnalysis::WifiOccupancy { occupied_channels, scans, .. } = analyze(&input).unwrap() else {
            panic!("wifi");
        };
        assert!(occupied_channels.is_empty());
        assert_eq!(scans.len(), 1);
        assert_eq!(scans[0].time, "-0.1");
        assert_eq!(scans[0].total_networks, 0);
    }

    #[test]
    fn numeric_time_and_checked_total() {
        let input = [series("1", vec![PlotPoint { x: 1.25.into(), y: Some(1.6), z: None }])];
        let PlotAnalysis::WifiOccupancy { scans, .. } = analyze(&input).unwrap() else {
            panic!("wifi");
        };
        assert_eq!(scans[0].time, "1.25");
        assert_eq!(scans[0].total_networks, 2);
        let overflow = [series("1", vec![point("1", Some(f64::MAX))]), series("6", vec![point("1", Some(1.0))])];
        assert!(analyze(&overflow).is_err());
    }
}
