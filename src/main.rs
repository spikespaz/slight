mod cli;
mod device;

use std::time::Duration;

use crate::cli::{SlightCommand, Value};
use crate::device::BacklightDevice;

enum DeltaValue {
    Increase(Value),
    Decrease(Value),
}

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
            let device = BacklightDevice::new(device);
            let target = value_to_absolute(value, &device);
            device.set_brightness(target).expect(FAIL_W_BRIGHTNESS);
        }
        Increase(ActionIncrease { amount, duration }) => {
            let device = BacklightDevice::new(device);
            change_brightness(DeltaValue::Increase(amount), &device, duration);
        }
        Decrease(ActionDecrease { amount, duration }) => {
            let device = BacklightDevice::new(device);
            change_brightness(DeltaValue::Decrease(amount), &device, duration);
        }
    };
}

fn value_to_absolute(value: Value, device: &BacklightDevice) -> u32 {
    match value {
        Value::Absolute(value) => value,
        Value::Percent(percent) => {
            let max = device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS);
            (percent * max as f32).round() as u32
        }
    }
}

fn change_brightness(amount: DeltaValue, device: &BacklightDevice, duration: Option<Duration>) {
    let value = match amount {
        DeltaValue::Increase(value) => {
            let delta = value_to_absolute(value, device);
            let actual = device.actual_brightness().expect(FAIL_R_ACTUAL_BRIGHTNESS);
            (actual + delta).max(device.max_brightness().expect(FAIL_R_MAX_BRIGHTNESS))
        }
        DeltaValue::Decrease(value) => {
            let delta = value_to_absolute(value, device);
            let actual = device.actual_brightness().expect(FAIL_R_ACTUAL_BRIGHTNESS);
            actual.saturating_sub(delta)
        }
    };
    device
        .set_brightness(value)
        .expect("failed to write brightness");
}
