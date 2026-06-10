use crate::event::{EventType, ForceProfile, OmidFlags};
use crate::packet::OmidPacket;
use crate::topology::TopologyDescriptor;

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new(object_id: u16, event_type: u8, flags: u8, payload: u32) -> OmidPacket {
    OmidPacket::new(object_id, event_type, flags, payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_f32(object_id: u16, event_type: u8, flags: u8, val: f32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_f32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val.to_bits())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_i32(object_id: u16, event_type: u8, flags: u8, val: i32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_i32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val as u32)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_u32(object_id: u16, event_type: u8, flags: u8, val: u32) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_u32(object_id, evt, OmidFlags(flags), val)
    } else {
        OmidPacket::new(object_id, event_type, flags, val)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_xy(object_id: u16, event_type: u8, flags: u8, x: u16, y: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_xy(object_id, evt, OmidFlags(flags), x, y)
    } else {
        let payload = (x as u32) | ((y as u32) << 16);
        OmidPacket::new(object_id, event_type, flags, payload)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_haptic(object_id: u16, profile: u8, intensity: f32) -> OmidPacket {
    if let Ok(prof) = ForceProfile::try_from(profile) {
        OmidPacket::new_haptic(object_id, prof, intensity)
    } else {
        OmidPacket::new(object_id, EventType::HapticFeedback as u8, profile, intensity.to_bits())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_adc12(object_id: u16, event_type: u8, flags: u8, val: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_adc12(object_id, evt, OmidFlags(flags), val)
    } else {
        let f = flags | OmidFlags::RAW_DATA;
        OmidPacket::new(object_id, event_type, f, (val & 0x0FFF) as u32)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_new_adc16(object_id: u16, event_type: u8, flags: u8, val: u16) -> OmidPacket {
    if let Ok(evt) = EventType::try_from(event_type) {
        OmidPacket::new_adc16(object_id, evt, OmidFlags(flags), val)
    } else {
        let f = flags | OmidFlags::RAW_DATA;
        OmidPacket::new(object_id, event_type, f, val as u32)
    }
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_packet_to_bytes(packet: OmidPacket, out_bytes: *mut u8) {
    if !out_bytes.is_null() {
        let bytes = packet.to_bytes();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, 8);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_f32(packet: OmidPacket) -> f32 {
    packet.payload_as_f32()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_i32(packet: OmidPacket) -> i32 {
    packet.payload_as_i32()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_u32(packet: OmidPacket) -> u32 {
    packet.payload_as_u32()
}

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

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_adc12(packet: OmidPacket) -> u16 {
    packet.payload_as_adc12()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_adc16(packet: OmidPacket) -> u16 {
    packet.payload_as_adc16()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_payload_as_normalized_f32(packet: OmidPacket, adc_bits: u8) -> f32 {
    packet.payload_as_normalized_f32(adc_bits)
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_haptic_intensity(packet: OmidPacket) -> f32 {
    packet.haptic_intensity()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_packet_haptic_force_profile(packet: OmidPacket) -> i32 {
    match packet.haptic_force_profile() {
        Ok(prof) => prof as i32,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_new(touched: bool, raw_data: bool, direction: bool, timer_delta: u8) -> u8 {
    OmidFlags::new(touched, raw_data, direction, timer_delta).0
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_is_touched(flags: u8) -> bool {
    OmidFlags(flags).is_touched()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_is_raw_data(flags: u8) -> bool {
    OmidFlags(flags).is_raw_data()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_direction(flags: u8) -> bool {
    OmidFlags(flags).direction()
}

#[unsafe(no_mangle)]
pub extern "C" fn omid_flags_timer_delta(flags: u8) -> u8 {
    OmidFlags(flags).timer_delta()
}

// Topology Descriptor FFI
#[unsafe(no_mangle)]
pub extern "C" fn omid_topology_new(object_id: u16, object_type: u8, spatial_x: u16, spatial_y: u16, resolution: u8) -> TopologyDescriptor {
    TopologyDescriptor::new(object_id, object_type, spatial_x, spatial_y, resolution)
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn omid_topology_to_bytes(desc: TopologyDescriptor, out_bytes: *mut u8) {
    if !out_bytes.is_null() {
        let bytes = desc.to_bytes();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, 8);
        }
    }
}
