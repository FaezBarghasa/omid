package omid

/*
#cgo LDFLAGS: -lomid
#include "../../../include/omid.h"
*/
import "C"
import (
	"errors"
)

type EventType uint8

const (
	EventTypeUnknown        EventType = 0x00
	EventTypeAbsoluteChange EventType = 0x01
	EventTypeRelativeDelta  EventType = 0x02
	EventTypeKeyPress       EventType = 0x03
	EventTypeKeyRelease      EventType = 0x04
	EventTypeHapticFeedback EventType = 0x05
	EventTypeVisualUpdate   EventType = 0x06
	EventTypeSystemHandshake EventType = 0x07
)

type ForceProfile uint8

const (
	ForceProfileUnknown          ForceProfile = 0x00
	ForceProfileHammerStrike     ForceProfile = 0x01
	ForceProfileSpringTension    ForceProfile = 0x02
	ForceProfileKineticDampening ForceProfile = 0x03
)

const (
	FlagTouched   uint8 = 0x01
	FlagRawData   uint8 = 0x02
	FlagDirection uint8 = 0x04
)

// Flags helper
func NewFlags(touched, rawData, direction bool, timerDelta uint8) uint8 {
	return uint8(C.omid_flags_new(C.bool(touched), C.bool(rawData), C.bool(direction), C.uint8_t(timerDelta)))
}

func FlagsIsTouched(flags uint8) bool {
	return bool(C.omid_flags_is_touched(C.uint8_t(flags)))
}

func FlagsIsRawData(flags uint8) bool {
	return bool(C.omid_flags_is_raw_data(C.uint8_t(flags)))
}

func FlagsDirection(flags uint8) bool {
	return bool(C.omid_flags_direction(C.uint8_t(flags)))
}

func FlagsTimerDelta(flags uint8) uint8 {
	return uint8(C.omid_flags_timer_delta(C.uint8_t(flags)))
}

type Packet struct {
	ObjectID  uint16
	EventType uint8
	Flags     uint8
	Payload   uint32
}

func toCPacket(p Packet) C.OmidPacket {
	return C.OmidPacket{
		object_id:  C.uint16_t(p.ObjectID),
		event_type: C.uint8_t(p.EventType),
		flags:      C.uint8_t(p.Flags),
		payload:    C.uint32_t(p.Payload),
	}
}

func fromCPacket(cp C.OmidPacket) Packet {
	return Packet{
		ObjectID:  uint16(cp.object_id),
		EventType: uint8(cp.event_type),
		Flags:     uint8(cp.flags),
		Payload:   uint32(cp.payload),
	}
}

func NewPacket(objectID uint16, eventType uint8, flags uint8, payload uint32) Packet {
	return fromCPacket(C.omid_packet_new(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.uint32_t(payload)))
}

func NewPacketF32(objectID uint16, eventType uint8, flags uint8, val float32) Packet {
	return fromCPacket(C.omid_packet_new_f32(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.float(val)))
}

func NewPacketI32(objectID uint16, eventType uint8, flags uint8, val int32) Packet {
	return fromCPacket(C.omid_packet_new_i32(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.int32_t(val)))
}

func NewPacketU32(objectID uint16, eventType uint8, flags uint8, val uint32) Packet {
	return fromCPacket(C.omid_packet_new_u32(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.uint32_t(val)))
}

func NewPacketXY(objectID uint16, eventType uint8, flags uint8, x, y uint16) Packet {
	return fromCPacket(C.omid_packet_new_xy(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.uint16_t(x), C.uint16_t(y)))
}

func NewPacketHaptic(objectID uint16, profile uint8, intensity float32) Packet {
	return fromCPacket(C.omid_packet_new_haptic(C.uint16_t(objectID), C.uint8_t(profile), C.float(intensity)))
}

func NewPacketAdc12(objectID uint16, eventType uint8, flags uint8, val uint16) Packet {
	return fromCPacket(C.omid_packet_new_adc12(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.uint16_t(val)))
}

func NewPacketAdc16(objectID uint16, eventType uint8, flags uint8, val uint16) Packet {
	return fromCPacket(C.omid_packet_new_adc16(C.uint16_t(objectID), C.uint8_t(eventType), C.uint8_t(flags), C.uint16_t(val)))
}

func FromBytes(bytes []byte) (Packet, error) {
	if len(bytes) < 8 {
		return Packet{}, errors.New("invalid byte slice length")
	}
	cp := C.omid_packet_from_bytes((*C.uint8_t)(&bytes[0]))
	return fromCPacket(cp), nil
}

func (p Packet) ToBytes() []byte {
	bytes := make([]byte, 8)
	C.omid_packet_to_bytes(toCPacket(p), (*C.uint8_t)(&bytes[0]))
	return bytes
}

func (p Packet) PayloadAsF32() float32 {
	return float32(C.omid_packet_payload_as_f32(toCPacket(p)))
}

func (p Packet) PayloadAsI32() int32 {
	return int32(C.omid_packet_payload_as_i32(toCPacket(p)))
}

func (p Packet) PayloadAsU32() uint32 {
	return uint32(C.omid_packet_payload_as_u32(toCPacket(p)))
}

func (p Packet) PayloadAsXY() (uint16, uint16) {
	var x, y C.uint16_t
	C.omid_packet_payload_as_xy(toCPacket(p), &x, &y)
	return uint16(x), uint16(y)
}

func (p Packet) PayloadAsAdc12() uint16 {
	return uint16(C.omid_packet_payload_as_adc12(toCPacket(p)))
}

func (p Packet) PayloadAsAdc16() uint16 {
	return uint16(C.omid_packet_payload_as_adc16(toCPacket(p)))
}

func (p Packet) PayloadAsNormalizedF32(adcBits uint8) float32 {
	return float32(C.omid_packet_payload_as_normalized_f32(toCPacket(p), C.uint8_t(adcBits)))
}

func (p Packet) HapticIntensity() float32 {
	return float32(C.omid_packet_haptic_intensity(toCPacket(p)))
}

func (p Packet) HapticForceProfile() int32 {
	return int32(C.omid_packet_haptic_force_profile(toCPacket(p)))
}

type Topology struct {
	ObjectID   uint16
	ObjectType uint8
	SpatialX   uint16
	SpatialY   uint16
	Resolution uint8
}

func toCTopology(t Topology) C.TopologyDescriptor {
	return C.TopologyDescriptor{
		object_id:   C.uint16_t(t.ObjectID),
		object_type: C.uint8_t(t.ObjectType),
		spatial_x:   C.uint16_t(t.SpatialX),
		spatial_y:   C.uint16_t(t.SpatialY),
		resolution:  C.uint8_t(t.Resolution),
	}
}

func fromCTopology(ct C.TopologyDescriptor) Topology {
	return Topology{
		ObjectID:   uint16(ct.object_id),
		ObjectType: uint8(ct.object_type),
		SpatialX:   uint16(ct.spatial_x),
		SpatialY:   uint16(ct.spatial_y),
		Resolution: uint8(ct.resolution),
	}
}

func NewTopology(objectID uint16, objectType uint8, spatialX, spatialY uint16, resolution uint8) Topology {
	return fromCTopology(C.omid_topology_new(C.uint16_t(objectID), C.uint8_t(objectType), C.uint16_t(spatialX), C.uint16_t(spatialY), C.uint8_t(resolution)))
}

func TopologyFromBytes(bytes []byte) (Topology, error) {
	if len(bytes) < 8 {
		return Topology{}, errors.New("invalid byte slice length")
	}
	ct := C.omid_topology_from_bytes((*C.uint8_t)(&bytes[0]))
	return fromCTopology(ct), nil
}

func (t Topology) ToBytes() []byte {
	bytes := make([]byte, 8)
	C.omid_topology_to_bytes(toCTopology(t), (*C.uint8_t)(&bytes[0]))
	return bytes
}
