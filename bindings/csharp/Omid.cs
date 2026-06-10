using System;
using System.Runtime.InteropServices;

namespace Omid
{
    public enum EventType : byte
    {
        Unknown = 0x00,
        AbsoluteChange = 0x01,
        RelativeDelta = 0x02,
        KeyPress = 0x03,
        KeyRelease = 0x04,
        HapticFeedback = 0x05,
        VisualUpdate = 0x06,
        SystemHandshake = 0x07
    }

    public enum ForceProfile : byte
    {
        Unknown = 0x00,
        HammerStrike = 0x01,
        SpringTension = 0x02,
        KineticDampening = 0x03
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct OmidPacket
    {
        public ushort object_id;
        public byte event_type;
        public byte flags;
        public uint payload;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct TopologyDescriptor
    {
        public ushort object_id;
        public byte object_type;
        public ushort spatial_x;
        public ushort spatial_y;
        public byte resolution;
    }

    public static class Native
    {
        private const string LibName = "omid";

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new(ushort object_id, byte event_type, byte flags, uint payload);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_f32(ushort object_id, byte event_type, byte flags, float val);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_i32(ushort object_id, byte event_type, byte flags, int val);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_u32(ushort object_id, byte event_type, byte flags, uint val);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_xy(ushort object_id, byte event_type, byte flags, ushort x, ushort y);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_haptic(ushort object_id, byte profile, float intensity);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_adc12(ushort object_id, byte event_type, byte flags, ushort val);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern OmidPacket omid_packet_new_adc16(ushort object_id, byte event_type, byte flags, ushort val);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern unsafe OmidPacket omid_packet_from_bytes(byte* bytes);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern unsafe void omid_packet_to_bytes(OmidPacket packet, byte* out_bytes);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern float omid_packet_payload_as_f32(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int omid_packet_payload_as_i32(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern uint omid_packet_payload_as_u32(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern unsafe void omid_packet_payload_as_xy(OmidPacket packet, ushort* out_x, ushort* out_y);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern ushort omid_packet_payload_as_adc12(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern ushort omid_packet_payload_as_adc16(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern float omid_packet_payload_as_normalized_f32(OmidPacket packet, byte adc_bits);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern float omid_packet_haptic_intensity(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int omid_packet_haptic_force_profile(OmidPacket packet);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern byte omid_flags_new(bool touched, bool raw_data, bool direction, byte timer_delta);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern bool omid_flags_is_touched(byte flags);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern bool omid_flags_is_raw_data(byte flags);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern bool omid_flags_direction(byte flags);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern byte omid_flags_timer_delta(byte flags);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern TopologyDescriptor omid_topology_new(ushort object_id, byte object_type, ushort spatial_x, ushort spatial_y, byte resolution);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern unsafe TopologyDescriptor omid_topology_from_bytes(byte* bytes);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern unsafe void omid_topology_to_bytes(TopologyDescriptor desc, byte* out_bytes);
    }
}
