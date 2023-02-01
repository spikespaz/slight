mod cli;
mod device;
mod discovery;

use std::path::Path;
use std::time::Duration;

use once_cell::unsync::Lazy;

use crate::cli::{ActionList, SlightCommand, Value};
use crate::device::{Backlight, BacklightDevice, Brightness, LedDevice};
use crate::discovery::{Capability, CapabilityCheckError, DeviceDetail};

const FAIL_R_MAX_BRIGHTNESS: &str = "failed to read max_brightness";
const FAIL_W_BRIGHTNESS: &str = "failed to write brightness";
const FAIL_R_BRIGHTNESS: &str = "failed to read brightness";
const FAIL_R_ACTUAL_BRIGHTNESS: &str = "failed to read actual_brightness";

const DEFAULT_DEVICE_PATHS: &[&str; 2] = &["/sys/class/backlight", "/sys/class/leds"];

fn main() {
    let args: SlightCommand = argh::from_env();

    let found_devices = Lazy::<Vec<DeviceDetail>>::new(find_devices);

    fn default_device(found: Lazy<Vec<DeviceDetail>>) -> DeviceDetail {
        Lazy::force(&found);
        let defaults = Lazy::into_value(found).unwrap();
        defaults.into_iter().next().expect("no default device")
    }

    let verbose = args.verbose;

    use cli::{Action::*, ActionDecrease, ActionGet, ActionIncrease, ActionSet};

    match args.command {
        List(ActionList { paths }) => {
            for device in found_devices.iter() {
                if paths {
                    println!("{}", device.path.display());
                } else {
                    println!("{}", device.name);
                }
            }
        }
        Get(ActionGet { percent }) => {}
        Set(ActionSet { value, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let target = value_to_absolute(value, &device);
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
        Increase(ActionIncrease { amount, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let delta = Delta::Increase(value_to_absolute(amount, &device));
            change_brightness(delta, &device, duration);
        }
        Decrease(ActionDecrease { amount, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let delta = Delta::Decrease(value_to_absolute(amount, &device));
            change_brightness(delta, &device, duration);
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

fn value_to_absolute(value: Value, device: &dyn Brightness) -> u32 {
    match value {
        Value::Absolute(value) => value,
        Value::Percent(percent) => {
            let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
            (percent * max as f32).round() as u32
        }
    }
}

pub enum Delta {
    Increase(u32),
    Decrease(u32),
}

fn change_brightness(delta: Delta, device: &dyn Brightness, duration: Option<Duration>) {
    let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
    let actual = device.brightness().expect(FAIL_R_ACTUAL_BRIGHTNESS);

    match delta {
        Delta::Increase(delta) => {
            let target = std::cmp::min(actual + delta, max);
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
        Delta::Decrease(delta) => {
            let target = actual.saturating_sub(delta);
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
    };
}
