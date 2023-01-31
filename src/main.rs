mod cli;
mod device;
mod discovery;

use std::time::Duration;

use crate::cli::{SlightCommand, Value};
use crate::device::{Backlight, BacklightDevice, Brightness, LedDevice};

const FAIL_R_MAX_BRIGHTNESS: &str = "failed to read max_brightness";
const FAIL_W_BRIGHTNESS: &str = "failed to write brightness";
const FAIL_R_BRIGHTNESS: &str = "failed to read brightness";
const FAIL_R_ACTUAL_BRIGHTNESS: &str = "failed to read actual_brightness";

fn main() {
    let args: SlightCommand = argh::from_env();

    let verbose = args.verbose;
    let device = args.device.unwrap();

    use cli::{Action::*, ActionDecrease, ActionGet, ActionIncrease, ActionSet};

    match args.command {
        List(_) => {}
        Get(ActionGet { percent }) => {}
        Set(ActionSet { value, duration }) => {
            let device = LedDevice::new(device);
            let target = value_to_absolute(value, &device);
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
        Increase(ActionIncrease { amount, duration }) => {
            let device = LedDevice::new(device);
            let delta = value_to_absolute(amount, &device) as i32;
            change_brightness(delta, &device, duration);
        }
        Decrease(ActionDecrease { amount, duration }) => {
            let device = LedDevice::new(device);
            let delta = value_to_absolute(amount, &device) as i32;
            change_brightness(-delta, &device, duration);
        }
    };
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

fn change_brightness(amount: i32, device: &dyn Brightness, duration: Option<Duration>) {
    let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
    let actual = device.brightness().expect(FAIL_R_ACTUAL_BRIGHTNESS);
    let value = (actual as i32 + amount).clamp(0, max as i32);
    device
        .set_brightness(value as u32)
        .expect("failed to write brightness");
}
