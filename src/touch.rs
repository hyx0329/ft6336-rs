//! Touch feature implementation.
//!
//! Not all variants support the gesture/weight/size(they might be just zeros).
//! Here only minimum touch detection is implemented.

use crate::{Error, Ft6336, I2c};
use num_enum::{FromPrimitive, IntoPrimitive};

const REG_TOUCH_COUNT: u8 = 0x02;

/// Point action.
#[repr(u8)]
#[derive(IntoPrimitive, FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PointAction {
    PressDown,
    LiftUp,
    Contact,
    #[num_enum(default)]
    NoAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Point {
    pub index: u8,
    pub action: PointAction,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PointsIter {
    data: [u8; 11],
}

impl Iterator for PointsIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data[0] > 0 {
            self.data[0] -= 1;
            let index_base = 1 + (self.data[0] as usize) * 6;
            let p = Point {
                index: self.data[index_base + 2] >> 4,
                action: PointAction::from_primitive(self.data[index_base] >> 6),
                x: (((self.data[index_base] & 0xF) as u16) << 8)
                    + (self.data[index_base + 1] as u16),
                y: (((self.data[index_base + 2] & 0xF) as u16) << 8)
                    + (self.data[index_base + 3] as u16),
            };
            Some(p)
        } else {
            None
        }
    }
}

impl<I2C: I2c> Ft6336<I2C> {
    /// Reads current touch count.
    pub fn touch_count(&mut self) -> Result<u8, Error> {
        self.read_u8(REG_TOUCH_COUNT)
    }

    /// Reads all current touch information.
    pub fn touches_raw(&mut self) -> Result<[u8; 13], Error> {
        let mut buf: [u8; 13] = [0; 13];
        self.read_buf(REG_TOUCH_COUNT, &mut buf)?;
        Ok(buf)
    }

    /// Get an iterator over current touch events.
    pub fn touch_points_iter(&mut self) -> Result<PointsIter, Error> {
        let mut buf: [u8; 11] = [0; 11];
        self.read_buf(REG_TOUCH_COUNT, &mut buf)?;
        Ok(PointsIter { data: buf })
    }
}
