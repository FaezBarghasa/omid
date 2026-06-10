#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OmidEventType {
    AbsoluteChange = 0x01,
    RelativeDelta = 0x02,
    KeyPress = 0x03,
    KeyRelease = 0x04,
    HapticFeedback = 0x05,
    VisualUpdate = 0x06,
    SystemHandshake = 0x07,
}

impl OmidEventType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::AbsoluteChange),
            0x02 => Some(Self::RelativeDelta),
            0x03 => Some(Self::KeyPress),
            0x04 => Some(Self::KeyRelease),
            0x05 => Some(Self::HapticFeedback),
            0x06 => Some(Self::VisualUpdate),
            0x07 => Some(Self::SystemHandshake),
            _ => None,
        }
    }
}

pub struct OmidFlags(pub u8);

impl OmidFlags {
    #[inline(always)]
    pub fn is_touched(&self) -> bool {
        (self.0 & 0b0000_0001) != 0
    }

    #[inline(always)]
    pub fn is_raw_data(&self) -> bool {
        (self.0 & 0b0000_0010) != 0
    }

    /// 0 for positive, 1 for negative increment.
    #[inline(always)]
    pub fn direction(&self) -> bool {
        (self.0 & 0b0000_0100) != 0
    }

    /// High-resolution microsecond timer delta (sub-sample offset adjustment)
    #[inline(always)]
    pub fn timer_delta(&self) -> u8 {
        (self.0 & 0b1111_1000) >> 3
    }

    #[inline(always)]
    pub fn new(touched: bool, raw_data: bool, direction: bool, timer_delta: u8) -> Self {
        let mut bits = 0;
        if touched {
            bits |= 0b0000_0001;
        }
        if raw_data {
            bits |= 0b0000_0010;
        }
        if direction {
            bits |= 0b0000_0100;
        }
        // Mask timer_delta to 5 bits (0..31) and shift it into position
        bits |= (timer_delta & 0b0001_1111) << 3;
        Self(bits)
    }
}

impl From<u8> for OmidFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<OmidFlags> for u8 {
    fn from(flags: OmidFlags) -> Self {
        flags.0
    }
}
