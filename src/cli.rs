use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use argh::FromArgs;

// TODO should the duration apply to interpolating between 0-100%,
// or between the current and target brightness?
// Which would be used more often, which should be the default, and
// is it worth complicating thing with a flag, and if so, what should
// the CLI for this look like?
//
// Currently `ActionSet` takes duration to mean between 0-100%,
// and increase an decrease take it as the duration over
// which to interpolate the delta.

/// Small CLI utility for Linux to control brightness on ACPI devices.
#[derive(FromArgs, PartialEq, Debug)]
pub struct SlightCommand {
    /// what to do?
    #[argh(subcommand)]
    pub command: Action,
    /// show errors
    #[argh(switch, short = 'v')]
    pub verbose: bool,
    /// the device to control
    #[argh(option, short = 'D')]
    pub device: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Action {
    List(ActionList),
    Get(ActionGet),
    Set(ActionSet),
    Increase(ActionIncrease),
    Decrease(ActionDecrease),
}

/// list all discovered backlight devices
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
pub struct ActionList {}

/// get the current brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "get")]
pub struct ActionGet {
    /// show the value as a percentage
    #[argh(switch, short = 'p')]
    pub percent: bool,
    // /// percentage curve function (raw to percent)
    // #[argh(option)]
    // pub curve: Option<String>,
}

/// set the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set")]
pub struct ActionSet {
    /// percentage or value to set
    #[argh(positional)]
    pub value: Value,
    // /// percentage curve function
    // #[argh(option)]
    // pub curve: Option<String>,
    /// duration of time when interpolating between 0% and 100%
    #[argh(option, short = 't', from_str_fn(duration_from_str))]
    pub duration: Option<Duration>,
}

/// increase the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "inc")]
pub struct ActionIncrease {
    /// percentage or value to add
    #[argh(positional)]
    pub amount: Value,
    // /// percentage curve function
    // #[argh(option)]
    // pub curve: Option<String>,
    /// duration of time over which to interpolate the change
    #[argh(option, short = 't', from_str_fn(duration_from_str))]
    pub duration: Option<Duration>,
}

/// decrease the brightness value
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "dec")]
pub struct ActionDecrease {
    /// percentage or value to subtract
    #[argh(positional)]
    pub amount: Value,
    // /// percentage curve function
    // #[argh(option)]
    // pub curve: Option<String>,
    /// duration of time over which to interpolate the change
    #[argh(option, short = 't', from_str_fn(duration_from_str))]
    pub duration: Option<Duration>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Value {
    Percent(f32),
    Absolute(u32),
}

// This is here because we get `FromArgValue` automatically
// for types that implement `FromStr` where `FromStr::Err` is `Display`.
impl FromStr for Value {
    type Err = ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.ends_with('%') {
            Ok(Self::Percent(
                value[0..value.len() - 1].parse::<u8>()? as f32 / 100.0,
            ))
        } else {
            Ok(Self::Absolute(value.parse()?))
        }
    }
}

// Just a function because it's tedious to properly wrap `Duration`
// in a manner that `argh` will accept.
pub fn duration_from_str(value: &str) -> Result<Duration, String> {
    macro_rules! parse {
        ($from_fun:path, $suf_len:literal) => {
            Ok($from_fun(
                value[0..value.len() - $suf_len]
                    .parse()
                    .map_err(|e| format!("{e}"))?,
            )
            .into())
        };
    }
    fn from_decis(ds: f64) -> Duration {
        Duration::from_secs_f64(ds / 10.0)
    }

    fn from_mins(m: f64) -> Duration {
        Duration::from_secs_f64(m * 60.0)
    }

    if value.ends_with("ms") {
        parse!(Duration::from_millis, 2)
    } else if value.ends_with("ds") {
        parse!(from_decis, 2)
    } else if value.ends_with('s') {
        parse!(Duration::from_secs_f64, 1)
    } else if value.ends_with('m') {
        parse!(from_mins, 1)
    } else {
        Err("unknown suffix".into())
    }
}
