use std::num::{ParseFloatError, ParseIntError};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use bpaf::Bpaf;

/// Small CLI utility for Linux to control brightness on ACPI devices.
#[derive(Debug, PartialEq, Bpaf)]
#[bpaf(options)]
pub struct SlightCommand {
    /// Show errors
    #[bpaf(short('v'), long)]
    pub verbose: bool,
    /// The device to control
    #[bpaf(short('D'), long, argument("DEVICE"))]
    pub device: Option<PathBuf>,
    /// What to do?
    #[bpaf(external(action))]
    pub command: Action,
}

#[derive(Debug, PartialEq, Bpaf)]
pub enum Action {
    /// Discover and list all backlight devices
    #[bpaf(command("list"))]
    List {
        /// List devices as full paths (not names)
        #[bpaf(short('P'), long)]
        paths: bool,
    },
    /// Get the current brightness of DEVICE
    #[bpaf(command("get"))]
    Get {
        /// Show the brightness as a percentage
        #[bpaf(short('p'), long)]
        percent: bool,
    },
    /// Set the brightness of DEVICE to VALUE
    #[bpaf(command("set"))]
    Set {
        /// Only increase, never decrease
        #[bpaf(short('I'), long("inc"), long("increase"))]
        increase: bool,
        /// Only decrease, never increase
        #[bpaf(short('D'), long("dec"), long("decrease"))]
        decrease: bool,
        //
        #[bpaf(external(interpolation_options))]
        interpolate: InterpolationOptions,
        /// Percentage or value to set
        #[bpaf(positional("VALUE"))]
        value: Value,
    },
    /// Increase the brightness of DEVICE by AMOUNT
    #[bpaf(command("inc"))]
    Increase {
        #[bpaf(external(interpolation_options))]
        interpolate: InterpolationOptions,
        /// Percentage or value to add
        #[bpaf(positional("AMOUNT"))]
        amount: Value,
    },
    /// Decrease the brightness of DEVICE by AMOUNT
    #[bpaf(command("dec"))]
    Decrease {
        #[bpaf(external(interpolation_options))]
        interpolate: InterpolationOptions,
        /// Percentage or value to subtract
        #[bpaf(positional("AMOUNT"))]
        amount: Value,
    },
}

#[derive(Debug, PartialEq, Bpaf)]
pub struct InterpolationOptions {
    /// Maximum duration of time over which to interpolate the change
    #[bpaf(
        short('t'),
        long,
        argument("DURATION"),
        fallback(DurationArgument::ZERO)
    )]
    pub duration: DurationArgument,
    /// The maximum frequency of brightness updates (Hz)
    #[bpaf(long("freq"), long("frequency"), argument("FREQUENCY"), fallback(30))]
    pub frequency: u32,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ParseValueError {
    #[error("percentage '{0}' must be between 0 and 100")]
    PercentOutOfRange(u8),
    #[error("{0} for percentage '{1}'")]
    ParsePercentError(ParseIntError, String),
    #[error("{0} for absolute value '{1}'")]
    ParseAbsoluteError(ParseIntError, String),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Value {
    Percent(f32),
    Absolute(u32),
}

impl Value {
    pub fn to_percent(self, max: u32) -> f32 {
        match self {
            Value::Percent(pct) => pct,
            Value::Absolute(abs) => abs as f32 / max as f32,
        }
    }

    pub fn to_absolute(self, max: u32) -> u32 {
        match self {
            Value::Percent(pct) => (pct.clamp(0.0, 1.0) * max as f32).round() as u32,
            Value::Absolute(abs) => abs,
        }
        .min(max)
    }

    pub fn as_percent(self, max: u32) -> Self {
        Value::Percent(self.to_percent(max))
    }

    pub fn as_absolute(self, max: u32) -> Self {
        Value::Absolute(self.to_absolute(max))
    }

    pub fn saturating_add(lhs: u32, rhs: Self, max: u32) -> u32 {
        lhs.saturating_add(rhs.to_absolute(max)).min(max)
    }

    pub fn saturating_sub(lhs: u32, rhs: Self, max: u32) -> u32 {
        lhs.saturating_sub(rhs.to_absolute(max))
    }
}

impl FromStr for Value {
    type Err = ParseValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        use ParseValueError as E;

        let value = value.trim();
        if value.ends_with('%') {
            let value = value[0..value.len() - 1]
                .parse::<u8>()
                .map_err(|e| E::ParsePercentError(e, value.to_string()))?;
            if !(0..=100).contains(&value) {
                Err(E::PercentOutOfRange(value))
            } else {
                Ok(Self::Percent(value as f32 / 100.0))
            }
        } else {
            let value = value
                .parse::<u32>()
                .map_err(|e| E::ParseAbsoluteError(e, value.to_string()))?;
            Ok(Self::Absolute(value))
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Percent(pct) => write!(f, "{}%", pct * 100.0),
            Value::Absolute(abs) => write!(f, "{abs}"),
        }
    }
}

/// A wrapper of [`Duration`] that is non-zero and implements [`FromStr`].
#[derive(Clone, Debug, PartialEq)]
pub struct DurationArgument(pub Duration);

impl DurationArgument {
    pub const ZERO: Self = DurationArgument(Duration::ZERO);
}

impl Deref for DurationArgument {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DurationArgument {
    type Err = ParseDurationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        match parse_duration(value) {
            Ok(dur) => Ok(Self(dur)),
            Err(ParseDurationError::MissingSuffix) => {
                let ms = value.parse().map_err(ParseDurationError::ParseIntError)?;
                Ok(Self(Duration::from_millis(ms)))
            }
            Err(e) => Err(e),
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

    use super::{parse_duration, slight_command, ParseDurationError};

    #[test]
    fn bpaf_check_invariants() {
        slight_command().check_invariants(false);
    }

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
