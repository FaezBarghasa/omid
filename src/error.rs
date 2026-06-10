use core::fmt;

/// Errors that can occur within the Omid library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmidError {
    /// The SPSC ring buffer queue has overflowed.
    QueueOverflow,
    /// The destination serialization buffer is too small.
    BufferTooSmall,
    /// The source deserialization buffer does not contain enough data.
    BufferUnderflow,
    /// The UACT demuxer internal buffer overflowed because the remaining data exceeded capacity.
    BufferOverflow,
    /// The transport interface (BLE, WiFi, or USB) is not connected.
    NotConnected,
    /// No worker queues were provided to the dispatcher.
    NoWorkerQueues,
    /// An invalid force profile code was supplied.
    InvalidForceProfile(u8),
    /// A hardware driver I/O error occurred.
    IoError,
}

impl fmt::Display for OmidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueOverflow => write!(f, "Queue Overflow - DSP Buffer Saturated"),
            Self::BufferTooSmall => write!(f, "Destination buffer too small"),
            Self::BufferUnderflow => write!(f, "Source buffer underflow"),
            Self::BufferOverflow => write!(f, "Internal buffer overflow: leftover chunk exceeds capacity"),
            Self::NotConnected => write!(f, "Device is not connected"),
            Self::NoWorkerQueues => write!(f, "No worker queues available"),
            Self::InvalidForceProfile(raw) => write!(f, "Invalid force profile: {:#04x}", raw),
            Self::IoError => write!(f, "Hardware driver I/O error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OmidError {}
