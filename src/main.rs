mod cli;
mod device;

use crate::cli::SlightCommand;

fn main() {
    let args: SlightCommand = argh::from_env();

    let verbose = args.verbose;
    let device = args.device;

    use cli::{Action::*, ActionDecrease, ActionGet, ActionIncrease, ActionSet};

    match args.command {
        List(_) => {}
        Get(ActionGet { percent, curve }) => {}
        Set(ActionSet {
            value,
            curve,
            duration,
        }) => {}
        Increase(ActionIncrease {
            amount,
            curve,
            duration,
        }) => {}
        Decrease(ActionDecrease {
            amount,
            curve,
            duration,
        }) => {}
    };
}
