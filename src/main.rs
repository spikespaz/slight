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

const FAIL_FIND_DEFAULT_DEVICE: &str = "failed to find a default device";
const FAIL_R_MAX_BRIGHTNESS: &str = "failed to read max_brightness";
const FAIL_W_BRIGHTNESS: &str = "failed to write brightness";
const FAIL_R_BRIGHTNESS: &str = "failed to read brightness";
const FAIL_R_ACTUAL_BRIGHTNESS: &str = "failed to read actual_brightness";
const CONFLICT_INCREASE_DECREASE: &str =
    "cannot specify increase (-I) and decrease (-D) at the same time";
const CURRENT_BRIGHTNESS_GREATER: &str = "current brightness is greater than target, doing nothing";
const CURRENT_BRIGHTNESS_LESS: &str = "current brightness is less than target, doing nothing";

const DEFAULT_DEVICE_PATHS: &[&str; 2] = &["/sys/class/backlight", "/sys/class/leds"];

fn main() {
    let args = slight_command().run();

    let found_devices = Lazy::<Vec<DeviceDetail>>::new(find_devices);

    fn default_device(found: Lazy<Vec<DeviceDetail>>) -> DeviceDetail {
        Lazy::force(&found);
        let defaults = Lazy::into_value(found).unwrap();
        defaults.into_iter().next().expect(FAIL_FIND_DEFAULT_DEVICE)
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
        }
        Action::Get { percent } => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let current = device.brightness().expect(FAIL_R_BRIGHTNESS);
            let current = Value::Absolute(current);
            if percent {
                let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
                let actual = current.as_percent(max);
                println!("{actual}");
            } else {
                println!("{current}");
            }
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
                panic!("{CONFLICT_INCREASE_DECREASE}");
            }

            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
            let current = device.brightness().expect(FAIL_R_BRIGHTNESS);
            let target = value.to_absolute(max);

            if target == current {
            } else if increase && target < current {
                println!("{CURRENT_BRIGHTNESS_GREATER}");
            } else if decrease && target > current {
                println!("{CURRENT_BRIGHTNESS_LESS}");
            } else {
                set_brightness(&device, current, target, duration.0, frequency, max);
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
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
            let amount = amount.to_absolute(max);
            let current = device.brightness().expect(FAIL_R_BRIGHTNESS);
            let target = (current + amount).clamp(0, max);

            set_brightness(&device, current, target, duration.0, frequency, amount);
        }
        Action::Decrease {
            amount,
            interpolate:
                InterpolationOptions {
                    duration,
                    frequency,
                },
        } => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
            let amount = amount.to_absolute(max);
            let current = device.brightness().expect(FAIL_R_BRIGHTNESS);
            let target = (current - amount).clamp(0, max);

            set_brightness(&device, current, target, duration.0, frequency, amount);
        }
    };
}

fn find_devices() -> Vec<DeviceDetail> {
    DEFAULT_DEVICE_PATHS
        .iter()
        .flat_map(|path| Path::new(path).read_dir())
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| DeviceDetail::try_from(entry.path()).ok())
        .collect()
}

fn set_brightness(
    device: &dyn Brightness,
    current: u32,
    target: u32,
    duration: Duration,
    frequency: u32,
    basis: u32,
) {
    if duration.is_zero() {
        device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
    } else {
        let delta = current.abs_diff(target);
        let duration = if basis == 0 {
            Duration::ZERO
        } else {
            duration.mul_f64(delta as f64 / basis as f64)
        };
        if duration.is_zero() {
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        } else {
            ramp_brightness(device, target, duration, frequency);
        }
    }
}

fn ramp_brightness(device: &dyn Brightness, target: u32, duration: Duration, frequency: u32) {
    assert!(!duration.is_zero() && frequency > 0);

    let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
    let start = device.brightness().expect(FAIL_R_BRIGHTNESS);
    let target = target.min(max);

    let delta = start.abs_diff(target);
    if delta == 0 {
        return;
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

        device.set_brightness(value).expect(FAIL_W_BRIGHTNESS);
    }
}
