// Omid (Object-MIDI) Dart FFI Bindings
// Designed for cross-platform integration in Flutter/Dart

import 'dart:ffi' as ffi;

// Struct representation in Dart FFI
final class OmidPacket extends ffi.Struct {
  @ffi.Uint16()
  external int objectId;

  @ffi.Uint8()
  external int eventType;

  @ffi.Uint8()
  external int flags;

  @ffi.Uint32()
  external int payload;
}

final class TopologyDescriptor extends ffi.Struct {
  @ffi.Uint16()
  external int objectId;

  @ffi.Uint8()
  external int objectType;

  @ffi.Uint16()
  external int spatialX;

  @ffi.Uint16()
  external int spatialY;

  @ffi.Uint8()
  external int resolution;
}

// Function signatures
typedef _omid_packet_new_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Uint32 payload,
);
typedef _omid_packet_new_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int payload,
);

typedef _omid_packet_new_f32_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Float val,
);
typedef _omid_packet_new_f32_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  double val,
);

typedef _omid_packet_new_i32_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Int32 val,
);
typedef _omid_packet_new_i32_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int val,
);

typedef _omid_packet_new_u32_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Uint32 val,
);
typedef _omid_packet_new_u32_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int val,
);

typedef _omid_packet_new_xy_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Uint16 x,
  ffi.Uint16 y,
);
typedef _omid_packet_new_xy_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int x,
  int y,
);

typedef _omid_packet_new_haptic_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 profile,
  ffi.Float intensity,
);
typedef _omid_packet_new_haptic_Dart = OmidPacket Function(
  int objectId,
  int profile,
  double intensity,
);

typedef _omid_packet_new_adc12_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Uint16 val,
);
typedef _omid_packet_new_adc12_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int val,
);

typedef _omid_packet_new_adc16_C = OmidPacket Function(
  ffi.Uint16 objectId,
  ffi.Uint8 eventType,
  ffi.Uint8 flags,
  ffi.Uint16 val,
);
typedef _omid_packet_new_adc16_Dart = OmidPacket Function(
  int objectId,
  int eventType,
  int flags,
  int val,
);

typedef _omid_packet_from_bytes_C = OmidPacket Function(ffi.Pointer<ffi.Uint8> bytes);
typedef _omid_packet_from_bytes_Dart = OmidPacket Function(ffi.Pointer<ffi.Uint8> bytes);

typedef _omid_packet_to_bytes_C = ffi.Void Function(OmidPacket packet, ffi.Pointer<ffi.Uint8> outBytes);
typedef _omid_packet_to_bytes_Dart = void Function(OmidPacket packet, ffi.Pointer<ffi.Uint8> outBytes);

typedef _omid_packet_payload_as_f32_C = ffi.Float Function(OmidPacket packet);
typedef _omid_packet_payload_as_f32_Dart = double Function(OmidPacket packet);

typedef _omid_packet_payload_as_i32_C = ffi.Int32 Function(OmidPacket packet);
typedef _omid_packet_payload_as_i32_Dart = int Function(OmidPacket packet);

typedef _omid_packet_payload_as_u32_C = ffi.Uint32 Function(OmidPacket packet);
typedef _omid_packet_payload_as_u32_Dart = int Function(OmidPacket packet);

typedef _omid_packet_payload_as_xy_C = ffi.Void Function(
  OmidPacket packet,
  ffi.Pointer<ffi.Uint16> outX,
  ffi.Pointer<ffi.Uint16> outY,
);
typedef _omid_packet_payload_as_xy_Dart = void Function(
  OmidPacket packet,
  ffi.Pointer<ffi.Uint16> outX,
  ffi.Pointer<ffi.Uint16> outY,
);

typedef _omid_packet_payload_as_adc12_C = ffi.Uint16 Function(OmidPacket packet);
typedef _omid_packet_payload_as_adc12_Dart = int Function(OmidPacket packet);

typedef _omid_packet_payload_as_adc16_C = ffi.Uint16 Function(OmidPacket packet);
typedef _omid_packet_payload_as_adc16_Dart = int Function(OmidPacket packet);

typedef _omid_packet_payload_as_normalized_f32_C = ffi.Float Function(OmidPacket packet, ffi.Uint8 adcBits);
typedef _omid_packet_payload_as_normalized_f32_Dart = double Function(OmidPacket packet, int adcBits);

typedef _omid_packet_haptic_intensity_C = ffi.Float Function(OmidPacket packet);
typedef _omid_packet_haptic_intensity_Dart = double Function(OmidPacket packet);

typedef _omid_packet_haptic_force_profile_C = ffi.Int32 Function(OmidPacket packet);
typedef _omid_packet_haptic_force_profile_Dart = int Function(OmidPacket packet);

typedef _omid_flags_new_C = ffi.Uint8 Function(
  ffi.Bool touched,
  ffi.Bool rawData,
  ffi.Bool direction,
  ffi.Uint8 timerDelta,
);
typedef _omid_flags_new_Dart = int Function(
  bool touched,
  bool rawData,
  bool direction,
  int timerDelta,
);

typedef _omid_flags_is_touched_C = ffi.Bool Function(ffi.Uint8 flags);
typedef _omid_flags_is_touched_Dart = bool Function(int flags);

typedef _omid_flags_is_raw_data_C = ffi.Bool Function(ffi.Uint8 flags);
typedef _omid_flags_is_raw_data_Dart = bool Function(int flags);

typedef _omid_flags_direction_C = ffi.Bool Function(ffi.Uint8 flags);
typedef _omid_flags_direction_Dart = bool Function(int flags);

typedef _omid_flags_timer_delta_C = ffi.Uint8 Function(ffi.Uint8 flags);
typedef _omid_flags_timer_delta_Dart = int Function(int flags);

typedef _omid_topology_new_C = TopologyDescriptor Function(
  ffi.Uint16 objectId,
  ffi.Uint8 objectType,
  ffi.Uint16 spatialX,
  ffi.Uint16 spatialY,
  ffi.Uint8 resolution,
);
typedef _omid_topology_new_Dart = TopologyDescriptor Function(
  int objectId,
  int objectType,
  int spatialX,
  int spatialY,
  int resolution,
);

typedef _omid_topology_from_bytes_C = TopologyDescriptor Function(ffi.Pointer<ffi.Uint8> bytes);
typedef _omid_topology_from_bytes_Dart = TopologyDescriptor Function(ffi.Pointer<ffi.Uint8> bytes);

typedef _omid_topology_to_bytes_C = ffi.Void Function(TopologyDescriptor desc, ffi.Pointer<ffi.Uint8> outBytes);
typedef _omid_topology_to_bytes_Dart = void Function(TopologyDescriptor desc, ffi.Pointer<ffi.Uint8> outBytes);

class OmidLibrary {
  final ffi.DynamicLibrary _dylib;

  late final _omid_packet_new_Dart packetNew;
  late final _omid_packet_new_f32_Dart packetNewF32;
  late final _omid_packet_new_i32_Dart packetNewI32;
  late final _omid_packet_new_u32_Dart packetNewU32;
  late final _omid_packet_new_xy_Dart packetNewXY;
  late final _omid_packet_new_haptic_Dart packetNewHaptic;
  late final _omid_packet_new_adc12_Dart packetNewAdc12;
  late final _omid_packet_new_adc16_Dart packetNewAdc16;
  late final _omid_packet_from_bytes_Dart packetFromBytes;
  late final _omid_packet_to_bytes_Dart packetToBytes;
  late final _omid_packet_payload_as_f32_Dart packetPayloadAsF32;
  late final _omid_packet_payload_as_i32_Dart packetPayloadAsI32;
  late final _omid_packet_payload_as_u32_Dart packetPayloadAsU32;
  late final _omid_packet_payload_as_xy_Dart packetPayloadAsXY;
  late final _omid_packet_payload_as_adc12_Dart packetPayloadAsAdc12;
  late final _omid_packet_payload_as_adc16_Dart packetPayloadAsAdc16;
  late final _omid_packet_payload_as_normalized_f32_Dart packetPayloadAsNormalizedF32;
  late final _omid_packet_haptic_intensity_Dart packetHapticIntensity;
  late final _omid_packet_haptic_force_profile_Dart packetHapticForceProfile;

  late final _omid_flags_new_Dart flagsNew;
  late final _omid_flags_is_touched_Dart flagsIsTouched;
  late final _omid_flags_is_raw_data_Dart flagsIsRawData;
  late final _omid_flags_direction_Dart flagsDirection;
  late final _omid_flags_timer_delta_Dart flagsTimerDelta;

  late final _omid_topology_new_Dart topologyNew;
  late final _omid_topology_from_bytes_Dart topologyFromBytes;
  late final _omid_topology_to_bytes_Dart topologyToBytes;

  OmidLibrary(String path) : _dylib = ffi.DynamicLibrary.open(path) {
    packetNew = _dylib
        .lookupFunction<_omid_packet_new_C, _omid_packet_new_Dart>('omid_packet_new');
    packetNewF32 = _dylib
        .lookupFunction<_omid_packet_new_f32_C, _omid_packet_new_f32_Dart>('omid_packet_new_f32');
    packetNewI32 = _dylib
        .lookupFunction<_omid_packet_new_i32_C, _omid_packet_new_i32_Dart>('omid_packet_new_i32');
    packetNewU32 = _dylib
        .lookupFunction<_omid_packet_new_u32_C, _omid_packet_new_u32_Dart>('omid_packet_new_u32');
    packetNewXY = _dylib
        .lookupFunction<_omid_packet_new_xy_C, _omid_packet_new_xy_Dart>('omid_packet_new_xy');
    packetNewHaptic = _dylib
        .lookupFunction<_omid_packet_new_haptic_C, _omid_packet_new_haptic_Dart>('omid_packet_new_haptic');
    packetNewAdc12 = _dylib
        .lookupFunction<_omid_packet_new_adc12_C, _omid_packet_new_adc12_Dart>('omid_packet_new_adc12');
    packetNewAdc16 = _dylib
        .lookupFunction<_omid_packet_new_adc16_C, _omid_packet_new_adc16_Dart>('omid_packet_new_adc16');
    packetFromBytes = _dylib
        .lookupFunction<_omid_packet_from_bytes_C, _omid_packet_from_bytes_Dart>('omid_packet_from_bytes');
    packetToBytes = _dylib
        .lookupFunction<_omid_packet_to_bytes_C, _omid_packet_to_bytes_Dart>('omid_packet_to_bytes');
    packetPayloadAsF32 = _dylib
        .lookupFunction<_omid_packet_payload_as_f32_C, _omid_packet_payload_as_f32_Dart>('omid_packet_payload_as_f32');
    packetPayloadAsI32 = _dylib
        .lookupFunction<_omid_packet_payload_as_i32_C, _omid_packet_payload_as_i32_Dart>('omid_packet_payload_as_i32');
    packetPayloadAsU32 = _dylib
        .lookupFunction<_omid_packet_payload_as_u32_C, _omid_packet_payload_as_u32_Dart>('omid_packet_payload_as_u32');
    packetPayloadAsXY = _dylib
        .lookupFunction<_omid_packet_payload_as_xy_C, _omid_packet_payload_as_xy_Dart>('omid_packet_payload_as_xy');
    packetPayloadAsAdc12 = _dylib
        .lookupFunction<_omid_packet_payload_as_adc12_C, _omid_packet_payload_as_adc12_Dart>('omid_packet_payload_as_adc12');
    packetPayloadAsAdc16 = _dylib
        .lookupFunction<_omid_packet_payload_as_adc16_C, _omid_packet_payload_as_adc16_Dart>('omid_packet_payload_as_adc16');
    packetPayloadAsNormalizedF32 = _dylib
        .lookupFunction<_omid_packet_payload_as_normalized_f32_C, _omid_packet_payload_as_normalized_f32_Dart>('omid_packet_payload_as_normalized_f32');
    packetHapticIntensity = _dylib
        .lookupFunction<_omid_packet_haptic_intensity_C, _omid_packet_haptic_intensity_Dart>('omid_packet_haptic_intensity');
    packetHapticForceProfile = _dylib
        .lookupFunction<_omid_packet_haptic_force_profile_C, _omid_packet_haptic_force_profile_Dart>('omid_packet_haptic_force_profile');

    flagsNew = _dylib
        .lookupFunction<_omid_flags_new_C, _omid_flags_new_Dart>('omid_flags_new');
    flagsIsTouched = _dylib
        .lookupFunction<_omid_flags_is_touched_C, _omid_flags_is_touched_Dart>('omid_flags_is_touched');
    flagsIsRawData = _dylib
        .lookupFunction<_omid_flags_is_raw_data_C, _omid_flags_is_raw_data_Dart>('omid_flags_is_raw_data');
    flagsDirection = _dylib
        .lookupFunction<_omid_flags_direction_C, _omid_flags_direction_Dart>('omid_flags_direction');
    flagsTimerDelta = _dylib
        .lookupFunction<_omid_flags_timer_delta_C, _omid_flags_timer_delta_Dart>('omid_flags_timer_delta');

    topologyNew = _dylib
        .lookupFunction<_omid_topology_new_C, _omid_topology_new_Dart>('omid_topology_new');
    topologyFromBytes = _dylib
        .lookupFunction<_omid_topology_from_bytes_C, _omid_topology_from_bytes_Dart>('omid_topology_from_bytes');
    topologyToBytes = _dylib
        .lookupFunction<_omid_topology_to_bytes_C, _omid_topology_to_bytes_Dart>('omid_topology_to_bytes');
  }
}
