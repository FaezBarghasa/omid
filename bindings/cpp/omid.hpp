#pragma once

#include "../../include/omid.h"
#include <cstdint>
#include <array>
#include <utility>

namespace omid {

enum class EventType : uint8_t {
    Unknown = 0x00,
    AbsoluteChange = 0x01,
    RelativeDelta = 0x02,
    KeyPress = 0x03,
    KeyRelease = 0x04,
    HapticFeedback = 0x05,
    VisualUpdate = 0x06,
    SystemHandshake = 0x07
};

enum class ForceProfile : uint8_t {
    Unknown = 0x00,
    HammerStrike = 0x01,
    SpringTension = 0x02,
    KineticDampening = 0x03
};

class Flags {
public:
    static constexpr uint8_t TOUCHED = 0x01;
    static constexpr uint8_t RAW_DATA = 0x02;
    static constexpr uint8_t DIRECTION = 0x04;

    static uint8_t create(bool touched, bool raw_data, bool direction, uint8_t timer_delta) {
        return omid_flags_new(touched, raw_data, direction, timer_delta);
    }

    static bool is_touched(uint8_t flags) {
        return omid_flags_is_touched(flags);
    }

    static bool is_raw_data(uint8_t flags) {
        return omid_flags_is_raw_data(flags);
    }

    static bool direction(uint8_t flags) {
        return omid_flags_direction(flags);
    }

    static uint8_t timer_delta(uint8_t flags) {
        return omid_flags_timer_delta(flags);
    }
};

class Packet {
public:
    ::OmidPacket inner;

    Packet() : inner{0, 0, 0, 0} {}
    Packet(::OmidPacket packet) : inner(packet) {}

    static Packet create(uint16_t object_id, uint8_t event_type, uint8_t flags, uint32_t payload) {
        return Packet(omid_packet_new(object_id, event_type, flags, payload));
    }

    static Packet create_f32(uint16_t object_id, uint8_t event_type, uint8_t flags, float val) {
        return Packet(omid_packet_new_f32(object_id, event_type, flags, val));
    }

    static Packet create_i32(uint16_t object_id, uint8_t event_type, uint8_t flags, int32_t val) {
        return Packet(omid_packet_new_i32(object_id, event_type, flags, val));
    }

    static Packet create_u32(uint16_t object_id, uint8_t event_type, uint8_t flags, uint32_t val) {
        return Packet(omid_packet_new_u32(object_id, event_type, flags, val));
    }

    static Packet create_xy(uint16_t object_id, uint8_t event_type, uint8_t flags, uint16_t x, uint16_t y) {
        return Packet(omid_packet_new_xy(object_id, event_type, flags, x, y));
    }

    static Packet create_haptic(uint16_t object_id, uint8_t profile, float intensity) {
        return Packet(omid_packet_new_haptic(object_id, profile, intensity));
    }

    static Packet create_adc12(uint16_t object_id, uint8_t event_type, uint8_t flags, uint16_t val) {
        return Packet(omid_packet_new_adc12(object_id, event_type, flags, val));
    }

    static Packet create_adc16(uint16_t object_id, uint8_t event_type, uint8_t flags, uint16_t val) {
        return Packet(omid_packet_new_adc16(object_id, event_type, flags, val));
    }

    static Packet from_bytes(const std::array<uint8_t, 8>& bytes) {
        return Packet(omid_packet_from_bytes(bytes.data()));
    }

    std::array<uint8_t, 8> to_bytes() const {
        std::array<uint8_t, 8> bytes;
        omid_packet_to_bytes(inner, bytes.data());
        return bytes;
    }

    uint16_t object_id() const { return inner.object_id; }
    uint8_t event_type() const { return inner.event_type; }
    uint8_t flags() const { return inner.flags; }
    uint32_t payload() const { return inner.payload; }

    float payload_as_f32() const { return omid_packet_payload_as_f32(inner); }
    int32_t payload_as_i32() const { return omid_packet_payload_as_i32(inner); }
    uint32_t payload_as_u32() const { return omid_packet_payload_as_u32(inner); }
    
    std::pair<uint16_t, uint16_t> payload_as_xy() const {
        uint16_t x = 0, y = 0;
        omid_packet_payload_as_xy(inner, &x, &y);
        return {x, y};
    }

    uint16_t payload_as_adc12() const { return omid_packet_payload_as_adc12(inner); }
    uint16_t payload_as_adc16() const { return omid_packet_payload_as_adc16(inner); }
    float payload_as_normalized_f32(uint8_t adc_bits) const { return omid_packet_payload_as_normalized_f32(inner, adc_bits); }

    float haptic_intensity() const { return omid_packet_haptic_intensity(inner); }
    int32_t haptic_force_profile() const { return omid_packet_haptic_force_profile(inner); }
};

class Topology {
public:
    ::TopologyDescriptor inner;

    Topology() : inner{0, 0, 0, 0, 0} {}
    Topology(::TopologyDescriptor desc) : inner(desc) {}

    static Topology create(uint16_t object_id, uint8_t object_type, uint16_t spatial_x, uint16_t spatial_y, uint8_t resolution) {
        return Topology(omid_topology_new(object_id, object_type, spatial_x, spatial_y, resolution));
    }

    static Topology from_bytes(const std::array<uint8_t, 8>& bytes) {
        return Topology(omid_topology_from_bytes(bytes.data()));
    }

    std::array<uint8_t, 8> to_bytes() const {
        std::array<uint8_t, 8> bytes;
        omid_topology_to_bytes(inner, bytes.data());
        return bytes;
    }
};

} // namespace omid
