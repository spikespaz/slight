use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use argh::FromArgs;

/// Small CLI utility for Linux to control brightness on ACPI devices.
#[derive(FromArgs, PartialEq, Debug)]
pub struct SlightCommand {
    /// what to do?
    #[argh(subcommand)]
    command: Action,
    /// show errors
    #[argh(switch, short = 'v')]
    verbose: bool,
    /// the device to control
    #[argh(option, short = 'D')]
    device: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Action {
    Get(ActionGet),
    Set(ActionSet),
    Increase(ActionIncrease),
    Decrease(ActionDecrease),
}

/// get the current brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "get")]
pub struct ActionGet {
    /// show the value as a percentage
    #[argh(switch, short = 'p')]
    percent: bool,
    /// percentage curve function (raw to percent)
    #[argh(option)]
    curve: Option<String>,
}

/// set the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set")]
pub struct ActionSet {
    /// percentage or value to set
    #[argh(positional)]
    value: Value,
    /// percentage curve function
    #[argh(option)]
    curve: Option<String>,
}

/// increase the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "increase")]
pub struct ActionIncrease {
    /// percentage or value to add
    #[argh(positional)]
    by: Value,
    /// percentage curve function
    #[argh(option)]
    curve: Option<String>,
}

/// decrease the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "decrease")]
pub struct ActionDecrease {
    /// percentage or value to subtract
    #[argh(positional)]
    by: Value,
    /// percentage curve function
    #[argh(option)]
    curve: Option<String>,
}

#[derive(PartialEq, Debug)]
pub enum Value {
    Percent(u8),
    Absolute(u32),
}

// This is here because we get `FromArgValue` automatically
// for types that implement `FromStr` where `FromStr::Err` is `Display`.
impl FromStr for Value {
    type Err = ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.ends_with('%') {
            Ok(Self::Percent(value[0..value.len() - 1].parse()?))
        } else {
            Ok(Self::Absolute(value.parse()?))
        }
    }
}
