use crate::event::{EventType, OmidFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, align(4))]
pub struct OmidPacket {
    pub object_id: u16,
    pub event_type: u8,
    pub flags: u8,
    pub payload: u32,
}

impl OmidPacket {
    pub const FLAG_TOUCHED: u8 = OmidFlags::TOUCHED;
    pub const FLAG_RAW_DATA: u8 = OmidFlags::RAW_DATA;
    pub const FLAG_DIRECTION: u8 = OmidFlags::DIRECTION;

    /// Creates a new OmidPacket with raw fields.
    pub fn new(object_id: u16, event_type: u8, flags: u8, payload: u32) -> Self {
        Self {
            object_id,
            event_type,
            flags,
            payload,
        }
    }

    /// Serializes the packet to a little-endian 8-byte array.
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        let obj_bytes = self.object_id.to_le_bytes();
        bytes[0] = obj_bytes[0];
        bytes[1] = obj_bytes[1];
        bytes[2] = self.event_type;
        bytes[3] = self.flags;
        let payload_bytes = self.payload.to_le_bytes();
        bytes[4..8].copy_from_slice(&payload_bytes);
        bytes
    }

    /// Deserializes a packet from a little-endian 8-byte array.
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        let object_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        let event_type = bytes[2];
        let flags = bytes[3];
        let payload = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Self {
            object_id,
            event_type,
            flags,
            payload,
        }
    }

    /// Returns the parsed event type.
    pub fn event(&self) -> EventType {
        EventType::from_u8(self.event_type).unwrap_or(EventType::Unknown)
    }

    /// Returns the typed event type, if valid (for backward compatibility).
    pub fn typed_event_type(&self) -> Option<EventType> {
        EventType::from_u8(self.event_type)
    }

    /// Returns the flags container for this packet.
    pub fn typed_flags(&self) -> OmidFlags {
        OmidFlags(self.flags)
    }

    /// Checks if the event type is KeyPress (0x03).
    #[inline(always)]
    pub fn is_keypress(&self) -> bool {
        self.event_type == 0x03
    }

    /// Checks if the touched flag is set.
    #[inline(always)]
    pub fn is_touched(&self) -> bool {
        self.typed_flags().is_touched()
    }

    /// Checks if the raw data flag is set.
    #[inline(always)]
    pub fn is_raw_data(&self) -> bool {
        self.typed_flags().is_raw_data()
    }

    /// Checks the direction flag.
    #[inline(always)]
    pub fn direction(&self) -> bool {
        self.typed_flags().direction()
    }

    /// Reads the sub-sample timer offset/delta from the flags.
    #[inline(always)]
    pub fn subsample_offset(&self) -> u8 {
        self.typed_flags().subsample_offset()
    }

    // Payload conversions

    /// Reads the payload as a raw 32-bit unsigned integer (u32).
    #[inline(always)]
    pub fn payload_as_u32(&self) -> u32 {
        self.payload
    }

    /// Reads the payload as a 32-bit single-precision float (f32).
    #[inline(always)]
    pub fn payload_as_f32(&self) -> f32 {
        f32::from_bits(self.payload)
    }

    /// Reads the payload as a 32-bit signed integer (i32).
    #[inline(always)]
    pub fn payload_as_i32(&self) -> i32 {
        self.payload as i32
    }

    /// Reads the payload as dual-axis coordinates split into X (lower 16 bits) and Y (upper 16 bits).
    #[inline(always)]
    pub fn payload_as_xy(&self) -> (u16, u16) {
        let x = (self.payload & 0xFFFF) as u16;
        let y = ((self.payload >> 16) & 0xFFFF) as u16;
        (x, y)
    }

    // Constructors with specific payload types

    pub fn new_f32(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, val: f32) -> Self {
        let f: OmidFlags = flags.into();
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload: val.to_bits(),
        }
    }

    pub fn new_i32(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, val: i32) -> Self {
        let f: OmidFlags = flags.into();
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload: val as u32,
        }
    }

    pub fn new_xy(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, x: u16, y: u16) -> Self {
        let f: OmidFlags = flags.into();
        let payload = (x as u32) | ((y as u32) << 16);
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload,
        }
    }

    pub fn new_u32(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, val: u32) -> Self {
        let f: OmidFlags = flags.into();
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload: val,
        }
    }

    /// Creates a new haptic feedback packet.
    /// The force profile ID is encoded in the flags field.
    pub fn new_haptic(object_id: u16, profile: crate::event::ForceProfile, intensity: f32) -> Self {
        Self {
            object_id,
            event_type: crate::event::EventType::HapticFeedback as u8,
            flags: profile as u8,
            payload: intensity.to_bits(),
        }
    }

    /// Parses the force profile from the flags field for a haptic packet.
    #[inline(always)]
    pub fn haptic_force_profile(&self) -> Result<crate::event::ForceProfile, u8> {
        crate::event::ForceProfile::try_from(self.flags)
    }

    /// Extracts the haptic intensity payload as a 32-bit float.
    #[inline(always)]
    pub fn haptic_intensity(&self) -> f32 {
        self.payload_as_f32()
    }

    /// Constructs a packet containing a raw 12-bit ADC value, setting the RAW_DATA flag.
    pub fn new_adc12(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, val: u16) -> Self {
        let mut f: OmidFlags = flags.into();
        f.0 |= OmidFlags::RAW_DATA;
        let capped_val = val & 0x0FFF;
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload: capped_val as u32,
        }
    }

    /// Constructs a packet containing a raw 16-bit ADC value, setting the RAW_DATA flag.
    pub fn new_adc16(object_id: u16, event_type: EventType, flags: impl Into<OmidFlags>, val: u16) -> Self {
        let mut f: OmidFlags = flags.into();
        f.0 |= OmidFlags::RAW_DATA;
        Self {
            object_id,
            event_type: event_type as u8,
            flags: f.0,
            payload: val as u32,
        }
    }

    /// Extracts the raw 12-bit value from the packet payload.
    #[inline(always)]
    pub fn payload_as_adc12(&self) -> u16 {
        (self.payload & 0x0FFF) as u16
    }

    /// Extracts the raw 16-bit value from the packet payload.
    #[inline(always)]
    pub fn payload_as_adc16(&self) -> u16 {
        (self.payload & 0xFFFF) as u16
    }

    /// Normalizes a raw ADC value to 0.0..=1.0 dynamically based on the resolution in bits.
    #[inline(always)]
    pub fn payload_as_normalized_f32(&self, adc_bits: u8) -> f32 {
        if adc_bits == 0 {
            return 0.0;
        }
        let max_val = (1u32 << adc_bits) - 1;
        let val = self.payload & max_val;
        val as f32 / max_val as f32
    }
}

