use crate::packet::OmidPacket;
use crate::event::{EventType, OmidFlags};
use crate::error::OmidError;

/// Translates standard MIDI 1.0 messages to OMID packets and vice-versa.
pub struct Midi1Translator;

impl Midi1Translator {
    /// Converts a 3-byte MIDI 1.0 message into an `OmidPacket`.
    ///
    /// The MIDI channel (0..15) is embedded as the upper 4 bits of the `object_id`,
    /// and the note/controller number occupies the lower 8 bits.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferUnderflow)` if `midi_msg` has less than 3 bytes.
    pub fn to_omid(midi_msg: &[u8]) -> Result<OmidPacket, OmidError> {
        if midi_msg.len() < 3 {
            return Err(OmidError::BufferUnderflow);
        }

        let status = midi_msg[0];
        let byte1 = midi_msg[1];
        let byte2 = midi_msg[2];

        let msg_type = status & 0xF0;
        let channel = status & 0x0F;

        // Combine channel and parameter/note index into a single 16-bit object_id
        let object_id = ((channel as u16) << 8) | (byte1 as u16);
        let flags = OmidFlags::new(false, false, false, 0);

        match msg_type {
            0x90 => {
                // Note On
                let velocity = byte2;
                if velocity > 0 {
                    let intensity = (velocity as f32) / 127.0;
                    Ok(OmidPacket::new_f32(object_id, EventType::KeyPress, flags, intensity))
                } else {
                    // Velocity 0 is treated as Note Off / KeyRelease
                    Ok(OmidPacket::new_f32(object_id, EventType::KeyRelease, flags, 0.0))
                }
            }
            0x80 => {
                // Note Off
                let velocity = byte2;
                let release_velocity = (velocity as f32) / 127.0;
                Ok(OmidPacket::new_f32(object_id, EventType::KeyRelease, flags, release_velocity))
            }
            0xB0 => {
                // Control Change
                let val = byte2;
                let norm_val = (val as f32) / 127.0;
                Ok(OmidPacket::new_f32(object_id, EventType::AbsoluteChange, flags, norm_val))
            }
            0xE0 => {
                // Pitch Bend (14-bit value)
                let lsb = byte1 as u16;
                let msb = byte2 as u16;
                let bend = (msb << 7) | lsb;
                let norm_bend = (bend as f32) / 16383.0;
                // Use a dedicated object_id for Pitch Bend on this channel
                let pb_object_id = ((channel as u16) << 8) | 0xFF;
                Ok(OmidPacket::new_f32(pb_object_id, EventType::AbsoluteChange, flags, norm_bend))
            }
            _ => Err(OmidError::IoError),
        }
    }

    /// Converts an `OmidPacket` into a 3-byte MIDI 1.0 message.
    ///
    /// Returns the length of the written message (typically 3) on success.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferTooSmall)` if the output buffer has less than 3 bytes.
    pub fn to_midi1(packet: OmidPacket, out_msg: &mut [u8]) -> Result<usize, OmidError> {
        if out_msg.len() < 3 {
            return Err(OmidError::BufferTooSmall);
        }

        let channel = ((packet.object_id >> 8) & 0x0F) as u8;
        let note_or_ctrl = (packet.object_id & 0xFF) as u8;

        match packet.event() {
            EventType::KeyPress => {
                out_msg[0] = 0x90 | channel;
                out_msg[1] = note_or_ctrl;
                let vel = (packet.payload_as_f32() * 127.0).clamp(0.0, 127.0) as u8;
                out_msg[2] = if vel == 0 { 1 } else { vel }; // avoid Note On with velocity 0
                Ok(3)
            }
            EventType::KeyRelease => {
                out_msg[0] = 0x80 | channel;
                out_msg[1] = note_or_ctrl;
                out_msg[2] = (packet.payload_as_f32() * 127.0).clamp(0.0, 127.0) as u8;
                Ok(3)
            }
            EventType::AbsoluteChange => {
                if note_or_ctrl == 0xFF {
                    // Pitch Bend
                    let bend = (packet.payload_as_f32() * 16383.0).clamp(0.0, 16383.0) as u16;
                    out_msg[0] = 0xE0 | channel;
                    out_msg[1] = (bend & 0x7F) as u8;
                    out_msg[2] = ((bend >> 7) & 0x7F) as u8;
                } else {
                    // Control Change
                    out_msg[0] = 0xB0 | channel;
                    out_msg[1] = note_or_ctrl;
                    out_msg[2] = (packet.payload_as_f32() * 127.0).clamp(0.0, 127.0) as u8;
                }
                Ok(3)
            }
            _ => Err(OmidError::IoError),
        }
    }
}

/// Helper for packing OMID packets into MIDI 2.0 UMP (Universal MIDI Packet) formats.
pub struct Midi2UmpTranslator;

impl Midi2UmpTranslator {
    /// Packs an 8-byte `OmidPacket` into a single 128-bit MIDI 2.0 Sysex8 UMP packet (4 x u32 words).
    ///
    /// Uses Message Type 0x5 (Data/Sysex8), Status Complete (0), and Stream ID.
    pub fn pack_to_sysex8(packet: OmidPacket, group: u8, stream_id: u8) -> [u32; 4] {
        let packet_bytes = packet.to_bytes();
        let mut ump = [0u32; 4];

        // Word 0: Type 5 (Sysex8), Group, Status = 0 (Complete), Number of Bytes = 9 (Stream ID + 8 packet bytes)
        let header_byte = 0x50 | (group & 0x0F);
        let status_len = 0x09; // Complete Sysex8 (0x00) | Length 9 (0x09)
        ump[0] = ((header_byte as u32) << 24)
            | ((status_len as u32) << 16)
            | ((stream_id as u32) << 8)
            | (packet_bytes[0] as u32);

        // Word 1: packet bytes 1, 2, 3, 4
        ump[1] = ((packet_bytes[1] as u32) << 24)
            | ((packet_bytes[2] as u32) << 16)
            | ((packet_bytes[3] as u32) << 8)
            | (packet_bytes[4] as u32);

        // Word 2: packet bytes 5, 6, 7 and 1 padding byte
        ump[2] = ((packet_bytes[5] as u32) << 24)
            | ((packet_bytes[6] as u32) << 16)
            | ((packet_bytes[7] as u32) << 8);

        // Word 3: unused padding
        ump[3] = 0;

        ump
    }

    /// Unpacks a 128-bit MIDI 2.0 Sysex8 UMP packet back into an `OmidPacket`.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferUnderflow)` if the UMP packet is not a valid Sysex8 OMID envelope.
    pub fn unpack_from_sysex8(ump: [u32; 4]) -> Result<OmidPacket, OmidError> {
        let word0 = ump[0];
        let msg_type = (word0 >> 28) & 0x0F;
        let status = (word0 >> 20) & 0x0F; // status type (0 = Complete, etc.)
        let len = (word0 >> 16) & 0x0F;

        if msg_type != 0x05 || status != 0 || len < 9 {
            return Err(OmidError::BufferUnderflow);
        }

        let mut packet_bytes = [0u8; 8];
        packet_bytes[0] = (word0 & 0xFF) as u8;

        let word1 = ump[1];
        packet_bytes[1] = ((word1 >> 24) & 0xFF) as u8;
        packet_bytes[2] = ((word1 >> 16) & 0xFF) as u8;
        packet_bytes[3] = ((word1 >> 8) & 0xFF) as u8;
        packet_bytes[4] = (word1 & 0xFF) as u8;

        let word2 = ump[2];
        packet_bytes[5] = ((word2 >> 24) & 0xFF) as u8;
        packet_bytes[6] = ((word2 >> 16) & 0xFF) as u8;
        packet_bytes[7] = ((word2 >> 8) & 0xFF) as u8;

        Ok(OmidPacket::from_bytes(&packet_bytes))
    }
}
