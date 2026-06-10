#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologyDescriptor {
    pub object_id: u16,
    pub object_type: u8,
    pub spatial_x: u16,
    pub spatial_y: u16,
    pub resolution: u8,
}

impl TopologyDescriptor {
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
    /// Uses little-endian byte ordering.
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
    /// Expects little-endian byte ordering.
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
