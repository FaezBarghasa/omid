use crate::packet::OmidPacket;

/// A single UACT (Unified Audio & Control Transport) Frame block.
/// Integrates synchronous PCM audio channels and a control packet under a single clock domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UactFrame<const C: usize> {
    pub audio: [f32; C],
    pub control: OmidPacket,
}

impl<const C: usize> UactFrame<C> {
    /// Total serialized size of the UACT frame in bytes.
    pub const FRAME_SIZE: usize = C * 4 + 8;

    /// Creates a new `UactFrame`.
    pub fn new(audio: [f32; C], control: OmidPacket) -> Self {
        Self { audio, control }
    }

    /// Serializes the frame into a byte slice.
    /// Returns an error if the destination buffer is too small.
    pub fn serialize(&self, dest: &mut [u8]) -> Result<(), &'static str> {
        if dest.len() < Self::FRAME_SIZE {
            return Err("Destination buffer too small");
        }

        for i in 0..C {
            let chan_bits = self.audio[i].to_bits();
            let chan_bytes = chan_bits.to_le_bytes();
            dest[i * 4..(i + 1) * 4].copy_from_slice(&chan_bytes);
        }

        let control_bytes = self.control.to_bytes();
        dest[C * 4..C * 4 + 8].copy_from_slice(&control_bytes);

        Ok(())
    }

    /// Deserializes a frame from a byte slice.
    /// Returns an error if the source buffer is too small.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::FRAME_SIZE {
            return Err("Source buffer underflow");
        }

        let mut audio = [0.0f32; C];
        for i in 0..C {
            let bits = u32::from_le_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
            audio[i] = f32::from_bits(bits);
        }

        let mut ctrl_bytes = [0u8; 8];
        ctrl_bytes.copy_from_slice(&bytes[C * 4..C * 4 + 8]);
        let control = OmidPacket::from_bytes(&ctrl_bytes);

        Ok(Self { audio, control })
    }
}

/// A zero-allocation stream demultiplexer for parsing continuous UACT frames from DMA buffers.
pub struct UactDemuxer<const C: usize> {
    buffer: [u8; 1024],
    len: usize,
}

impl<const C: usize> UactDemuxer<C> {
    /// Creates a new `UactDemuxer`.
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 1024],
            len: 0,
        }
    }

    /// Feeds raw bytes into the demuxer, invoking the frame callback for every fully parsed UactFrame.
    pub fn process_bytes<F>(&mut self, bytes: &[u8], mut frame_callback: F) -> Result<(), &'static str>
    where
        F: FnMut(UactFrame<C>),
    {
        let frame_size = UactFrame::<C>::FRAME_SIZE;
        let mut read_ptr = 0;

        // Process leftover buffer content
        if self.len > 0 {
            let space_left = self.buffer.len() - self.len;
            let copy_len = core::cmp::min(bytes.len(), space_left);
            self.buffer[self.len..self.len + copy_len].copy_from_slice(&bytes[..copy_len]);
            self.len += copy_len;
            read_ptr += copy_len;

            while self.len >= frame_size {
                if let Ok(frame) = UactFrame::<C>::from_bytes(&self.buffer[..frame_size]) {
                    frame_callback(frame);
                }
                // Shift buffer down
                self.buffer.copy_within(frame_size..self.len, 0);
                self.len -= frame_size;
            }
        }

        // Direct zero-copy processing of the remaining incoming stream
        while bytes.len() - read_ptr >= frame_size {
            if let Ok(frame) = UactFrame::<C>::from_bytes(&bytes[read_ptr..read_ptr + frame_size]) {
                frame_callback(frame);
            }
            read_ptr += frame_size;
        }

        // Buffer any remaining partial bytes for the next read
        let remaining = bytes.len() - read_ptr;
        if remaining > 0 {
            if remaining > self.buffer.len() {
                return Err("Internal buffer overflow: leftover chunk exceeds capacity");
            }
            self.buffer[..remaining].copy_from_slice(&bytes[read_ptr..]);
            self.len = remaining;
        }

        Ok(())
    }
}

impl<const C: usize> Default for UactDemuxer<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// Clock synchronizer for sample-accurate sub-sample event timestamping.
pub struct ClockSynchronizer {
    pub sample_rate: u32,
    pub clock_frequency: f64,
}

impl ClockSynchronizer {
    /// Creates a new `ClockSynchronizer`.
    pub fn new(sample_rate: u32, clock_frequency: f64) -> Self {
        Self {
            sample_rate,
            clock_frequency,
        }
    }

    /// Converts the sub-sample timer offset (ticks since sample start) into time in seconds.
    pub fn timer_delta_to_seconds(&self, timer_delta: u8) -> f64 {
        let tick_duration = 1.0 / self.clock_frequency;
        (timer_delta as f64) * tick_duration
    }

    /// Computes the fractional sample offset relative to the start of the current sample block.
    pub fn sample_offset(&self, timer_delta: u8) -> f64 {
        let seconds = self.timer_delta_to_seconds(timer_delta);
        seconds * (self.sample_rate as f64)
    }
}
