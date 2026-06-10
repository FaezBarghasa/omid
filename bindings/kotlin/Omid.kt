package omid

import com.sun.jna.Pointer
import com.sun.jna.Memory

enum class EventType(val value: Byte) {
    Unknown(0x00),
    AbsoluteChange(0x01),
    RelativeDelta(0x02),
    KeyPress(0x03),
    KeyRelease(0x04),
    HapticFeedback(0x05),
    VisualUpdate(0x06),
    SystemHandshake(0x07);

    companion object {
        fun fromByte(value: Byte): EventType =
            values().find { it.value == value } ?: Unknown
    }
}

enum class ForceProfile(val value: Byte) {
    Unknown(0x00),
    HammerStrike(0x01),
    SpringTension(0x02),
    KineticDampening(0x03);

    companion object {
        fun fromByte(value: Byte): ForceProfile =
            values().find { it.value == value } ?: Unknown
    }
}

class OmidFlags(val raw: Byte) {
    val isTouched: Boolean get() = Omid.INSTANCE.omid_flags_is_touched(raw)
    val isRawData: Boolean get() = Omid.INSTANCE.omid_flags_is_raw_data(raw)
    val direction: Boolean get() = Omid.INSTANCE.omid_flags_direction(raw)
    val timerDelta: Byte get() = Omid.INSTANCE.omid_flags_timer_delta(raw)

    companion object {
        fun create(touched: Boolean, rawData: Boolean, direction: Boolean, timerDelta: Byte): OmidFlags {
            val raw = Omid.INSTANCE.omid_flags_new(touched, rawData, direction, timerDelta)
            return OmidFlags(raw)
        }
    }
}

class Packet(val inner: Omid.OmidPacket.ByValue) {
    val objectId: Short get() = inner.objectId
    val eventType: EventType get() = EventType.fromByte(inner.eventType)
    val flags: OmidFlags get() = OmidFlags(inner.flags)
    val payload: Int get() = inner.payload

    fun payloadAsF32(): Float = Omid.INSTANCE.omid_packet_payload_as_f32(inner)
    fun payloadAsI32(): Int = Omid.INSTANCE.omid_packet_payload_as_i32(inner)
    fun payloadAsU32(): Int = Omid.INSTANCE.omid_packet_payload_as_u32(inner)
    
    fun payloadAsXY(): Pair<Short, Short> {
        val outX = Memory(2)
        val outY = Memory(2)
        Omid.INSTANCE.omid_packet_payload_as_xy(inner, outX, outY)
        return Pair(outX.getShort(0), outY.getShort(0))
    }

    fun payloadAsAdc12(): Short = Omid.INSTANCE.omid_packet_payload_as_adc12(inner)
    fun payloadAsAdc16(): Short = Omid.INSTANCE.omid_packet_payload_as_adc16(inner)
    fun payloadAsNormalizedF32(adcBits: Byte): Float = Omid.INSTANCE.omid_packet_payload_as_normalized_f32(inner, adcBits)
    
    fun hapticIntensity(): Float = Omid.INSTANCE.omid_packet_haptic_intensity(inner)
    fun hapticForceProfile(): ForceProfile {
        val profileVal = Omid.INSTANCE.omid_packet_haptic_force_profile(inner)
        return if (profileVal >= 0) ForceProfile.fromByte(profileVal.toByte()) else ForceProfile.Unknown
    }

    fun toBytes(): ByteArray {
        val mem = Memory(8)
        Omid.INSTANCE.omid_packet_to_bytes(inner, mem)
        return mem.getByteArray(0, 8)
    }

    companion object {
        fun create(objectId: Short, eventType: EventType, flags: OmidFlags, payload: Int): Packet =
            Packet(Omid.INSTANCE.omid_packet_new(objectId, eventType.value, flags.raw, payload))

        fun createF32(objectId: Short, eventType: EventType, flags: OmidFlags, value: Float): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_f32(objectId, eventType.value, flags.raw, value))

        fun createI32(objectId: Short, eventType: EventType, flags: OmidFlags, value: Int): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_i32(objectId, eventType.value, flags.raw, value))

        fun createXY(objectId: Short, eventType: EventType, flags: OmidFlags, x: Short, y: Short): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_xy(objectId, eventType.value, flags.raw, x, y))

        fun createHaptic(objectId: Short, profile: ForceProfile, intensity: Float): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_haptic(objectId, profile.value, intensity))

        fun createAdc12(objectId: Short, eventType: EventType, flags: OmidFlags, value: Short): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_adc12(objectId, eventType.value, flags.raw, value))

        fun createAdc16(objectId: Short, eventType: EventType, flags: OmidFlags, value: Short): Packet =
            Packet(Omid.INSTANCE.omid_packet_new_adc16(objectId, eventType.value, flags.raw, value))

        fun fromBytes(bytes: ByteArray): Packet {
            require(bytes.size == 8) { "Data must be exactly 8 bytes" }
            val mem = Memory(8)
            mem.write(0, bytes, 0, 8)
            return Packet(Omid.INSTANCE.omid_packet_from_bytes(mem))
        }
    }
}
