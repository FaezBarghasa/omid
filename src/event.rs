/// The type of action or transaction represented by an Omid control packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    /// Unrecognized or placeholder event.
    Unknown = 0x00,
    /// Absolute parameter change (e.g. fader move).
    AbsoluteChange = 0x01,
    /// Relative parameter adjustment (e.g. rotary encoder tick).
    RelativeDelta = 0x02,
    /// Key press event on a piano keyboard or pad.
    KeyPress = 0x03,
    /// Key release event on a piano keyboard or pad.
    KeyRelease = 0x04,
    /// Direct command/haptic vibration to the device's actuator.
    HapticFeedback = 0x05,
    /// Event to update local LEDs or displays on the controller.
    VisualUpdate = 0x06,
    /// Initial startup or clock sync handshake between host and controller.
    SystemHandshake = 0x07,
}

/// Alias for backward compatibility.
pub type OmidEventType = EventType;

impl EventType {
    /// Alias for relative delta changes.
    #[allow(non_upper_case_globals)]
    pub const RelativeChange: Self = Self::RelativeDelta;

    /// Attempts to parse an `EventType` from a raw byte.
    ///
    /// Returns `None` if the byte does not correspond to any known event type.
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        Self::try_from(value).ok()
    }
}

/// The force feedback profiles available for haptic actuators on OMID devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ForceProfile {
    /// Unspecified force feedback curve.
    Unknown = 0x00,
    /// Emulation of an acoustic piano hammer strike response.
    HammerStrike = 0x01,
    /// Simulated spring tension pull.
    SpringTension = 0x02,
    /// Simulated fluid or kinetic resistance dampening.
    KineticDampening = 0x03,
}

impl TryFrom<u8> for ForceProfile {
    type Error = u8;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Unknown),
            0x01 => Ok(Self::HammerStrike),
            0x02 => Ok(Self::SpringTension),
            0x03 => Ok(Self::KineticDampening),
            other => Err(other),
        }
    }
}

impl From<ForceProfile> for u8 {
    #[inline]
    fn from(profile: ForceProfile) -> Self {
        profile as u8
    }
}

impl TryFrom<u8> for EventType {
    type Error = u8;

    #[inline]
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
    #[inline]
    fn from(event_type: EventType) -> Self {
        event_type as u8
    }
}

/// A wrapper struct for the 8-bit packet configuration flags field.
///
/// Layout:
/// - Bit 0: Touched flag (active touch sensor)
/// - Bit 1: Raw Data flag (specifies if payload is ADC/raw integer vs normalized f32)
/// - Bit 2: Direction flag (0 for positive, 1 for negative increment)
/// - Bits 3..7: Sub-sample microsecond timer delta offset (0..31 ticks)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmidFlags(pub u8);

impl OmidFlags {
    /// Bitmask for the touched state.
    pub const TOUCHED: u8 = 0b0000_0001;
    /// Bitmask for the raw data flag.
    pub const RAW_DATA: u8 = 0b0000_0010;
    /// Bitmask for the direction flag.
    pub const DIRECTION: u8 = 0b0000_0100;

    /// Checks if the touched flag is active.
    #[inline(always)]
    pub fn is_touched(&self) -> bool {
        (self.0 & Self::TOUCHED) != 0
    }

    /// Checks if the raw data flag is active.
    #[inline(always)]
    pub fn is_raw_data(&self) -> bool {
        (self.0 & Self::RAW_DATA) != 0
    }

    /// Returns the direction bit (false for positive/forward, true for negative/backward).
    #[inline(always)]
    pub fn direction(&self) -> bool {
        (self.0 & Self::DIRECTION) != 0
    }

    /// Retrieves the sub-sample timer delta offset (0..=31).
    #[inline(always)]
    pub fn timer_delta(&self) -> u8 {
        (self.0 & 0b1111_1000) >> 3
    }

    /// Alias for `timer_delta()` to support host DSP alignment.
    #[inline(always)]
    pub fn subsample_offset(&self) -> u8 {
        self.timer_delta()
    }

    /// Creates a new `OmidFlags` wrapper from discrete flags and delta.
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
        bits |= (timer_delta & 0b0001_1111) << 3;
        Self(bits)
    }
}

impl From<u8> for OmidFlags {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<i32> for OmidFlags {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value as u8)
    }
}

impl From<OmidFlags> for u8 {
    #[inline]
    fn from(flags: OmidFlags) -> Self {
        flags.0
    }
}
