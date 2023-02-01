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
        Get(ActionGet { percent }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            println!("{}", device.brightness().expect(FAIL_R_BRIGHTNESS));
        }
        Set(ActionSet { value, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let value = value_to_absolute(value, &device);
            set_brightness(value, &device, duration);
        }
        Increase(ActionIncrease { amount, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let delta = value_to_absolute(amount, &device);
            let value = device
                .brightness()
                .expect(FAIL_R_BRIGHTNESS)
                .saturating_add(delta);
            set_brightness(value, &device, duration);
        }
        Decrease(ActionDecrease { amount, duration }) => {
            let device = args.device.unwrap_or(default_device(found_devices).path);
            let device = LedDevice::new(device);
            let delta = value_to_absolute(amount, &device);
            let value = device
                .brightness()
                .expect(FAIL_R_BRIGHTNESS)
                .saturating_sub(delta);
            set_brightness(value, &device, duration);
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

fn set_brightness(value: u32, device: &dyn Brightness, duration: Option<Duration>) {
    let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
    let actual = device.brightness().expect(FAIL_R_BRIGHTNESS);
    let target = std::cmp::min(value, max);

    use std::cmp::Ordering;
    match (target.cmp(&actual), duration) {
        (Ordering::Greater, Some(duration)) => {
            let interval = duration / (value - actual);
            for i in (actual + 1)..=target {
                device.set_brightness(i).expect(FAIL_W_BRIGHTNESS);
                if i != target {
                    std::thread::sleep(interval);
                }
            }
        }
        (Ordering::Less, Some(duration)) => {
            let interval = duration / (actual - value);
            for i in (target..actual).rev() {
                device.set_brightness(i).expect(FAIL_W_BRIGHTNESS);
                if i != target {
                    std::thread::sleep(interval);
                }
            }
        }
        (Ordering::Greater | Ordering::Less, None) => {
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
        (Ordering::Equal, _) => {}
    }
}
