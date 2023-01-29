mod cli;
mod device;

use crate::cli::{SlightCommand, Value};
use crate::device::BacklightDevice;

fn main() {
    let args: SlightCommand = argh::from_env();

    let verbose = args.verbose;
    let device = args.device;

    use cli::{Action::*, ActionDecrease, ActionGet, ActionIncrease, ActionSet};

    match args.command {
        List(_) => {}
        Get(ActionGet { percent }) => {}
        Set(ActionSet { value, duration }) => {
            let dev = BacklightDevice::new(device.unwrap());
            let target = match value {
                Value::Absolute(value) => value,
                Value::Percent(mul) => {
                    let max = dev.max_brightness().expect("could not read max_brightness");
                    (mul * max as f32) as u32
                }
            };
            dev.set_brightness(target)
                .expect("failed to set brightness");
        }
        Increase(ActionIncrease { amount, duration }) => {}
        Decrease(ActionDecrease { amount, duration }) => {}
    };
}
