// Omid (Object-MIDI) TypeScript FFI Bindings
// Designed for ultra-high performance in Bun, Deno, or Node.js environments

export interface OmidPacket {
  objectId: number;
  eventType: number;
  flags: number;
  payload: number;
}

export interface TopologyDescriptor {
  objectId: number;
  objectType: number;
  spatialX: number;
  spatialY: number;
  resolution: number;
}

export enum EventType {
  Unknown = 0x00,
  AbsoluteChange = 0x01,
  RelativeDelta = 0x02,
  KeyPress = 0x03,
  KeyRelease = 0x04,
  HapticFeedback = 0x05,
  VisualUpdate = 0x06,
  SystemHandshake = 0x07,
}

export enum ForceProfile {
  Unknown = 0x00,
  HammerStrike = 0x01,
  SpringTension = 0x02,
  KineticDampening = 0x03,
}

export const Flags = {
  TOUCHED: 0x01,
  RAW_DATA: 0x02,
  DIRECTION: 0x04,
};

// Bun FFI Definitions helper
// Note: Since OmidPacket and TopologyDescriptor are both exactly 8 bytes (64 bits),
// they can be passed/returned by value as "u64" in JS engines for maximum efficiency.
export const BunFFIDefinitions = {
  omid_packet_new: {
    args: ["u16", "u8", "u8", "u32"],
    returns: "u64",
  },
  omid_packet_new_f32: {
    args: ["u16", "u8", "u8", "f32"],
    returns: "u64",
  },
  omid_packet_new_i32: {
    args: ["u16", "u8", "u8", "i32"],
    returns: "u64",
  },
  omid_packet_new_u32: {
    args: ["u16", "u8", "u8", "u32"],
    returns: "u64",
  },
  omid_packet_new_xy: {
    args: ["u16", "u8", "u8", "u16", "u16"],
    returns: "u64",
  },
  omid_packet_new_haptic: {
    args: ["u16", "u8", "f32"],
    returns: "u64",
  },
  omid_packet_new_adc12: {
    args: ["u16", "u8", "u8", "u16"],
    returns: "u64",
  },
  omid_packet_new_adc16: {
    args: ["u16", "u8", "u8", "u16"],
    returns: "u64",
  },
  omid_packet_from_bytes: {
    args: ["ptr"],
    returns: "u64",
  },
  omid_packet_to_bytes: {
    args: ["u64", "ptr"],
    returns: "void",
  },
  omid_packet_payload_as_f32: {
    args: ["u64"],
    returns: "f32",
  },
  omid_packet_payload_as_i32: {
    args: ["u64"],
    returns: "i32",
  },
  omid_packet_payload_as_u32: {
    args: ["u64"],
    returns: "u32",
  },
  omid_packet_payload_as_xy: {
    args: ["u64", "ptr", "ptr"],
    returns: "void",
  },
  omid_packet_payload_as_adc12: {
    args: ["u64"],
    returns: "u16",
  },
  omid_packet_payload_as_adc16: {
    args: ["u64"],
    returns: "u16",
  },
  omid_packet_payload_as_normalized_f32: {
    args: ["u64", "u8"],
    returns: "f32",
  },
  omid_packet_haptic_intensity: {
    args: ["u64"],
    returns: "f32",
  },
  omid_packet_haptic_force_profile: {
    args: ["u64"],
    returns: "i32",
  },
  omid_flags_new: {
    args: ["bool", "bool", "bool", "u8"],
    returns: "u8",
  },
  omid_flags_is_touched: {
    args: ["u8"],
    returns: "bool",
  },
  omid_flags_is_raw_data: {
    args: ["u8"],
    returns: "bool",
  },
  omid_flags_direction: {
    args: ["u8"],
    returns: "bool",
  },
  omid_flags_timer_delta: {
    args: ["u8"],
    returns: "u8",
  },
  omid_topology_new: {
    args: ["u16", "u8", "u16", "u16", "u8"],
    returns: "u64",
  },
  omid_topology_from_bytes: {
    args: ["ptr"],
    returns: "u64",
  },
  omid_topology_to_bytes: {
    args: ["u64", "ptr"],
    returns: "void",
  },
} as const;

// Unpack/Pack OmidPacket from/to a 64-bit BigInt (u64 FFI value)
export function unpackPacket(val: bigint): OmidPacket {
  return {
    objectId: Number(val & 0xffffn),
    eventType: Number((val >> 16n) & 0xffn),
    flags: Number((val >> 24n) & 0xffn),
    payload: Number((val >> 32n) & 0xffffffffn),
  };
}

export function packPacket(p: OmidPacket): bigint {
  return (
    BigInt(p.objectId & 0xffff) |
    (BigInt(p.eventType & 0xff) << 16n) |
    (BigInt(p.flags & 0xff) << 24n) |
    (BigInt(p.payload & 0xffffffff) << 32n)
  );
}

// Unpack/Pack TopologyDescriptor from/to a 64-bit BigInt (u64 FFI value)
export function unpackTopology(val: bigint): TopologyDescriptor {
  return {
    objectId: Number(val & 0xffffn),
    objectType: Number((val >> 16n) & 0xffn),
    spatialX: Number((val >> 24n) & 0xffffn),
    spatialY: Number((val >> 40n) & 0xffffn),
    resolution: Number((val >> 56n) & 0xffn),
  };
}

export function packTopology(t: TopologyDescriptor): bigint {
  return (
    BigInt(t.objectId & 0xffff) |
    (BigInt(t.objectType & 0xff) << 16n) |
    (BigInt(t.spatialX & 0xffff) << 24n) |
    (BigInt(t.spatialY & 0xffff) << 40n) |
    (BigInt(t.resolution & 0xff) << 56n)
  );
}
