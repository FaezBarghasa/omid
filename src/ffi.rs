use crate::event::{EventType, ForceProfile, OmidFlags};
use crate::packet::OmidPacket;
use crate::topology::TopologyDescriptor;

/// Creates a new `OmidPacket` with raw fields.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new(object_id: u16, event_type: u8, flags: u8, payload: u32) -> OmidPacket {
    OmidPacket::new(object_id, event_type, flags, payload)
}

/// Creates a new `OmidPacket` carrying a 32-bit single-precision float.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_f32(object_id: u16, event_type: u8, flags: u8, val: f32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_f32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val.to_bits())
    }
}

/// Creates a new `OmidPacket` carrying a 32-bit signed integer.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_i32(object_id: u16, event_type: u8, flags: u8, val: i32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_i32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val as u32)
    }
}

/// Creates a new `OmidPacket` carrying a 32-bit unsigned integer.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_u32(object_id: u16, event_type: u8, flags: u8, val: u32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_u32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val)
    }
}

/// Creates a new `OmidPacket` carrying two 16-bit coordinates (X and Y).
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_xy(object_id: u16, event_type: u8, flags: u8, x: u16, y: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_xy(object_id, evt, OmidFlags(flags), x, y)
    } else {
        let payload = (x as u32) | ((y as u32) << 16);
        OmidPacket::new(object_id, event_type, flags, payload)
    }
}

/// Creates a new `OmidPacket` representing haptic feedback.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_haptic(object_id: u16, profile: u8, intensity: f32) -> OmidPacket {
    if let Ok(prof) = ForceProfile::try_from(profile) {
        OmidPacket::new_haptic(object_id, prof, intensity)
    } else {
        OmidPacket::new(object_id, EventType::HapticFeedback as u8, profile, intensity.to_bits())
    }
}

/// Creates a new `OmidPacket` carrying a 12-bit ADC value.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_adc12(object_id: u16, event_type: u8, flags: u8, val: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_adc12(object_id, evt, OmidFlags(flags), val)
    } else {
        let f = flags | OmidFlags::RAW_DATA;
        OmidPacket::new(object_id, event_type, f, (val & 0x0FFF) as u32)
    }
}

/// Creates a new `OmidPacket` carrying a 16-bit ADC value.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_adc16(object_id: u16, event_type: u8, flags: u8, val: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_adc16(object_id, evt, OmidFlags(flags), val)
    } else {
        let f = flags | OmidFlags::RAW_DATA;
        OmidPacket::new(object_id, event_type, f, val as u32)
    }
}

/// Deserializes a packet from a raw 8-byte array.
///
/// # Safety
///
/// The caller must ensure that the `bytes` pointer points to a valid, initialized block of at least 8 bytes
/// of memory and is safe to read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_packet_from_bytes(bytes: *const u8) -> OmidPacket {
    if bytes.is_null() {
        return OmidPacket::new(0, 0, 0, 0);
    }
    let mut arr = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 8);
    }
    OmidPacket::from_bytes(&arr)
}

/// Serializes a packet into a raw 8-byte buffer.
///
/// # Safety
///
/// The caller must ensure that the `out_bytes` pointer points to a valid, writable block of at least 8 bytes
/// of memory and is safe to write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_packet_to_bytes(packet: OmidPacket, out_bytes: *mut u8) {
    if !out_bytes.is_null() {
        let bytes = packet.to_bytes();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, 8);
        }
    }
}

/// Extracts the payload as a 32-bit float.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_f32(packet: OmidPacket) -> f32 {
    packet.payload_as_f32()
}

/// Extracts the payload as a 32-bit signed integer.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_i32(packet: OmidPacket) -> i32 {
    packet.payload_as_i32()
}

/// Extracts the payload as a 32-bit unsigned integer.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_u32(packet: OmidPacket) -> u32 {
    packet.payload_as_u32()
}

/// Extracts the XY coordinate payload from a packet.
///
/// # Safety
///
/// The caller must ensure that `out_x` and `out_y` are either null or point to valid, writable
/// 16-bit memory locations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_packet_payload_as_xy(packet: OmidPacket, out_x: *mut u16, out_y: *mut u16) {
    let (x, y) = packet.payload_as_xy();
    unsafe {
        if !out_x.is_null() {
            *out_x = x;
        }
        if !out_y.is_null() {
            *out_y = y;
        }
    }
}

/// Extracts the 12-bit ADC value from the packet payload.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_adc12(packet: OmidPacket) -> u16 {
    packet.payload_as_adc12()
}

/// Extracts the 16-bit ADC value from the packet payload.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_adc16(packet: OmidPacket) -> u16 {
    packet.payload_as_adc16()
}

/// Normalizes raw ADC values based on resolution bits.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_normalized_f32(packet: OmidPacket, adc_bits: u8) -> f32 {
    packet.payload_as_normalized_f32(adc_bits)
}

/// Returns the intensity payload for haptic events.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_haptic_intensity(packet: OmidPacket) -> f32 {
    packet.haptic_intensity()
}

/// Returns the force profile ID for haptic events, or -1 if invalid.
#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_haptic_force_profile(packet: OmidPacket) -> i32 {
    match packet.haptic_force_profile() {
        Ok(prof) => prof as i32,
        Err(_) => -1,
    }
}

/// Helper to construct a flags byte.
#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_new(touched: bool, raw_data: bool, direction: bool, timer_delta: u8) -> u8 {
    OmidFlags::new(touched, raw_data, direction, timer_delta).0
}

/// Helper to check the touched bit from raw flags.
#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_is_touched(flags: u8) -> bool {
    OmidFlags(flags).is_touched()
}

/// Helper to check the raw data bit from raw flags.
#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_is_raw_data(flags: u8) -> bool {
    OmidFlags(flags).is_raw_data()
}

/// Helper to check the direction bit from raw flags.
#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_direction(flags: u8) -> bool {
    OmidFlags(flags).direction()
}

/// Helper to check the timer delta from raw flags.
#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_timer_delta(flags: u8) -> u8 {
    OmidFlags(flags).timer_delta()
}

/// Creates a new `TopologyDescriptor`.
#[unsafe(no_mangle)]
pub extern "C" fn omid_topology_new(object_id: u16, object_type: u8, spatial_x: u16, spatial_y: u16, resolution: u8) -> TopologyDescriptor {
    TopologyDescriptor::new(object_id, object_type, spatial_x, spatial_y, resolution)
}

/// Deserializes a topology descriptor from a raw 8-byte array.
///
/// # Safety
///
/// The caller must ensure that the `bytes` pointer points to a valid, initialized block of at least 8 bytes
/// of memory and is safe to read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_topology_from_bytes(bytes: *const u8) -> TopologyDescriptor {
    if bytes.is_null() {
        return TopologyDescriptor::new(0, 0, 0, 0, 0);
    }
    let mut arr = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 8);
    }
    TopologyDescriptor::from_bytes(&arr)
}

/// Serializes a topology descriptor into a raw 8-byte buffer.
///
/// # Safety
///
/// The caller must ensure that the `out_bytes` pointer points to a valid, writable block of at least 8 bytes
/// of memory and is safe to write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_topology_to_bytes(desc: TopologyDescriptor, out_bytes: *mut u8) {
    if !out_bytes.is_null() {
        let bytes = desc.to_bytes();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, 8);
        }
    }
}
