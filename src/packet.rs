use crate::event::{OmidEventType, OmidFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, align(4))]
pub struct OmidPacket {
    pub object_id: u16,
    pub event_type: u8,
    pub flags: u8,
    pub payload: u32,
}

impl OmidPacket {
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

    /// Returns the typed event type, if valid.
    pub fn typed_event_type(&self) -> Option<OmidEventType> {
        OmidEventType::from_u8(self.event_type)
    }

    /// Returns the flags container for this packet.
    pub fn typed_flags(&self) -> OmidFlags {
        OmidFlags(self.flags)
    }

    // Payload conversions

    /// Reads the payload as a 32-bit single-precision float (f32).
    pub fn payload_as_f32(&self) -> f32 {
        f32::from_bits(self.payload)
    }

    /// Reads the payload as a 32-bit signed integer (i32).
    pub fn payload_as_i32(&self) -> i32 {
        self.payload as i32
    }

    /// Reads the payload as dual-axis coordinates split into X (lower 16 bits) and Y (upper 16 bits).
    pub fn payload_as_xy(&self) -> (u16, u16) {
        let x = (self.payload & 0xFFFF) as u16;
        let y = ((self.payload >> 16) & 0xFFFF) as u16;
        (x, y)
    }

    // Constructors with specific payload types

    pub fn new_f32(object_id: u16, event_type: OmidEventType, flags: OmidFlags, val: f32) -> Self {
        Self {
            object_id,
            event_type: event_type as u8,
            flags: flags.0,
            payload: val.to_bits(),
        }
    }

    pub fn new_i32(object_id: u16, event_type: OmidEventType, flags: OmidFlags, val: i32) -> Self {
        Self {
            object_id,
            event_type: event_type as u8,
            flags: flags.0,
            payload: val as u32,
        }
    }

    pub fn new_xy(object_id: u16, event_type: OmidEventType, flags: OmidFlags, x: u16, y: u16) -> Self {
        let payload = (x as u32) | ((y as u32) << 16);
        Self {
            object_id,
            event_type: event_type as u8,
            flags: flags.0,
            payload,
        }
    }

    pub fn new_u32(object_id: u16, event_type: OmidEventType, flags: OmidFlags, val: u32) -> Self {
        Self {
            object_id,
            event_type: event_type as u8,
            flags: flags.0,
            payload: val,
        }
    }
}
