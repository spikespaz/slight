use std::num::{ParseFloatError, ParseIntError};
use std::ops::Deref;
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
pub struct ActionList {
    /// list devices as full paths (not names)
    #[argh(switch, short = 'P')]
    pub paths: bool,
}

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
    /// only increase, never decrease
    #[argh(switch, short = 'I')]
    pub increase: bool,
    /// only decrease, never increase
    #[argh(switch, short = 'D')]
    pub decrease: bool,
    // /// percentage curve function
    // #[argh(option)]
    // pub curve: Option<String>,
    /// duration of time when interpolating between 0% and 100%
    #[argh(option, short = 't')]
    pub duration: Option<DurationInterval>,
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
    #[argh(option, short = 't')]
    pub duration: Option<DurationInterval>,
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
    #[argh(option, short = 't')]
    pub duration: Option<DurationInterval>,
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

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum DurationIntervalError {
    #[error("duration must be greater than zero")]
    IsZero,
    #[error("{0}")]
    Parse(#[from] ParseDurationError),
}

/// A wrapper of [`Duration`] that is non-zero and implements [`FromStr`].
#[derive(Clone, Debug, PartialEq)]
pub struct DurationInterval(pub Duration);

impl TryFrom<Duration> for DurationInterval {
    type Error = DurationIntervalError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        if value.is_zero() {
            Err(DurationIntervalError::IsZero)
        } else {
            Ok(Self(value))
        }
    }
}

impl Deref for DurationInterval {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DurationInterval {
    type Err = DurationIntervalError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        match parse_duration(value) {
            Ok(dur) => Self::try_from(dur),
            Err(ParseDurationError::MissingSuffix) => {
                let ms = value.parse().map_err(ParseDurationError::ParseIntError)?;
                Duration::from_millis(ms).try_into()
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ParseDurationError {
    #[error("duration is missing a value")]
    MissingNumber,
    #[error("durations cannot be negative")]
    NegativeNumber,
    #[error("duration is missing a suffix")]
    MissingSuffix,
    #[error("unknown suffix '{0}' for duration, must be one of: `ms`, `ds`, `s`, `m`")]
    InvalidSuffix(String),
    #[error("{0} for duration")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("{0} for duration")]
    ParseIntError(#[from] ParseIntError),
}

fn parse_duration(value: &str) -> Result<Duration, ParseDurationError> {
    use ParseDurationError as E;

    if value.is_empty() {
        return Err(E::MissingNumber);
    }

    macro_rules! parse_with_suffix {
        ($suffix:literal, $parse_ty:ty, $map:expr) => {
            if value.ends_with($suffix) {
                let number = &value[0..value.len() - $suffix.len()];
                if number.is_empty() {
                    return Err(E::MissingNumber);
                }
                let number = number.parse::<$parse_ty>()?;
                if number < 0 as $parse_ty {
                    return Err(E::NegativeNumber);
                } else {
                    return Ok($map(number));
                }
            }
        };
    }

    parse_with_suffix!("ms", u64, Duration::from_millis);
    parse_with_suffix!("ds", u64, |ds| Duration::from_millis(ds * 100));
    parse_with_suffix!("s", f64, Duration::from_secs_f64);
    parse_with_suffix!("m", f64, |m| Duration::from_secs_f64(m * 60.0));

    let number = value.trim_end_matches(|ch: char| !ch.is_numeric() && ch != '.');
    let suffix = &value[number.len()..];
    if suffix.is_empty() {
        Err(E::MissingSuffix)
    } else {
        Err(E::InvalidSuffix(suffix.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use test_case::{test_case, test_matrix};

    use super::{parse_duration, ParseDurationError};

    #[test_case("100ms" => Duration::from_millis(100))]
    #[test_case("10ds" => Duration::from_secs_f64(1.0))]
    #[test_case("1s" => Duration::from_secs(1))]
    #[test_case("1m" => Duration::from_secs(60))]
    #[test_case("1.0s" => Duration::from_secs_f64(1.0))]
    #[test_case("1.0m" => Duration::from_secs_f64(60.0))]
    fn test_parse_duration(input: &str) -> Duration {
        parse_duration(input).unwrap()
    }

    #[test_matrix(["ms", "ds", "s", "m"] => ParseDurationError::MissingNumber)]
    #[test_matrix(["-1.0s", "-1.0m"] => ParseDurationError::NegativeNumber)]
    #[test_matrix(["1.0", "1"] => ParseDurationError::MissingSuffix)]
    #[test_matrix(["1h", "h"] => ParseDurationError::InvalidSuffix("h".to_owned()))]
    #[test_case("100.0ms" => ParseDurationError::ParseIntError("100.0".parse::<u64>().unwrap_err()))]
    #[test_case("10.0ms" => ParseDurationError::ParseIntError("10.0".parse::<u64>().unwrap_err()))]
    #[test_case("-100ms" => ParseDurationError::ParseIntError("-100".parse::<u64>().unwrap_err()))]
    #[test_case("0x01s" => ParseDurationError::ParseFloatError("0x01".parse::<f64>().unwrap_err()))]
    #[test_case("" => ParseDurationError::MissingNumber)]
    fn test_parse_duration_error(input: &str) -> ParseDurationError {
        parse_duration(input).unwrap_err()
    }
}
