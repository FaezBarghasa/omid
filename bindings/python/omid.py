import ctypes
import os
import sys

# Try to find and load the shared library
lib_names = {
    "darwin": "libomid.dylib",
    "win32": "omid.dll",
    "linux": "libomid.so"
}

lib_file = lib_names.get(sys.platform, "libomid.so")
# Search patterns for cargo build outputs
search_paths = [
    os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", lib_file),
    os.path.join(os.path.dirname(__file__), "..", "..", "target", "release", lib_file),
    os.path.join(os.path.dirname(__file__), lib_file),
]

_lib = None
for path in search_paths:
    if os.path.exists(path):
        try:
            _lib = ctypes.CDLL(path)
            break
        except Exception:
            pass

# Structs
class OmidPacket(ctypes.Structure):
    _fields_ = [
        ("object_id", ctypes.c_uint16),
        ("event_type", ctypes.c_uint8),
        ("flags", ctypes.c_uint8),
        ("payload", ctypes.c_uint32)
    ]

class TopologyDescriptor(ctypes.Structure):
    _fields_ = [
        ("object_id", ctypes.c_uint16),
        ("object_type", ctypes.c_uint8),
        ("spatial_x", ctypes.c_uint16),
        ("spatial_y", ctypes.c_uint16),
        ("resolution", ctypes.c_uint8)
    ]

if _lib:
    # Configure argument and return types
    _lib.omid_packet_new.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint32]
    _lib.omid_packet_new.restype = OmidPacket

    _lib.omid_packet_new_f32.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_float]
    _lib.omid_packet_new_f32.restype = OmidPacket

    _lib.omid_packet_new_i32.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_int32]
    _lib.omid_packet_new_i32.restype = OmidPacket

    _lib.omid_packet_new_u32.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint32]
    _lib.omid_packet_new_u32.restype = OmidPacket

    _lib.omid_packet_new_xy.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint16, ctypes.c_uint16]
    _lib.omid_packet_new_xy.restype = OmidPacket

    _lib.omid_packet_new_haptic.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_float]
    _lib.omid_packet_new_haptic.restype = OmidPacket

    _lib.omid_packet_new_adc12.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint16]
    _lib.omid_packet_new_adc12.restype = OmidPacket

    _lib.omid_packet_new_adc16.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint16]
    _lib.omid_packet_new_adc16.restype = OmidPacket

    _lib.omid_packet_from_bytes.argtypes = [ctypes.c_char_p]
    _lib.omid_packet_from_bytes.restype = OmidPacket

    _lib.omid_packet_to_bytes.argtypes = [OmidPacket, ctypes.c_char_p]
    _lib.omid_packet_to_bytes.restype = None

    _lib.omid_packet_payload_as_f32.argtypes = [OmidPacket]
    _lib.omid_packet_payload_as_f32.restype = ctypes.c_float

    _lib.omid_packet_payload_as_i32.argtypes = [OmidPacket]
    _lib.omid_packet_payload_as_i32.restype = ctypes.c_int32

    _lib.omid_packet_payload_as_u32.argtypes = [OmidPacket]
    _lib.omid_packet_payload_as_u32.restype = ctypes.c_uint32

    _lib.omid_packet_payload_as_xy.argtypes = [OmidPacket, ctypes.POINTER(ctypes.c_uint16), ctypes.POINTER(ctypes.c_uint16)]
    _lib.omid_packet_payload_as_xy.restype = None

    _lib.omid_packet_payload_as_adc12.argtypes = [OmidPacket]
    _lib.omid_packet_payload_as_adc12.restype = ctypes.c_uint16

    _lib.omid_packet_payload_as_adc16.argtypes = [OmidPacket]
    _lib.omid_packet_payload_as_adc16.restype = ctypes.c_uint16

    _lib.omid_packet_payload_as_normalized_f32.argtypes = [OmidPacket, ctypes.c_uint8]
    _lib.omid_packet_payload_as_normalized_f32.restype = ctypes.c_float

    _lib.omid_packet_haptic_intensity.argtypes = [OmidPacket]
    _lib.omid_packet_haptic_intensity.restype = ctypes.c_float

    _lib.omid_packet_haptic_force_profile.argtypes = [OmidPacket]
    _lib.omid_packet_haptic_force_profile.restype = ctypes.c_int32

    _lib.omid_flags_new.argtypes = [ctypes.c_bool, ctypes.c_bool, ctypes.c_bool, ctypes.c_uint8]
    _lib.omid_flags_new.restype = ctypes.c_uint8

    _lib.omid_flags_is_touched.argtypes = [ctypes.c_uint8]
    _lib.omid_flags_is_touched.restype = ctypes.c_bool

    _lib.omid_flags_is_raw_data.argtypes = [ctypes.c_uint8]
    _lib.omid_flags_is_raw_data.restype = ctypes.c_bool

    _lib.omid_flags_direction.argtypes = [ctypes.c_uint8]
    _lib.omid_flags_direction.restype = ctypes.c_bool

    _lib.omid_flags_timer_delta.argtypes = [ctypes.c_uint8]
    _lib.omid_flags_timer_delta.restype = ctypes.c_uint8

    _lib.omid_topology_new.argtypes = [ctypes.c_uint16, ctypes.c_uint8, ctypes.c_uint16, ctypes.c_uint16, ctypes.c_uint8]
    _lib.omid_topology_new.restype = TopologyDescriptor

    _lib.omid_topology_from_bytes.argtypes = [ctypes.c_char_p]
    _lib.omid_topology_from_bytes.restype = TopologyDescriptor

    _lib.omid_topology_to_bytes.argtypes = [TopologyDescriptor, ctypes.c_char_p]
    _lib.omid_topology_to_bytes.restype = None

# Python wrappers for clean usability
class Omid:
    @staticmethod
    def _check_lib():
        if not _lib:
            raise RuntimeError("Omid shared library could not be loaded. Ensure the dynamic library (e.g. libomid.so) is compiled and in target/debug, target/release, or the script's directory.")

    @classmethod
    def create_packet(cls, object_id: int, event_type: int, flags: int, payload: int) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new(object_id, event_type, flags, payload)

    @classmethod
    def create_packet_f32(cls, object_id: int, event_type: int, flags: int, val: float) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_f32(object_id, event_type, flags, val)

    @classmethod
    def create_packet_i32(cls, object_id: int, event_type: int, flags: int, val: int) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_i32(object_id, event_type, flags, val)

    @classmethod
    def create_packet_xy(cls, object_id: int, event_type: int, flags: int, x: int, y: int) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_xy(object_id, event_type, flags, x, y)

    @classmethod
    def create_packet_haptic(cls, object_id: int, profile: int, intensity: float) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_haptic(object_id, profile, intensity)

    @classmethod
    def create_packet_adc12(cls, object_id: int, event_type: int, flags: int, val: int) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_adc12(object_id, event_type, flags, val)

    @classmethod
    def create_packet_adc16(cls, object_id: int, event_type: int, flags: int, val: int) -> OmidPacket:
        cls._check_lib()
        return _lib.omid_packet_new_adc16(object_id, event_type, flags, val)

    @classmethod
    def packet_from_bytes(cls, data: bytes) -> OmidPacket:
        cls._check_lib()
        if len(data) != 8:
            raise ValueError("Data must be exactly 8 bytes")
        return _lib.omid_packet_from_bytes(data)

    @classmethod
    def packet_to_bytes(cls, packet: OmidPacket) -> bytes:
        cls._check_lib()
        buf = ctypes.create_string_buffer(8)
        _lib.omid_packet_to_bytes(packet, buf)
        return buf.raw

    @classmethod
    def get_payload_f32(cls, packet: OmidPacket) -> float:
        cls._check_lib()
        return _lib.omid_packet_payload_as_f32(packet)

    @classmethod
    def get_payload_i32(cls, packet: OmidPacket) -> int:
        cls._check_lib()
        return _lib.omid_packet_payload_as_i32(packet)

    @classmethod
    def get_payload_u32(cls, packet: OmidPacket) -> int:
        cls._check_lib()
        return _lib.omid_packet_payload_as_u32(packet)

    @classmethod
    def get_payload_xy(cls, packet: OmidPacket) -> tuple:
        cls._check_lib()
        x = ctypes.c_uint16(0)
        y = ctypes.c_uint16(0)
        _lib.omid_packet_payload_as_xy(packet, ctypes.byref(x), ctypes.byref(y))
        return (x.value, y.value)

    @classmethod
    def get_payload_adc12(cls, packet: OmidPacket) -> int:
        cls._check_lib()
        return _lib.omid_packet_payload_as_adc12(packet)

    @classmethod
    def get_payload_adc16(cls, packet: OmidPacket) -> int:
        cls._check_lib()
        return _lib.omid_packet_payload_as_adc16(packet)

    @classmethod
    def get_payload_normalized(cls, packet: OmidPacket, bits: int) -> float:
        cls._check_lib()
        return _lib.omid_packet_payload_as_normalized_f32(packet, bits)

    @classmethod
    def get_haptic_intensity(cls, packet: OmidPacket) -> float:
        cls._check_lib()
        return _lib.omid_packet_haptic_intensity(packet)

    @classmethod
    def get_haptic_force_profile(cls, packet: OmidPacket) -> int:
        cls._check_lib()
        return _lib.omid_packet_haptic_force_profile(packet)

    @classmethod
    def create_flags(cls, touched: bool, raw_data: bool, direction: bool, timer_delta: int) -> int:
        cls._check_lib()
        return _lib.omid_flags_new(touched, raw_data, direction, timer_delta)

    @classmethod
    def flags_is_touched(cls, flags: int) -> bool:
        cls._check_lib()
        return _lib.omid_flags_is_touched(flags)

    @classmethod
    def flags_is_raw_data(cls, flags: int) -> bool:
        cls._check_lib()
        return _lib.omid_flags_is_raw_data(flags)

    @classmethod
    def flags_direction(cls, flags: int) -> bool:
        cls._check_lib()
        return _lib.omid_flags_direction(flags)

    @classmethod
    def flags_timer_delta(cls, flags: int) -> int:
        cls._check_lib()
        return _lib.omid_flags_timer_delta(flags)

    @classmethod
    def create_topology(cls, object_id: int, object_type: int, spatial_x: int, spatial_y: int, resolution: int) -> TopologyDescriptor:
        cls._check_lib()
        return _lib.omid_topology_new(object_id, object_type, spatial_x, spatial_y, resolution)

    @classmethod
    def topology_from_bytes(cls, data: bytes) -> TopologyDescriptor:
        cls._check_lib()
        if len(data) != 8:
            raise ValueError("Data must be exactly 8 bytes")
        return _lib.omid_topology_from_bytes(data)

    @classmethod
    def topology_to_bytes(cls, desc: TopologyDescriptor) -> bytes:
        cls._check_lib()
        buf = ctypes.create_string_buffer(8)
        _lib.omid_topology_to_bytes(desc, buf)
        return buf.raw
