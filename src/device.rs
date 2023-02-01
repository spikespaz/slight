use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use derive_more::Display;
use once_cell::unsync::OnceCell;
use strum::EnumString;
use thiserror::Error;

#[derive(Error, Display, Debug)]
pub enum ReadNumError {
    Read(#[from] std::io::Error),
    Parse(#[from] std::num::ParseIntError),
}

pub type ReadNumResult<T> = Result<T, ReadNumError>;
pub type WriteResult = std::io::Result<()>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PowerState {
    Unblank = 0,
    Powerdown = 4,
}

#[derive(EnumString, PartialEq, Debug, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum DeviceType {
    Firmware,
    Platform,
    Raw,
}

/// This API corresponds to the basics that `sysfs-class-led` and
/// `sysfs-class-backlight` have in common.
///
/// TODO: missing `trigger`.
///
/// There is more to LED devices that is not specified by this trait.
/// <https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-led>
pub trait Brightness {
    fn brightness(&self) -> ReadNumResult<u32>;
    fn set_brightness(&self, value: u32) -> WriteResult;
    fn max_brightness(&self) -> ReadNumResult<u32>;
}

/// This API corresponds to:
/// <https://www.kernel.org/doc/Documentation/ABI/stable/sysfs-class-backlight>
pub trait Backlight: Brightness {
    fn bl_power(&self) -> std::io::Result<PowerState>;
    fn set_bl_power(&self, value: PowerState) -> WriteResult;
    fn actual_brightness(&self) -> ReadNumResult<u32>;
    fn device_type(&self) -> std::io::Result<DeviceType>;
}

#[derive(Debug)]
pub struct LedDevice {
    path: PathBuf,
    file_brightness: OnceCell<File>,
    max_brightness: OnceCell<u32>,
}

#[derive(Debug)]
pub struct BacklightDevice {
    path: PathBuf,
    file_bl_power: OnceCell<File>,
    file_brightness: OnceCell<File>,
    file_actual_brightness: OnceCell<File>,
    max_brightness: OnceCell<u32>,
    device_type: OnceCell<DeviceType>,
}

impl LedDevice {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            file_brightness: OnceCell::new(),
            max_brightness: OnceCell::new(),
        }
    }
}

impl BacklightDevice {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            file_bl_power: OnceCell::new(),
            file_brightness: OnceCell::new(),
            file_actual_brightness: OnceCell::new(),
            max_brightness: OnceCell::new(),
            device_type: OnceCell::new(),
        }
    }
}

macro_rules! device_file {
    ($self:ident, $field:ident, $subpath:literal, $write:literal) => {
        $self.$field.get_or_try_init(|| {
            OpenOptions::new()
                .read(true)
                .write($write)
                .open($self.path.join($subpath))
        })
    };
}

macro_rules! impl_brightness {
    ($struct:path) => {
        impl Brightness for $struct {
            fn brightness(&self) -> ReadNumResult<u32> {
                let mut file = device_file!(self, file_brightness, "brightness", true)?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                file.rewind()?;
                Ok(buf.trim().parse()?)
            }

            fn set_brightness(&self, value: u32) -> WriteResult {
                let mut file = device_file!(self, file_brightness, "brightness", true)?;
                file.write_fmt(format_args!("{}", value))?;
                file.rewind()?;
                file.flush()
            }

            fn max_brightness(&self) -> ReadNumResult<u32> {
                self.max_brightness
                    .get_or_try_init(|| {
                        let mut file = File::open(self.path.join("max_brightness"))?;
                        let mut buf = String::new();
                        file.read_to_string(&mut buf)?;
                        Ok(buf.trim().parse()?)
                    })
                    .copied()
            }
        }
    };
}

impl_brightness!(LedDevice);
impl_brightness!(BacklightDevice);

impl Backlight for BacklightDevice {
    fn bl_power(&self) -> std::io::Result<PowerState> {
        let mut file = device_file!(self, file_bl_power, "bl_power", true)?;
        let mut buf = [0_u8];
        file.read_exact(&mut buf)?;
        match &buf {
            b"0" => Ok(PowerState::Unblank),
            b"4" => Ok(PowerState::Powerdown),
            _ => unreachable!(),
        }
    }

    fn set_bl_power(&self, value: PowerState) -> WriteResult {
        let mut file = device_file!(self, file_bl_power, "bl_power", true)?;
        file.write_fmt(format_args!("{}", value as u8))?;
        file.rewind()?;
        file.flush()
    }

    fn actual_brightness(&self) -> ReadNumResult<u32> {
        let mut file = device_file!(self, file_actual_brightness, "brightness", false)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        file.rewind()?;
        Ok(buf.trim().parse()?)
    }

    fn device_type(&self) -> std::io::Result<DeviceType> {
        self.device_type
            .get_or_try_init(|| {
                let mut file = File::open(self.path.join("type"))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                Ok(buf.trim().parse().unwrap())
            })
            .copied()
    }
}
