#![no_std]

pub mod event;
pub mod packet;
pub mod topology;

// Re-export core types for convenience
pub use event::{OmidEventType, OmidFlags, ForceProfile};
pub use packet::OmidPacket;
pub use topology::TopologyDescriptor;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventType;

    #[test]
    fn test_packet_f32_roundtrip() {
        let flags = OmidFlags::new(true, false, false, 15);
        let packet = OmidPacket::new_f32(0x1234, OmidEventType::AbsoluteChange, flags, 0.75f32);
        
        let bytes = packet.to_bytes();
        let parsed = OmidPacket::from_bytes(&bytes);
        
        assert_eq!(parsed.object_id, 0x1234);
        assert_eq!(parsed.typed_event_type(), Some(OmidEventType::AbsoluteChange));
        assert!(parsed.typed_flags().is_touched());
        assert!(!parsed.typed_flags().is_raw_data());
        assert_eq!(parsed.typed_flags().timer_delta(), 15);
        assert_eq!(parsed.payload_as_f32(), 0.75f32);
    }

    #[test]
    fn test_packet_xy_roundtrip() {
        let flags = OmidFlags::new(false, true, true, 0);
        let packet = OmidPacket::new_xy(0xABCD, OmidEventType::AbsoluteChange, flags, 1000, 2000);
        
        let bytes = packet.to_bytes();
        let parsed = OmidPacket::from_bytes(&bytes);
        
        assert_eq!(parsed.object_id, 0xABCD);
        assert!(!parsed.typed_flags().is_touched());
        assert!(parsed.typed_flags().is_raw_data());
        assert!(parsed.typed_flags().direction());
        assert_eq!(parsed.payload_as_xy(), (1000, 2000));
    }

    #[test]
    fn test_topology_roundtrip() {
        let desc = TopologyDescriptor::new(0x4321, 0x02, 65000, 12000, 12);
        let bytes = desc.to_bytes();
        let parsed = TopologyDescriptor::from_bytes(&bytes);
        
        assert_eq!(parsed.object_id, 0x4321);
        assert_eq!(parsed.object_type, 0x02);
        assert_eq!(parsed.spatial_x, 65000);
        assert_eq!(parsed.spatial_y, 12000);
        assert_eq!(parsed.resolution, 12);
    }

    #[test]
    fn test_haptic_roundtrip() {
        let packet = OmidPacket::new_haptic(0x5678, ForceProfile::SpringTension, 0.85f32);
        let bytes = packet.to_bytes();
        let parsed = OmidPacket::from_bytes(&bytes);

        assert_eq!(parsed.object_id, 0x5678);
        assert_eq!(parsed.event(), EventType::HapticFeedback);
        assert_eq!(parsed.haptic_force_profile(), Ok(ForceProfile::SpringTension));
        assert_eq!(parsed.haptic_intensity(), 0.85f32);
    }

    #[test]
    fn test_adc_roundtrips() {
        // Test 12-bit ADC fader (value 2048 / 4095)
        let flags = OmidFlags::new(false, false, false, 0);
        let packet12 = OmidPacket::new_adc12(1, EventType::AbsoluteChange, flags, 2048);
        assert!(packet12.is_raw_data());
        assert_eq!(packet12.payload_as_adc12(), 2048);
        let norm12 = packet12.payload_as_normalized_f32(12);
        assert!((norm12 - (2048.0 / 4095.0)).abs() < 1e-6);

        // Test 16-bit ADC key (value 32768 / 65535)
        let packet16 = OmidPacket::new_adc16(2, EventType::KeyPress, flags, 32768);
        assert!(packet16.is_raw_data());
        assert_eq!(packet16.payload_as_adc16(), 32768);
        let norm16 = packet16.payload_as_normalized_f32(16);
        assert!((norm16 - (32768.0 / 65535.0)).abs() < 1e-6);
    }
}
