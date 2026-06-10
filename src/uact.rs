use crate::packet::OmidPacket;
use crate::error::OmidError;

/// A single UACT (Unified Audio & Control Transport) Frame block.
///
/// Integrates synchronous PCM audio channels and a control packet under a single clock domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UactFrame<const C: usize> {
    /// The synchronous audio channel samples for this frame.
    pub audio: [f32; C],
    /// The control packet associated with this frame.
    pub control: OmidPacket,
}

impl<const C: usize> UactFrame<C> {
    /// Total serialized size of the UACT frame in bytes.
    pub const FRAME_SIZE: usize = C * 4 + 8;

    /// Creates a new `UactFrame` with the given audio samples and control packet.
    #[inline]
    pub fn new(audio: [f32; C], control: OmidPacket) -> Self {
        Self { audio, control }
    }

    /// Serializes the frame into a byte slice.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferTooSmall)` if the destination buffer is smaller than `Self::FRAME_SIZE`.
    pub fn serialize(&self, dest: &mut [u8]) -> Result<(), OmidError> {
        if dest.len() < Self::FRAME_SIZE {
            return Err(OmidError::BufferTooSmall);
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
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferUnderflow)` if the source buffer is smaller than `Self::FRAME_SIZE`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OmidError> {
        if bytes.len() < Self::FRAME_SIZE {
            return Err(OmidError::BufferUnderflow);
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
///
/// Uses an optimized internal ring buffer pattern to minimize copying and shifting operations,
/// and processes fully aligned frames in-place directly from incoming slices where possible.
pub struct UactDemuxer<const C: usize> {
    buffer: [u8; 1024],
    head: usize,
    tail: usize,
}

impl<const C: usize> UactDemuxer<C> {
    /// Creates a new, empty `UactDemuxer`.
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 1024],
            head: 0,
            tail: 0,
        }
    }

    /// Feeds raw bytes into the demuxer, invoking the frame callback for every fully parsed `UactFrame`.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::BufferOverflow)` if the internal accumulation buffer overflows.
    pub fn process_bytes<F>(&mut self, bytes: &[u8], mut frame_callback: F) -> Result<(), OmidError>
    where
        F: FnMut(UactFrame<C>),
    {
        let frame_size = UactFrame::<C>::FRAME_SIZE;

        // Optimized path: if no leftovers exist, parse directly from the incoming buffer
        if self.head == self.tail {
            let mut read_idx = 0;
            while bytes.len() - read_idx >= frame_size {
                if let Ok(frame) = UactFrame::<C>::from_bytes(&bytes[read_idx..read_idx + frame_size]) {
                    frame_callback(frame);
                }
                read_idx += frame_size;
            }

            let remaining = bytes.len() - read_idx;
            if remaining > 0 {
                if remaining > self.buffer.len() {
                    return Err(OmidError::BufferOverflow);
                }
                self.buffer[..remaining].copy_from_slice(&bytes[read_idx..]);
                self.head = 0;
                self.tail = remaining;
            }
            return Ok(());
        }

        // Shift data to the beginning if we need room at the end of the buffer
        if self.tail + bytes.len() > self.buffer.len() {
            let len = self.tail - self.head;
            if len > 0 {
                self.buffer.copy_within(self.head..self.tail, 0);
            }
            self.head = 0;
            self.tail = len;
        }

        // Check if the incoming data actually fits
        if self.tail + bytes.len() > self.buffer.len() {
            return Err(OmidError::BufferOverflow);
        }

        // Append incoming bytes
        self.buffer[self.tail..self.tail + bytes.len()].copy_from_slice(bytes);
        self.tail += bytes.len();

        // Process all complete frames from the accumulator buffer
        while self.tail - self.head >= frame_size {
            if let Ok(frame) = UactFrame::<C>::from_bytes(&self.buffer[self.head..self.head + frame_size]) {
                frame_callback(frame);
            }
            self.head += frame_size;
        }

        // Reset indexes to 0 if the buffer is empty to avoid unnecessary shifts
        if self.head == self.tail {
            self.head = 0;
            self.tail = 0;
        }

        Ok(())
    }
}

impl<const C: usize> Default for UactDemuxer<C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Clock synchronizer for sample-accurate sub-sample event timestamping.
pub struct ClockSynchronizer {
    /// The host audio stream sample rate (e.g. 48000, 96000, 192000).
    pub sample_rate: u32,
    /// The physical clock frequency in Hz used to measure sub-sample offsets.
    pub clock_frequency: f64,
}

impl ClockSynchronizer {
    /// Creates a new `ClockSynchronizer`.
    #[inline]
    pub fn new(sample_rate: u32, clock_frequency: f64) -> Self {
        Self {
            sample_rate,
            clock_frequency,
        }
    }

    /// Converts the sub-sample timer offset (ticks since sample start) into time in seconds.
    #[inline]
    pub fn timer_delta_to_seconds(&self, timer_delta: u8) -> f64 {
        let tick_duration = 1.0 / self.clock_frequency;
        (timer_delta as f64) * tick_duration
    }

    /// Computes the fractional sample offset relative to the start of the current sample block.
    #[inline]
    pub fn sample_offset(&self, timer_delta: u8) -> f64 {
        let seconds = self.timer_delta_to_seconds(timer_delta);
        seconds * (self.sample_rate as f64)
    }
}
