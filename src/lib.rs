#![no_std]

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod event;
pub mod packet;
pub mod queue;
pub mod topology;
pub mod uact;

#[cfg(feature = "std")]
pub mod dispatcher;
#[cfg(feature = "std")]
pub mod driver;

// Re-export core types for convenience
pub use event::{OmidEventType, OmidFlags, ForceProfile};
pub use packet::OmidPacket;
pub use topology::TopologyDescriptor;
pub use uact::{UactFrame, UactDemuxer, ClockSynchronizer};

#[cfg(feature = "std")]
pub use dispatcher::{OmidHostDispatcher, DispatcherStats};
#[cfg(feature = "std")]
pub use driver::{OmidDriver, MockHardwareDriver, LinuxDriver, WindowsDriver, MacosDriver, BleDriver, WifiDriver};


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

    #[test]
    #[cfg(feature = "std")]
    fn test_dispatcher_routing() {
        use std::sync::Arc;
        use std::sync::atomic::Ordering;
        use crate::queue::SpscRingBuffer;

        let stats = Arc::new(DispatcherStats::default());
        let q0 = Arc::new(SpscRingBuffer::new());
        let q1 = Arc::new(SpscRingBuffer::new());
        let queues = std::vec![q0.clone(), q1.clone()];

        let dispatcher = OmidHostDispatcher::new(2, queues.clone(), stats.clone());

        // Dispatch a KeyPress on object_id 0 (should route to q0: 0 % 2 == 0)
        let flags = OmidFlags::new(false, false, false, 0);
        let p0 = OmidPacket::new_adc16(0, EventType::KeyPress, flags, 1000);
        dispatcher.dispatch(p0, &queues).unwrap();

        // Dispatch a KeyRelease on object_id 1 (should route to q1: 1 % 2 == 1)
        let p1 = OmidPacket::new(1, EventType::KeyRelease as u8, 0, 0);
        dispatcher.dispatch(p1, &queues).unwrap();

        // Let the threads process
        std::thread::sleep(std::time::Duration::from_millis(50));

        assert_eq!(stats.key_presses.load(Ordering::Relaxed), 1);
        assert_eq!(stats.key_releases.load(Ordering::Relaxed), 1);

        dispatcher.shutdown();
    }

    #[test]
    fn test_uact_stream_and_sync() {
        // Create a UactFrame with 2 audio channels and a key press event
        let flags = OmidFlags::new(false, false, false, 16); // 16 ticks delta
        let packet = OmidPacket::new_adc16(10, EventType::KeyPress, flags, 32768);
        let audio_data = [0.1f32, -0.2f32];
        let frame = UactFrame::new(audio_data, packet);

        // Serialize
        let mut buffer = [0u8; 16];
        frame.serialize(&mut buffer).unwrap();

        // Deserialize
        let parsed = UactFrame::<2>::from_bytes(&buffer).unwrap();
        assert_eq!(parsed.audio, audio_data);
        assert_eq!(parsed.control.object_id, 10);
        assert_eq!(parsed.control.payload_as_adc16(), 32768);
        assert_eq!(parsed.control.typed_flags().subsample_offset(), 16);

        // Test Demuxer with chunking
        let mut demuxer = UactDemuxer::<2>::new();
        let mut frames_parsed = std::vec![];
        // Feed in two halves
        demuxer.process_bytes(&buffer[..8], |f| frames_parsed.push(f)).unwrap();
        assert_eq!(frames_parsed.len(), 0); // shouldn't parse a full frame yet
        demuxer.process_bytes(&buffer[8..], |f| frames_parsed.push(f)).unwrap();
        assert_eq!(frames_parsed.len(), 1);
        assert_eq!(frames_parsed[0], frame);

        // Test Clock Synchronizer
        // Clock = 122.88 MHz, sample rate = 192000 Hz.
        let sync = ClockSynchronizer::new(192000, 122880000.0);
        let seconds = sync.timer_delta_to_seconds(16);
        let expected_seconds = 16.0 / 122880000.0;
        assert!((seconds - expected_seconds).abs() < 1e-12);
        
        let sample_offset = sync.sample_offset(16);
        let expected_sample_offset = expected_seconds * 192000.0;
        assert!((sample_offset - expected_sample_offset).abs() < 1e-12);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_driver_and_overflow() {
        let hardware = MockHardwareDriver::new();
        let driver = LinuxDriver::new(hardware);

        // Test submission and polling of control packets
        let flags = OmidFlags::new(false, false, false, 0);
        let p = OmidPacket::new_adc12(5, EventType::AbsoluteChange, flags, 4000);
        driver.submit_control(p).unwrap();
        let polled = driver.poll_control().unwrap();
        assert_eq!(polled.object_id, 5);
        assert_eq!(polled.payload_as_adc12(), 4000);

        // Test audio submission and polling
        driver.submit_audio(0.75f32).unwrap();
        // Since submit_audio pushes to ep3_out, let's verify it got queued
        assert_eq!(driver.hardware.ep3_out.pop(), Some(0.75f32));

        // Test high-throughput control queue saturation (capacity is 4096)
        for i in 0..4096 {
            let p_i = OmidPacket::new_adc12(i as u16, EventType::AbsoluteChange, flags, i as u16);
            driver.submit_control(p_i).unwrap();
        }
        // Next push should fail due to saturation/overflow
        let p_overflow = OmidPacket::new_adc12(9999, EventType::AbsoluteChange, flags, 0);
        let res = driver.submit_control(p_overflow);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Queue Overflow - DSP Buffer Saturated");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_ble_driver() {
        let hardware = MockHardwareDriver::new();
        let mut driver = BleDriver::new(hardware, true, 256, true);
        
        // Assert initial state
        assert!(!driver.is_connected());
        assert_eq!(driver.mtu(), 256);
        assert!(driver.is_l2cap_coc_active());

        // When disconnected, submit should fail
        let flags = OmidFlags::new(false, false, false, 0);
        let p = OmidPacket::new_adc16(1, EventType::KeyPress, flags, 1000);
        assert!(driver.submit_control(p).is_err());

        // Connect and test
        driver.connect();
        assert!(driver.is_connected());
        driver.submit_control(p).unwrap();
        assert_eq!(driver.poll_control().unwrap().payload_as_adc16(), 1000);

        // MTU negotiations
        assert_eq!(driver.negotiate_mtu(600), 512); // Clamped at 512
        assert_eq!(driver.negotiate_mtu(10), 23);   // Clamped at 23
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_wifi_driver() {
        let hardware = MockHardwareDriver::new();
        let driver = WifiDriver::new(hardware, true, std::string::String::from("192.168.1.50"), 8000, true);

        // Assert initial state
        assert!(!driver.is_connected());
        assert_eq!(driver.ip_address(), "192.168.1.50");
        assert_eq!(driver.port(), 8000);
        assert!(driver.tcp_nodelay());
        assert!(driver.is_tcp());

        // Test connected state submission
        driver.connect();
        assert!(driver.is_connected());
        let flags = OmidFlags::new(false, false, false, 0);
        let p = OmidPacket::new_adc12(2, EventType::AbsoluteChange, flags, 2000);
        driver.submit_control(p).unwrap();
        assert_eq!(driver.poll_control().unwrap().payload_as_adc12(), 2000);
    }
}
