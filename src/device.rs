/// This API corresponds to:
/// <https://www.kernel.org/doc/Documentation/ABI/stable/sysfs-class-backlight>
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
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

#[derive(Debug)]
pub struct BacklightDevice {
    path: PathBuf,
    file_bl_power: OnceCell<File>,
    file_brightness: OnceCell<File>,
    file_actual_brightness: OnceCell<File>,
    file_max_brightness: OnceCell<File>,
    file_type: OnceCell<File>,
}

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

impl BacklightDevice {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            file_bl_power: OnceCell::new(),
            file_brightness: OnceCell::new(),
            file_actual_brightness: OnceCell::new(),
            file_max_brightness: OnceCell::new(),
            file_type: OnceCell::new(),
        }
    }

    pub fn bl_power(&self) -> std::io::Result<PowerState> {
        let mut file = device_file!(self, file_bl_power, "bl_power", true)?;
        let mut buf = Vec::<u8>::new();
        let mut buf = [0_u8; 1];
        file.read_exact(&mut buf)?;
        match &buf {
            b"0" => Ok(PowerState::Unblank),
            b"4" => Ok(PowerState::Powerdown),
            _ => unreachable!(),
        }
    }

    pub fn set_bl_power(&self, value: PowerState) -> WriteResult {
        let mut file = device_file!(self, file_bl_power, "bl_power", true)?;
        file.write_fmt(format_args!("{}", value as u8))?;
        file.flush()
    }

    pub fn brightness(&self) -> ReadNumResult<u32> {
        let mut file = device_file!(self, file_brightness, "brightness", true)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.parse()?)
    }

    pub fn set_brightness(&self, value: u32) -> WriteResult {
        let mut file = device_file!(self, file_brightness, "brightness", true)?;
        file.write_fmt(format_args!("{}", value))?;
        file.flush()
    }

    pub fn actual_brightness(&self) -> ReadNumResult<u32> {
        let mut file = device_file!(self, file_actual_brightness, "brightness", false)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.parse()?)
    }

    pub fn max_brightness(&self) -> ReadNumResult<u32> {
        let mut file = device_file!(self, file_max_brightness, "max_brightness", false)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.parse()?)
    }

    pub fn r#type(&self) -> std::io::Result<DeviceType> {
        let mut file = device_file!(self, file_type, "type", false)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.parse::<DeviceType>().unwrap())
    }
}
