/// Spatial placement and resolution descriptor for OMID physical control objects.
///
/// Sent during system handshakes to inform the host of a control's physical layout
/// and resolution properties for UI and haptic map rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct TopologyDescriptor {
    /// The unique identifier of the target control object.
    pub object_id: u16,
    /// The classification of the control (e.g. 0x01 = Fader, 0x02 = Key, 0x03 = XY Pad).
    pub object_type: u8,
    /// Physical location offset on the horizontal X axis (in millimeters).
    pub spatial_x: u16,
    /// Physical location offset on the vertical Y axis (in millimeters).
    pub spatial_y: u16,
    /// The bit resolution of the sensor (e.g. 12 for 12-bit ADC, 16 for 16-bit ADC).
    pub resolution: u8,
}

impl TopologyDescriptor {
    /// Creates a new `TopologyDescriptor`.
    #[inline]
    pub fn new(object_id: u16, object_type: u8, spatial_x: u16, spatial_y: u16, resolution: u8) -> Self {
        Self {
            object_id,
            object_type,
            spatial_x,
            spatial_y,
            resolution,
        }
    }

    /// Serializes the topology descriptor to an 8-byte array.
    ///
    /// Uses little-endian byte ordering.
    #[inline]
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        
        let id_bytes = self.object_id.to_le_bytes();
        bytes[0] = id_bytes[0];
        bytes[1] = id_bytes[1];
        
        bytes[2] = self.object_type;
        
        let x_bytes = self.spatial_x.to_le_bytes();
        bytes[3] = x_bytes[0];
        bytes[4] = x_bytes[1];
        
        let y_bytes = self.spatial_y.to_le_bytes();
        bytes[5] = y_bytes[0];
        bytes[6] = y_bytes[1];
        
        bytes[7] = self.resolution;
        
        bytes
    }

    /// Deserializes a topology descriptor from an 8-byte array.
    ///
    /// Expects little-endian byte ordering.
    #[inline]
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        let object_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        let object_type = bytes[2];
        let spatial_x = u16::from_le_bytes([bytes[3], bytes[4]]);
        let spatial_y = u16::from_le_bytes([bytes[5], bytes[6]]);
        let resolution = bytes[7];
        
        Self {
            object_id,
            object_type,
            spatial_x,
            spatial_y,
            resolution,
        }
    }
}
