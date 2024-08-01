#![doc = include_str!("../README.md")]
#![warn(unsafe_code)]
#![no_std]

use embedded_hal::i2c::{Error as I2cError, ErrorKind as I2cErrorKind, I2c};
use num_enum::{FromPrimitive, IntoPrimitive};

mod touch;

const FT6336_ADDR: u8 = 0x38;

/// FT6336 error type.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// An I2C error occurred during the transaction.
    I2cError(I2cErrorKind),
    /// Other error. The original error converted from may contain more information.
    Other,
}

impl<T: I2cError> From<T> for Error {
    fn from(value: T) -> Self {
        Self::I2cError(value.kind())
    }
}

impl embedded_hal::digital::Error for Error {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

/// FT6336 power modes.
#[repr(u8)]
#[derive(IntoPrimitive, FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerMode {
    #[num_enum(default)]
    Active,
    Monitor,
    Standby,
    Hibernate,
}

/// FT6336 struct.
#[derive(Debug)]
pub struct Ft6336<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Ft6336<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Switchs to work mode, to avoid rare case that the controller
    /// is left in factory mode.
    ///
    /// This method is safe to call at anytime.
    pub fn init(&mut self) -> Result<(), Error> {
        self.write_u8(0x00, 0x00)
    }

    /// Returns chip code.
    ///
    /// - FT6236G: 0x00, ?, ?
    /// - FT6336G: 0x01, ?, ?
    /// - FT6336U: 0x02, ?, ?
    /// - FT6426: 0x03, ?, ?
    pub fn chip_code(&mut self) -> Result<(u8, u8, u8), Error> {
        let low = self.read_u8(0xA0)?;
        let mid = self.read_u8(0x9F)?;
        let high = self.read_u8(0xA3)?;
        Ok((low, mid, high))
    }

    /// Returns app lib version.
    pub fn applib_version(&mut self) -> Result<(u8, u8), Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read_buf(0xA1, &mut buf)?;
        let low = buf[1];
        let high = buf[0];
        Ok((low, high))
    }

    /// Returns firmware version.
    pub fn firmware_version(&mut self) -> Result<u8, Error> {
        self.read_u8(0xA6)
    }

    /// Returns vendor ID.
    pub fn vender_id(&mut self) -> Result<u8, Error> {
        self.read_u8(0xA8)
    }

    /// Returns release code ID on custom reference version.
    pub fn release_code(&mut self) -> Result<u8, Error> {
        self.read_u8(0xAF)
    }

    /// Sets frequency hopping enable status.
    ///
    /// Set true to enable frequency hopping(useful when plugged to a power source).
    /// But it seems not necessary under most cases.
    pub fn set_use_freqency_hopping(&mut self, value: bool) -> Result<(), Error> {
        match value {
            true => self.write_u8(0x8B, 0x01),
            false => self.write_u8(0x8B, 0x00),
        }
    }

    /// Sets the INT pin behavior to generate a pulse when there's new touch event.
    ///
    /// In either interrupt mode, the touch released events will not generate an
    /// iterrupt signal.
    pub fn interrupt_by_pulse(&mut self) -> Result<(), Error> {
        self.write_u8(0xA4, 0x01)
    }

    /// Sets the INT pin behavior to drive low when there's new touch event to process.
    ///
    /// In either interrupt mode, the touch released events will not generate an
    /// iterrupt signal.
    pub fn interrupt_by_state(&mut self) -> Result<(), Error> {
        self.write_u8(0xA4, 0x00)
    }

    /// Sets whether to automatically enter monitor mode(simpler scan mode, saves energy).
    pub fn set_auto_monitor_mode(&mut self, value: bool) -> Result<(), Error> {
        match value {
            true => self.write_u8(0x86, 0x01),
            false => self.write_u8(0x86, 0x00),
        }
    }

    /// Sets the time limit(in second) to enter monitor mode automatically.
    ///
    /// Default is 30 seconds. Maximum value is 0x64
    pub fn set_auto_monitor_mode_delay(&mut self, value: u8) -> Result<(), Error> {
        if value > 0x64 {
            self.write_u8(0x87, 0x64)
        } else {
            self.write_u8(0x87, value)
        }
    }

    /// Sets the scan rate under active mode, in Hertz.
    ///
    /// minimum 0x04, maximum 0x14
    pub fn set_scan_rate(&mut self, value: u8) -> Result<(), Error> {
        if value < 0x04 {
            self.write_u8(0x88, 0x04)
        } else if value > 0x14 {
            self.write_u8(0x88, 0x14)
        } else {
            self.write_u8(0x88, value)
        }
    }

    /// Sets the scan rate under monitor mode, in Hertz.
    ///
    /// minimum 0x04, maximum 0x14
    pub fn set_monitor_scan_rate(&mut self, value: u8) -> Result<(), Error> {
        if value < 0x04 {
            self.write_u8(0x89, 0x04)
        } else if value > 0x14 {
            self.write_u8(0x89, 0x14)
        } else {
            self.write_u8(0x89, value)
        }
    }

    /// Sets touch driver [`PowerMode`].
    pub fn set_power_mode(&mut self, value: PowerMode) -> Result<(), Error> {
        self.write_u8(0xA5, value.into())
    }

    /// Reads one u8 integer.
    fn read_u8(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf: [u8; 1] = [0; 1];

        match self.i2c.write_read(FT6336_ADDR, &[reg], &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e.into()),
        }
    }

    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), Error> {
        Ok(self.i2c.write(FT6336_ADDR, &[reg, value])?)
    }

    #[inline]
    fn read_buf(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), Error> {
        Ok(self.i2c.write_read(FT6336_ADDR, &[reg], buf)?)
    }

    // /// Write a single bit.
    // #[inline]
    // fn write_bit(&mut self, reg: u8, bit: usize, value: bool) -> Result<(), Error> {
    //     let mut reg_val = self.read_u8(reg)?;
    //     if reg_val.get_bit(bit) == value {
    //         Ok(())
    //     } else {
    //         reg_val.set_bit(bit, value);
    //         self.write_u8(reg, reg_val)
    //     }
    // }

    // /// Writes bits.
    // #[inline]
    // fn write_bits<T: RangeBounds<usize>>(
    //     &mut self,
    //     reg: u8,
    //     range: T,
    //     value: u8,
    // ) -> Result<(), Error> {
    //     let mut reg_val = self.read_u8(reg)?;
    //     reg_val.set_bits(range, value);
    //     self.write_u8(reg, reg_val)
    // }
}
