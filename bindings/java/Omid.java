package omid;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;

public interface Omid extends Library {
    Omid INSTANCE = Native.load("omid", Omid.class);

    @Structure.FieldOrder({"objectId", "eventType", "flags", "payload"})
    class OmidPacket extends Structure {
        public short objectId;
        public byte eventType;
        public byte flags;
        public int payload;

        public static class ByValue extends OmidPacket implements Structure.ByValue {}
    }

    @Structure.FieldOrder({"objectId", "objectType", "spatialX", "spatialY", "resolution"})
    class TopologyDescriptor extends Structure {
        public short objectId;
        public byte objectType;
        public short spatialX;
        public short spatialY;
        public byte resolution;

        public static class ByValue extends TopologyDescriptor implements Structure.ByValue {}
    }

    OmidPacket.ByValue omid_packet_new(short objectId, byte eventType, byte flags, int payload);
    OmidPacket.ByValue omid_packet_new_f32(short objectId, byte eventType, byte flags, float val);
    OmidPacket.ByValue omid_packet_new_i32(short objectId, byte eventType, byte flags, int val);
    OmidPacket.ByValue omid_packet_new_u32(short objectId, byte eventType, byte flags, int val);
    OmidPacket.ByValue omid_packet_new_xy(short objectId, byte eventType, byte flags, short x, short y);
    OmidPacket.ByValue omid_packet_new_haptic(short objectId, byte profile, float intensity);
    OmidPacket.ByValue omid_packet_new_adc12(short objectId, byte eventType, byte flags, short val);
    OmidPacket.ByValue omid_packet_new_adc16(short objectId, byte eventType, byte flags, short val);
    OmidPacket.ByValue omid_packet_from_bytes(Pointer bytes);
    void omid_packet_to_bytes(OmidPacket.ByValue packet, Pointer out_bytes);
    float omid_packet_payload_as_f32(OmidPacket.ByValue packet);
    int omid_packet_payload_as_i32(OmidPacket.ByValue packet);
    int omid_packet_payload_as_u32(OmidPacket.ByValue packet);
    void omid_packet_payload_as_xy(OmidPacket.ByValue packet, Pointer out_x, Pointer out_y);
    short omid_packet_payload_as_adc12(OmidPacket.ByValue packet);
    short omid_packet_payload_as_adc16(OmidPacket.ByValue packet);
    float omid_packet_payload_as_normalized_f32(OmidPacket.ByValue packet, byte adc_bits);
    float omid_packet_haptic_intensity(OmidPacket.ByValue packet);
    int omid_packet_haptic_force_profile(OmidPacket.ByValue packet);

    byte omid_flags_new(boolean touched, boolean raw_data, boolean direction, byte timer_delta);
    boolean omid_flags_is_touched(byte flags);
    boolean omid_flags_is_raw_data(byte flags);
    boolean omid_flags_direction(byte flags);
    byte omid_flags_timer_delta(byte flags);

    TopologyDescriptor.ByValue omid_topology_new(short objectId, byte objectType, short spatialX, short spatialY, byte resolution);
    TopologyDescriptor.ByValue omid_topology_from_bytes(Pointer bytes);
    void omid_topology_to_bytes(TopologyDescriptor.ByValue desc, Pointer out_bytes);
}
