mod cli;
mod device;
mod discovery;

use std::path::Path;
use std::time::{Duration, Instant};

use once_cell::unsync::Lazy;

use crate::cli::{slight_command, Action, Value};
use crate::device::{Backlight, BacklightDevice, Brightness, LedDevice};
use crate::discovery::{Capability, CapabilityCheckError, DeviceDetail};

use self::cli::InterpolationOptions;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to find a default device")]
    NoDefaultDevice,
    #[error("reading device attribute '{0}' failed: {1}")]
    DeviceReadFailed(&'static str, Box<dyn std::error::Error>),
    #[error("writing device attribute '{0}' failed: {1}")]
    DeviceWriteFailed(&'static str, std::io::Error),
    #[error("the arguments are incorrect: {0}")]
    MalformedArguments(Box<dyn std::error::Error>),
}

const CONFLICT_INCREASE_DECREASE: &str =
    "cannot specify increase (-I) and decrease (-D) at the same time";
const CURRENT_BRIGHTNESS_GREATER: &str = "current brightness is greater than target, doing nothing";
const CURRENT_BRIGHTNESS_LESS: &str = "current brightness is less than target, doing nothing";

const DEFAULT_DEVICE_PATHS: &[&str; 2] = &["/sys/class/backlight", "/sys/class/leds"];

fn main() -> Result<()> {
    let args = slight_command().run();

    let found_devices = Lazy::<Vec<DeviceDetail>>::new(find_devices);

    fn default_device(found: Lazy<Vec<DeviceDetail>>) -> Result<DeviceDetail> {
        Lazy::force(&found);
        let defaults = Lazy::into_value(found).unwrap();
        defaults.into_iter().next().ok_or(Error::NoDefaultDevice)
    }

    let verbose = args.verbose;

    match args.command {
        Action::List { paths } => {
            for device in found_devices.iter() {
                if paths {
                    println!("{}", device.path.display());
                } else {
                    println!("{}", device.name);
                }
            }
            Ok(())
        }
        Action::Get { percent } => {
            let device = args.device.unwrap_or(default_device(found_devices)?.path);
            let device = LedDevice::new(device);
            let current = read_brightness(&device)?;
            let current = Value::Absolute(current);
            if percent {
                let max = read_max_brightness(&device)?;
                let actual = current.as_percent(max);
                println!("{actual}");
            } else {
                println!("{current}");
            }
            Ok(())
        }
        Action::Set {
            value,
            increase,
            decrease,
            interpolate:
                InterpolationOptions {
                    duration,
                    frequency,
                },
        } => {
            if increase && decrease {
                return Err(Error::MalformedArguments(CONFLICT_INCREASE_DECREASE.into()));
            }

            let device = args.device.unwrap_or(default_device(found_devices)?.path);
            let device = LedDevice::new(device);
            let max = read_max_brightness(&device)?;
            let current = read_brightness(&device)?;
            let target = value.to_absolute(max);

            if target == current {
                Ok(())
            } else if increase && target < current {
                eprintln!("{CURRENT_BRIGHTNESS_GREATER}");
                Ok(())
            } else if decrease && target > current {
                eprintln!("{CURRENT_BRIGHTNESS_LESS}");
                Ok(())
            } else {
                set_brightness(&device, current, target, duration.0, frequency, max)
            }
        }
        Action::Increase {
            amount,
            interpolate:
                InterpolationOptions {
                    duration,
                    frequency,
                },
        } => {
            let device = args.device.unwrap_or(default_device(found_devices)?.path);
            let device = LedDevice::new(device);
            let max = read_max_brightness(&device)?;
            let amount = amount.to_absolute(max);
            let current = read_brightness(&device)?;
            let target = (current + amount).clamp(0, max);

            set_brightness(&device, current, target, duration.0, frequency, amount)
        }
        Action::Decrease {
            amount,
            interpolate:
                InterpolationOptions {
                    duration,
                    frequency,
                },
        } => {
            let device = args.device.unwrap_or(default_device(found_devices)?.path);
            let device = LedDevice::new(device);
            let max = read_max_brightness(&device)?;
            let amount = amount.to_absolute(max);
            let current = read_brightness(&device)?;
            let target = (current - amount).clamp(0, max);

            set_brightness(&device, current, target, duration.0, frequency, amount)
        }
    }
}

fn find_devices() -> Vec<DeviceDetail> {
    DEFAULT_DEVICE_PATHS
        .iter()
        .flat_map(|path| Path::new(path).read_dir())
        .flatten()
        .filter_map(|res| res.ok())
        .filter_map(|entry| DeviceDetail::try_from(entry.path()).ok())
        .collect()
}

fn read_brightness(device: &dyn Brightness) -> Result<u32> {
    device
        .brightness()
        .map_err(|e| Error::DeviceReadFailed("brightness", e.into()))
}

fn read_max_brightness(device: &dyn Brightness) -> Result<u32> {
    device
        .max_brightness()
        .map_err(|e| Error::DeviceReadFailed("max_brightness", e.into()))
}

fn write_brightness(device: &dyn Brightness, value: u32) -> Result<()> {
    device
        .set_brightness(value)
        .map_err(|e| Error::DeviceWriteFailed("brightness", e))
}

fn set_brightness(
    device: &dyn Brightness,
    current: u32,
    target: u32,
    duration: Duration,
    frequency: u32,
    basis: u32,
) -> Result<()> {
    if duration.is_zero() {
        write_brightness(device, target)
    } else {
        let delta = current.abs_diff(target);
        let duration = if basis == 0 {
            Duration::ZERO
        } else {
            duration.mul_f64(delta as f64 / basis as f64)
        };
        if duration.is_zero() {
            write_brightness(device, target)
        } else {
            ramp_brightness(device, target, duration, frequency)
        }
    }
}

fn ramp_brightness(
    device: &dyn Brightness,
    target: u32,
    duration: Duration,
    frequency: u32,
) -> Result<()> {
    assert!(!duration.is_zero() && frequency > 0);

    let max = read_max_brightness(device)?;
    let start = read_brightness(device)?;
    let target = target.min(max);

    let delta = start.abs_diff(target);
    if delta == 0 {
        return Ok(());
    }
    let steps = ((duration.as_secs_f64() * frequency as f64).floor() as u32)
        .max(1)
        .min(delta);
    let interval = duration / steps;
    let mut next_update = Instant::now() + interval;

    for step in 1..=steps {
        let now = Instant::now();
        if now < next_update {
            std::thread::sleep(next_update - now);
        }
        next_update += interval;
        let prog = (delta as u64 * step as u64 / steps as u64) as u32;
        let value = if target >= start {
            start + prog
        } else {
            start - prog
        };

        write_brightness(device, value)?;
    }

    Ok(())
}
