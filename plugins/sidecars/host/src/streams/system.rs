/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host System Collection: One Reading per Tick, Fanned Out by Availability

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::domain::readings::System;
use crate::domain::telemetry::{
    Battery, BatteryCycles, BatteryIdentity, BatteryTemperature, BatteryTimeToEmpty, BatteryTimeToFull, CpuCapacity, PowerPlan,
    SYSTEM_PROBE, SystemGit, SystemIdentity, SystemResources, SystemTemperature,
};
use crate::settings;
use crate::streams;
use dokime::Result;
use dokime::operations::Context;
use dokime::plugin::{Collection, CollectionOutput, Plugin};
use harmos::plugin::Stream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the System Collection and Every Stream It Feeds.
pub fn catalog(plugin: Plugin, host: Arc<Host>) -> Result<Plugin> {
    let collection = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect(&ctx, &rows).await
        }
    })
    .stream(streams::probe(SYSTEM_PROBE))
    .stream(Stream::<SystemIdentity>::new(SystemIdentity::STREAM).description(SystemIdentity::LABEL))
    .stream(Stream::<SystemGit>::new(SystemGit::STREAM).description(SystemGit::LABEL))
    .stream(Stream::<SystemResources>::new(SystemResources::STREAM).description(SystemResources::LABEL))
    .stream(Stream::<SystemTemperature>::new(SystemTemperature::STREAM).description(SystemTemperature::LABEL))
    .stream(Stream::<CpuCapacity>::new(CpuCapacity::STREAM).description(CpuCapacity::LABEL))
    .stream(Stream::<Battery>::new(Battery::STREAM).description(Battery::LABEL))
    .stream(Stream::<BatteryIdentity>::new(BatteryIdentity::STREAM).description(BatteryIdentity::LABEL))
    .stream(Stream::<BatteryTemperature>::new(BatteryTemperature::STREAM).description(BatteryTemperature::LABEL))
    .stream(Stream::<BatteryTimeToFull>::new(BatteryTimeToFull::STREAM).description(BatteryTimeToFull::LABEL))
    .stream(Stream::<BatteryTimeToEmpty>::new(BatteryTimeToEmpty::STREAM).description(BatteryTimeToEmpty::LABEL))
    .stream(Stream::<BatteryCycles>::new(BatteryCycles::STREAM).description(BatteryCycles::LABEL))
    .stream(Stream::<PowerPlan>::new(PowerPlan::STREAM).description(PowerPlan::LABEL))
    .interval_setting(settings::SYSTEM_INTERVAL)
    .on_start({
        let host = host.clone();
        async move |_| host.system.start()
    })
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.system.stop();
            Ok(())
        }
    });
    Ok(plugin.collection(collection))
}

/// Publish Each Group of a Reading Whose Fields Were All Read; the Outcome Row Always.
pub fn publish(reading: System, rows: &CollectionOutput) {
    streams::outcome(rows, SYSTEM_PROBE, reading.probe_status, reading.probe_error);
    let identity = (
        reading.host_name,
        reading.user,
        reading.os_name,
        reading.os_version,
        reading.kernel_version,
        reading.cpu_architecture,
        reading.cpu_name,
        reading.cpu_brand,
        reading.cpu_vendor_id,
        reading.cpu_frequency_mhz,
        reading.physical_cpu_count,
        reading.logical_cpu_count,
    );
    if let (
        Some(host_name),
        Some(user),
        Some(os_name),
        Some(os_version),
        Some(kernel_version),
        Some(cpu_architecture),
        Some(cpu_name),
        Some(cpu_brand),
        Some(cpu_vendor_id),
        Some(cpu_frequency_mhz),
        Some(physical_cpu_count),
        Some(logical_cpu_count),
    ) = identity
    {
        let row = SystemIdentity {
            host_name,
            user,
            os_name,
            os_version,
            kernel_version,
            cpu_architecture,
            cpu_name,
            cpu_brand,
            cpu_vendor_id,
            cpu_frequency_mhz,
            physical_cpu_count,
            logical_cpu_count,
        };
        rows.publish(&Stream::new(SystemIdentity::STREAM), row);
    }
    if let (Some(git_user_name), Some(git_user_email)) = (reading.git_user_name, reading.git_user_email) {
        rows.publish(&Stream::new(SystemGit::STREAM), SystemGit { git_user_name, git_user_email });
    }
    let resources = (
        reading.cpu_load_percent,
        reading.memory_total_bytes,
        reading.memory_used_bytes,
        reading.memory_pressure_percent,
        reading.memory_used_gib,
        reading.memory_total_gib,
        reading.system_uptime_s,
        reading.temperature_sensor_count,
    );
    if let (
        Some(cpu_load_percent),
        Some(memory_total_bytes),
        Some(memory_used_bytes),
        Some(memory_pressure_percent),
        Some(memory_used_gib),
        Some(memory_total_gib),
        Some(system_uptime_s),
        Some(temperature_sensor_count),
    ) = resources
    {
        let row = SystemResources {
            cpu_load_percent,
            memory_total_bytes,
            memory_used_bytes,
            memory_pressure_percent,
            memory_used_gib,
            memory_total_gib,
            system_uptime_s,
            temperature_sensor_count,
        };
        rows.publish(&Stream::new(SystemResources::STREAM), row);
    }
    if let Some(max_temperature_c) = reading.max_temperature_c {
        rows.publish(&Stream::new(SystemTemperature::STREAM), SystemTemperature { max_temperature_c });
    }
    if let (Some(cpu_capacity_gflops), Some(cpu_load_gflops)) = (reading.cpu_capacity_gflops, reading.cpu_load_gflops) {
        rows.publish(&Stream::new(CpuCapacity::STREAM), CpuCapacity { cpu_capacity_gflops, cpu_load_gflops });
    }
    let battery = (
        reading.battery_percent,
        reading.battery_state,
        reading.battery_power_plugged,
        reading.battery_technology,
        reading.battery_state_of_charge_ratio,
        reading.battery_energy_wh,
        reading.battery_energy_full_wh,
        reading.battery_energy_full_design_wh,
        reading.battery_energy_rate_w,
        reading.battery_voltage_v,
        reading.battery_state_of_health_ratio,
    );
    if let (
        Some(battery_percent),
        Some(battery_state),
        Some(battery_power_plugged),
        Some(battery_technology),
        Some(battery_state_of_charge_ratio),
        Some(battery_energy_wh),
        Some(battery_energy_full_wh),
        Some(battery_energy_full_design_wh),
        Some(battery_energy_rate_w),
        Some(battery_voltage_v),
        Some(battery_state_of_health_ratio),
    ) = battery
    {
        let row = Battery {
            battery_percent,
            battery_state,
            battery_power_plugged,
            battery_technology,
            battery_state_of_charge_ratio,
            battery_energy_wh,
            battery_energy_full_wh,
            battery_energy_full_design_wh,
            battery_energy_rate_w,
            battery_voltage_v,
            battery_state_of_health_ratio,
        };
        rows.publish(&Stream::new(Battery::STREAM), row);
    }
    if let (Some(battery_vendor), Some(battery_model), Some(battery_serial_number)) =
        (reading.battery_vendor, reading.battery_model, reading.battery_serial_number)
    {
        rows.publish(
            &Stream::new(BatteryIdentity::STREAM),
            BatteryIdentity {
                battery_vendor,
                battery_model,
                battery_serial_number,
            },
        );
    }
    if let Some(battery_temperature_c) = reading.battery_temperature_c {
        rows.publish(&Stream::new(BatteryTemperature::STREAM), BatteryTemperature { battery_temperature_c });
    }
    if let Some(battery_time_to_full_s) = reading.battery_time_to_full_s {
        rows.publish(&Stream::new(BatteryTimeToFull::STREAM), BatteryTimeToFull { battery_time_to_full_s });
    }
    if let Some(battery_time_to_empty_s) = reading.battery_time_to_empty_s {
        rows.publish(&Stream::new(BatteryTimeToEmpty::STREAM), BatteryTimeToEmpty { battery_time_to_empty_s });
    }
    if let Some(battery_cycle_count) = reading.battery_cycle_count {
        rows.publish(&Stream::new(BatteryCycles::STREAM), BatteryCycles { battery_cycle_count });
    }
    if let (Some(power_plan_name), Some(power_plan_guid), Some(power_mode), Some(power_mode_performance_bias)) =
        (reading.power_plan_name, reading.power_plan_guid, reading.power_mode, reading.power_mode_performance_bias)
    {
        rows.publish(
            &Stream::new(PowerPlan::STREAM),
            PowerPlan {
                power_plan_name,
                power_plan_guid,
                power_mode,
                power_mode_performance_bias,
            },
        );
    }
}

/* === Private Functions === */

async fn collect(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let reading = ctx.plugin::<Arc<Host>>()?.system.collect(ctx).await?;
    publish(reading, rows);
    Ok(())
}

/* === Tests === */
