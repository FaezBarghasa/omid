#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    Unknown = 0x00,
    AbsoluteChange = 0x01,
    RelativeDelta = 0x02,
    KeyPress = 0x03,
    KeyRelease = 0x04,
    HapticFeedback = 0x05,
    VisualUpdate = 0x06,
    SystemHandshake = 0x07,
}

pub type OmidEventType = EventType;

impl EventType {
    pub const RelativeChange: Self = Self::RelativeDelta;

    pub fn from_u8(value: u8) -> Option<Self> {
        Self::try_from(value).ok()
    }
}

impl TryFrom<u8> for EventType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Unknown),
            0x01 => Ok(Self::AbsoluteChange),
            0x02 => Ok(Self::RelativeDelta),
            0x03 => Ok(Self::KeyPress),
            0x04 => Ok(Self::KeyRelease),
            0x05 => Ok(Self::HapticFeedback),
            0x06 => Ok(Self::VisualUpdate),
            0x07 => Ok(Self::SystemHandshake),
            other => Err(other),
        }
    }
}

impl From<EventType> for u8 {
    fn from(event_type: EventType) -> Self {
        event_type as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmidFlags(pub u8);

impl OmidFlags {
    pub const TOUCHED: u8 = 0b0000_0001;
    pub const RAW_DATA: u8 = 0b0000_0010;
    pub const DIRECTION: u8 = 0b0000_0100;

    #[inline(always)]
    pub fn is_touched(&self) -> bool {
        (self.0 & Self::TOUCHED) != 0
    }

    #[inline(always)]
    pub fn is_raw_data(&self) -> bool {
        (self.0 & Self::RAW_DATA) != 0
    }

    /// 0 for positive, 1 for negative increment.
    #[inline(always)]
    pub fn direction(&self) -> bool {
        (self.0 & Self::DIRECTION) != 0
    }

    /// High-resolution microsecond timer delta (sub-sample offset adjustment)
    #[inline(always)]
    pub fn timer_delta(&self) -> u8 {
        (self.0 & 0b1111_1000) >> 3
    }

    /// Alias for timer_delta() to support host DSP alignment
    #[inline(always)]
    pub fn subsample_offset(&self) -> u8 {
        self.timer_delta()
    }

    #[inline(always)]
    pub fn new(touched: bool, raw_data: bool, direction: bool, timer_delta: u8) -> Self {
        let mut bits = 0;
        if touched {
            bits |= Self::TOUCHED;
        }
        if raw_data {
            bits |= Self::RAW_DATA;
        }
        if direction {
            bits |= Self::DIRECTION;
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

impl From<i32> for OmidFlags {
    fn from(value: i32) -> Self {
        Self(value as u8)
    }
}

impl From<OmidFlags> for u8 {
    fn from(flags: OmidFlags) -> Self {
        flags.0
    }
}

