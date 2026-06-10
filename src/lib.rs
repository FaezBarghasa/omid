#![no_std]

pub mod event;
pub mod packet;
pub mod topology;

// Re-export core types for convenience
pub use event::{OmidEventType, OmidFlags};
pub use packet::OmidPacket;
pub use topology::TopologyDescriptor;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

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
}
